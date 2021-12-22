from abc import ABC, abstractmethod
from typing import Any

from pre.common import Address


class AbstractLedgerCrypto(ABC):
    """Abstract ledger id crypto class."""

    @abstractmethod
    def get_address(self) -> str:
        """Get address."""

    @abstractmethod
    def get_pubkey_as_str(self) -> str:
        """Get public key as string."""

    @abstractmethod
    def get_pubkey_as_bytes(self) -> bytes:
        """Get public key as bytes."""

    @abstractmethod
    def save_key_to_file(self, filename: str):
        """Save private key to file."""

    @abstractmethod
    def as_str(self) -> str:
        """Get private key as string."""

    @abstractmethod
    def __bytes__(self) -> bytes:
        """Get private key as bytes."""


class AbstractLedger:
    """Abstract ledger class."""

    @abstractmethod
    def sign_tx(self, *args, **kwargs) -> Any:
        """Sign a transaction."""

    @abstractmethod
    def broadcast_tx(self, *args, **kwargs) -> Any:
        """Publish transaction over the ledger network."""

    @abstractmethod
    def generate_tx(self, *args, **kwargs) -> Any:
        """Generate a transaction."""

    @abstractmethod
    def get_balance(self, address: Address, *args, **kwargs) -> Any:
        """Get balance for address."""

    @abstractmethod
    def load_crypto_from_file(
        self, keyfile_path: str, **kwargs
    ) -> AbstractLedgerCrypto:
        """Load ledger crypto private key from file."""

    @abstractmethod
    def check_availability(self):
        """Check ledger host avaiable."""


class LedgerServerNotAvailable(Exception):
    """Ledger server is not avaiable by address provided."""
