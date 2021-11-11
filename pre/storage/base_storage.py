from abc import ABC, abstractmethod
from typing import IO, Union

from pre.common import Capsule, DataID, EncryptedData, ReencryptedFragment


class AbstractStorage(ABC):
    @abstractmethod
    def store_encrypted_data(self, encrypted_data: EncryptedData) -> DataID:
        pass

    @abstractmethod
    def get_capsule(self, data_id: DataID) -> Capsule:
        pass

    @abstractmethod
    def get_data(self, data_id: DataID, stream: bool = False) -> Union[bytes, IO]:
        pass

    @abstractmethod
    def get_encrypted_data(
        self, data_id: DataID, stream: bool = False
    ) -> EncryptedData:
        pass

    @abstractmethod
    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> DataID:
        pass

    @abstractmethod
    def get_encrypted_part(self, data_id: DataID) -> ReencryptedFragment:
        pass
