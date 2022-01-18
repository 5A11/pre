from abc import ABC, abstractmethod
from typing import IO, Union

from pre.common import EncryptedData, HashID


class AbstractStorage(ABC):
    """Abstract storage class."""

    @abstractmethod
    def store_encrypted_data(self, encrypted_data: EncryptedData) -> HashID:
        """
        Store encrypted data container to storage and return hash_id of the container.

        :param encrypted_data: EncryptedData instance
        :return: str, hash_id
        """

    @abstractmethod
    def get_data(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        """
        Get data of encrypted data by container hash id.

        :param hash_id:, str, hash_id of the encrypted data stored
        :param stream: bool, return as IO stream if True else return bytes

        :return: bytes or IO stream
        """

    @abstractmethod
    def get_encrypted_data(
        self, hash_id: HashID, stream: bool = False
    ) -> EncryptedData:
        """
        Get encrypted data by container hash id.

        :param hash_id:, str, hash_id of the encrypted data stored
        :param stream: bool, return as IO stream if True else return bytes

        :return: EncryptedData instance
        """

    @abstractmethod
    def connect(self):
        """Connect storage."""

    @abstractmethod
    def disconnect(self):
        """Disconnect storage."""


class StorageError(Exception):
    """Generic storage error."""


class StorageNotConnected(StorageError):
    """Storage not connected error."""


class StorageNetworkError(StorageError):
    """Network relateed storage error."""


class StorageTimeout(StorageError):
    """Storage timeout exception."""


class ServerUnreachable(StorageNetworkError):
    """Server is not reachable exception."""


class StorageNotAvailable(StorageNetworkError):
    """Storage server not available exception."""
