import json
from abc import ABC, abstractclassmethod, abstractmethod
from collections import UserDict
from enum import Enum
from typing import IO, Any, NamedTuple, Union

DataID = str
Address = str


class AbstractSerializable(ABC):
    @abstractmethod
    def serialize(self) -> bytes:
        pass

    @abstractclassmethod
    def deserialize(self, data: bytes) -> Any:
        pass


class Capsule(ABC):
    def serialize(self) -> bytes:
        pass

    @classmethod
    def deserialize(cls, data: bytes) -> "Capsule":
        return cls()


class PublicKey(AbstractSerializable):
    @property
    def address(self) -> Address:
        return ""

    def serialize(self) -> bytes:
        return b""

    @classmethod
    def deserialize(cls, data: bytes) -> "PublicKey":
        return PublicKey()


class PrivateKey:
    @property
    def public_key(self) -> PublicKey:
        pass


class EncryptionPrivateKey(PrivateKey):
    pass


class LedgerPrivateKey(PrivateKey):
    pass


class Delegation(UserDict, AbstractSerializable):
    def serialize(self) -> bytes:
        return b""

    @classmethod
    def deserialize(cls, data: bytes) -> "Delegation":
        return Delegation()


class EncryptedData(NamedTuple):
    data: Union[bytes, IO]
    capsule: Capsule


class ReencryptedFragment(AbstractSerializable):
    def serialize(self) -> bytes:
        return b""

    @classmethod
    def deserialize(cls, data: bytes) -> "ReencryptedFragment":
        return ReencryptedFragment()


class ReencryptionRequest(AbstractSerializable):
    data_id: DataID

    def __init__(
        self, data_id: DataID, delegation: Delegation, delegatee_public_key: PublicKey
    ):
        self.data_id = data_id
        self.delegation = delegation
        self.delegatee_public_key = delegatee_public_key

    def serialize(self) -> bytes:
        return json.dumps(
            {
                "data_id": self.data_id,
                "delegation": self.delegation.serialize(),
                "delegatee_public_key": self.delegatee_public_key.serialize(),
            }
        ).encode("utf-8")

    @classmethod
    def deserialize(cls, data: bytes) -> "ReencryptionRequest":
        dict_data = json.loads(data.decode("utf-8"))
        return cls(
            data_id=dict_data["data_id"],
            delegation=Delegation.deserialize(dict_data["delegation"]),
            delegatee_public_key=PublicKey.deserialize(
                dict_data["delegatee_public_key"]
            ),
        )


ProxyTask = ReencryptionRequest
