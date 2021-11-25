from abc import ABC, abstractmethod
from typing import Any

from pre.common import Address


class AbstractLedgerCrypto(ABC):
    @abstractmethod
    def get_address(self) -> str:
        pass

    @abstractmethod
    def get_pubkey_as_str(self) -> str:
        pass

    @abstractmethod
    def get_pubkey_as_bytes(self) -> bytes:
        pass

    @abstractmethod
    def save_key_to_file(self, filename: str):
        pass

    @abstractmethod
    def as_str(self) -> str:
        pass

    @abstractmethod
    def __bytes__(self) -> bytes:
        pass


class AbstractLedger:
    @abstractmethod
    def sign_tx(self, *args, **kwargs) -> Any:
        pass

    @abstractmethod
    def broadcast_tx(self, *args, **kwargs) -> Any:
        pass

    @abstractmethod
    def generate_tx(self, *args, **kwargs) -> Any:
        pass

    @abstractmethod
    def get_balance(self, address: Address, *args, **kwargs) -> Any:
        pass

    @abstractmethod
    def load_crypto_from_file(
        self, keyfile_path: str, **kwargs
    ) -> AbstractLedgerCrypto:
        pass
