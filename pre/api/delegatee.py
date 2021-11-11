from typing import IO, Union

from pre.api.base_api import BaseAPI
from pre.common import DataID
from pre.contract.base_contract import AbstractContractQueries


class Delegatee(BaseAPI):
    _contract: AbstractContractQueries

    def read_data(self, data_id: DataID) -> Union[bytes, IO]:
        threshold, fragments_ids = self._contract.get_fragments_response(
            data_id, self._encryption_private_key.public_key
        )
        if threshold > len(fragments_ids):
            raise ValueError("Data is not ready!")

        encrypted_fragments = [
            self._storage.get_encrypted_part(i) for i in fragments_ids
        ]
        encrypted_data = self._storage.get_encrypted_data(data_id)
        data = self._crypto.decrypt(
            encrypted_data, encrypted_fragments, self._encryption_private_key
        )
        return data
