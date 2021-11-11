from pre.common import PrivateKey
from pre.contract.base_contract import BaseAbstractContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.storage.base_storage import AbstractStorage


class BaseAPI:
    def __init__(
        self,
        ledger_private_key: PrivateKey,
        encryption_private_key: PrivateKey,
        contract: BaseAbstractContract,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
    ):
        self._ledger_private_key = ledger_private_key
        self._encryption_private_key = encryption_private_key
        self._contract = contract
        self._storage = storage
        self._crypto = crypto
