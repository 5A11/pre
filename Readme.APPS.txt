`ipfs_config.yaml`
addr: localhost
port: 5001

`ledger_config.yaml`
chain_id: capricorn-1
denom: atestfet
node_address: http://127.0.0.1:1317
prefix: fetch
validator_pk: bbaef7511f275dc15f47436d14d6d3c92d4d01befea073d23d0c2750a46f6cb3





full workflow:

> ./apps/keys.py generate-crypto-key /tmp/tmpplikfwmn/admin_encryption.key
Private key written to `/tmp/tmpplikfwmn/admin_encryption.key`


> ./apps/keys.py generate-crypto-key /tmp/tmpplikfwmn/delegator_encryption.key
Private key written to `/tmp/tmpplikfwmn/delegator_encryption.key`


> ./apps/keys.py generate-crypto-key /tmp/tmpplikfwmn/proxy_encryption.key
Private key written to `/tmp/tmpplikfwmn/proxy_encryption.key`


> ./apps/keys.py generate-crypto-key /tmp/tmpplikfwmn/delegatee_encryption.key
Private key written to `/tmp/tmpplikfwmn/delegatee_encryption.key`


> ./apps/keys.py generate-ledger-key --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/admin_ledger.key
Private key written to `/tmp/tmpplikfwmn/admin_ledger.key`


> ./apps/keys.py generate-ledger-key --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/delegator_ledger.key
Private key written to `/tmp/tmpplikfwmn/delegator_ledger.key`


> ./apps/keys.py generate-ledger-key --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/proxy_ledger.key
Private key written to `/tmp/tmpplikfwmn/proxy_ledger.key`


> ./apps/keys.py get-ledger-address --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/admin_ledger.key
Ledger address for key /tmp/tmpplikfwmn/admin_ledger.key is fetch1y3j62ak9a9052dh2vd3gl26yn2qk06ljdr6tes


> ./apps/keys.py get-ledger-address --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/proxy_ledger.key
Ledger address for key /tmp/tmpplikfwmn/proxy_ledger.key is fetch1rpwq4hfggqqfffj6yalkwhxdt5609p6ehvpr8j


> ./apps/keys.py get-ledger-address --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml /tmp/tmpplikfwmn/delegator_ledger.key
Ledger address for key /tmp/tmpplikfwmn/delegator_ledger.key is fetch1dnvw0mqzd809g8y0jh3axdhml3yzntdqnnrxhh


> ./apps/admin.py instantiate-contract --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/admin_ledger.key
instantiate contract with options:
 * admin_address: fetch1y3j62ak9a9052dh2vd3gl26yn2qk06ljdr6tes
 * threshold: 1
 * proxies: []

Contract was set succesfully. Contract address is fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr


> ./apps/admin.py add-proxy --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/admin_ledger.key --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr fetch1rpwq4hfggqqfffj6yalkwhxdt5609p6ehvpr8j
Proxy fetch1rpwq4hfggqqfffj6yalkwhxdt5609p6ehvpr8j added


> ./apps/proxy.py register --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/proxy_ledger.key --encryption-private-key /tmp/tmpplikfwmn/proxy_encryption.key --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr
Proxy was registered


> ./apps/owner.py add-data --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/delegator_ledger.key --encryption-private-key /tmp/tmpplikfwmn/delegator_encryption.key --ipfs-config /tmp/tmpplikfwmn/ipfs_config.yaml --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr /tmp/tmpplikfwmn/data.file
Data was settled: hash_id is QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ


> ./apps/keys.py get-encryption-pubkey /tmp/tmpplikfwmn/delegatee_encryption.key
Public key hex for /tmp/tmpplikfwmn/delegatee_encryption.key is 020d6e47307c220898eeec5da6ca2778d3777dc3751e6f3bd4d91a09ff425da8dd


> ./apps/owner.py grant-access --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/delegator_ledger.key --encryption-private-key /tmp/tmpplikfwmn/delegator_encryption.key --ipfs-config /tmp/tmpplikfwmn/ipfs_config.yaml --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ 020d6e47307c220898eeec5da6ca2778d3777dc3751e6f3bd4d91a09ff425da8dd
Access to hash_id QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ granted to 020d6e47307c220898eeec5da6ca2778d3777dc3751e6f3bd4d91a09ff425da8dd


> ./apps/proxy.py run --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --ledger-private-key /tmp/tmpplikfwmn/proxy_ledger.key --encryption-private-key /tmp/tmpplikfwmn/proxy_encryption.key --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr --run-once-and-exit
Proxy was registered already
Got a reencryption task: QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ: delegator: 028dc8d207eb5c990a2adf03e119dc90dc14de59e35c322686c9fd1b4a074d835f delegatee: 020d6e47307c220898eeec5da6ca2778d3777dc3751e6f3bd4d91a09ff425da8dd
Reencryption task processed: QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ: delegator: 028dc8d207eb5c990a2adf03e119dc90dc14de59e35c322686c9fd1b4a074d835f delegatee: 020d6e47307c220898eeec5da6ca2778d3777dc3751e6f3bd4d91a09ff425da8dd
Proxy was unregistered


> ./apps/reader.py get-data-status --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --encryption-private-key /tmp/tmpplikfwmn/delegatee_encryption.key --ipfs-config /tmp/tmpplikfwmn/ipfs_config.yaml --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ
Data QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ is ready!


> ./apps/keys.py get-encryption-pubkey /tmp/tmpplikfwmn/delegator_encryption.key
Public key hex for /tmp/tmpplikfwmn/delegator_encryption.key is 028dc8d207eb5c990a2adf03e119dc90dc14de59e35c322686c9fd1b4a074d835f


> ./apps/reader.py get-data --ledger-config /tmp/tmpplikfwmn/ledger_config.yaml --encryption-private-key /tmp/tmpplikfwmn/delegatee_encryption.key --ipfs-config /tmp/tmpplikfwmn/ipfs_config.yaml --contract-address fetch18vd8fpwxzck93qlwghaj6arh4p7c5n890l3amr QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ 028dc8d207eb5c990a2adf03e119dc90dc14de59e35c322686c9fd1b4a074d835f /tmp/tmpplikfwmn/decrypted.data
Data QmbVaAgEY9v7WTZ9v4mizq42s9ZfeiE3E6MKPBND8pPQXZ decrypted and stored at /tmp/tmpplikfwmn/decrypted.data

