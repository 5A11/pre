from typing import IO, List, Tuple, Union

from pre.common import HashID, PrivateKey, ReencryptionRequestState
from pre.contract.base_contract import AbstractContractQueries
from pre.crypto.base_crypto import AbstractCrypto
from pre.storage.base_storage import AbstractStorage


class DelegateeAPI:
    _contract: AbstractContractQueries

    def __init__(
        self,
        encryption_private_key: PrivateKey,
        contract: AbstractContractQueries,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
        **_  # FIXME rm
    ):
        self._encryption_private_key = encryption_private_key
        self._contract = contract
        self._storage = storage
        self._crypto = crypto

    def is_data_ready(self, hash_id: HashID) -> Tuple[bool, int, List[HashID]]:
        response = self._contract.get_fragments_response(
            hash_id=hash_id,
            delegatee_pubkey_bytes=bytes(self._encryption_private_key.public_key),
        )
        is_ready = (
            response.reencryption_request_state == ReencryptionRequestState.granted
        )
        return is_ready, response.threshold, response.fragments

    def read_data(
        self, hash_id: HashID, delegator_pubkey_bytes: bytes
    ) -> Union[bytes, IO]:
        is_ready, _, fragments_ids = self.is_data_ready(hash_id)

        if not is_ready:  # pragma: nocover
            raise ValueError("Data is not ready!")

        encrypted_fragments = [
            self._storage.get_encrypted_part(i) for i in fragments_ids
        ]
        encrypted_data = self._storage.get_encrypted_data(hash_id)
        data = self._crypto.decrypt(
            encrypted_data=encrypted_data,
            encrypted_data_fragments_bytes=encrypted_fragments,
            delegatee_private_key=self._encryption_private_key,
            delegator_pubkey_bytes=delegator_pubkey_bytes,
        )
        return data
