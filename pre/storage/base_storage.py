from abc import ABC, abstractmethod
from typing import IO, Union

from pre.common import Capsule, EncryptedData, HashID, ReencryptedFragment


class AbstractStorage(ABC):
    @abstractmethod
    def store_encrypted_data(self, encrypted_data: EncryptedData) -> HashID:
        pass

    @abstractmethod
    def get_capsule(self, hash_id: HashID) -> Capsule:
        pass

    @abstractmethod
    def get_data(self, hash_id: HashID, stream: bool = False) -> Union[bytes, IO]:
        pass

    @abstractmethod
    def get_encrypted_data(
        self, hash_id: HashID, stream: bool = False
    ) -> EncryptedData:
        pass

    @abstractmethod
    def store_encrypted_part(self, encrypted_part: ReencryptedFragment) -> HashID:
        pass

    @abstractmethod
    def get_encrypted_part(self, hash_id: HashID) -> ReencryptedFragment:
        pass
