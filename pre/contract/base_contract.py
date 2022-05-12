from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Dict, List, Optional

from pre.common import (
    Address,
    Coin,
    ContractState,
    Delegation,
    DelegationStatus,
    GetFragmentsResponse,
    HashID,
    ProxyAvailability,
    ProxyStatus,
    ProxyTask,
    StakingConfig,
)
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto


class BaseAbstractContract(ABC):
    """Base abstract contract."""

    def __init__(self, ledger: AbstractLedger, contract_address: Address):
        """
        Init the contract.

        :param ledger: ledger instance
        :param contract_address: str, address of contract deployed
        """
        self._contract_address = contract_address
        self.ledger = ledger

    @property
    def contract_address(self) -> Address:
        """Get contract address."""
        if not self._contract_address:
            raise ValueError("Empty contract address!")  # pragma: nocover
        return self._contract_address


@dataclass
class DataEntry:
    """Pubkey container."""

    pubkey: bytes


class AbstractContractQueries(BaseAbstractContract):
    """Interface for contract queries."""

    @abstractmethod
    def get_available_proxies(self) -> List[ProxyAvailability]:
        """
        Get proxies registered with contract.

        :return: list of proxies pubkeys as bytes and stake_amount as Uint128
        """

    @abstractmethod
    def get_contract_state(self) -> ContractState:
        """
        Get contract default parameters.

        :return: ContractState instance
        """

    @abstractmethod
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

    @abstractmethod
    def get_staking_config(self) -> StakingConfig:
        """Get contract staking config."""

    @abstractmethod
    def get_data_entry(self, data_id: HashID) -> Optional[DataEntry]:
        """
        Get data entry.

        :param data_id: str, hash id of the data set on contract

        :return: DataEntry instance or None
        """

    @abstractmethod
    def get_proxy_tasks(self, proxy_pubkey_bytes: bytes) -> List[ProxyTask]:
        """
        Get next proxy task for proxy specified by proxy public key.

        :param proxy_pubkey_bytes: bytes, proxy public key

        :return: ProxyTask instance or None if no tasks left
        """

    @abstractmethod
    def get_fragments_response(
        self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> GetFragmentsResponse:
        """
        Get reencryption fragments for data_id and specific delegatee.

        :param hash_id: str, hash id of the data set on contract
        :param delegatee_pubkey_bytes: Delegator public key as bytes

        :return: GetFragmentsResponse instance
        """


class AbstractAdminContract(BaseAbstractContract, ABC):
    """Interface for admin contract."""

    @classmethod
    @abstractmethod
    def instantiate_contract(
        cls,
        ledger: AbstractLedger,
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
        :param admin_addr: address of contract administator
        :param stake_denom: str,
        :param minimum_proxy_stake_amount: Optional[int]
        :param per_proxy_task_reward_amount: Optional[int] = None
        :param per_task_slash_stake_amount: Optional[int] = None
        :param threshold: int threshold ,
        :param proxies: optional list of proxies addresses,
        :param timeout_height: Timeout height
        :param proxy_whitelisting: Proxy whitelisting
        :param label: str, contract label

        :return: str, deloyed contract address
        """

    @abstractmethod
    def add_proxy(self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address):
        """
        Add proxy to allowed proxies list.

        :param admin_private_key: private ledger key instance
        :param proxy_addres: str

        :return: None
        """

    @abstractmethod
    def remove_proxy(
        self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        """
        Remove proxy from allowed proxies list.

        :param admin_private_key: private ledger key instance
        :param proxy_addres: str

        :return: None
        """

    @abstractmethod
    def terminate_contract(self, admin_private_key: AbstractLedgerCrypto):
        """
        Terminate contract.

        :param admin_private_key: private ledger key instance

        :return: None
        """

    @abstractmethod
    def withdraw_contract(
        self, admin_private_key: AbstractLedgerCrypto, recipient_addr: Address
    ):
        """
        Withdraw all remaining stake from contract after termination.

        :param admin_private_key: private ledger key instance
        :param recipient_addr: str

        :return: None
        """


class AbstractDelegatorContract(BaseAbstractContract, ABC):
    """Interface for delegator contract."""

    @abstractmethod
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

    @abstractmethod
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
        :param deleations: list of Delegation for the proies selected
        """

    @abstractmethod
    def get_delegation_status(
        self,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> DelegationStatus:
        """
        Get state of delegation.

        :param delegator_pubkey_bytes: Delegator public key as bytes
        :param delegatee_pubkey_bytes: Delegatee public key as bytes

        :return: DelegationStatus instance
        """

    @abstractmethod
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

    @abstractmethod
    def get_available_proxies(self) -> List[ProxyAvailability]:
        """
        Get list of proxies registered.

        :return:  List of proxy public keys as bytes and stake_amount as Uint128
        """


class AbstractProxyContract(BaseAbstractContract, ABC):
    """Interface for proxy contract."""

    @abstractmethod
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

    @abstractmethod
    def proxy_unregister(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        """
        Unregister the proxy.

        :param proxy_private_key: Proxy ledger private key
        """

    @abstractmethod
    def proxy_deactivate(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        """
        Deactivate the proxy.

        :param proxy_private_key: Proxy ledger private key
        """

    @abstractmethod
    def get_proxy_tasks(self, proxy_pubkey_bytes: bytes) -> List[ProxyTask]:
        """
        Get next proxy task.

        :param proxy_pubkey_bytes: Proxy public key as bytes
        :return: None or ProxyTask instance
        """

    @abstractmethod
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

    @abstractmethod
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

    @abstractmethod
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

    @abstractmethod
    def add_stake(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        stake_amount: Coin,
    ):
        """Add stake to the proxy."""

    @abstractmethod
    def get_contract_state(self) -> ContractState:
        """
        Get contract constants.

        :return: ContractState instance
        """

    @abstractmethod
    def get_staking_config(self) -> StakingConfig:
        """
        Get contract staking config.

        :return: StakingConfig instance
        """

    @abstractmethod
    def get_proxy_status(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyStatus]:
        """
        Get proxy status.

        :param proxy_pubkey_bytes: proxy public key as bytes

        :return: None or ProxyStatus instance
        """


class ContractExecutionError(Exception):
    """Generic contract execution error."""

    def __init__(
        self, msg: str, resp_code: Optional[int] = None, resp: Optional[Dict] = None
    ):
        """
        Init exception.

        :param msg: str, error message
        :param resp_code: optional int
        :param resp: body of the network request response
        """
        self.msg = msg
        self.resp = resp
        self.resp_code = resp_code
        super().__init__(msg)


class ProxyAlreadyExist(ContractExecutionError):
    """Proxy already exists exception."""


class ProxyNotActive(ContractExecutionError):
    """Proxy already deactivated exception."""


class NotAdminError(ContractExecutionError):
    """Only admin can execute some contract methods."""


class UnknownProxy(ContractExecutionError):
    """Proxy not allowed to be registered exception."""


class ProxyNotRegistered(ContractExecutionError):
    """Proxy not registered exception."""


class ReencryptionAlreadyRequested(ContractExecutionError):
    """Reencryption request already exists exception."""


class DataAlreadyExist(ContractExecutionError):
    """Data wa already registered exception."""


class ContractInstantiateFailure(ContractExecutionError):
    """Error on contract instantiation."""


class ReencryptedCapsuleFragAlreadyProvided(ContractExecutionError):
    """Reencrypted fragment already exists exception."""


class NotOwner(ContractExecutionError):
    """Private key does not belongs to owner of data."""


class DataDoesNotExist(ContractExecutionError):
    """Data was not registered exception."""


class DataEntryDoesNotExist(ContractExecutionError):
    """Data entry was not registered exception."""


class ProxyAlreadyRegistered(ContractExecutionError):
    """Proxy already registered exception."""


class ProxyNotInDelegation(ContractExecutionError):
    """Proxy specified is not in delegation exception."""


class DelegationAlreadyExist(ContractExecutionError):
    """Delegation already exists exception."""


class UnkownReencryptionRequest(ContractExecutionError):
    """Unknown reencryption request exception."""


class ContractQueryError(Exception):
    """Contract query exception."""


class BadContractAddress(ContractExecutionError):
    """Bad contract address specified exception."""


class DelegationAlreadyAdded(ContractExecutionError):
    """Delegation already added."""


class NotEnoughStakeToWithdraw(ContractExecutionError):
    """Not enough stake to withdraw."""


class NotEnoughProxies(ContractExecutionError):
    """More proxies needs to be selected."""


class ProxiesAreTooBusy(ContractExecutionError):
    """Proxies are too busy error."""


class FragmentVerificationFailed(ContractExecutionError):
    """Fragment verification failed."""


class QueryDataEntryDoesNotExist(ContractQueryError):
    """Data entry was not registered exception."""


class ContractTerminated(ContractExecutionError):
    """Contract terminated."""


class ContractNotTerminated(ContractExecutionError):
    """Contract not terminated."""
