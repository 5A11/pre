import re
from base64 import b64decode, b64encode
from pathlib import Path
from typing import Dict, List, Optional

from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin

from pre.common import (
    Address,
    ContractState,
    Delegation,
    DelegationState,
    GetFragmentsResponse,
    HashID,
    ProxyStatus,
    ProxyTask,
    ReencryptionRequestState,
    ProxyState,
    DelegationStatus,
    StakingConfig,
)
from pre.contract.base_contract import (
    AbstractAdminContract,
    AbstractContractQueries,
    AbstractDelegatorContract,
    AbstractProxyContract,
    BadContractAddress,
    ContractExecutionError,
    ContractInstantiateFailure,
    ContractQueryError,
    DataAlreadyExist,
    DataEntry,
    DataEntryDoesNotExist,
    DelegationAlreadyAdded,
    DelegationAlreadyExist,
    NotAdminError,
    NotEnoughStakeToWithdraw,
    ProxyAlreadyExist,
    ProxyAlreadyRegistered,
    ProxyNotRegistered,
    ReencryptedCapsuleFragAlreadyProvided,
    ReencryptionAlreadyRequested,
    UnknownProxy,
    UnkownReencryptionRequest, ProxiesAreTooBusy,
)
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.ledger.cosmos.ledger import BroadcastException, CosmosLedger
from pre.utils.loggers import get_logger

CONTRACT_WASM_FILE = str(
    Path(__file__).parent.parent.parent / "contract" / "./cw_proxy_reencryption.wasm"
)

_logger = get_logger(__name__)


def encode_bytes(data: bytes) -> str:
    return b64encode(data).decode("ascii")


class ContractExecuteExceptionMixIn:
    @classmethod
    def _exception_from_res(cls, error_code, res):
        if error_code == 0:
            return

        raw_log = ""

        try:
            raw_log = res["txResponse"]["rawLog"]
        except KeyError:  # pragma: nocover
            raw_log = ""

        if "Pubkey already used" in raw_log:
            raise ProxyAlreadyRegistered(raw_log, error_code, res)
        elif "Sender is not a proxy." in raw_log:
            raise UnknownProxy(raw_log, error_code, res)
        elif "Proxy already registered." in raw_log:
            raise ProxyAlreadyRegistered(raw_log, error_code, res)
        elif re.search("Delegator .* already registered with this pubkey", raw_log):
            raise DelegationAlreadyAdded(raw_log, error_code, res)
        elif "is already proxy" in raw_log:
            raise ProxyAlreadyExist(raw_log, error_code, res)
        elif "is not a proxy" in raw_log:
            raise ProxyNotRegistered(raw_log, error_code, res)
        elif "Proxy already unregistered" in raw_log:
            raise ProxyNotRegistered(raw_log, error_code, res)
        elif "No proxies selected for this delegation" in raw_log:
            raise UnknownProxy(raw_log, error_code, res)
        elif "Reencryption already requested" in raw_log:
            raise ReencryptionAlreadyRequested(raw_log, error_code, res)
        elif "Fragment already provided" in raw_log:
            raise ReencryptedCapsuleFragAlreadyProvided(raw_log, error_code, res)
        elif "Entry with ID hash_id already exist" in raw_log:
            raise DataAlreadyExist(raw_log, error_code, res)
        elif "Delegation strings already provided" in raw_log:
            raise DelegationAlreadyExist(raw_log, error_code, res)
        elif "This fragment was not requested" in raw_log:
            raise UnkownReencryptionRequest(raw_log, error_code, res)
        elif "contract: not found" in raw_log:
            raise BadContractAddress(raw_log, error_code, res)
        elif re.search("Data entry doesn't exist", raw_log):
            raise DataEntryDoesNotExist(raw_log, error_code, res)
        elif re.search("Only admin can execute this method", raw_log):
            raise NotAdminError(raw_log, error_code, res)
        elif "Not enough stake to withdraw: execute wasm contract failed" in raw_log:
            raise NotEnoughStakeToWithdraw(raw_log, error_code, res)
        elif "Proxies are too busy, try again later" in raw_log:
            raise ProxiesAreTooBusy(raw_log, error_code, res)
        raise ContractExecutionError(
            f"Contract execution failed: {raw_log}", error_code, res
        )  # pragma: nocover


