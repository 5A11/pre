import contextlib
from io import BytesIO
from typing import Dict, IO, Optional, Union

import ipfshttpclient  # type: ignore
from ipfshttpclient.client import Client  # type: ignore
from ipfshttpclient.exceptions import CommunicationError
from ipfshttpclient.http_common import ReadableStreamWrapper  # type: ignore

from pre.common import AbstractConfig, EncryptedData, HashID
from pre.storage.base_storage import (
    AbstractStorage,
    StorageError,
    StorageNetworkError,
    StorageNotAvailable,
    StorageNotConnected,
    StorageTimeout,
)


class IpfsStorageConfig(AbstractConfig):
    """Ipfs storage config class."""

    @classmethod
    def validate(cls, data: Dict) -> Dict:
        """
        Validate config and construct a valid one.

        :param data: dictinput config

        :return: cleaned up config as dict
        """
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
        """
        Generate default config.

        :return: dict
        """
        return {"addr": "localhost", "port": 5001, "timeout": 10}


class IpfsStorage(AbstractStorage):
    """IPFS storage implementation."""

    CONFIG_CLASS = IpfsStorageConfig

    def __init__(
        self,
        addr: Optional[str] = None,
        port: Optional[int] = None,
        timeout: int = 10,
        storage_config: Optional[Dict] = None,
    ):
        """
        Init ipfs storage.

        :param addr: optional str
        :param port: option int port number
        :param timeout: int, network operations timeout in seconds
        :param storage_config" dict with kwargs for ipfs client
        """
        self._storage_config = storage_config or {}
        self._client: Optional[Client] = None
        if addr and port:
            url = "/dns/" + str(addr) + "/tcp/" + str(port) + "/http"
            self._storage_config["addr"] = url
        elif "addr" not in self._storage_config:
            raise ValueError("Provide `addr` value in storage_config or addr and port ")
        self.timeout = timeout

    def _check_connected(self):
        """Check storage connected - raise exception if not."""
        if self._client is None:
            raise StorageNotConnected("IPFS storage is not connected! Connect first!")

    @staticmethod
    @contextlib.contextmanager
    def _wrap_exceptions():
        """Wrap exception with context manager to reraise proper exception."""
        try:
            yield
        except ipfshttpclient.exceptions.TimeoutError as e:
            raise StorageTimeout(e) from e
        except ipfshttpclient.exceptions.ConnectionError as e:  # pragma: nocover
            raise StorageNetworkError(e.original) from e
        except ipfshttpclient.exceptions.CommunicationError as e:  # pragma: nocover
            raise StorageError(e) from e

    def connect(self):
        """Connect storage."""
        if self._client is not None:
            raise StorageError("Already connected!")

        try:
            self._client = ipfshttpclient.connect(
                **self._storage_config, timeout=self.timeout
            )
        except CommunicationError as e:
            raise StorageNotAvailable(
                f"Storage is not available with address: {self._storage_config['addr']}"
            ) from e

    def disconnect(self):
        """Disconnect storage."""
        self._check_connected()
        self._client.close()
        self._client = None

    def _get_object(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        """Get object from ipfs client."""
        self._check_connected()
        with self._wrap_exceptions():
            return (
                ReadableStreamWrapper(self._client.cat(hash_id, stream=True))  # type: ignore
                if stream
                else self._client.cat(hash_id, stream=False, timeout=self.timeout)  # type: ignore
            )

    def _add_object(self, data: Union[bytes, IO]) -> HashID:
        """Add object to ipfs."""
        self._check_connected()
        with self._wrap_exceptions():
            if isinstance(data, bytes):
                data = BytesIO(data)
            res = self._client.add(file=data, timeout=self.timeout)  # type: ignore
            return HashID(res["Hash"])

    def store_encrypted_data(self, encrypted_data: EncryptedData) -> HashID:
        """
        Store encrypted data container to storage and return hash_id of the container.

        :param encrypted_data: EncryptedData instance
        :return: hash_id
        """
        objects = {
            "data": self._add_object(encrypted_data.data),
        }
        return self._add_to_container(objects)

    def get_encrypted_data(
        self, hash_id: HashID, stream: bool = False
    ) -> EncryptedData:
        """
        Get encrypted data by container hash id.

        :param hash_id:, str, hash_id of the encrypted data stored
        :param stream: bool, return as IO stream if True else return bytes

        :return: EncryptedData instance
        """
        links = self._read_container(hash_id)
        return EncryptedData(
            data=self._get_object(links["data"], stream=stream),
        )

    def get_data(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        """
        Get data of encrypted data by container hash id.

        :param hash_id:, str, hash_id of the encrypted data stored
        :param stream: bool, return as IO stream if True else return bytes

        :return: bytes or IO stream
        """
        links = self._read_container(hash_id)
        return self._get_object(links["data"], stream=stream)

    def _add_to_container(self, objects: Dict) -> HashID:
        """Add object to one container."""
        self._check_connected()
        with self._wrap_exceptions():
            res = self._client.object.new()  # type: ignore
            container_id = res["Hash"]
            for name, cid in objects.items():
                res = self._client.object.patch.add_link(container_id, name=name, ref=cid, timeout=self.timeout)  # type: ignore
                container_id = res["Hash"]
            return HashID(container_id)

    def _read_container(self, container_id: HashID) -> Dict[str, HashID]:
        """Read content of container."""
        with self._wrap_exceptions():
            return {
                i["Name"]: i["Hash"] for i in self._client.object.links(container_id, timeout=self.timeout)["Links"]  # type: ignore
            }
