from typing import IO, List, Union, cast

from umbral import Capsule as _Capsule
from umbral import KeyFrag
from umbral import PublicKey as _PublicKey
from umbral import SecretKey as _SecretKey
from umbral import Signer as _Signer
from umbral import VerifiedCapsuleFrag as _VerifiedCapsuleFrag
from umbral import decrypt_original, encrypt, generate_kfrags, reencrypt
from umbral.capsule_frag import CapsuleFrag
from umbral.pre import decrypt_reencrypted

from pre.common import Delegation, EncryptedData, PrivateKey, PublicKey
from pre.crypto.base_crypto import AbstractCrypto


class UmbralPublicKey(PublicKey):
    def __init__(self, umbral_key: _PublicKey):
        self._key = umbral_key

    @property
    def _umbral_key(self) -> _PublicKey:
        return self._key

    def __bytes__(self) -> bytes:
        return bytes(self._key)

    @classmethod
    def from_bytes(cls, data: bytes) -> "UmbralPublicKey":
        return cls(_PublicKey.from_bytes(data))


class UmbralPrivateKey(PrivateKey):
    def __init__(self, umbral_key: _SecretKey) -> None:
        self._key = umbral_key

    @property
    def _umbral_key(self) -> _SecretKey:
        return self._key

    @property
    def public_key(self) -> PublicKey:
        return UmbralPublicKey(self._key.public_key())

    def __bytes__(self) -> bytes:
        return bytes(self._key)

    @classmethod
    def from_bytes(cls, data: bytes) -> "UmbralPrivateKey":
        return cls(_SecretKey.from_bytes(data))

    @classmethod
    def random(cls) -> "UmbralPrivateKey":
        return cls(_SecretKey.random())


class UmbralCapsule:
    def __init__(self, umbral_capsule: _Capsule) -> None:
        self._capsule = umbral_capsule

    @property
    def _umbral_capsule(self) -> _Capsule:
        return self._capsule

    def __bytes__(self) -> bytes:
        return bytes(self._capsule)

    @classmethod
    def from_bytes(cls, data: bytes) -> "UmbralCapsule":
        return cls(_Capsule.from_bytes(data))


class _Delegation:
    def __init__(self, capsule: _Capsule, data: bytes):
        self.capsule = capsule
        self.data = data

    def __bytes__(self) -> bytes:
        return bytes(self.capsule) + self.data

    @classmethod
    def from_bytes(cls, data: bytes) -> "_Delegation":
        return cls(
            capsule=_Capsule.from_bytes(data[: _Capsule.serialized_size()]),
            data=data[_Capsule.serialized_size() :],
        )


class UmbralDelegation:
    def __init__(self, umbral_delegation: _Delegation) -> None:
        self._delegation = umbral_delegation

    @property
    def _umbral_delegation(self) -> _Delegation:
        return self._delegation

    def __bytes__(self) -> bytes:
        return self._delegation.__bytes__()

    @classmethod
    def from_bytes(cls, data: bytes) -> "UmbralDelegation":
        return cls(_Delegation.from_bytes(data))


class UmbralReencryptedFragment:
    def __init__(self, umbral_reenc_cap_frag: _VerifiedCapsuleFrag) -> None:
        self._reenc_cap_frag = umbral_reenc_cap_frag

    @property
    def _umbral_reencrypted_cap_frag(self) -> _VerifiedCapsuleFrag:
        return self._reenc_cap_frag

    def __bytes__(self) -> bytes:
        return bytes(self._reenc_cap_frag)

    @classmethod
    def from_bytes(cls, data: bytes) -> "UmbralReencryptedFragment":
        return cls(_VerifiedCapsuleFrag.from_verified_bytes(data))


