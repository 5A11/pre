from pre.contract.cosmos_contracts import CosmosContract
from pre.crypto.umbral_crypto import UmbralCrypto
from pre.ledger.cosmos.ledger import CosmosLedger
from pre.storage.ipfs_storage import IpfsStorage


CRYPTO_CLASS = UmbralCrypto
STORAGE_CLASS = IpfsStorage
LEDGER_CLASS = CosmosLedger
CONTRACT_CLASS = CosmosContract
