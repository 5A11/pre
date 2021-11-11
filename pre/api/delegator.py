from typing import IO, List, Optional, Tuple, Union, cast

from pre.api.base_api import BaseAPI
from pre.common import Address, Capsule, DataID, PublicKey
from pre.contract.base_contract import AbstractDelegatorContract


class DelegatorAPI(BaseAPI):
    _contract: AbstractDelegatorContract

    def add_data(self, data: Union[bytes, IO]) -> DataID:
        return self._add_data(data)[0]

    def _add_data(self, data: Union[bytes, IO]) -> Tuple[DataID, Capsule]:
        encrypted_data = self._crypto.encrypt(
            data, self._encryption_private_key.public_key
        )
        data_id = self._storage.store_encrypted_data(encrypted_data)
        self._contract.add_data(self._ledger_private_key, data_id)
        return data_id, encrypted_data.capsule

    def get_proxies_list(
        self,
    ) -> List[Tuple[Address, PublicKey]]:
        return self._contract.get_avaiable_proxies()

    def _set_delegation(
        self,
        delegatee_public_key: PublicKey,
        proxies_list: List[Tuple[Address, PublicKey]],
        capsule: Capsule,
    ):
        delegations = self._crypto.generate_delegations(
            capsule, delegatee_public_key, proxies_list
        )
        self._contract.add_delegation(
            self._ledger_private_key, delegatee_public_key, delegations
        )

    def grant_access(
        self,
        data_id: DataID,
        delegatee_public_key: PublicKey,
        proxies_list: Optional[List[Tuple[Address, PublicKey]]] = None,
        capsule: Optional[Capsule] = None,
    ):
        if not self._contract.does_delegation_exist(
            self._ledger_private_key, delegatee_public_key
        ):
            if not capsule:
                capsule = self._storage.get_capsule(data_id)
            if not proxies_list:
                proxies_list = cast(
                    List[Tuple[Address, PublicKey]], self.get_proxies_list()
                )
            self._set_delegation(delegatee_public_key, proxies_list, capsule)

        self._contract.request_reencryption(
            self._ledger_private_key, data_id, delegatee_public_key
        )

    def add_data_and_grant(
        self,
        data: Union[bytes, IO],
        delegatee_public_key: PublicKey,
        proxies_list: Optional[List[Tuple[Address, PublicKey]]] = None,
    ):
        data_id, capsule = self._add_data(data)
        self.grant_access(data_id, delegatee_public_key, proxies_list, capsule=capsule)
