from io import BytesIO
from typing import IO, Dict, Optional, Union, cast

import ipfshttpclient  # type: ignore
from ipfshttpclient.client import Client  # type: ignore
from ipfshttpclient.http_common import ReadableStreamWrapper  # type: ignore

from pre.common import Capsule, DataID, EncryptedData, ReencryptedFragment
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

    def _get_object(self, data_id: DataID, stream: bool = False) -> Union[bytes, IO]:
        self._check_connected()
        return (
            ReadableStreamWrapper(self._client.cat(data_id, stream=True))  # type: ignore
            if stream
            else self._client.cat(data_id, stream=False)  # type: ignore
        )

    def _add_object(self, data: Union[bytes, IO]) -> DataID:
        self._check_connected()
        if isinstance(data, bytes):
            data = BytesIO(data)
        res = self._client.add(file=data)  # type: ignore
        return DataID(res["Hash"])

    def store_encrypted_data(self, encrypted_data: EncryptedData) -> DataID:
        objects = {
            "data": self._add_object(encrypted_data.data),
            "capsule": self._add_object(encrypted_data.capsule.serialize()),
        }
        return self._add_to_container(objects)

    def get_encrypted_data(
        self, data_id: DataID, stream: bool = False
    ) -> EncryptedData:
        links = self._read_container(data_id)
        return EncryptedData(
            data=self._get_object(links["data"], stream=stream),
            capsule=Capsule.deserialize(
                cast(bytes, self._get_object(links["capsule"], stream=False))
            ),
        )

    def get_capsule(self, data_id: DataID) -> Capsule:
        links = self._read_container(data_id)
        object_bytes = cast(bytes, self._get_object(links["capsule"], stream=False))
        return Capsule.deserialize(object_bytes)

    def get_data(self, data_id: DataID, stream: bool = False) -> Union[bytes, IO]:
        links = self._read_container(data_id)
        return self._get_object(links["capsule"], stream=stream)

    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> DataID:
        return self._add_object(encrypted_part.serialize())

    def get_encrypted_part(self, data_id: DataID) -> ReencryptedFragment:
        return ReencryptedFragment.deserialize(
            cast(bytes, self._get_object(data_id, stream=False))
        )

    def _add_to_container(self, objects: Dict) -> DataID:
        self._check_connected()
        res = self._client.object.new()  # type: ignore
        container_id = res["Hash"]
        for name, cid in objects.items():
            res = self._client.object.patch.add_link(container_id, name=name, ref=cid)  # type: ignore
            container_id = res["Hash"]
        return DataID(container_id)

    def _read_container(self, container_id: DataID) -> Dict[str, DataID]:
        return {
            i["Name"]: i["Hash"] for i in self._client.object.links(container_id)["Links"]  # type: ignore
        }
