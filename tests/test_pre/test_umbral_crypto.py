from typing import List

import pytest

from pre.common import ReencryptedFragment
from pre.crypto.base_crypto import (
    DecryptionError,
    IncorrectFormatOfDelegationString,
    NotEnoughFragments,
)
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey, UmbralPublicKey


def new_umbral_key():
    return UmbralPrivateKey.random()


def test_encryption_delegation_reencryption_decryption_cycle():
    n_proxies = 10
    threshold = 7
    data = b"Valuable text to reencrypt."

    assert threshold <= n_proxies, "Threshold must be smaller than number of proxies"

    # generate keys
    delegator = new_umbral_key()
    delegatee = new_umbral_key()
    proxies: List[UmbralPrivateKey] = []
    for _ in range(n_proxies):
        proxies.append(new_umbral_key())

    some_key = new_umbral_key()

    crypto = UmbralCrypto()

    encrypted_data, capsule = crypto.encrypt(data, delegator.public_key)

    delegations = crypto.generate_delegations(
        threshold,
        bytes(delegatee.public_key),
        [bytes(p.public_key) for p in proxies],
        [str(addr) for addr in range(len(proxies))],
        delegator,
    )

    reencrypted_cap_frags: List[ReencryptedFragment] = []
    for i, p in enumerate(proxies):
        reencrypted_cap_frags.append(
            crypto.reencrypt(
                capsule,
                bytes(delegations[i].delegation_string),
                p,
                bytes(delegator.public_key),
                bytes(delegatee.public_key),
            )
        )

    # delegation does not match proxy key
    with pytest.raises(DecryptionError, match="Decryption of ciphertext failed"):
        crypto.reencrypt(
            capsule,
            bytes(delegations[1].delegation_string),
            some_key,
            bytes(delegator.public_key),
            bytes(delegatee.public_key),
        )

    with pytest.raises(IncorrectFormatOfDelegationString):
        crypto.reencrypt(
            capsule,
            bytes(b"1" * 98),
            proxies[1],
            bytes(delegator.public_key),
            bytes(delegatee.public_key),
        )

    # bad delegator key:
    with pytest.raises(DecryptionError):
        crypto.decrypt(
            encrypted_data,
            capsule,
            reencrypted_cap_frags[:threshold],
            delegatee,
            bytes(delegatee.public_key),
        )

    # bad delegatee key:
    with pytest.raises(DecryptionError):
        crypto.decrypt(
            encrypted_data,
            capsule,
            reencrypted_cap_frags[:threshold],
            some_key,
            bytes(delegator.public_key),
        )

    # not enough reencrypted capsules
    with pytest.raises(NotEnoughFragments):
        crypto.decrypt(
            encrypted_data,
            capsule,
            reencrypted_cap_frags[: threshold - 1],
            delegatee,
            bytes(delegator.public_key),
        )

    decrypted_data = crypto.decrypt(
        encrypted_data,
        capsule,
        reencrypted_cap_frags[:threshold],
        delegatee,
        bytes(delegator.public_key),
    )

    assert (
        decrypted_data == data
    ), f"Original data and decrypted data differs!\n{data}\n{decrypted_data}\n"


def test_keys_dump_load():
    key = UmbralCrypto.make_new_key()
    restored_key = UmbralCrypto.load_key(bytes(key))
    assert bytes(key.public_key) == bytes(restored_key.public_key)
    pub_key = UmbralPublicKey.from_bytes(bytes(key.public_key))
    assert pub_key
