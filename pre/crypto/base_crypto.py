from abc import ABC, abstractmethod
from typing import IO, List, Tuple, Union

from pre.common import Capsule, Delegation, EncryptedData, PrivateKey, PublicKey


class AbstractCrypto(ABC):
    """Abstract crypto class."""

    @abstractmethod
    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> Tuple[EncryptedData, Capsule]:
        """
        Encrypt data with delegatorm public key.

        :param data: bytes or IO stream
        :param delegator_public_key: delegator encryption public key

        :return: EncryptedData instance, Capsule bytes
        """

    @abstractmethod
    def generate_delegations(
        self,
        threshold: int,
        delegatee_pubkey_bytes: bytes,
        proxies_pubkeys_bytes: List[bytes],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        """
        Generate delegations.

        :param threshold: int
        :param delegatee_pubkey_bytes: reader public key in bytes
        :param proxies_pubkeys_bytes: List[bytes], list of proxies public keys in bytes
        :param delegator_private_key:delegator encryption private key

        :return: List of Delegation
        """

    @abstractmethod
    def reencrypt(
        self,
        capsule_bytes: bytes,
        delegation_bytes: bytes,
        proxy_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bytes:
        """
        Reencrypt data using capsule, proxy private key.

        :param capsule_bytes: capsule in bytes
        :param delegation_bytes: delegation in bytes
        :param proxy_private_key: proxy encryption private key
        :param delegator_public_key: delegator encryption public key
        :param delegatee_pubkey_bytes: reader public key in bytes

        :return: bytes representation of reencryption fragment
        """

    @abstractmethod
    def decrypt(
        self,
        encrypted_data: EncryptedData,
        capsule: Capsule,
        encrypted_data_fragments_bytes: List[bytes],
        delegatee_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
    ) -> Union[bytes, IO]:
        """
        Decrypt data using reencryption fragments and private key.

        :param encrypted_data: EncryptedData instance
        :param capsule: Capsule
        :param encrypted_data_fragments_bytes: list of bytes of reencryption fragments
        :param delegatee_private_key: delegatee encryption private key
        :param delegator_pubkey_bytes: delegator encryption public

        :return: bytes of the decrypted data
        """

    @classmethod
    @abstractmethod
    def make_new_key(cls) -> PrivateKey:
        """
        Make new private key.

        :return: new private key instance
        """

    @classmethod
    @abstractmethod
    def load_key(cls, data: bytes) -> PrivateKey:
        """
        Load private key from bytes.

        :param data: bytes of private key

        :return: private key instance
        """


class CryptoError(Exception):
    """Generic crypto error."""


class DecryptionError(CryptoError):
    """Deryption error."""


class IncorrectFormatOfDelegationString(CryptoError):
    """Incorrect format of delegations tring error."""


class NotEnoughFragments(CryptoError):
    """Not enough fragments to decrypt."""
