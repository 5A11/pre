from typing import Optional

from pre.common import PrivateKey, ProxyTask
from pre.contract.base_contract import AbstractProxyContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.storage.base_storage import AbstractStorage


class ProxyAPI:
    def __init__(
        self,
        encryption_private_key: PrivateKey,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractProxyContract,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
    ):
        self._encryption_private_key = encryption_private_key
        self._ledger_crypto = ledger_crypto
        self._contract = contract
        self._storage = storage
        self._crypto = crypto

    def _pub_key_as_bytes(self) -> bytes:
        return bytes(self._encryption_private_key.public_key)

    def register(self):
        self._contract.proxy_register(
            proxy_private_key=self._ledger_crypto,
            proxy_pubkey_bytes=self._pub_key_as_bytes(),
        )

    def unregister(self):
        self._contract.proxy_unregister(self._ledger_crypto)

    def get_next_reencryption_request(
        self,
    ) -> Optional[ProxyTask]:
        return self._contract.get_next_proxy_task(self._pub_key_as_bytes())

    def process_reencryption_request(self, proxy_task: ProxyTask):
        hash_id = proxy_task.hash_id
        delegatee_pubkey_bytes = proxy_task.delegatee_pubkey
        capsule = self._storage.get_capsule(hash_id)
        encrypted_fragment = self._crypto.reencrypt(
            capsule_bytes=capsule,
            delegation_bytes=proxy_task.delegation_string,
            proxy_private_key=self._encryption_private_key,
            delegator_pubkey_bytes=proxy_task.delegator_pubkey,
            delegatee_pubkey_bytes=proxy_task.delegatee_pubkey,
        )
        encryption_fragment_hash_id = self._storage.store_encrypted_part(
            encrypted_fragment
        )
        self._contract.provide_reencrypted_fragment(
            proxy_private_key=self._ledger_crypto,
            hash_id=hash_id,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            fragment_hash_id=encryption_fragment_hash_id,
        )
