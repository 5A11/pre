from typing import IO, List, NamedTuple, Tuple, Union, cast
from umbral import (
    SecretKey as _SecretKey,
    PublicKey as _PublicKey,
    Signer as _Signer,
    encrypt,
    decrypt_original,
    generate_kfrags,
    Capsule as _Capsule,
    KeyFrag,
    reencrypt,
    VerifiedCapsuleFrag as _VerifiedCapsuleFrag
)
from umbral.capsule_frag import CapsuleFrag
from umbral.pre import decrypt_reencrypted

from pre.common import (
    Address,
    Capsule,
    Delegation,
    EncryptedData,
    PrivateKey,
    PublicKey,
    ReencryptedFragment,
)
from pre.crypto.base_crypto import AbstractCrypto


class UmbralPublicKey(PublicKey):
    def __init__(self, umbral_key: _PublicKey) -> None:
        self._key = umbral_key

    @property
    def _umbral_key(self) -> _PublicKey:
        return self._key


class UmbralPrivateKey(PrivateKey):
    def __init__(self, umbral_key: _SecretKey) -> None:
        self._key = umbral_key

    @property
    def _umbral_key(self) -> _SecretKey:
        return self._key

    @property
    def public_key(self) -> PublicKey:
        return UmbralPublicKey(self._key.public_key())


class UmbralCapsule(Capsule):
    def __init__(self, umbral_capsule: _Capsule) -> None:
        self._capsule = umbral_capsule

    @property
    def _umbral_capsule(self) -> _Capsule:
        return self._capsule


class _Delegation(NamedTuple):
    capsule: _Capsule
    data: bytes


class UmbralDelegation(Delegation):
    def __init__(self, umbral_delegation: _Delegation) -> None:
        self._delegation = umbral_delegation

    @property
    def _umbral_delegation(self) -> _Delegation:
        return self._delegation


class UmbralReencryptedFragment(ReencryptedFragment):
    def __init__(self, umbral_reenc_cap_frag: _VerifiedCapsuleFrag) -> None:
        self._reenc_cap_frag = umbral_reenc_cap_frag

    @property
    def _umbral_reencrypted_cap_frag(self) -> _VerifiedCapsuleFrag:
        return self._reenc_cap_frag


class UmbralCrypto(AbstractCrypto):
    def __init__(self) -> None:
        pass

    def encrypt(
        self, data: Union[bytes, IO], delegator_public_key: PublicKey
    ) -> EncryptedData:
        # FIXME(LR) maybe composition with an enum would be better than inheritence
        umb_public_key = cast(UmbralPublicKey, delegator_public_key)._umbral_key

        capsule, ciphertext = encrypt(umb_public_key, data)
        return EncryptedData(ciphertext, UmbralCapsule(capsule))

    def generate_delegations(
        self,
        _capsule: Capsule,
        threshold: int,
        delegatee_public_key: PublicKey,
        proxies_public_keys: List[PublicKey],
        delegator_private_key: PrivateKey,
    ) -> List[Delegation]:
        # umb_capsule = cast(UmbralCapsule, capsule)
        umb_delegatee_public_key = cast(
            UmbralPublicKey, delegatee_public_key
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
            shares=len(proxies_public_keys),
        )

        delegations: List[Delegation] = []
        for i, kfrag in enumerate(kfrags):
            proxy_public_key = cast(UmbralPublicKey, proxies_public_keys[i])._umbral_key
            encrypted_frag = encrypt(proxy_public_key, bytes(kfrag))
            delegations.append(UmbralDelegation(encrypted_frag))

        return delegations

    def reencrypt(
        self,
        capsule: Capsule,
        delegation: Delegation,
        proxy_private_key: PrivateKey,
        delegator_public_key: PublicKey,
        delegatee_public_key: PublicKey,
    ) -> ReencryptedFragment:
        umb_capsule = cast(UmbralCapsule, capsule)._umbral_capsule
        umb_delegation = cast(UmbralDelegation, delegation)._umbral_delegation
        umb_proxy_private_key = cast(UmbralPrivateKey, proxy_private_key)._umbral_key
        umb_delegator_public_key = cast(
            UmbralPublicKey, delegator_public_key
        )._umbral_key
        umb_delegatee_public_key = cast(
            UmbralPublicKey, delegatee_public_key
        )._umbral_key

        dec_kfrag = decrypt_original(
            umb_proxy_private_key, umb_delegation[0], umb_delegation[1]
        )
        kfrag = KeyFrag.from_bytes(dec_kfrag).verify(
            umb_delegator_public_key, umb_delegator_public_key, umb_delegatee_public_key
        )

        return UmbralReencryptedFragment(reencrypt(capsule=umb_capsule, kfrag=kfrag))

    def decrypt(
        self,
        encrypted_data: EncryptedData,
        encrypted_data_fragments: List[ReencryptedFragment],
        delegatee_private_key: PrivateKey,
        delegator_public_key: PublicKey,
    ) -> Union[bytes, IO]:
        umb_delegator_public_key = cast(
            UmbralPublicKey, delegator_public_key
        )._umbral_key
        umb_delegatee_private_key = cast(
            UmbralPrivateKey, delegatee_private_key
        )._umbral_key
        umb_capsule = cast(UmbralCapsule, encrypted_data.capsule)._umbral_capsule

        cfrags = []
        for i, frag in enumerate(encrypted_data_fragments):
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
