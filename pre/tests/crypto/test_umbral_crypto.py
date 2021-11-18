from typing import List

from umbral import SecretKey

from pre.common import ReencryptedFragment
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey


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

    crypto = UmbralCrypto()

    encrypted_data = crypto.encrypt(data, delegator.public_key)
    delegations = crypto.generate_delegations(
        encrypted_data.capsule,
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

    decrypted_data = crypto.decrypt(
        encrypted_data,
        reencrypted_cap_frags[:threshold],
        delegatee,
        bytes(delegator.public_key),
    )

    assert (
        decrypted_data == data
    ), f"Original data and decrypted data differs!\n{data}\n{decrypted_data}\n"
