from io import BytesIO
from typing import Dict, IO, Optional, Union, cast

import ipfshttpclient  # type: ignore
from ipfshttpclient.client import Client  # type: ignore
from ipfshttpclient.http_common import ReadableStreamWrapper  # type: ignore

from pre.common import Capsule, EncryptedData, HashID, ReencryptedFragment
from pre.storage.base_storage import AbstractStorage


class IpfsStorage(AbstractStorage):
    def __init__(self, storage_config: Optional[Dict] = None):
        self._storage_config = storage_config
        self._client: Optional[Client] = None

    def _check_connected(self):
        if self._client is None:
            raise ValueError("IPFS storage is not connected! Connect first!")

    def connect(self):
        if self._client is not None:
            raise ValueError("Already connected!")
        self._client = ipfshttpclient.connect(**self._storage_config or {})

    def disconnect(self):
        self._check_connected()
        self._client.close()
        self._client = None

    def _get_object(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        self._check_connected()
        return (
            ReadableStreamWrapper(self._client.cat(hash_id, stream=True))  # type: ignore
            if stream
            else self._client.cat(hash_id, stream=False)  # type: ignore
        )

    def _add_object(self, data: Union[bytes, IO]) -> HashID:
        self._check_connected()
        if isinstance(data, bytes):
            data = BytesIO(data)
        res = self._client.add(file=data)  # type: ignore
        return HashID(res["Hash"])

    def store_encrypted_data(self, encrypted_data: EncryptedData) -> HashID:
        objects = {
            "data": self._add_object(encrypted_data.data),
            "capsule": self._add_object(encrypted_data.capsule),
        }
        return self._add_to_container(objects)

    def get_encrypted_data(
        self, hash_id: HashID, stream: bool = False
    ) -> EncryptedData:
        links = self._read_container(hash_id)
        return EncryptedData(
            data=self._get_object(links["data"], stream=stream),
            capsule=cast(bytes, self._get_object(links["capsule"], stream=False)),
        )

    def get_capsule(self, hash_id: HashID) -> Capsule:
        links = self._read_container(hash_id)
        object_bytes = cast(bytes, self._get_object(links["capsule"], stream=False))
        return object_bytes

    def get_data(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        links = self._read_container(hash_id)
        return self._get_object(links["capsule"], stream=stream)

    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> HashID:
        return self._add_object(encrypted_part)

    def get_encrypted_part(self, hash_id: HashID) -> ReencryptedFragment:
        return cast(ReencryptedFragment, self._get_object(hash_id, stream=False))

    def _add_to_container(self, objects: Dict) -> HashID:
        self._check_connected()
        res = self._client.object.new()  # type: ignore
        container_id = res["Hash"]
        for name, cid in objects.items():
            res = self._client.object.patch.add_link(container_id, name=name, ref=cid)  # type: ignore
            container_id = res["Hash"]
        return HashID(container_id)

    def _read_container(self, container_id: HashID) -> Dict[str, HashID]:
        return {
            i["Name"]: i["Hash"] for i in self._client.object.links(container_id)["Links"]  # type: ignore
        }
