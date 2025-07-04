import binascii
import gzip
import json
import re
import time
from enum import Enum
from os import urandom
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Union

import certifi
import grpc
import requests
from cosmpy.common.types import JSONLike
from cosmpy.crypto.address import Address
from cosmpy.crypto.keypairs import PrivateKey
from cosmpy.protos.cosmos.auth.v1beta1.auth_pb2 import BaseAccount
from cosmpy.protos.cosmos.auth.v1beta1.query_pb2 import QueryAccountRequest
from cosmpy.protos.cosmos.auth.v1beta1.query_pb2_grpc import QueryStub as AuthGrpcClient
from cosmpy.protos.cosmos.bank.v1beta1.query_pb2 import QueryBalanceRequest
from cosmpy.protos.cosmos.bank.v1beta1.query_pb2_grpc import QueryStub as BankGrpcClient
from cosmpy.protos.cosmos.bank.v1beta1.tx_pb2 import MsgSend
from cosmpy.protos.cosmos.base.tendermint.v1beta1.query_pb2 import GetNodeInfoRequest
from cosmpy.protos.cosmos.base.tendermint.v1beta1.query_pb2_grpc import (
    ServiceStub as TendermintGrpcClient,
)
from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin
from cosmpy.protos.cosmos.crypto.secp256k1.keys_pb2 import PubKey as ProtoPubKey
from cosmpy.protos.cosmos.params.v1beta1.query_pb2 import QueryParamsRequest
from cosmpy.protos.cosmos.params.v1beta1.query_pb2_grpc import (
    QueryStub as QueryParamsGrpcClient,
)
from cosmpy.protos.cosmos.tx.signing.v1beta1.signing_pb2 import SignMode
from cosmpy.protos.cosmos.tx.v1beta1.service_pb2 import (
    BroadcastMode,
    BroadcastTxRequest,
    GetTxRequest,
    GetTxResponse,
    SimulateRequest,
    SimulateResponse,
)
from cosmpy.protos.cosmos.tx.v1beta1.service_pb2_grpc import ServiceStub as TxGrpcClient
from cosmpy.protos.cosmos.tx.v1beta1.tx_pb2 import (
    AuthInfo,
    Fee,
    ModeInfo,
    SignDoc,
    SignerInfo,
    Tx,
    TxBody,
)
from cosmpy.protos.cosmwasm.wasm.v1.query_pb2 import QuerySmartContractStateRequest
from cosmpy.protos.cosmwasm.wasm.v1.query_pb2_grpc import (
    QueryStub as CosmWasmGrpcClient,
)
from cosmpy.protos.cosmwasm.wasm.v1.tx_pb2 import (
    MsgExecuteContract,
    MsgInstantiateContract,
    MsgStoreCode,
)
from google.protobuf.any_pb2 import Any as ProtoAny
from google.protobuf.json_format import MessageToDict
from grpc import insecure_channel

from pre.common import (
    AbstractConfig,
    filter_data_with_types,
    get_defaults,
    types_from_annotations,
)
from pre.contract.base_contract import WalletInsufficientFunds
from pre.ledger.base_ledger import AbstractLedger, LedgerServerNotAvailable
from pre.ledger.cosmos.crypto import CosmosCrypto
from pre.utils.loggers import get_logger


_logger = get_logger(__name__)


# Pre-configured CosmWasm nodes
class NodeConfigPreset(Enum):
    local_net = 0
    dorado = 1


# CosmWasm client response codes
CLIENT_CODE_ERROR_EXCEPTION = 4
CLIENT_CODE_MESSAGE_SUCCESSFUL = 0

# maximum gas limit - tx will fail with higher limit
DEFAULT_TX_MAXIMUM_GAS_LIMIT = 3000000
DEFAULT_CONTRACT_TX_GAS = 900000
DEFAULT_SEND_TX_GAS = 120000
DEFAULT_MINIMUM_GAS_PRICE_AMOUNT = 500000000000
DEFAULT_FUNDS_AMOUNT = 9 * 10 ** 18

# Gas estimates will be increased by this multiplier
DEFAULT_TX_ESTIMATE_MULTIPLIER = 1.2
# When tx fails due to insufficient gas we increase multiplier by this factor
INCREASE_TX_ESTIMATE_MULTIPLIER = 1.2


class BroadcastException(Exception):
    pass


class FailedToGetReceiptException(Exception):
    txhash: str

    def __init__(self, message, txhash):
        super().__init__(message)
        self.txhash = txhash


class CosmosLedgerConfig(AbstractConfig):
    DEFAULT_FETCH_CHAIN_ID = "dorado-1"
    DEFAULT_DENOMINATION = "atestfet"
    PREFIX = "fetch"
    FETCHD_URL = "127.0.0.1:9090"

    @classmethod
    def validate(cls, data: Dict) -> Dict:
        types = types_from_annotations(CosmosLedger.__init__)
        new_data = cls.make_default()
        new_data.update(data)
        return filter_data_with_types(new_data, types)

    @classmethod
    def make_default(cls) -> Dict:
        defaults = get_defaults(CosmosLedger.__init__)
        defaults.update(
            dict(
                denom=cls.DEFAULT_DENOMINATION,
                chain_id=cls.DEFAULT_FETCH_CHAIN_ID,
                prefix=cls.PREFIX,
                node_address=cls.FETCHD_URL,
            )
        )
        return defaults


