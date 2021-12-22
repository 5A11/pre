from abc import ABC, abstractmethod
from typing import IO, Union

from pre.common import Capsule, EncryptedData, HashID, ReencryptedFragment


class AbstractStorage(ABC):
    """Abstract storage class."""

    @abstractmethod
    def store_encrypted_data(self, encrypted_data: EncryptedData) -> HashID:
        """Store encrypted data container to storage and return hash_id of the container."""

    @abstractmethod
    def get_capsule(self, hash_id: HashID) -> Capsule:
        """Get capsule of encrypted data by container hash id."""

    @abstractmethod
    def get_data(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        """Get data of encrypted data by container hash id."""

    @abstractmethod
    def get_encrypted_data(
        self, hash_id: HashID, stream: bool = False
    ) -> EncryptedData:
        """Get encrypted data by container hash id."""

    @abstractmethod
    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> HashID:
        """Store reencryption part and return hash id."""

    @abstractmethod
    def get_encrypted_part(self, hash_id: HashID) -> ReencryptedFragment:
        """Get reencryption part by it's hash id."""

    @abstractmethod
    def connect(self):
        pass

    @abstractmethod
    def disconnect(self):
        pass


class StorageError(Exception):
    """Generic storage error."""


class StorageNotConnected(StorageError):
    """Storage not connected error."""


class StorageNetworkError(StorageError):
    """Network relateed storage error."""


class StorageTimeout(StorageError):
    pass


class ServerUnreachable(StorageNetworkError):
    pass


class StorageNotAvailable(StorageNetworkError):
    pass
