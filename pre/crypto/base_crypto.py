from abc import ABC, abstractmethod
from typing import IO, List, Union

from pre.common import Delegation, EncryptedData, PrivateKey, PublicKey


class AbstractCrypto(ABC):
    """Abstract crypto class."""

    @abstractmethod
    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> EncryptedData:
        """Encrypt data with delegatorm public key."""

    @abstractmethod
    def generate_delegations(
        self,
        threshold: int,
        delegatee_pubkey_bytes: bytes,
        proxies_pubkeys_bytes: List[bytes],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        """Generate delegations."""

    @abstractmethod
    def reencrypt(
        self,
        capsule_bytes: bytes,
        delegation_bytes: bytes,
        proxy_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bytes:
        """Reencrypt data using capsule, proxy private key."""

    @abstractmethod
    def decrypt(
        self,
        encrypted_data: EncryptedData,
        encrypted_data_fragments_bytes: List[bytes],
        delegatee_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
    ) -> Union[bytes, IO]:
        """Decrypt data using reencryption fragments and private key."""

    @classmethod
    @abstractmethod
    def make_new_key(cls) -> PrivateKey:
        """Make new private key."""

    @classmethod
    @abstractmethod
    def load_key(cls, data: bytes) -> PrivateKey:
        """Load private key from bytes."""


class CryptoError(Exception):
    pass


class WrongDecryptionKey(CryptoError):
    pass


class NotEnoughFragments(CryptoError):
    pass
