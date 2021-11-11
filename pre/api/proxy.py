from typing import Optional

from pre.api.base_api import BaseAPI
from pre.common import ReencryptionRequest
from pre.contract.base_contract import AbstractProxyContract


class Proxy(BaseAPI):
    _contract: AbstractProxyContract

    def register(self):
        self._contract.proxy_register()

    def unregister(self):
        self._contract.proxy_unregister()

    def get_next_reencryption_request(
        self,
    ) -> Optional[ReencryptionRequest]:
        return self._contract.get_next_proxy_task(
            self._ledger_private_key.public_key.address
        )

    def process_reencryption_request(self, reencryption_request: ReencryptionRequest):
        data_id = reencryption_request.data_id
        delegatee_public_key = reencryption_request.delegatee_public_key
        capsule = self._storage.get_capsule(data_id)
        encrypted_fragment = self._crypto.reencrypt(
            capsule, reencryption_request.delegation, self._encryption_private_key
        )
        encryption_fragment_data_id = self._storage.store_encrypted_part(
            encrypted_fragment
        )
        self._contract.provide_reencrypted_fragment(
            self._ledger_private_key,
            data_id,
            delegatee_public_key,
            encryption_fragment_data_id,
        )
