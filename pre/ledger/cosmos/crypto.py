from pathlib import Path
from typing import Optional

from cosmpy.crypto.address import Address
from cosmpy.crypto.keypairs import PrivateKey

from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.utils.loggers import get_logger


_logger = get_logger(__name__)


class CosmosCrypto(AbstractLedgerCrypto):
    """
    Class that represents user on a blockchain.
    Hold private key to one user and can sign transactions.
    """

    def __init__(
        self,
        private_key: PrivateKey,
        prefix: Optional[str] = None,
        account_number: Optional[int] = None,
    ):
        self.private_key = private_key
        self.prefix = prefix
        self.account_number = account_number

    def get_address(self) -> str:
        return str(Address(self.private_key, prefix=self.prefix))

    def get_pubkey_as_str(self) -> str:
        return self.private_key.public_key

    def get_pubkey_as_bytes(self) -> bytes:
        return self.private_key.public_key_bytes

    def save_key_to_file(self, filename: str):
        Path(filename).write_text(self.as_str())

    def as_str(self) -> str:
        return self.private_key.private_key_hex

    def __bytes__(self) -> bytes:
        return self.private_key.private_key_bytes
