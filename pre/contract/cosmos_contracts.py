import re
from base64 import b64decode, b64encode
from pathlib import Path
from typing import Dict, List, Optional, cast

from pre.common import (
    Address,
    Coin,
    ContractState,
    Delegation,
    DelegationState,
    DelegationStatus,
    GetFragmentsResponse,
    HashID,
    JSONLike,
    ProxyAvailability,
    ProxyState,
    ProxyStatus,
    ProxyTask,
    ReencryptionRequestState,
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
    ContractNotTerminated,
    ContractQueryError,
    ContractTerminated,
    DataAlreadyExist,
    DataEntry,
    DataEntryDoesNotExist,
    DelegationAlreadyAdded,
    DelegationAlreadyExist,
    FragmentVerificationFailed,
    NotAdminError,
    NotEnoughProxies,
    NotEnoughStakeToWithdraw,
    ProxiesAreTooBusy,
    ProxyAlreadyExist,
    ProxyAlreadyRegistered,
    ProxyNotActive,
    ProxyNotRegistered,
    QueryDataEntryDoesNotExist,
    ReencryptedCapsuleFragAlreadyProvided,
    ReencryptionAlreadyRequested,
    ReencryptionNotPermitted,
    UnknownProxy,
    UnkownReencryptionRequest,
    WalletInsufficientFunds,
    WithdrawalNotPossibleYet,
)
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.ledger.cosmos.ledger import BroadcastException, CosmosLedger
from pre.utils.loggers import get_logger


CONTRACT_WASM_FILE = str(
    Path(__file__).parent.parent.parent / "contract" / "./cw_proxy_reencryption.wasm"
)

_logger = get_logger(__name__)


def encode_bytes(data: bytes) -> str:
    """Encode bytes to string."""
    return b64encode(data).decode("ascii")


class ContractExecuteExceptionMixIn:  # pylint: disable=too-few-public-methods
    """Contract mixin to handle exceptions."""

    @classmethod
    def _exception_from_res(cls, error_code: int, res: Dict):
        """Construct and raise proper exception depends on reponse."""
        if error_code == 0:
            return

        raw_log = ""

        try:
            raw_log = res["txResponse"]["rawLog"]
        except KeyError:  # pragma: nocover
            raw_log = ""

        if "Sender is not a proxy" in raw_log:
            raise UnknownProxy(raw_log, error_code, res)
        if "Proxy already registered" in raw_log:
            raise ProxyAlreadyRegistered(raw_log, error_code, res)
        if re.search("Delegator .* already registered with this pubkey", raw_log):
            raise DelegationAlreadyAdded(raw_log, error_code, res)
        if "is already proxy" in raw_log:
            raise ProxyAlreadyExist(raw_log, error_code, res)
        if "Proxy not registered" in raw_log:
            raise ProxyNotRegistered(raw_log, error_code, res)
        if "Proxy already unregistered" in raw_log:
            raise ProxyNotRegistered(raw_log, error_code, res)
        if "Proxy already deactivated" in raw_log:
            raise ProxyNotActive(raw_log, error_code, res)
        if "Unknown proxy with pubkey" in raw_log:
            raise UnknownProxy(raw_log, error_code, res)
        if "Reencryption already requested" in raw_log:
            raise ReencryptionAlreadyRequested(raw_log, error_code, res)
        if "Fragment already provided" in raw_log:
            raise ReencryptedCapsuleFragAlreadyProvided(raw_log, error_code, res)
        if "Task was already completed" in raw_log:
            raise ReencryptedCapsuleFragAlreadyProvided(raw_log, error_code, res)
        if "Entry with ID hash_id already exist" in raw_log:
            raise DataAlreadyExist(raw_log, error_code, res)
        if "Delegation string was already provided" in raw_log:
            raise DelegationAlreadyExist(raw_log, error_code, res)
        if "Delegation already exists" in raw_log:
            raise DelegationAlreadyExist(raw_log, error_code, res)
        if "This fragment was not requested" in raw_log:
            raise UnkownReencryptionRequest(raw_log, error_code, res)
        if "Task doesn't exist" in raw_log:
            raise UnkownReencryptionRequest(raw_log, error_code, res)
        if "contract: not found" in raw_log:
            raise BadContractAddress(raw_log, error_code, res)
        if re.search("Data entry doesn't exist", raw_log):
            raise DataEntryDoesNotExist(raw_log, error_code, res)
        if re.search("Only admin can execute this method", raw_log):
            raise NotAdminError(raw_log, error_code, res)
        if "Not enough stake to withdraw: execute wasm contract failed" in raw_log:
            raise NotEnoughStakeToWithdraw(raw_log, error_code, res)
        if "Proxies are too busy, try again later" in raw_log:
            raise ProxiesAreTooBusy(raw_log, error_code, res)
        if "Fragment verification failed" in raw_log:
            raise FragmentVerificationFailed(raw_log, error_code, res)
        if "Fragment already provided by other proxy" in raw_log:
            raise FragmentVerificationFailed(raw_log, error_code, res)
        if "Required at least" in raw_log:
            raise NotEnoughProxies(raw_log, error_code, res)
        if "Contract was terminated." in raw_log:
            raise ContractTerminated(raw_log, error_code, res)
        if "Contract not terminated" in raw_log:
            raise ContractNotTerminated(raw_log, error_code, res)
        if "Nothing to withdraw" in raw_log:
            raise NotEnoughStakeToWithdraw(raw_log, error_code, res)
        if "Reencryption is not permitted" in raw_log:
            raise ReencryptionNotPermitted(raw_log, error_code, res)
        if "insufficient funds" in raw_log:
            raise WalletInsufficientFunds(raw_log, error_code, res)
        if "Withdrawal will be possible at height" in raw_log:
            raise WithdrawalNotPossibleYet(raw_log, error_code, res)

        raise ContractExecutionError(
            f"Contract execution failed: {raw_log}", error_code, res
        )  # pragma: nocover


