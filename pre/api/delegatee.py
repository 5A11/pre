from typing import IO, List, Tuple, Union

from pre.common import HashID, PrivateKey, ReencryptionRequestState
from pre.contract.base_contract import AbstractContractQueries
from pre.crypto.base_crypto import AbstractCrypto
from pre.storage.base_storage import AbstractStorage


class DelegateeAPI:
    """Delegatee API to read reencrypted data provided."""

    _contract: AbstractContractQueries

    def __init__(
        self,
        encryption_private_key: PrivateKey,
        contract: AbstractContractQueries,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
        **_  # FIXME rm
    ):
        """
        Init Delegatee API instance.

        :param encryption_private_key: PrivateKey,
        :param contract: instance of contrsact query implementation,
        :param storage: instance of abstract storage implementation,
        :param crypto: instance of abstract crypto implementation,
        """
        self._encryption_private_key = encryption_private_key
        self._contract = contract
        self._storage = storage
        self._crypto = crypto

    def is_data_ready(self, hash_id: HashID) -> Tuple[bool, int, List[bytes], bytes]:
        """
        Check is data ready to be decrypted.

        :param hash_id: str, data hash id to check
        :return: Tuple[is_ready:bool, threshold: int, reencryption_fragmetns_list:List of hash id, capsule:bytes]
        """
        response = self._contract.get_fragments_response(
            hash_id=hash_id,
            delegatee_pubkey_bytes=bytes(self._encryption_private_key.public_key),
        )
        is_ready = (
            response.reencryption_request_state == ReencryptionRequestState.granted
        )
        return is_ready, response.threshold, response.fragments, response.capsule

    def read_data(
        self, hash_id: HashID, delegator_pubkey_bytes: bytes
    ) -> Union[bytes, IO]:
        """
        Decrypt data ready to be decrypted.

        :param hash_id: str, hash_id of data
        :param delegator_pubkey_bytes: public key of the initial data owner,

        :return: bytes of the data decrypted
        """
        is_ready, _, encrypted_fragments, capsule = self.is_data_ready(hash_id)

        if not is_ready:  # pragma: nocover
            raise ValueError("Data is not ready!")

        encrypted_data = self._storage.get_encrypted_data(hash_id)
        data = self._crypto.decrypt(
            encrypted_data=encrypted_data,
            capsule=capsule,
            encrypted_data_fragments_bytes=encrypted_fragments,
            delegatee_private_key=self._encryption_private_key,
            delegator_pubkey_bytes=delegator_pubkey_bytes,
        )
        return data
