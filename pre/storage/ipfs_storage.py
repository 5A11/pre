import contextlib
from io import BytesIO
from typing import Dict, IO, Optional, Union, cast

import ipfshttpclient  # type: ignore
from ipfshttpclient.client import Client  # type: ignore
from ipfshttpclient.exceptions import CommunicationError
from ipfshttpclient.http_common import ReadableStreamWrapper  # type: ignore

from pre.common import (
    AbstractConfig,
    Capsule,
    EncryptedData,
    HashID,
    ReencryptedFragment,
)
from pre.storage.base_storage import (
    AbstractStorage,
    StorageError,
    StorageNetworkError,
    StorageNotAvailable,
    StorageNotConnected,
    StorageTimeout,
)


class IpfsStorageConfig(AbstractConfig):
    @classmethod
    def validate(cls, data: Dict):
        config_dict = cls.make_default()
        config_dict["addr"] = data.get("addr") or config_dict["addr"]
        config_dict["port"] = data.get("port") or config_dict["port"]
        config_dict["timeout"] = data.get("timeout") or config_dict["timeout"]

        if not isinstance(config_dict["addr"], str):
            raise ValueError("ipfs storage parameter `addr` has to be a string")

        if not isinstance(config_dict["port"], int):
            raise ValueError("ipfs storage parameter `port` has to be an integer")

        if not isinstance(config_dict["timeout"], int):
            raise ValueError("ipfs storage parameter `timeout` has to be an integer")

        return config_dict

    @classmethod
    def make_default(cls) -> Dict:
        return {"addr": "localhost", "port": 5001, "timeout": 10}


class IpfsStorage(AbstractStorage):
    CONFIG_CLASS = IpfsStorageConfig

    def __init__(
        self,
        addr: Optional[str] = None,
        port: Optional[int] = None,
        timeout: int = 10,
        storage_config: Optional[Dict] = None,
    ):
        self._storage_config = storage_config or {}
        self._client: Optional[Client] = None
        if addr and port:
            url = "/dns/" + str(addr) + "/tcp/" + str(port) + "/http"
            self._storage_config["addr"] = url
        self.timeout = timeout

    def _check_connected(self):
        if self._client is None:
            raise StorageNotConnected("IPFS storage is not connected! Connect first!")

    @contextlib.contextmanager
    def _wrap_exceptions(self):
        try:
            yield
        except ipfshttpclient.exceptions.TimeoutError as e:
            raise StorageTimeout(e)
        except ipfshttpclient.exceptions.ConnectionError as e:
            raise StorageNetworkError(e.original)
        except ipfshttpclient.exceptions.CommunicationError as e:  # pragma: nocover
            raise StorageError(e)

    def connect(self):
        if self._client is not None:
            raise StorageError("Already connected!")

        try:
            self._client = ipfshttpclient.connect(
                **self._storage_config, timeout=self.timeout
            )
        except CommunicationError:
            raise StorageNotAvailable(
                f"Storage is not avaiable with address: {self._storage_config['addr']}"
            )

    def disconnect(self):
        self._check_connected()
        self._client.close()
        self._client = None

    def _get_object(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        self._check_connected()
        with self._wrap_exceptions():
            return (
                ReadableStreamWrapper(self._client.cat(hash_id, stream=True))  # type: ignore
                if stream
                else self._client.cat(hash_id, stream=False, timeout=self.timeout)  # type: ignore
            )

    def _add_object(self, data: Union[bytes, IO]) -> HashID:
        self._check_connected()
        with self._wrap_exceptions():
            if isinstance(data, bytes):
                data = BytesIO(data)
            res = self._client.add(file=data, timeout=self.timeout)  # type: ignore
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
        return self._get_object(links["data"], stream=stream)

    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> HashID:
        return self._add_object(encrypted_part)

    def get_encrypted_part(self, hash_id: HashID) -> ReencryptedFragment:
        return cast(ReencryptedFragment, self._get_object(hash_id, stream=False))

    def _add_to_container(self, objects: Dict) -> HashID:
        self._check_connected()
        with self._wrap_exceptions():
            res = self._client.object.new()  # type: ignore
            container_id = res["Hash"]
            for name, cid in objects.items():
                res = self._client.object.patch.add_link(container_id, name=name, ref=cid, timeout=self.timeout)  # type: ignore
                container_id = res["Hash"]
            return HashID(container_id)

    def _read_container(self, container_id: HashID) -> Dict[str, HashID]:
        with self._wrap_exceptions():
            return {
                i["Name"]: i["Hash"] for i in self._client.object.links(container_id, timeout=self.timeout)["Links"]  # type: ignore
            }