class ContractQueries(AbstractContractQueries):
    """Cosmos contract queries."""

    ledger: CosmosLedger

    def _send_query(self, state_msg: Dict) -> JSONLike:
        """
        Send  query to contract.

        :param state_msg: dict

        :return: dict
        """
        try:
            return self.ledger.send_query_msg(self.contract_address, state_msg)
        except BroadcastException as e:
            if "contract: not found" in str(e):
                raise ContractQueryError(str(e)) from e
            if "Data entry doesn't exist" in str(e):
                raise QueryDataEntryDoesNotExist(e)
            if "Generic error" in str(e):
                raise ContractQueryError(str(e)) from e
            raise

    def get_available_proxies(self) -> List[ProxyAvailability]:
        """
        Get proxies registered with contract.

        :return: list of proxies pubkeys as bytes
        """
        state_msg: Dict = {"get_available_proxies": {}}
        json_res = self._send_query(state_msg)

        return [
            ProxyAvailability(
                proxy_addr=proxy["proxy_addr"],
                proxy_pubkey=b64decode(proxy["proxy_pubkey"]),
                stake_amount=proxy["stake_amount"],
            )
            for proxy in cast(Dict, json_res["proxies"])
        ]

    def get_data_entry(self, data_id: HashID) -> Optional[DataEntry]:
        """
        Get data entry.

        :param data_id: str, hash id of the data set on contract

        :return: DataEntry instance or None
        """
        state_msg: Dict = {"get_data_i_d": {"data_id": data_id}}
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        if json_res["data_entry"]:
            return DataEntry(
                pubkey=b64decode(json_res["data_entry"]["delegator_pubkey"]),
            )
        return None

    def get_contract_state(self) -> ContractState:
        """
        Get contract default parameters.

        :return: ContractState instance
        """
        state_msg: Dict = {"get_contract_state": {}}
        json_res = self._send_query(state_msg)
        return ContractState(
            admin=cast(str, json_res["admin"]),
            threshold=cast(int, json_res["threshold"]),
            terminated=cast(bool, json_res["terminated"]),
            withdrawn=cast(bool, json_res["withdrawn"]),
        )

    def get_staking_config(self) -> StakingConfig:
        """
        Get contract staking config.

        :return: StakingConfig instance
        """
        state_msg: Dict = {"get_staking_config": {}}
        json_res = self._send_query(state_msg)
        return StakingConfig(
            stake_denom=cast(str, json_res["stake_denom"]),
            minimum_proxy_stake_amount=cast(
                str, json_res["minimum_proxy_stake_amount"]
            ),
            per_proxy_task_reward_amount=cast(
                str, json_res["per_proxy_task_reward_amount"]
            ),
        )

    def get_proxy_tasks(self, proxy_address: str) -> List[ProxyTask]:
        """
        Get proxy tasks for proxy specified by proxy public key.

        :param proxy_pubkey_bytes: bytes, proxy public key

        :return: List of ProxyTask
        """
        state_msg: Dict = {"get_proxy_tasks": {"proxy_addr": proxy_address}}
        json_res = self._send_query(state_msg)

        proxy_tasks: Dict = cast(Dict, json_res.get("proxy_tasks"))

        tasks = []
        for proxy_task in proxy_tasks:
            tasks.append(
                ProxyTask(
                    hash_id=cast(str, proxy_task["data_id"]),
                    capsule=b64decode(cast(str, proxy_task["capsule"])),
                    delegatee_pubkey=b64decode(
                        cast(str, proxy_task["delegatee_pubkey"])
                    ),
                    delegator_pubkey=b64decode(
                        cast(str, proxy_task["delegator_pubkey"])
                    ),
                    delegation_string=b64decode(
                        cast(str, proxy_task["delegation_string"])
                    ),
                )
            )
        return tasks

    def get_fragments_response(
        self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> GetFragmentsResponse:
        """
        Get reencryption fragments for data_id and specific delegatee.

        :param hash_id: str, hash id of the data set on contract
        :param delegatee_pubkey_bytes: Delegator public key as bytes

        :return: GetFragmentsResponse instance
        """
        state_msg: Dict = {
            "get_fragments": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        json_res = self._send_query(state_msg)

        return GetFragmentsResponse(
            reencryption_request_state=ReencryptionRequestState[
                cast(str, json_res["reencryption_request_state"])
            ],
            capsule=b64decode(cast(str, json_res["capsule"])),
            fragments=[b64decode(i) for i in cast(List[str], json_res["fragments"])],
            threshold=cast(int, json_res["threshold"]),
        )

    def get_delegation_status(
        self,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> DelegationStatus:
        """
        Get satatus of delegation.

        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param delegatee_pubkey_bytes: Delegatee public key as bytes

        :return: DelegationStatus instance
        """

        state_msg: Dict = {
            "get_delegation_status": {
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
            }
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return DelegationStatus(
            delegation_state=DelegationState[json_res["delegation_state"]],
            total_request_reward_amount=Coin(
                denom=str(json_res["total_request_reward_amount"]["denom"]),
                amount=str(json_res["total_request_reward_amount"]["amount"]),
            ),
        )

    def get_proxy_status(self, proxy_address: str) -> Optional[ProxyStatus]:
        """
        Get proxy status.

        :param proxy_address: proxy address as str

        :return: None or ProxyStatus instance
        """
        state_msg: Dict = {"get_proxy_status": {"proxy_addr": proxy_address}}
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)

        if json_res["proxy_status"]:
            proxy_status = ProxyStatus(
                proxy_pubkey=None,
                stake_amount=json_res["proxy_status"]["stake_amount"],
                withdrawable_stake_amount=json_res["proxy_status"][
                    "withdrawable_stake_amount"
                ],
                proxy_state=ProxyState[json_res["proxy_status"]["proxy_state"]],
            )
            if "proxy_pubkey" in json_res["proxy_status"]:
                proxy_status.proxy_pubkey = json_res["proxy_status"]["proxy_pubkey"]

            return proxy_status
        return None


class AdminContract(AbstractAdminContract, ContractExecuteExceptionMixIn):
    """Cosmos admin contract."""

    CONTRACT_WASM_FILE = CONTRACT_WASM_FILE
    ledger: CosmosLedger

    @classmethod
    def instantiate_contract(
        cls,
        ledger: CosmosLedger,
        admin_private_key: AbstractLedgerCrypto,
        admin_addr: Address,
        stake_denom: str,
        minimum_proxy_stake_amount: Optional[int] = None,
        per_proxy_task_reward_amount: Optional[int] = None,
        per_task_slash_stake_amount: Optional[int] = None,
        threshold: Optional[int] = None,
        proxies: Optional[List[Address]] = None,
        timeout_height: Optional[int] = None,
        proxy_whitelisting: Optional[bool] = None,
        label: str = "PRE",
    ) -> Address:
        """
        Instantiate contract.
        Deploys contract over the ledger.

        :param ledger: ledger instance to perform contract deployment
        :param admin_private_key: private ledger key instance
        :param admin_addr: address of contract administrator
        :param stake_denom: str,
        :param minimum_proxy_stake_amount: Optional[int]
        :param per_proxy_task_reward_amount: Optional[int] = None
        :param per_task_slash_stake_amount: Optional[int] = None
        :param threshold: int threshold ,
        :param proxies: optional list of proxies addresses,
        :param timeout_height: Timeout height
        :param proxy_whitelisting: Proxy whitelisting
        :param label: str, contract label

        :return: str, deployed contract address
        """
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
            init_msg["minimum_proxy_stake_amount"] = str(minimum_proxy_stake_amount)

        if per_proxy_task_reward_amount is not None:
            init_msg["per_proxy_task_reward_amount"] = str(per_proxy_task_reward_amount)

        if per_task_slash_stake_amount is not None:
            init_msg["per_task_slash_stake_amount"] = str(per_task_slash_stake_amount)

        if timeout_height is not None:
            init_msg["timeout_height"] = timeout_height

        if proxy_whitelisting is not None:
            init_msg["proxy_whitelisting"] = proxy_whitelisting

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
        """
        Add proxy to allowed proxies list.

        :param admin_private_key: private ledger key instance
        :param proxy_addres: str

        :return: None
        """
        submit_msg = {"add_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def remove_proxy(
        self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        """
        Remove proxy from allowed proxies list.

        :param admin_private_key: private ledger key instance
        :param proxy_addres: str

        :return: None
        """
        submit_msg = {"remove_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def terminate_contract(self, admin_private_key: AbstractLedgerCrypto):
        """
        Terminate contract.

        :param admin_private_key: private ledger key instance

        :return: None
        """
        submit_msg: Dict = {"terminate_contract": {}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def withdraw_contract(
        self, admin_private_key: AbstractLedgerCrypto, recipient_addr: Address
    ):
        """
        Withdraw all remaining stake from contract after termination.

        :param admin_private_key: private ledger key instance
        :param recipient_addr: str

        :return: None
        """
        submit_msg = {"withdraw_contract": {"recipient_addr": recipient_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)


class DelegatorContract(AbstractDelegatorContract, ContractExecuteExceptionMixIn):
    """Cosmos delegator contract."""

    ledger: CosmosLedger

    def add_data(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        capsule: bytes,
    ):
        """
        Register data in the contract.

        :param delegator_private_key: Delegator ledger private key
        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param hash_id: str, hash_id the encrypteed data published
        :param capsule: Encrypted capsule
        """
        submit_msg = {
            "add_data": {
                "data_id": str(hash_id),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "capsule": encode_bytes(capsule),
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
        """
        Add delegations.

        :param delegator_private_key: Delegator ledger private key
        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param delegatee_pubkey_bytes: Delegatee public key as bytes
        :param delegations: list of Delegation for the proxies selected
        """
        submit_msg = {
            "add_delegation": {
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "proxy_delegations": [
                    {
                        "proxy_addr": i.proxy_address,
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
        """
        Get status of delegation.

        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param delegatee_pubkey_bytes: Delegatee public key as bytes

        :return: DelegationStatus instance
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_delegation_status(delegator_pubkey_bytes, delegatee_pubkey_bytes)

    def request_reencryption(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        stake_amount: Coin,
    ):
        """
        Request reencryption for the data with proxies selected for delegation.
        :param delegator_private_key: Delegator ledger private key
        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param hash_id: str, hash_id the encrypteed data published
        :param delegatee_pubkey_bytes: Delegatee public key as bytes
        :param stake_amount: Coin instance
        """
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
            amount=[stake_amount.to_pb()],
        )
        self._exception_from_res(error_code, res)

    def get_available_proxies(self) -> List[ProxyAvailability]:
        """
        Get list of proxies registered.

        :return:  List of proxy public keys as bytes
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_available_proxies()


class ProxyContract(AbstractProxyContract, ContractExecuteExceptionMixIn):
    """Cosmos proxy contract."""

    ledger: CosmosLedger

    def proxy_register(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        proxy_pubkey_bytes: bytes,
        stake_amount: Optional[Coin] = None,
    ):
        """
        Register the proxy with contract.

        :param proxy_private_key: Proxy ledger private key
        :param proxy_pubkey_bytes: Proxy public key as bytes
        :param stake_amount: Coin instance
        """

        submit_msg = {
            "register_proxy": {
                "proxy_pubkey": encode_bytes(proxy_pubkey_bytes),
            }
        }

        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key,
            self.contract_address,
            submit_msg,
            amount=[stake_amount.to_pb()] if stake_amount is not None else None,
        )
        self._exception_from_res(error_code, res)

    def proxy_deactivate(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        """
        Deactivate the proxy.

        :param proxy_private_key: Proxy ledger private key
        """
        submit_msg: Dict = {"deactivate_proxy": {}}
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def proxy_unregister(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        """
        Unregister the proxy.

        :param proxy_private_key: Proxy ledger private key
        """
        submit_msg: Dict = {"unregister_proxy": {}}
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def get_proxy_tasks(self, proxy_address: str) -> List[ProxyTask]:
        """
        Get next proxy task.

        :param proxy_address: Proxy address as str
        :return: None or ProxyTask instance
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_proxy_tasks(proxy_address)

    def provide_reencrypted_fragment(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        fragment_bytes: bytes,
    ):
        """
        Provide reencrypted fragment for specific reencryption request.

        :param proxy_private_key: Proxy ledger private key
        :param hash_id: str, hash_id the encrypteed data published
        :param delegatee_pubkey_bytes: Delegatee public key as bytes
        :param fragment_bytes: reencrypted fragment
        """
        submit_msg = {
            "provide_reencrypted_fragment": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "fragment": encode_bytes(fragment_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def skip_reencryption_task(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
    ):
        """
        Skip reencryption task.

        :param proxy_private_key: Proxy ledger private key
        :param hash_id: str, hash_id the encrypteed data published
        :param delegatee_pubkey_bytes: Delegatee public key as bytes
        """
        submit_msg = {
            "skip_reencryption_task": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def withdraw_stake(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        stake_amount: Optional[int] = None,
    ):
        """
        Withdraw proxy stake.

        :param proxy_private_key: Proxy ledger private key
        :param stake_amount: Optional str, amount to transfer
        """
        if stake_amount is not None:
            submit_msg = {
                "withdraw_stake": {
                    "stake_amount": str(stake_amount),
                }
            }
        else:
            submit_msg = {"withdraw_stake": {}}

        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        self._exception_from_res(error_code, res)

    def add_stake(self, proxy_private_key: AbstractLedgerCrypto, stake_amount: Coin):
        submit_msg: Dict = {"add_stake": {}}
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key,
            self.contract_address,
            submit_msg,
            amount=[stake_amount.to_pb()],
        )
        self._exception_from_res(error_code, res)

    def get_contract_state(self) -> ContractState:
        """
        Get contract constants.

        :return: ContractState instance
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_contract_state()

    def get_staking_config(self) -> StakingConfig:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_staking_config()

    def get_proxy_status(self, proxy_address: str) -> Optional[ProxyStatus]:
        """
        Get proxy status.

        :param proxy_address: proxy address as str

        :return: None or ProxyStatus instance
        """
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_proxy_status(proxy_address)


class CosmosContract:  # pylint: disable=too-few-public-methods
    """Cosmos contrct class to define all contract classes."""

    _CONTRACT_ADDR_RE = re.compile("^fetch[0-9a-z]{59}$")
    ADMIN_CONTRACT = AdminContract
    DELEGATOR_CONTRACT = DelegatorContract
    QUERIES_CONTRACT = ContractQueries
    PROXY_CONTRACT = ProxyContract

    @classmethod
    def validate_contract_address(cls, address: str):
        """
        Raise exception if address is invalid.

        :param address: str
        """
        if not cls._CONTRACT_ADDR_RE.match(address):
            raise ValueError(
                f"result = self.runner.invoke(Cosmos address {address} is invalid"
            )
