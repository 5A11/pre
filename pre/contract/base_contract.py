from abc import ABC, abstractmethod
from typing import List, Optional, Tuple

from pre.common import (Address, Capsule, DataID, Delegation, PrivateKey,
                        ProxyTask, PublicKey, ReencryptedFragment,
                        ReencryptionRequest)
from pre.ledger.base_ledger import AbstractLedger


class BaseAbstractContract(ABC):
    def __init__(self, contract_address: Address, ledger: AbstractLedger):
        self.contract_address = contract_address
        self.ledger = ledger


class AbstractContractQueries(BaseAbstractContract):
    def __init__(self, contract_address: Address, ledger: AbstractLedger):
        self.contract_address = contract_address
        self.ledger = ledger

    @abstractmethod
    def get_avaiable_proxies(self) -> List[Tuple[Address, PublicKey]]:
        pass

    @abstractmethod
    def get_threshold(self) -> int:
        pass

    @abstractmethod
    def get_next_proxy_task(self, proxy_addr: Address) -> ProxyTask:
        pass

    @abstractmethod
    def get_fragments_response(
        self, data_id: DataID, delegatee_pubkey: PublicKey
    ) -> Tuple[int, List[DataID]]:
        pass


class AbstractAdminContract(BaseAbstractContract, ABC):
    @abstractmethod
    def instantiate_contract(
        self,
        admin_private_key: PrivateKey,
        admin_addr: Address,
        threshold: int,
        n_max_proxies: int,
        proxies: List[Address],
    ):
        pass

    @abstractmethod
    def add_proxy(self, admin_private_key: PrivateKey, proxy_addr: Address):
        pass

    @abstractmethod
    def remove_proxy(self, admin_private_key: PrivateKey, proxy_addr: Address):
        pass


class AbstractDelegatorContract(BaseAbstractContract, ABC):
    @abstractmethod
    def add_data(
        self,
        delegator_private_key: PrivateKey,
        data_id: DataID,
    ):
        pass

    @abstractmethod
    def add_delegation(
        self,
        delegator_private_key: PrivateKey,
        delegatee_public_key: PublicKey,
        delegations: List[Delegation],
    ):
        pass

    @abstractmethod
    def does_delegation_exist(
        self,
        delegator_private_key: PrivateKey,
        delegatee_public_key: PublicKey,
    ) -> bool:
        pass

    @abstractmethod
    def request_reencryption(
        self,
        delegator_private_key: PrivateKey,
        data_id: DataID,
        delegatee_public_key: PublicKey,
    ):
        pass

    @abstractmethod
    def get_avaiable_proxies(self) -> List[Tuple[Address, PublicKey]]:
        pass


class AbstractProxyContract(BaseAbstractContract, ABC):
    @abstractmethod
    def proxy_register(
        self,
        proxy_private_key: PrivateKey,
    ):
        pass

    @abstractmethod
    def proxy_unregister(
        self,
        proxy_private_key: PrivateKey,
    ):
        pass

    @abstractmethod
    def get_next_proxy_task(self, proxy_addr: Address) -> ProxyTask:
        pass

    @abstractmethod
    def provide_reencrypted_fragment(
        self,
        proxy_private_key: PrivateKey,
        data_id: DataID,
        delegatee_public_key: PublicKey,
        fragment_data_id: DataID,
    ):
        pass
