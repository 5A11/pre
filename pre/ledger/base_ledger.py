from abc import ABC, abstractmethod
from typing import Any, List, Optional

from pre.common import Address


class AbstractLedgerCrypto(ABC):
    """Abstract ledger id crypto class."""

    @abstractmethod
    def get_address(self) -> str:
        """
        Get address.

        :return: address as str
        """

    @abstractmethod
    def get_pubkey_as_str(self) -> str:
        """
        Get public key as string.

        :return: public key as str
        """

    @abstractmethod
    def get_pubkey_as_bytes(self) -> bytes:
        """
        Get public key as bytes.

        :return: public key as bytes
        """

    @abstractmethod
    def save_key_to_file(self, filename: str):
        """
        Save private key to file.

        :param filename: str, path to file to save key
        """

    @abstractmethod
    def as_str(self) -> str:
        """
        Get private key as string.

        :return: str
        """

    @abstractmethod
    def __bytes__(self) -> bytes:
        """
        Get private key as bytes.

        :return: bytes
        """


class AbstractLedger:
    """Abstract ledger class."""

    @abstractmethod
    def sign_tx(self, tx: Any, crypto: AbstractLedgerCrypto) -> Any:
        """
        Sign a transaction.

        :param tx: transaction body to sign
        :param crypto: ledger crypto to sign with

        :return: signed transaction
        """

    @abstractmethod
    def broadcast_tx(self, tx: Any) -> Any:
        """
        Publish transaction over the ledger network.

        :param tx: signed transaction to broadcast.

        :return: broadcast result response
        """

    @abstractmethod
    def generate_tx(
        self,
        packed_msgs: List[Any],
        from_addresses: List[Address],
        pub_keys: List[bytes],
    ) -> Any:
        """
        Generate a transaction.

        :param packed_msgs: list of messages to generate transaction
        :param from_addresses: list of source addresses
        :param pub_keys: list of public keys as bytes
        """

    @abstractmethod
    def get_balance(self, address: Address) -> int:
        """
        Get balance for address.

        :param address: str, address
        :return: current balance
        """

    @abstractmethod
    def load_crypto_from_file(self, keyfile_path: str) -> AbstractLedgerCrypto:
        """
        Load ledger crypto private key from file.

        :param keyfile_path: str, path to file with key data

        :return: private key instance
        """

    @abstractmethod
    def check_availability(self):
        """
        Check ledger host available.

        Raise exception if not available.
        """

    @abstractmethod
    def ensure_funds(self, addresses: List[str], amount: Optional[int] = None):
        """
        Refill funds of addresses using faucet or validator

        :param addresses: Address to be refilled
        :param amount: Amount of refill

        :return: Nothing
        """

    @classmethod
    @abstractmethod
    def validate_address(cls, address: str):
        """Validate address provided: raise exception if not valid."""


class LedgerServerNotAvailable(Exception):
    """Ledger server is not avaiable by address provided."""
