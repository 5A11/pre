from abc import ABC, abstractmethod
from typing import Any, Dict, IO, List, NamedTuple, Optional, Union


Primitive = Union[str, int, bool, float]
_JSONDict = Dict[Any, Any]  # temporary placeholder
_JSONList = List[Any]  # temporary placeholder
_JSONType = Optional[Union[Primitive, _JSONDict, _JSONList]]
# Added Dict[str, _JSONDict] as workaround to not properly resolving recursive types - _JSONDict should be subset of _JSONType
JSONLike = Union[Dict[str, _JSONType], Dict[str, _JSONDict]]

HashID = str
Address = str


Capsule = bytes


class PublicKey(ABC):
    @property
    def address(self) -> Address:
        return ""

    @abstractmethod
    def __bytes__(self) -> bytes:
        pass

    @classmethod
    @abstractmethod
    def from_bytes(cls, data: bytes) -> Any:
        pass


class PrivateKey(ABC):
    @property
    def public_key(self) -> PublicKey:
        pass

    @abstractmethod
    def __bytes__(self) -> bytes:
        pass

    @classmethod
    @abstractmethod
    def from_bytes(cls, data: bytes) -> Any:
        pass


class EncryptionPrivateKey(PrivateKey):
    pass


class LedgerPrivateKey(PrivateKey):
    pass


class Delegation:
    proxy_pub_key: bytes
    delegation_string: bytes

    def __init__(self, proxy_pub_key: bytes, delegation_string: bytes):
        self.proxy_pub_key = proxy_pub_key
        self.delegation_string = delegation_string


class EncryptedData(NamedTuple):
    data: Union[bytes, IO]
    capsule: bytes


ReencryptedFragment = bytes


class ProxyTask:
    def __init__(
        self,
        hash_id: HashID,
        delegatee_pubkey: bytes,
        delegator_pubkey: bytes,
        delegation_string: bytes,
    ):
        self.hash_id = hash_id
        self.delegatee_pubkey = delegatee_pubkey
        self.delegator_pubkey = delegator_pubkey
        self.delegation_string = delegation_string
