from abc import ABC, abstractmethod
from typing import IO, List, Union

from pre.common import Delegation, EncryptedData, PrivateKey, PublicKey


class AbstractCrypto(ABC):
    @abstractmethod
    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> EncryptedData:
        pass

    @abstractmethod
    def generate_delegations(
        self,
        capsule_bytes: bytes,
        threshold: int,
        delegatee_pubkey_bytes: bytes,
        proxies_pubkeys_bytes: List[bytes],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        pass

    @abstractmethod
    def reencrypt(
        self,
        capsule_bytes: bytes,
        delegation_bytes: bytes,
        proxy_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bytes:
        pass

    @abstractmethod
    def decrypt(
        self,
        encrypted_data: EncryptedData,
        encrypted_data_fragments_bytes: List[bytes],
        delegatee_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
    ) -> Union[bytes, IO]:
        pass

    @classmethod
    @abstractmethod
    def make_new_key(cls) -> PrivateKey:
        pass

    @classmethod
    @abstractmethod
    def load_key(cls, data: bytes) -> PrivateKey:
        pass
