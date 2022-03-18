import typing
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any, Callable, Dict, IO, List, NamedTuple, Optional, Union

from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin


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
    """Interface for component config."""

    @classmethod
    @abstractmethod
    def validate(cls, data: Any) -> Any:
        """
        Validate config and construct a valid one.

        :param data: input config

        :return: cleaned up config
        """

    @classmethod
    @abstractmethod
    def make_default(cls) -> Any:
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
    """Delegation dataclass."""

    proxy_pub_key: bytes
    delegation_string: bytes


class EncryptedData(NamedTuple):
    """Encrypted data dataclass."""

    data: Union[bytes, IO]


ReencryptedFragment = bytes


class ProxyTask:
    """Proxy task data class."""

    def __init__(
        self,
        hash_id: HashID,
        capsule: Capsule,
        delegatee_pubkey: bytes,
        delegator_pubkey: bytes,
        delegation_string: bytes,
    ):
        self.hash_id = hash_id
        self.capsule = capsule
        self.delegatee_pubkey = delegatee_pubkey
        self.delegator_pubkey = delegator_pubkey
        self.delegation_string = delegation_string

    def __str__(self) -> str:  # pragma: nocover
        return f"{self.hash_id}: capsule: {self.capsule.hex()} delegator: {self.delegator_pubkey.hex()} delegatee: {self.delegatee_pubkey.hex()}"


class DelegationState(Enum):
    """Delegation states enumeration."""

    # pylint: disable=invalid-name
    # for compatibility with contract api responses
    non_existing = 1
    waiting_for_delegation_strings = 2
    active = 3
    proxies_are_too_busy = 4


@dataclass
class DelegationStatus:
    """Delegation status data class."""

    delegation_state: DelegationState
    total_request_reward_amount: Coin


class ReencryptionRequestState(Enum):
    """Reencryption request states enumeration."""

    # pylint: disable=invalid-name
    # for compatibility with contract api responses
    inaccessible = 1
    ready = 2
    granted = 3
    abandoned = 4
    timed_out = 5


@dataclass
class GetFragmentsResponse:
    """Get reencypteed fragments response data class."""

    reencryption_request_state: ReencryptionRequestState
    fragments: List[bytes]
    capsule: bytes
    threshold: int


@dataclass
class ContractState:
    """Contract state data class."""

    admin: Address
    threshold: int


@dataclass
class StakingConfig:
    stake_denom: str
    minimum_proxy_stake_amount: str
    per_proxy_request_reward_amount: str


@dataclass
class DataEntry:
    """DataEntry data class."""

    pubkey: bytes


class ProxyState(Enum):
    """Proxy state enumeration."""

    # pylint: disable=invalid-name
    # for compatibility with contract api responses
    authorised = 1
    registered = 2
    leaving = 3


@dataclass
class ProxyStatus:
    """Proxy status data class."""

    proxy_address: Address
    stake_amount: str
    withdrawable_stake_amount: str
    proxy_state: ProxyState


@dataclass
class ProxyAvailability:
    """Proxy availability data class."""

    proxy_pubkey: bytes
    stake_amount: str


def types_from_annotations(func: Callable) -> Dict:
    """Get types annotation for the function."""
    types = {}
    for name, type_ in func.__annotations__.items():
        if typing.get_origin(type_) is typing.Union:  # for optional
            type_ = typing.get_args(type_)
        types[name] = type_

    return types


def filter_data_with_types(data: Dict, types: Dict, allow_extras=True) -> Dict:
    """Filter data according to data types defined."""
    extra_keys = set(data.keys()) - set(types.keys())
    if extra_keys and not allow_extras:  # pragma: nocover
        raise ValueError(f'Extra keys found `{", ".join(extra_keys)}`')

    validated_data = {}

    for name, type_ in types.items():
        if name in data and isinstance(data.get(name), type_):
            validated_data[name] = data.get(name)
    return validated_data


def get_defaults(func: Callable) -> Dict:
    """Get default values for kwargs of the function."""
    varnames = func.__code__.co_varnames
    defaults = func.__defaults__  # type: ignore
    return dict(zip(varnames[-len(defaults) :], defaults))