class UmbralCrypto(AbstractCrypto):
    def __init__(self) -> None:
        pass

    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> EncryptedData:
        # FIXME(LR) maybe composition with an enum would be better than inheritence
        umb_public_key = cast(UmbralPublicKey, delegator_public_key)._umbral_key

        capsule, ciphertext = encrypt(umb_public_key, data)
        return EncryptedData(ciphertext, bytes(UmbralCapsule(capsule)))

    def generate_delegations(
        self,
        capsule_bytes: bytes,
        threshold: int,
        delegatee_pubkey_bytes: bytes,
        proxies_pubkeys_bytes: List[bytes],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        # umb_capsule = cast(UmbralCapsule, capsule)
        umb_delegatee_public_key = UmbralPublicKey.from_bytes(
            delegatee_pubkey_bytes
        )._umbral_key
        umb_delegator_private_key = cast(
            UmbralPrivateKey, delegator_private_key
        )._umbral_key

        kfrags = generate_kfrags(
            delegating_sk=umb_delegator_private_key,
            receiving_pk=umb_delegatee_public_key,
            # FIXME(LR) using same key for delegating and signing
            signer=_Signer(umb_delegator_private_key),
            threshold=threshold,
            shares=len(proxies_pubkeys_bytes),
        )

        delegations: List[Delegation] = []
        for i, kfrag in enumerate(kfrags):
            proxy_public_key = UmbralPublicKey.from_bytes(
                proxies_pubkeys_bytes[i]
            )._umbral_key

            encrypted_frag = encrypt(proxy_public_key, bytes(kfrag))
            delegations.append(
                Delegation(
                    proxy_pub_key=proxies_pubkeys_bytes[i],
                    delegation_string=bytes(
                        UmbralDelegation(_Delegation(*encrypted_frag))
                    ),
                )
            )

        return delegations

    def reencrypt(
        self,
        capsule_bytes: bytes,
        delegation_bytes: bytes,
        proxy_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bytes:
        umb_capsule = UmbralCapsule.from_bytes(capsule_bytes)._umbral_capsule
        umb_delegation = UmbralDelegation.from_bytes(
            delegation_bytes
        )._umbral_delegation
        umb_proxy_private_key = cast(UmbralPrivateKey, proxy_private_key)._umbral_key
        umb_delegator_public_key = UmbralPublicKey.from_bytes(
            delegator_pubkey_bytes
        )._umbral_key
        umb_delegatee_public_key = UmbralPublicKey.from_bytes(
            delegatee_pubkey_bytes
        )._umbral_key

        dec_kfrag = decrypt_original(
            umb_proxy_private_key, umb_delegation.capsule, umb_delegation.data
        )
        kfrag = KeyFrag.from_bytes(dec_kfrag).verify(
            umb_delegator_public_key, umb_delegator_public_key, umb_delegatee_public_key
        )

        return bytes(
            UmbralReencryptedFragment(reencrypt(capsule=umb_capsule, kfrag=kfrag))
        )

    def decrypt(
        self,
        encrypted_data: EncryptedData,
        encrypted_data_fragments_bytes: List[bytes],
        delegatee_private_key: PrivateKey,
        delegator_pubkey_bytes: bytes,
    ) -> Union[bytes, IO]:
        umb_delegator_public_key = UmbralPublicKey.from_bytes(
            delegator_pubkey_bytes
        )._umbral_key
        umb_delegatee_private_key = cast(
            UmbralPrivateKey, delegatee_private_key
        )._umbral_key
        umb_capsule = UmbralCapsule.from_bytes(encrypted_data.capsule)._umbral_capsule

        cfrags = []
        for i, frag_bytes in enumerate(encrypted_data_fragments_bytes):
            frag = UmbralReencryptedFragment.from_bytes(frag_bytes)
            reenc_frag = cast(UmbralReencryptedFragment, frag)._reenc_cap_frag
            cfrag = CapsuleFrag.from_bytes(bytes(reenc_frag))
            cfrag = cfrag.verify(
                umb_capsule,
                umb_delegator_public_key,
                umb_delegator_public_key,
                umb_delegatee_private_key.public_key(),
            )
            cfrags.append(cfrag)

        return decrypt_reencrypted(
            receiving_sk=umb_delegatee_private_key,
            delegating_pk=umb_delegator_public_key,
            capsule=umb_capsule,
            verified_cfrags=cfrags,
            ciphertext=encrypted_data.data,
        )