# Class that provides interface to communicate with CosmWasm/Fetch blockchain
class CosmosLedger(AbstractLedger):
    CONFIG_CLASS = CosmosLedgerConfig
    PRIVATE_KEY_LENGTH = 32

    _ADDR_RE = re.compile("^fetch[0-9a-z]{39}$")
    _CONTRACT_ADDR_RE = re.compile("^fetch[0-9a-z]{59}$")

    @staticmethod
    def prepare_grpc_client(secure_channel: bool, node_address: str):
        """
        Prepare grpc client

        :param secure_channel: Secure/insecure grpc channel
        :param node_address: web address of the GRPC node
        """

        # Clients to communicate with Cosmos/CosmWasm GRPC node
        if secure_channel:
            with open(certifi.where(), "rb") as f:
                trusted_certs = f.read()
            credentials = grpc.ssl_channel_credentials(root_certificates=trusted_certs)
            return grpc.secure_channel(node_address, credentials)
        else:
            return insecure_channel(node_address)

    def __init__(
        self,
        denom: str,
        chain_id: str,
        prefix: str,
        node_address: str,
        secure_channel: bool = False,
        validator_pk: Optional[str] = None,
        faucet_url: Optional[str] = None,
        msg_retry_interval: int = 2,
        msg_failed_retry_interval: int = 10,
        faucet_retry_interval: int = 20,
        n_total_msg_retries: int = 10,  # 10,
        get_response_retry_interval: float = 1,  # 2,
        n_get_response_retries: int = 90,  # 30,
        minimum_gas_price_amount: int = DEFAULT_MINIMUM_GAS_PRICE_AMOUNT,
        *args,
        **kwargs,
    ):
        """
        Create new instance to deploy and communicate with smart contract

        :param denom: Denom of stake
        :param chain_id: ID of a blockchain
        :param prefix: Cosmos string addresses prefix
        :param node_address: web address of the GRPC node
        :param secure_channel: Secure/insecure grpc channel
        :param validator_pk: Path to Validator's private key - for funding from validator
        :param faucet_url: Address of testnet faucet - for funding from testnet
        :param msg_retry_interval: Interval between message partial steps retries
        :param msg_failed_retry_interval: Interval between complete send/settle message attempts
        :param faucet_retry_interval: Get wealth from faucet retry interval
        :param n_total_msg_retries: Number of total send/settle transaction retries
        :param get_response_retry_interval: Retry interval for getting receipt
        :param n_get_response_retries: Number of get receipt retries
        :param minimum_gas_price_amount: Minimum gas price amount
        """
        # Override presets when parameters are specified
        self.chain_id = chain_id
        self.node_address = node_address
        self.faucet_url = faucet_url
        self.denom = denom
        self.prefix = prefix

        self.validator_crypto: Optional[CosmosCrypto] = None
        if validator_pk is not None:
            self.validator_crypto = self.load_crypto_from_str(
                validator_pk, prefix=prefix
            )

        # Clients to communicate with Cosmos/CosmWasm GRPC node
        self.rpc_client = self.prepare_grpc_client(secure_channel, node_address)
        self.tx_client = TxGrpcClient(self.rpc_client)
        self.auth_client = AuthGrpcClient(self.rpc_client)
        self.wasm_client = CosmWasmGrpcClient(self.rpc_client)
        self.bank_client = BankGrpcClient(self.rpc_client)
        self.tendermint_client = TendermintGrpcClient(self.rpc_client)
        self.params_client = QueryParamsGrpcClient(self.rpc_client)

        self.msg_retry_interval = msg_retry_interval
        self.msg_failed_retry_interval = msg_failed_retry_interval
        self.faucet_retry_interval = faucet_retry_interval
        self.n_get_response_retries = n_get_response_retries
        self.n_total_msg_retries = n_total_msg_retries
        self.get_response_retry_interval = get_response_retry_interval
        self.minimum_gas_price_amount = minimum_gas_price_amount

    @staticmethod
    def sign_transaction(
        tx: Tx,
        signer: PrivateKey,
        chain_id: str,
        account_number: int,
        deterministic: bool = False,
    ):
        """
        Sign transaction

        :param tx: Transaction to be signed
        :param signer: Signer of transaction
        :param chain_id: Chain ID
        :param account_number: Account Number
        :param deterministic: Deterministic mode flag
        """
        sd = SignDoc()
        sd.body_bytes = tx.body.SerializeToString()
        sd.auth_info_bytes = tx.auth_info.SerializeToString()
        sd.chain_id = chain_id
        sd.account_number = account_number

        data_for_signing = sd.SerializeToString()

        # Generating deterministic signature:
        signature = signer.sign(
            data_for_signing, deterministic=deterministic, canonicalise=True
        )
        tx.signatures.extend([signature])

    def load_crypto_from_file(
        self, keyfile_path: str, prefix: Optional[str] = None
    ) -> CosmosCrypto:
        return self.load_crypto_from_str(Path(keyfile_path).read_text(), prefix)

    def load_crypto_from_str(
        self, key_str: str, prefix: Optional[str] = None
    ) -> CosmosCrypto:
        prefix = prefix or self.prefix
        private_key = PrivateKey(bytes.fromhex(key_str))
        return CosmosCrypto(private_key=private_key, prefix=prefix)

    def make_new_crypto(self, prefix: Optional[str] = None) -> CosmosCrypto:
        key_str = self._generate_key()
        return self.load_crypto_from_str(key_str, prefix)

    def _sleep(self, seconds: Union[float, int]):
        time.sleep(seconds)

    def deploy_contract(
        self,
        sender_crypto: CosmosCrypto,
        contract_filename: str,
        gas_limit: Optional[int] = None,
    ) -> Tuple[int, JSONLike]:
        """
        Deploy smart contract on a blockchain

        :param sender_crypto: Crypto of deployer to sign deploy transaction
        :param contract_filename: Path to contract .wasm bytecode
        :param gas_limit:  Maximum amount of gas to be used on executing command
        :return: Deployment transaction response
        """

        attempt = 0
        res = None
        code_id: Optional[int] = None
        last_exception: Optional[Exception] = None
        while code_id is None and attempt < self.n_total_msg_retries:
            try:
                msg = self.get_packed_store_msg(
                    sender_address=sender_crypto.get_address(),
                    contract_filename=Path(contract_filename),
                )

                tx = self.generate_tx(
                    [msg],
                    [sender_crypto.get_address()],
                    [sender_crypto.get_pubkey_as_bytes()],
                    gas_limit=gas_limit,
                )
                self.sign_tx(tx, sender_crypto)

                res = self.broadcast_tx(tx)

                raw_log = json.loads(res.tx_response.raw_log)  # pylint: disable=E1101
                assert raw_log[0]["events"][1]["attributes"][0]["key"] == "code_id"
                code_id = int(raw_log[0]["events"][1]["attributes"][0]["value"])
            except BroadcastException as e:
                # Failure due to wrong sequence, signature, etc.
                last_exception = e
                _logger.warning(
                    f"Failed to deploy contract code due BroadcastException: {e}"
                )
                self._sleep(self.msg_failed_retry_interval)
                attempt += 1
                continue

        if code_id is None or res is None:  # pragma: nocover
            raise BroadcastException(
                f"Failed to deploy contract code after multiple attempts: {last_exception}"
            )

        return code_id, MessageToDict(res)

    def send_init_msg(
        self,
        sender_crypto: CosmosCrypto,
        code_id: int,
        init_msg: JSONLike,
        label: str,
        gas_limit: Union[int, str] = "auto",
    ) -> Tuple[str, JSONLike]:
        """
        Send init contract message

        :param sender_crypto: Deployer crypto to sign init message
        :param code_id: ID of binary code stored on chain
        :param init_msg: Init message in json format
        :param label: Label of current instance of contract
        :param gas_limit: Gas limit

        :return: Contract address string, transaction response
        """

        msg = self.get_packed_init_msg(
            sender_address=sender_crypto.get_address(),
            code_id=code_id,
            init_msg=init_msg,
            label=label,
        )

        elapsed_time = 0
        res: Optional[GetTxResponse] = None
        contract_address: Optional[str] = None
        last_exception: Optional[Exception] = None
        while contract_address is None and elapsed_time < self.n_total_msg_retries:
            try:
                tx = self.generate_tx(
                    [msg],
                    [sender_crypto.get_address()],
                    [sender_crypto.get_pubkey_as_bytes()],
                    gas_limit=DEFAULT_TX_MAXIMUM_GAS_LIMIT,
                )
                self.sign_tx(tx, sender_crypto)

                if gas_limit == "auto":
                    estimated_gas_limit = int(
                        self.estimate_tx_gas(tx) * DEFAULT_TX_ESTIMATE_MULTIPLIER
                    )

                    tx = self.generate_tx(
                        [msg],
                        [sender_crypto.get_address()],
                        [sender_crypto.get_pubkey_as_bytes()],
                        gas_limit=estimated_gas_limit,
                    )

                    self.sign_tx(tx, sender_crypto)

                res = self.broadcast_tx(tx)

                raw_log = json.loads(res.tx_response.raw_log)  # pylint: disable=E1101
                if (
                    raw_log[0]["events"][0]["attributes"][0]["key"]
                    == "_contract_address"
                ):
                    contract_address = str(
                        raw_log[0]["events"][0]["attributes"][0]["value"]
                    )
            except BroadcastException as e:
                # Failure due to wrong sequence, signature, etc.
                last_exception = e
                _logger.warning(f"Failed to init contract due BroadcastException: {e}")
            except json.decoder.JSONDecodeError as e:
                # Failure due to response parsing error
                last_exception = e
                _logger.warning(
                    f"Failed to parse init Contract response {res.tx_response.raw_log if res is not None else None} : {e}"
                )

            if contract_address is None:
                self._sleep(self.msg_failed_retry_interval)
                elapsed_time += 1

        if contract_address is None or res is None:
            error_msg = ""
            if res:
                error_msg = res.tx_response.raw_log

            raise BroadcastException(
                f"Failed to init contract after multiple attempts: {last_exception} {error_msg}"
            )

        return contract_address, MessageToDict(res)

    def send_query_msg(
        self,
        contract_address: str,
        query_msg: JSONLike,
        num_retries: Optional[int] = None,
    ) -> JSONLike:
        """
        Generate and send query message to get state of smart contract
        - No signing is required because it works with contract as read only

        :param contract_address: Address of contract running on chain
        :param query_msg: Query message in json format
        :param num_retries: Optional number of retries

        :return: Query json response
        """
        request = QuerySmartContractStateRequest(
            address=contract_address, query_data=json.dumps(query_msg).encode("UTF8")
        )

        if num_retries is None:
            num_retries = self.n_total_msg_retries

        res = None
        last_exception: Optional[Exception] = None
        for _ in range(num_retries):
            try:
                res = self.wasm_client.SmartContractState(request)
                if res is not None:
                    break
            except Exception as e:  # pylint: disable=W0703
                last_exception = e
                _logger.warning(f"Cannot get contract state: {e}")
                self._sleep(self.msg_failed_retry_interval)

        if res is None:
            raise BroadcastException(
                f"Getting contract state failed after multiple attempts: {last_exception}"
            ) from last_exception
        return json.loads(res.data)  # pylint: disable=E1101

    def send_execute_msg(
        self,
        sender_crypto: CosmosCrypto,
        contract_address: str,
        execute_msg: JSONLike,
        gas_limit: Optional[Union[int, str]] = "auto",
        amount: Optional[List[Coin]] = None,
        retries: Optional[int] = None,
    ) -> Tuple[JSONLike, int]:
        """
        Generate, sign and send handle message

        :param sender_crypto: Sender's crypto to sign init message
        :param contract_address: Address of contract running on chain
        :param execute_msg: Execute message in json format
        :param gas_limit: Gas limit
        :param amount: Funds to be transferred to contract address
        :param retries: Optional number of retries

        :return: Execute message response
        """

        res: Optional[GetTxResponse] = None
        last_exception: Optional[Exception] = None

        if retries is None:
            retries = self.n_total_msg_retries

        msg = self.get_packed_exec_msg(
            sender_address=sender_crypto.get_address(),
            contract_address=contract_address,
            msg=execute_msg,
            funds=amount,
        )

        tx_estimate_multiplier = DEFAULT_TX_ESTIMATE_MULTIPLIER
        for _ in range(retries):
            try:
                tx = self.generate_tx(
                    [msg],
                    [sender_crypto.get_address()],
                    [sender_crypto.get_pubkey_as_bytes()],
                    gas_limit=DEFAULT_TX_MAXIMUM_GAS_LIMIT,
                )

                self.sign_tx(tx, sender_crypto)

                if gas_limit == "auto":
                    estimated_gas_limit = int(
                        self.estimate_tx_gas(tx) * tx_estimate_multiplier
                    )

                    tx = self.generate_tx(
                        [msg],
                        [sender_crypto.get_address()],
                        [sender_crypto.get_pubkey_as_bytes()],
                        gas_limit=estimated_gas_limit,
                    )

                    self.sign_tx(tx, sender_crypto)

                res = self.broadcast_tx(tx)
                if res is not None:
                    if "out of gas" in res.tx_response.raw_log:
                        _logger.warning(
                            f"Failed to execute contract code due out of gas: {res.tx_response.raw_log}"
                        )

                        # Increase gas estimate multiplier for this transaction
                        tx_estimate_multiplier *= INCREASE_TX_ESTIMATE_MULTIPLIER
                        self._sleep(self.msg_failed_retry_interval)
                        continue
                    break
            except BroadcastException as e:
                # Failure due to wrong sequence, signature, etc.
                last_exception = e
                _logger.warning(
                    f"Failed to deploy contract code due BroadcastException: {e}"
                )
                self._sleep(self.msg_failed_retry_interval)

        if res is None:
            raise BroadcastException(
                f"Failed to execute contract after multiple attempts: {last_exception}"
            ) from last_exception

        # err_code >0 in case of exceptions inside rust contract
        err_code = res.tx_response.code  # pylint: disable=E1101
        return MessageToDict(res), err_code

    def ensure_funds(
        self,
        addresses: List[str],
        amount: Optional[int] = None,
        denom: Optional[str] = None,
        faucet_url: Optional[str] = None,
        validator_crypto: Optional[CosmosCrypto] = None,
    ):
        """
        Refill funds of addresses using faucet or validator
        Refilling from validator is preferred if both options are present

        :param addresses: Address to be refilled
        :param amount: Amount of refill
        :param denom: Denomination to refill
        :param faucet_url: Faucet URL for refill
        :param validator_crypto: Private key of validator for refill

        :return: Nothing
        """

        denom = denom if denom is not None else self.denom
        assert denom is not None

        used_faucet_url = None
        used_validator_crypto = None

        if validator_crypto is not None:
            used_validator_crypto = validator_crypto
        elif faucet_url is not None:
            used_faucet_url = faucet_url
        else:
            used_faucet_url = self.faucet_url
            used_validator_crypto = self.validator_crypto

        if used_validator_crypto is not None:
            self._refill_wealth_from_validator(
                used_validator_crypto, denom, addresses, amount
            )
        elif used_faucet_url is not None:
            self._refill_wealth_from_faucet(used_faucet_url, denom, addresses, amount)
        else:
            raise RuntimeError(
                "Faucet or validator was not specified, cannot refill addresses"
            )

    def query_funds(self, address: str) -> str:
        """
        Query funds of address using faucet or validator. Returns the string 'unknown' if it
        cannot query the network

        :param address: Address to be query

        :return: String representation of funds: i.e. 10000FET
        """

        balance = str(self.get_balance(address))
        ret = "unknown" if balance is None else balance

        return ret

    def get_balance(self, address: Address, denom: Optional[str] = None) -> int:
        """
        Query funds of address and denom

        :param address: Address to be query
        :param denom: Denom of coins

        :return: Integer representation of funds: i.e. 10000
        """

        if denom is None:
            denom = self.denom

        res = None
        last_exception: Optional[Exception] = None
        for _ in range(self.n_total_msg_retries):
            try:
                res = self.bank_client.Balance(
                    QueryBalanceRequest(address=str(address), denom=denom)
                )
                if res is not None:
                    break
            except Exception as e:  # pylint: disable=W0703
                last_exception = e
                _logger.warning(f"Cannot get balance: {e}")
                self._sleep(self.msg_retry_interval)
                continue

        if res is None:
            raise BroadcastException(
                f"Getting balance failed after multiple attempts: {last_exception}"
            )

        return int(res.balance.amount)

    def _refill_wealth_from_faucet(
        self, faucet_url: str, denom: str, addresses, amount: Optional[int] = None
    ):
        """
        Uses faucet api to refill balance of addresses

        :param faucet_url: Faucet URL
        :param denom: Denom
        :param addresses: List of addresses to be refilled
        :param amount: Amount to be refilled

        :return: Nothing
        """

        if amount:
            min_amount_required = amount
        else:
            min_amount_required = DEFAULT_FUNDS_AMOUNT

        for address in addresses:
            attempts_allowed = 10

            # Retry in case of network issues
            while attempts_allowed > 0:
                try:
                    attempts_allowed -= 1
                    balance = self.get_balance(address, denom=denom)

                    if balance < min_amount_required:
                        _logger.info(
                            f"Refilling balance of {str(address)} from faucet. Currently: {balance}"
                        )
                        # Send faucet request
                        response = requests.post(
                            f"{faucet_url}/api/v3/claims",
                            json={"address": address},
                        )

                        if response.status_code != 200:
                            _logger.exception(
                                f"Failed to refill the balance from faucet, retry in {self.faucet_retry_interval} seconds: {str(response)}"
                            )

                        # Wait for wealth to be refilled
                        self._sleep(self.faucet_retry_interval)
                        continue
                    _logger.info(f"Balance of {str(address)} is {str(balance)}")
                    break
                except Exception as e:  # pylint: disable=W0703
                    _logger.exception(
                        f"Failed to refill the balance from faucet, retry in {self.faucet_retry_interval} second: {e} ({type(e)})"
                    )
                    self._sleep(self.faucet_retry_interval)
                    continue
        # todo: add result of execution?

    def send_funds(
        self,
        from_crypto: CosmosCrypto,
        to_address: str,
        amount: int,
        denom: Optional[str] = None,
        gas_limit: Optional[int] = DEFAULT_SEND_TX_GAS,
    ):
        """
        Transfer funds from one address to another address

        :param from_crypto: Crypto with funds to be sent
        :param to_address: Address to receive funds
        :param amount: Amount of funds
        :param denom: Denomination

        :return: Transaction response
        """
        if denom is None:
            denom = self.denom

        amount_coins = [Coin(amount=str(amount), denom=denom)]
        from_address = str(from_crypto.get_address())

        msg = self.get_packed_send_msg(
            from_address=from_address, to_address=to_address, amount=amount_coins
        )

        tx = self.generate_tx(
            [msg],
            [from_address],
            [from_crypto.get_pubkey_as_bytes()],
            gas_limit=gas_limit,
        )
        self.sign_tx(tx, from_crypto)

        return self.broadcast_tx(tx)

    def sign_tx(self, tx: Tx, crypto: CosmosCrypto):
        """
        Sign tx using crypto
        - network is used to query account_number if not already stored in crypto

        :param tx: Transaction to be signed
        :param crypto: Crypto used to sign transaction

        :return: Nothing
        """

        # Update account number if needed - Getting account data might fail if address is not funded
        self._ensure_accont_number(crypto)

        self.sign_transaction(
            tx, crypto.private_key, self.chain_id, crypto.account_number
        )

    def _ensure_accont_number(self, crypto: CosmosCrypto):
        if crypto.account_number is None:
            account = self._query_account_data(crypto.get_address())
            crypto.account_number = account.account_number  # pylint: disable=E1101

    def _refill_wealth_from_validator(
        self,
        validator_crypto: CosmosCrypto,
        denom: str,
        addresses: List[str],
        amount: Optional[int] = None,
    ):
        """
        Refill funds of addresses using validator
        - Works only for network with validator account

        :param validator_crypto: Validator crypto
        :param denom: Denom to be refilled
        :param addresses: Address to be refilled
        :param amount: Amount to be refilled

        :raises BroadcastException: if refilling fails.

        :return: Nothing
        """
        if amount:
            min_amount_required = amount
        else:
            min_amount_required = DEFAULT_FUNDS_AMOUNT

        for address in addresses:
            balance = self.get_balance(address, denom=denom)
            assert isinstance(balance, int)
            if balance < min_amount_required:

                elapsed_time = 0
                done = False
                last_exception: Optional[Exception] = None
                while not done and elapsed_time < self.n_total_msg_retries:
                    elapsed_time += 1
                    try:
                        _logger.info(
                            f"Refilling balance of {str(address)} from validator {str(validator_crypto.get_address())}"
                        )
                        res = self.send_funds(
                            validator_crypto,
                            address,
                            min_amount_required - balance,
                            denom=denom,
                        )
                        if res is not None and res.tx_response.code == 0:
                            done = True
                    except Exception as e:  # pylint: disable=W0703
                        last_exception = e
                        _logger.warning(f"Cannot refill funds from validator: {e}")
                        time.sleep(self.msg_failed_retry_interval)
                        continue
                    if not done:
                        raise BroadcastException(
                            f"Refilling funds from validator failed after multiple attempts: {last_exception}"
                        )

    @classmethod
    def _generate_key(cls) -> str:
        """
        Generate random private key

        :return: Random hex string representation of private key
        """

        return binascii.b2a_hex(urandom(cls.PRIVATE_KEY_LENGTH)).decode("utf-8")

    def generate_tx(
        self,
        packed_msgs: List[ProtoAny],
        from_addresses: List[Address],
        pub_keys: List[bytes],
        fee: Optional[List[Coin]] = None,
        memo: str = "",
        gas_limit: Optional[int] = None,
    ) -> Tx:
        """
        Generate transaction that can be later signed

        :param packed_msgs: Messages to be in transaction
        :param from_addresses: List of addresses of each sender
        :param pub_keys: List of public keys
        :param fee: Transaction fee
        :param memo: Memo
        :param gas_limit: Gas limit

        :return: Tx
        """

        max_gas_limit = self.query_max_gas_limit()

        gas_limit = gas_limit if gas_limit else max_gas_limit

        # Tx with higher than maximum gas limit cannot be broadcast
        if gas_limit > max_gas_limit:
            gas_limit = max_gas_limit

        fee = fee if fee else self.calculate_tx_fee(gas_limit)

        # Get account and signer info for each sender
        accounts: List[BaseAccount] = []
        signer_infos: List[SignerInfo] = []
        for from_address, pub_key in zip(from_addresses, pub_keys):
            account = self._query_account_data(from_address)
            accounts.append(account)
            signer_infos.append(self._get_signer_info(account, pub_key))

        # Prepare auth info
        auth_info = AuthInfo(
            signer_infos=signer_infos,
            fee=Fee(amount=fee, gas_limit=gas_limit),
        )

        # Prepare Tx body
        tx_body = TxBody()
        tx_body.memo = memo
        tx_body.messages.extend(packed_msgs)  # pylint: disable=E1101

        # Prepare Tx
        tx = Tx(body=tx_body, auth_info=auth_info)
        return tx

    def _query_account_data(self, address: Address) -> BaseAccount:
        """
        Query account data for signing

        :param address: Address of account to query data about

        :raises TypeError: in case of wrong account type.
        :raises BroadcastException: if broadcasting fails.

        :return: BaseAccount
        """
        # Get account data for signing

        last_exception: Optional[Exception] = None
        account_response = None
        for _ in range(self.n_total_msg_retries):
            try:
                account_response = self.auth_client.Account(
                    QueryAccountRequest(address=str(address))
                )
                break
            except Exception as e:  # pylint: disable=W0703
                last_exception = e
                _logger.warning(f"Cannot query account data: {e}")
                self._sleep(self.msg_retry_interval)
                continue

        if account_response is None:
            raise BroadcastException(
                f"Getting account data failed after multiple attempts: {last_exception}"
            )

        account = BaseAccount()
        if account_response.account.Is(BaseAccount.DESCRIPTOR):
            account_response.account.Unpack(account)
        else:
            raise TypeError("Unexpected account type")
        return account

    @staticmethod
    def _get_signer_info(from_acc: BaseAccount, pub_key: bytes) -> SignerInfo:
        """
        Generate signer info

        :param from_acc: Account info of signer
        :param pub_key: Public key bytes

        :return: SignerInfo
        """

        from_pub_key_packed = ProtoAny()
        from_pub_key_pb = ProtoPubKey(key=pub_key)
        from_pub_key_packed.Pack(from_pub_key_pb, type_url_prefix="/")  # type: ignore

        # Prepare auth info
        single = ModeInfo.Single(mode=SignMode.SIGN_MODE_DIRECT)
        mode_info = ModeInfo(single=single)
        signer_info = SignerInfo(
            public_key=from_pub_key_packed,
            mode_info=mode_info,
            sequence=from_acc.sequence,
        )
        return signer_info

    @staticmethod
    def get_packed_send_msg(
        from_address: Address, to_address: Address, amount: List[Coin]
    ) -> ProtoAny:
        """
        Generate and pack MsgSend

        :param from_address: Address of sender
        :param to_address: Address of recipient
        :param amount: List of Coins to be sent

        :return: packer ProtoAny type message
        """
        msg_send = MsgSend(
            from_address=str(from_address), to_address=str(to_address), amount=amount
        )
        send_msg_packed = ProtoAny()
        send_msg_packed.Pack(msg_send, type_url_prefix="/")  # type: ignore

        return send_msg_packed

    def broadcast_tx(self, tx: Tx, retries: Optional[int] = None) -> GetTxResponse:
        """
        Broadcast transaction and get receipt

        :param tx: Transaction

        :raises BroadcastException: if broadcasting fails.

        :return: GetTxResponse
        """

        tx_data = tx.SerializeToString()
        broad_tx_req = BroadcastTxRequest(
            tx_bytes=tx_data, mode=BroadcastMode.BROADCAST_MODE_SYNC
        )

        if retries is None:
            retries = self.n_total_msg_retries

        last_exception = None
        broad_tx_resp = None
        for _ in range(retries):
            try:
                broad_tx_resp = self.tx_client.BroadcastTx(broad_tx_req)
                break
            except Exception as e:  # pylint: disable=W0703
                last_exception = e
                _logger.warning(f"Transaction broadcasting failed: {e}")
                self._sleep(self.msg_retry_interval)

        if broad_tx_resp is None:
            raise BroadcastException(
                f"Broadcasting tx failed after multiple attempts: {last_exception}"
            )

        # Transaction cannot be broadcast because of wrong format, sequence, signature, etc.
        if broad_tx_resp.tx_response.code != 0:
            raw_log = broad_tx_resp.tx_response.raw_log
            raise BroadcastException(f"Transaction cannot be broadcast: {raw_log}")

        # Wait for transaction to settle
        return self.make_tx_request(txhash=broad_tx_resp.tx_response.txhash)

    def is_tx_settled(self, txhash: str) -> bool:
        """
        Get tx receipt and check error code

        :param txhash: Transaction hash

        :return: true if transaction was successful
        """

        res = None
        try:
            res = self.make_tx_request(txhash)
        except FailedToGetReceiptException:
            return False

        if res is not None:
            return True

        return False

    def estimate_tx_gas(self, tx) -> int:
        """
        Simulate transaction and get estimated tx_limit

        :param tx: Transaction

        :raises WalletInsufficientFunds: If there is not enough funds

        :return: int estimated gas limit
        """

        estimated_gas_limit = DEFAULT_CONTRACT_TX_GAS
        try:
            estimated_gas_limit = int(self.simulate_tx(tx).gas_info.gas_used)
        except Exception as e:
            if "insufficient funds" in str(e):
                raise WalletInsufficientFunds(str(e)) from e
            _logger.warning(f"Failed to simulate tx: {e}")
        return estimated_gas_limit

    def simulate_tx(self, tx: Tx) -> SimulateResponse:
        """
        Simulate transaction and get estimated tx_limit

        :param tx: Transaction
        :param retries: Number of attempts

        :raises BroadcastException: if simulate fails.

        :return: int estimated gas limit
        """

        simulate_req = SimulateRequest(tx=tx)
        simulate_resp = self.tx_client.Simulate(simulate_req)

        return simulate_resp

    def make_tx_request(self, txhash):
        tx_request = GetTxRequest(hash=txhash)
        last_exception = None
        tx_response = None

        for _ in range(self.n_get_response_retries):
            try:
                # Send GetTx request
                tx_response = self.tx_client.GetTx(tx_request)
                break
            except Exception as e:  # pylint: disable=W0703
                # This fails when Tx is not on chain yet - not an actual error
                last_exception = e
                self._sleep(self.get_response_retry_interval)

        if tx_response is None:
            raise FailedToGetReceiptException(
                f"Getting tx {txhash} response failed after multiple attempts: {last_exception}",
                txhash=txhash,
            ) from last_exception

        return tx_response

    @staticmethod
    def get_packed_store_msg(
        sender_address: Address, contract_filename: Path
    ) -> ProtoAny:
        """
        Loads contract bytecode, generate and return packed MsgStoreCode

        :param sender_address: Address of transaction sender
        :param contract_filename: Path to smart contract bytecode

        :return: Packed MsgStoreCode
        """
        with open(contract_filename, "rb") as contract_file:
            wasm_byte_code = gzip.compress(contract_file.read(), 9)

        msg_send = MsgStoreCode(
            sender=str(sender_address),
            wasm_byte_code=wasm_byte_code,
        )
        send_msg_packed = ProtoAny()
        send_msg_packed.Pack(msg_send, type_url_prefix="/")  # type: ignore

        return send_msg_packed

    @staticmethod
    def get_packed_init_msg(
        sender_address: Address,
        code_id: int,
        init_msg: JSONLike,
        label="contract",
        funds: Optional[List[Coin]] = None,
    ) -> ProtoAny:
        """
        Create and pack MsgInstantiateContract

        :param sender_address: Sender's address
        :param code_id: code_id of stored contract bytecode
        :param init_msg: Parameters to be passed to smart contract constructor
        :param label: Label
        :param funds: Funds transferred to new contract

        :return: Packed MsgInstantiateContract
        """
        msg_send = MsgInstantiateContract(
            sender=str(sender_address),
            code_id=code_id,
            msg=json.dumps(init_msg).encode("UTF8"),
            label=label,
            funds=funds,
        )
        send_msg_packed = ProtoAny()
        send_msg_packed.Pack(msg_send, type_url_prefix="/")  # type: ignore

        return send_msg_packed

    @staticmethod
    def get_packed_exec_msg(
        sender_address: Address,
        contract_address: str,
        msg: JSONLike,
        funds: Optional[List[Coin]] = None,
    ) -> ProtoAny:
        """
        Create and pack MsgExecuteContract

        :param sender_address: Address of sender
        :param contract_address: Address of contract
        :param msg: Parameters to be passed to smart contract
        :param funds: Funds to be sent to smart contract

        :return: Packed MsgExecuteContract
        """
        msg_send = MsgExecuteContract(
            sender=str(sender_address),
            contract=contract_address,
            msg=json.dumps(msg).encode("UTF8"),
            funds=funds,
        )
        send_msg_packed = ProtoAny()
        send_msg_packed.Pack(msg_send, type_url_prefix="/")  # type: ignore

        return send_msg_packed

    def check_availability(self):

        last_exception: Optional[Exception] = None
        done = False
        for _ in range(self.n_total_msg_retries):
            try:
                node_info = self.tendermint_client.GetNodeInfo(GetNodeInfoRequest())
                if node_info.default_node_info.network != self.chain_id:
                    last_exception = ValueError(
                        f"Bad chain id, expected {node_info.default_node_info.network}, got {self.chain_id}"
                    )
                    break
                done = True
                break
            except Exception as e:
                last_exception = e

        if not done and last_exception is not None:
            raise LedgerServerNotAvailable(
                f"ledger server is not available with address: {self.node_address}: {last_exception}"
            ) from last_exception

    @classmethod
    def validate_address(cls, address: str):
        if not cls._ADDR_RE.match(address):
            raise ValueError(f"Cosmos address {address} is invalid")

    @classmethod
    def validate_contract_address(cls, address: str):
        if not cls._CONTRACT_ADDR_RE.match(address):
            raise ValueError(f"Contract address {address} is invalid")

    def calculate_tx_fee(self, gas_limit) -> List[Coin]:
        """
        Calculate tx fee

        :param gas_limit: Gas amount limit

        :return: tx fee
        """

        gas_price = self.query_minimum_gas_price()

        # tx_fee = gas_price * gas
        return [
            Coin(denom=gas_price.denom, amount=str(gas_limit * int(gas_price.amount)))
        ]

    def query_params(self, subspace: str, key: str) -> str:
        """
        Query node params

        :param subspace: Subspace
        :param key: Key

        :return: String from QueryParamsResponse.params.value
        """

        request = QueryParamsRequest(subspace=subspace, key=key)

        last_exception: Optional[Exception] = None
        resp = None
        for _ in range(self.n_total_msg_retries):
            try:
                resp = self.params_client.Params(request)
                break
            except Exception as e:
                last_exception = e

        if resp is None and last_exception is not None:
            raise LedgerServerNotAvailable(
                f"ledger server is not available with address: {self.node_address}: {last_exception}"
            ) from last_exception

        assert resp is not None
        return resp.param.value

    def query_max_gas_limit(self) -> int:
        """
        Query maximum gas from node

        :return: Maximum gas limit
        """

        params_value = json.loads(
            self.query_params(subspace="baseapp", key="BlockParams")
        )
        max_gas = int(params_value["max_gas"])

        if max_gas == -1:
            return DEFAULT_TX_MAXIMUM_GAS_LIMIT
        else:
            return max_gas

    def query_minimum_gas_price(self) -> Coin:
        """
        Query minimum price per gas unit from node

        :return: Coin price per gas unit
        """

        return Coin(denom=self.denom, amount=str(self.minimum_gas_price_amount))
