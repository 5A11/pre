from typing import List
from umbral import SecretKey
from pre.common import ReencryptedFragment
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey
from base64 import b64encode


# Script used for generating capsule with re-encrypted fragments mainly for testing purposes

def new_umbral_key():
    return UmbralPrivateKey(SecretKey.random())


def reencrypt(delegations, delegator, delegatee):
    reencrypted_cap_frags: List[ReencryptedFragment] = []
    for i, p in enumerate(proxies):
        reencrypted_cap_frags.append(
            crypto.reencrypt(
                capsule1,
                bytes(delegations[i].delegation_string),
                p,
                bytes(delegator.public_key),
                bytes(delegatee.public_key),
            )
        )
    return reencrypted_cap_frags


n_proxies = 3
threshold = 2
data = b"Valuable text to reencrypt."

assert threshold <= n_proxies, "Threshold must be smaller than number of proxies"

# generate keys
delegator1 = UmbralPrivateKey.from_bytes(b"DELEGATOR00000000000000000000001")
delegator2 = UmbralPrivateKey.from_bytes(b"DELEGATOR00000000000000000000002")

delegatee1 = UmbralPrivateKey.from_bytes(b"DELEGATEE00000000000000000000001")
delegatee2 = UmbralPrivateKey.from_bytes(b"DELEGATEE00000000000000000000002")

proxies: List[UmbralPrivateKey] = []
for _ in range(n_proxies):
    proxies.append(new_umbral_key())

crypto = UmbralCrypto()

# Encrypt data
encrypted_data1, capsule1 = crypto.encrypt(data, delegator1.public_key)

# Generate delegation strings
delegations1_1 = crypto.generate_delegations(
    threshold,
    bytes(delegatee1.public_key),
    [bytes(p.public_key) for p in proxies],
    delegator1,
)

delegations1_2 = crypto.generate_delegations(
    threshold,
    bytes(delegatee2.public_key),
    [bytes(p.public_key) for p in proxies],
    delegator1,
)

delegations2_1 = crypto.generate_delegations(
    threshold,
    bytes(delegatee1.public_key),
    [bytes(p.public_key) for p in proxies],
    delegator2,
)

delegations2_2 = crypto.generate_delegations(
    threshold,
    bytes(delegatee2.public_key),
    [bytes(p.public_key) for p in proxies],
    delegator2,
)

# Do reencryption
reencrypted_cap_frags1_1 = reencrypt(delegations1_1, delegator1, delegatee1)
reencrypted_cap_frags1_2 = reencrypt(delegations1_2, delegator1, delegatee2)
reencrypted_cap_frags2_1 = reencrypt(delegations2_1, delegator2, delegatee1)
reencrypted_cap_frags2_2 = reencrypt(delegations2_2, delegator2, delegatee2)

# Print results
print(f'capsule1: {b64encode(capsule1).decode("ascii")}')

print(f'delegator1 pubkey: {b64encode(bytes(delegator1.public_key)).decode("ascii")}')
print(f'delegator2 pubkey: {b64encode(bytes(delegator2.public_key)).decode("ascii")}')

print(f'delegatee1 pubkey: {b64encode(bytes(delegatee1.public_key)).decode("ascii")}')
print(f'delegatee2 pubkey: {b64encode(bytes(delegatee2.public_key)).decode("ascii")}')

for frag in reencrypted_cap_frags1_1:
    print(f'fragment delegator1 to delegatee1: {b64encode(frag).decode("ascii")}')

for frag in reencrypted_cap_frags1_2:
    print(f'fragment delegator1 to delegatee2: {b64encode(frag).decode("ascii")}')

for frag in reencrypted_cap_frags2_1:
    print(f'fragment delegator2 to delegatee1: {b64encode(frag).decode("ascii")}')

for frag in reencrypted_cap_frags2_2:
    print(f'fragment delegator2 to delegatee2: {b64encode(frag).decode("ascii")}')
