from typing import List

import pytest
from umbral import SecretKey

from pre.common import ReencryptedFragment
from pre.crypto.base_crypto import NotEnoughFragments, WrongDecryptionKey
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey, UmbralPublicKey


def new_umbral_key():
    return UmbralPrivateKey(SecretKey.random())


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

    encrypted_data = crypto.encrypt(data, delegator.public_key)

    delegations = crypto.generate_delegations(
        threshold,
        bytes(delegatee.public_key),
        [bytes(p.public_key) for p in proxies],
        delegator,
    )

    reencrypted_cap_frags: List[ReencryptedFragment] = []
    for i, p in enumerate(proxies):
        reencrypted_cap_frags.append(
            crypto.reencrypt(
                encrypted_data.capsule,
                bytes(delegations[i].delegation_string),
                p,
                bytes(delegator.public_key),
                bytes(delegatee.public_key),
            )
        )

    # delegation does not match proxy key
    with pytest.raises(WrongDecryptionKey):
        crypto.reencrypt(
            encrypted_data.capsule,
            bytes(delegations[1].delegation_string),
            some_key,
            bytes(delegator.public_key),
            bytes(delegatee.public_key),
        )

    # bad delegatee key:
    with pytest.raises(WrongDecryptionKey):
        crypto.decrypt(
            encrypted_data,
            reencrypted_cap_frags[:threshold],
            some_key,
            bytes(delegator.public_key),
        )

    # not enough reencrypted capsules
    with pytest.raises(NotEnoughFragments):
        crypto.decrypt(
            encrypted_data,
            reencrypted_cap_frags[: threshold - 1],
            delegatee,
            bytes(delegator.public_key),
        )

    decrypted_data = crypto.decrypt(
        encrypted_data,
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
