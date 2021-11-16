from abc import ABC, abstractmethod
from typing import IO, List, Tuple, Union

from pre.common import (Address, Capsule, Delegation, EncryptedData,
                        PrivateKey, PublicKey, ReencryptedFragment)


class AbstractCrypto(ABC):
    @abstractmethod
    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> EncryptedData:
        pass

    @abstractmethod
    def generate_delegations(
        self,
        capsule: Capsule,
        threshold: int,
        delegatee_public_key: PublicKey,
        proxies_public_keys: List[PublicKey],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        pass

    @abstractmethod
    def reencrypt(
        self,
        capsule: Capsule,
        delegation: Delegation,
        proxy_private_key: PrivateKey,
        delegator_public_key: PublicKey,
        delegatee_public_key: PublicKey,
    ) -> ReencryptedFragment:
        pass

    @abstractmethod
    def decrypt(
        self,
        encrypted_data: EncryptedData,
        encrypted_data_fragments: List[ReencryptedFragment],
        delegatee_private_key: PrivateKey,
        delegator_public_key: PublicKey,
    ) -> Union[bytes, IO]:
        pass