class ContractQueries(AbstractContractQueries):
    ledger: CosmosLedger

    def _send_query(self, state_msg):
        try:
            return self.ledger.send_query_msg(self.contract_address, state_msg)
        except BroadcastException as e:
            if "contract: not found: invalid request" in str(e):
                raise ContractQueryError(str(e))
            raise

    def get_avaiable_proxies(self) -> List[bytes]:
        state_msg: Dict = {"get_available_proxies": {}}
        json_res = self._send_query(state_msg)
        return [b64decode(i) for i in json_res["proxy_pubkeys"]]

    def get_selected_proxies_for_delegation(
            self,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        state_msg: Dict = {
            "get_selected_proxies_for_delegation": {
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        json_res = self._send_query(state_msg)
        return [b64decode(i) for i in json_res["proxy_pubkeys"]]

    def get_data_entry(self, data_id: HashID) -> Optional[DataEntry]:
        state_msg: Dict = {"get_data_i_d": {"data_id": data_id}}
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        if json_res["data_entry"]:
            return DataEntry(
                pubkey=b64decode(json_res["data_entry"]["delegator_pubkey"]),
            )
        return None

    def get_contract_state(self) -> ContractState:
        state_msg: Dict = {"get_contract_state": {}}
        json_res = self._send_query(state_msg)
        return ContractState(admin=json_res["admin"],
                             threshold=json_res["threshold"],
                             n_max_proxies=json_res["n_max_proxies"],
                             )

    def get_staking_config(self) -> StakingConfig:
        state_msg: Dict = {"get_staking_config": {}}
        json_res = self._send_query(state_msg)
        return StakingConfig(stake_denom=json_res["stake_denom"],
                             minimum_proxy_stake_amount=json_res["minimum_proxy_stake_amount"],
                             minimum_request_reward_amount=json_res["minimum_request_reward_amount"],
                             )

    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        state_msg: Dict = {
            "get_next_proxy_task": {"proxy_pubkey": encode_bytes(proxy_pubkey_bytes)}
        }
        json_res = self._send_query(state_msg)

        if json_res["proxy_task"]:
            return ProxyTask(
                hash_id=json_res["proxy_task"]["data_id"],
                delegatee_pubkey=b64decode(json_res["proxy_task"]["delegatee_pubkey"]),
                delegator_pubkey=b64decode(json_res["proxy_task"]["delegator_pubkey"]),
                delegation_string=b64decode(
                    json_res["proxy_task"]["delegation_string"]
                ),
            )
        return None

    def get_fragments_response(
            self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> GetFragmentsResponse:
        state_msg: Dict = {
            "get_fragments": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        json_res = self._send_query(state_msg)

        return GetFragmentsResponse(
            reencryption_request_state=ReencryptionRequestState[
                json_res["reencryption_request_state"]
            ],
            fragments=json_res["fragments"],
            threshold=json_res["threshold"],
        )

    def get_delegation_status(
            self,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
    ) -> DelegationStatus:
        state_msg: Dict = {
            "get_delegation_status": {
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
            }
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return DelegationStatus(delegation_state=DelegationState[json_res["delegation_state"]],
                                minimum_request_reward=Coin(
                                    denom=str(json_res["minimum_request_reward"]["denom"]),
                                    amount=str(json_res["minimum_request_reward"]["amount"]))
                                )

    def get_proxy_status(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyStatus]:
        state_msg: Dict = {
            "get_proxy_status": {"proxy_pubkey": encode_bytes(proxy_pubkey_bytes)}
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)

        if json_res["proxy_status"]:
            return ProxyStatus(
                proxy_address=json_res["proxy_status"]["proxy_address"],
                stake_amount=json_res["proxy_status"]["stake_amount"],
                withdrawable_stake_amount=json_res["proxy_status"]["withdrawable_stake_amount"],
                proxy_state=ProxyState[json_res["proxy_status"]["proxy_state"]],
            )
        return None


class AdminContract(AbstractAdminContract, ContractExecuteExceptionMixIn):
    CONTRACT_WASM_FILE = CONTRACT_WASM_FILE
    ledger: CosmosLedger

    @classmethod
    def instantiate_contract(
            cls,
            ledger: CosmosLedger,
            admin_private_key: AbstractLedgerCrypto,
            admin_addr: Address,
            stake_denom: str,
            minimum_proxy_stake_amount: Optional[str] = None,
            minimum_request_reward_amount: Optional[str] = None,
            per_request_slash_stake_amount: Optional[str] = None,
            threshold: Optional[int] = None,
            n_max_proxies: Optional[int] = None,
            proxies: Optional[List[Address]] = None,
            label: str = "PRE",
    ) -> Address:
        _logger.info("Deploying contract")
        code_id, res = ledger.deploy_contract(admin_private_key, cls.CONTRACT_WASM_FILE)

        if not isinstance(code_id, int):  # pragma: nocover
            cls._exception_from_res(1, res)

        _logger.info("Initializing deployed contract")
        init_msg: Dict = {
            "admin": admin_addr,
            "stake_denom": stake_denom,
            "proxies": proxies or [],
        }
        if threshold is not None:
            init_msg["threshold"] = threshold

        if minimum_proxy_stake_amount is not None:
            init_msg["minimum_proxy_stake_amount"] = minimum_proxy_stake_amount

        if minimum_request_reward_amount is not None:
            init_msg["minimum_request_reward_amount"] = minimum_request_reward_amount

        if per_request_slash_stake_amount is not None:
            init_msg["per_request_slash_stake_amount"] = per_request_slash_stake_amount

        if n_max_proxies is not None:
            init_msg["n_max_proxies"] = n_max_proxies

        try:
            contract_address, res = ledger.send_init_msg(
                admin_private_key, code_id, init_msg, label
            )
        except BroadcastException as e:
            if (
                    "Error parsing into type cw_proxy_reencryption::msg::InstantiateMsg"
                    in str(e)
            ):
                raise ContractInstantiateFailure(str(e)) from e
            raise
        return Address(contract_address)

    def add_proxy(self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address):
        submit_msg = {"add_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def remove_proxy(
            self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        submit_msg = {"remove_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)


class DelegatorContract(AbstractDelegatorContract, ContractExecuteExceptionMixIn):
    ledger: CosmosLedger

    def add_data(
            self,
            delegator_private_key: AbstractLedgerCrypto,
            delegator_pubkey_bytes: bytes,
            hash_id: HashID,
    ):
        submit_msg = {
            "add_data": {
                "data_id": str(hash_id),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def add_delegations(
            self,
            delegator_private_key: AbstractLedgerCrypto,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
            delegations: List[Delegation],
    ):
        submit_msg = {
            "add_delegation": {
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "proxy_delegations": [
                    {
                        "proxy_pubkey": encode_bytes(i.proxy_pub_key),
                        "delegation_string": encode_bytes(i.delegation_string),
                    }
                    for i in delegations
                ],
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def get_delegation_status(
            self,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
    ) -> DelegationStatus:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_delegation_status(delegator_pubkey_bytes, delegatee_pubkey_bytes)

    def get_selected_proxies_for_delegation(
            self,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_selected_proxies_for_delegation(
            delegator_pubkey_bytes, delegatee_pubkey_bytes
        )

    def request_proxies_for_delegation(
            self,
            delegator_private_key: AbstractLedgerCrypto,
            delegator_pubkey_bytes: bytes,
            delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        submit_msg = {
            "request_proxies_for_delegation": {
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

        # FIXME(LR) parse `res`` instead
        return self.get_selected_proxies_for_delegation(
            delegator_pubkey_bytes,
            delegatee_pubkey_bytes,
        )

    def request_reencryption(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        stake_amount: Coin
    ):
        submit_msg = {
            "request_reencryption": {
                "delegator_public_key": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "data_id": str(hash_id),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key,
            self.contract_address,
            submit_msg,
            amount=[stake_amount],
        )
        self._exception_from_res(error_code, res)

    def get_avaiable_proxies(self) -> List[bytes]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_avaiable_proxies()


class ProxyContract(AbstractProxyContract, ContractExecuteExceptionMixIn):
    ledger: CosmosLedger

    def proxy_register(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        proxy_pubkey_bytes: bytes,
        stake_amount: Coin
    ):
        submit_msg = {
            "register_proxy": {
                "proxy_pubkey": encode_bytes(proxy_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg, amount=[stake_amount]
        )
        self._exception_from_res(error_code, res)

    def proxy_unregister(
            self,
            proxy_private_key: AbstractLedgerCrypto,
    ):
        submit_msg: Dict = {"unregister_proxy": {}}
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_next_proxy_task(proxy_pubkey_bytes)

    def provide_reencrypted_fragment(
            self,
            proxy_private_key: AbstractLedgerCrypto,
            hash_id: HashID,
            delegatee_pubkey_bytes: bytes,
            fragment_hash_id: HashID,
    ):
        submit_msg = {
            "provide_reencrypted_fragment": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "fragment": fragment_hash_id,
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def withdraw_stake(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        stake_amount: Optional[str] = None,
    ):
        if stake_amount is not None:
            submit_msg = {
                "withdraw_stake": {
                    "stake_amount": stake_amount,
                }
            }
        else:
            submit_msg = {"withdraw_stake": {}}

        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def add_stake(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        stake_amount: Coin
    ):
        submit_msg = {
            "add_stake": {}
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg, amount=[stake_amount]
        )
        self._exception_from_res(error_code, res)

    def get_contract_state(self) -> ContractState:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_contract_state()

    def get_staking_config(self) -> StakingConfig:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_staking_config()

    def get_proxy_status(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyStatus]:
        """
        Get proxy status.

        :param proxy_pubkey_bytes: proxy public key as bytes

        :return: None or ProxyStatus instance
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_proxy_status(proxy_pubkey_bytes)


class CosmosContract:  # pylint: disable=too-few-public-methods
    ADMIN_CONTRACT = AdminContract
    DELEGATOR_CONTRACT = DelegatorContract
    QUERIES_CONTRACT = ContractQueries
    PROXY_CONTRACT = ProxyContract
