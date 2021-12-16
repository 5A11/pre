from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import List, Optional

from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin

from pre.common import (
    Address,
    Delegation,
    ProxyInfo,
    GetFragmentsResponse,
    HashID,
    ProxyTask,
    ContractState,
    GetDelegationStateResponse,
)
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto


class BaseAbstractContract(ABC):
    def __init__(self, ledger: AbstractLedger, contract_address: Address):
        self._contract_address = contract_address
        self.ledger = ledger

    @property
    def contract_address(self):
        if not self._contract_address:
            raise ValueError("Empty contract address!")  # pragma: nocover
        return self._contract_address


@dataclass
class DataEntry:
    pubkey: bytes


class AbstractContractQueries(BaseAbstractContract):
    @abstractmethod
    def get_avaiable_proxies(self) -> List[bytes]:
        """Get proxies registered with contract."""

    @abstractmethod
    def get_contract_state(self) -> ContractState:
        """Get contract default parameters."""

    @abstractmethod
    def get_data_entry(self, data_id: HashID) -> Optional[DataEntry]:
        """Get data entry."""

    @abstractmethod
    def get_selected_proxies_for_delegation(
        self,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        """Get selected proxy for delegation."""

    @abstractmethod
    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        """Get next proxy task."""

    @abstractmethod
    def get_fragments_response(
        self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> GetFragmentsResponse:
        """Get fragments."""


class AbstractAdminContract(BaseAbstractContract, ABC):
    @classmethod
    @abstractmethod
    def instantiate_contract(
        cls,
        ledger: AbstractLedger,
        admin_private_key: AbstractLedgerCrypto,
        admin_addr: Address,
        stake_denom: str,
        minimum_proxy_stake_amount: Optional[str],
        minimum_request_reward_amount: Optional[str],
        threshold: Optional[int],
        n_max_proxies: Optional[int],
        proxies: List[Address],
        label: str = "PRE",
    ) -> Address:
        """Instantiate contract."""

    @abstractmethod
    def add_proxy(self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address):
        """Add proxy."""

    @abstractmethod
    def remove_proxy(
        self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        """Remove proxy."""


class AbstractDelegatorContract(BaseAbstractContract, ABC):
    @abstractmethod
    def add_data(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
    ):
        """Register data in contract."""

    @abstractmethod
    def add_delegations(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
        delegations: List[Delegation],
    ):
        """Add delegations."""

    @abstractmethod
    def get_delegation_state(  # FIXME(LR) duplicate of get_selected_proxies_for_delegation
        self,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> GetDelegationStateResponse:
        """Check delegation exists."""

    @abstractmethod
    def get_selected_proxies_for_delegation(
        self,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        """Get selected proxies for delegation."""

    @abstractmethod
    def request_proxies_for_delegation(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        """Request proxies for delegation."""

    @abstractmethod
    def request_reencryption(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        stake_amount: Coin
    ):
        """Request reencryption for the data with proxies selected for delegation."""

    @abstractmethod
    def get_avaiable_proxies(self) -> List[bytes]:
        """Get list of proxies registered."""


class AbstractProxyContract(BaseAbstractContract, ABC):
    @abstractmethod
    def proxy_register(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        proxy_pubkey_bytes: bytes,
        stake_amount: Coin
    ):
        """Register the proxy."""

    @abstractmethod
    def proxy_unregister(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        """Unregister the proxy."""

    @abstractmethod
    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        """Get next proxy task."""

    @abstractmethod
    def provide_reencrypted_fragment(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        fragment_hash_id: HashID,
    ):
        """Provide reencrypted fragment for specific reencryption request."""

    @abstractmethod
    def withdraw_stake(
        self,
        stake_amount: Optional[str],
    ):
        """Withdraw proxy stake."""

    @abstractmethod
    def get_contract_state(self) -> ContractState:
        """Get contract constants."""

    @abstractmethod
    def get_proxy_info(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyInfo]:
        """Get proxy state."""


class ContractExecutionError(Exception):
    def __init__(self, msg, resp_code=None, resp=None):
        self.msg = msg
        self.resp = resp
        self.resp_code = resp_code
        super().__init__(msg)


class ProxyAlreadyExist(ContractExecutionError):
    """Proxy already exists exception."""


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
