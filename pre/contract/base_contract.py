from abc import ABC, abstractmethod
from typing import List, Optional, Tuple

from pre.common import Address, Delegation, HashID, ProxyTask
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto


class BaseAbstractContract(ABC):
    def __init__(self, ledger: AbstractLedger, contract_address: Address):
        self._contract_address = contract_address
        self.ledger = ledger

    @property
    def contract_address(self):
        if not self._contract_address:
            raise ValueError("Empty contract address!")
        return self._contract_address


class AbstractContractQueries(BaseAbstractContract):
    @abstractmethod
    def get_avaiable_proxies(self) -> List[bytes]:
        pass

    @abstractmethod
    def get_threshold(self) -> int:
        pass

    @abstractmethod
    def get_selected_proxies_for_delegation(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        pass

    @abstractmethod
    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        pass

    @abstractmethod
    def get_fragments_response(
        self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> Tuple[int, List[HashID]]:
        pass


class AbstractAdminContract(BaseAbstractContract, ABC):
    @classmethod
    @abstractmethod
    def instantiate_contract(
        cls,
        ledger: AbstractLedger,
        admin_private_key: AbstractLedgerCrypto,
        admin_addr: Address,
        threshold: Optional[int],
        n_max_proxies: Optional[int],
        proxies: List[Address],
        label: str = "PRE",
    ) -> Address:
        pass

    @abstractmethod
    def add_proxy(self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address):
        pass

    @abstractmethod
    def remove_proxy(
        self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        pass


class AbstractDelegatorContract(BaseAbstractContract, ABC):
    @abstractmethod
    def add_data(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
    ):
        pass

    @abstractmethod
    def add_delegations(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
        delegations: List[Delegation],
    ):
        pass

    @abstractmethod
    def does_delegation_exist(  # FIXME(LR) duplicate of get_selected_proxies_for_delegation
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bool:
        pass

    @abstractmethod
    def get_selected_proxies_for_delegation(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        pass

    @abstractmethod
    def request_proxies_for_delegation(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        pass

    @abstractmethod
    def request_reencryption(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
    ):
        pass

    @abstractmethod
    def get_avaiable_proxies(self) -> List[bytes]:
        pass


class AbstractProxyContract(BaseAbstractContract, ABC):
    @abstractmethod
    def proxy_register(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        proxy_pubkey_bytes: bytes,
    ):
        pass

    @abstractmethod
    def proxy_unregister(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        pass

    @abstractmethod
    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        pass

    @abstractmethod
    def provide_reencrypted_fragment(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        fragment_hash_id: HashID,
    ):
        pass
