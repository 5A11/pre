import typing
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any, Callable, Dict, IO, List, NamedTuple, Optional, Union


Primitive = Union[str, int, bool, float]
_JSONDict = Dict[Any, Any]  # temporary placeholder
_JSONList = List[Any]  # temporary placeholder
_JSONType = Optional[Union[Primitive, _JSONDict, _JSONList]]
# Added Dict[str, _JSONDict] as workaround to not properly resolving recursive types - _JSONDict should be subset of _JSONType
JSONLike = Union[Dict[str, _JSONType], Dict[str, _JSONDict]]

HashID = str
Address = str


Capsule = bytes


class AbstractConfig(ABC):
    @classmethod
    @abstractmethod
    def validate(cls, data: Dict) -> Dict:
        """Validate config."""

    @classmethod
    @abstractmethod
    def make_default(cls) -> Dict:
        """Generate default config."""


class PublicKey(ABC):
    """Abstract public key."""

    @abstractmethod
    def __bytes__(self) -> bytes:
        """Get public key bytes."""

    @classmethod
    @abstractmethod
    def from_bytes(cls, data: bytes) -> Any:
        """Make public key from bytes."""


class PrivateKey(ABC):
    """Abstract private key."""

    @property
    @abstractmethod
    def public_key(self) -> PublicKey:
        """Get public key."""

    @abstractmethod
    def __bytes__(self) -> bytes:
        """Get private key bytes."""

    @classmethod
    @abstractmethod
    def from_bytes(cls, data: bytes) -> Any:
        """Make private key from bytes."""


@dataclass
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

    def __str__(self) -> str:  # pragma: nocover
        return f"{self.hash_id}: delegator: {self.delegator_pubkey.hex()} delegatee: {self.delegatee_pubkey.hex()}"


class DelegationState(Enum):
    non_existing = 1
    waiting_for_delegation_strings = 2
    active = 3


class ReencryptionRequestState(Enum):
    inaccessible = 1
    ready = 2
    granted = 3


@dataclass
class GetFragmentsResponse:
    reencryption_request_state: ReencryptionRequestState
    fragments: List[HashID]
    threshold: int


@dataclass
class DataEntry:
    pubkey: bytes
    addr: Address


def types_from_annotations(func: Callable) -> Dict:
    types = {}
    for name, type_ in func.__annotations__.items():
        if typing.get_origin(type_) is typing.Union:  # for optional
            type_ = typing.get_args(type_)
        types[name] = type_

    return types


def filter_data_with_types(data: Dict, types: Dict, allow_extras=True) -> Dict:
    extra_keys = set(data.keys()) - set(types.keys())
    if extra_keys and not allow_extras:  # pragma: nocover
        raise ValueError(f'Extra keys found `{", ".join(extra_keys)}`')

    validated_data = {}

    for name, type_ in types.items():
        if name in data and isinstance(data.get(name), type_):
            validated_data[name] = data.get(name)
    return validated_data


def get_defaults(func: Callable) -> Dict:
    varnames = func.__code__.co_varnames
    defaults = func.__defaults__  # type: ignore
    return dict(zip(varnames[-len(defaults) :], defaults))
