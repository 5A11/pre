from typing import IO, List, Optional, Tuple, Union, cast

from pre.common import Capsule, HashID, PrivateKey
from pre.contract.base_contract import AbstractDelegatorContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.storage.base_storage import AbstractStorage


class DelegatorAPI:
    _contract: AbstractDelegatorContract

    def __init__(
        self,
        encryption_private_key: PrivateKey,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractDelegatorContract,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
    ):
        self._encryption_private_key = encryption_private_key
        self._ledger_crypto = ledger_crypto
        self._contract = contract
        self._storage = storage
        self._crypto = crypto

    def add_data(self, data: Union[bytes, IO]) -> HashID:
        return self._add_data(data)[0]

    def _add_data(self, data: Union[bytes, IO]) -> Tuple[HashID, Capsule]:
        encrypted_data = self._crypto.encrypt(
            data, self._encryption_private_key.public_key
        )
        hash_id = self._storage.store_encrypted_data(encrypted_data)
        self._contract.add_data(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            hash_id=hash_id,
        )
        return hash_id, encrypted_data.capsule

    def get_proxies_list(
        self,
    ) -> List[bytes]:
        return self._contract.get_avaiable_proxies()

    def _set_delegation(
        self,
        delegatee_pubkey_bytes: bytes,
        proxies_list: List[bytes],
        capsule_bytes: bytes,
        threshold: int,
    ):
        if not proxies_list:
            raise ValueError("proxies_list can not be empty")
        delegations = self._crypto.generate_delegations(
            capsule_bytes=capsule_bytes,
            threshold=threshold,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            proxies_pubkeys_bytes=proxies_list,
            delegator_private_key=self._encryption_private_key,
        )
        self._contract.add_delegations(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            delegations=delegations,
        )

    def grant_access(
        self,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        threshold: int,
        proxies_list: Optional[List[bytes]] = None,
        capsule: Optional[Capsule] = None,
    ):
        if not self._contract.does_delegation_exist(
            delegator_addr=self._ledger_crypto.get_address(),
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        ):
            if not capsule:
                capsule = self._storage.get_capsule(hash_id)
            if not proxies_list:
                proxies_list = cast(List[bytes], self.get_proxies_list())
            self._set_delegation(
                delegatee_pubkey_bytes=delegatee_pubkey_bytes,
                proxies_list=proxies_list,
                capsule_bytes=capsule,
                threshold=threshold,
            )
        self._contract.request_reencryption(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            hash_id=hash_id,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        )

    def add_data_and_grant(
        self,
        data: Union[bytes, IO],
        delegatee_pubkey_bytes: bytes,
        threshold: int,
        proxies_list: Optional[List[bytes]] = None,
    ):
        hash_id, capsule = self._add_data(data)
        self.grant_access(
            hash_id=hash_id,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            threshold=threshold,
            proxies_list=proxies_list,
            capsule=capsule,
        )
