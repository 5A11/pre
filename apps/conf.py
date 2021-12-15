from enum import Enum
from functools import update_wrapper
from pathlib import Path
from typing import Callable, Dict, List, cast

import click

from apps.utils import config_option, private_key_file_option
from pre.common import PrivateKey
from pre.contract.base_contract import (
    AbstractAdminContract,
    AbstractContractQueries,
    AbstractDelegatorContract,
    AbstractProxyContract,
)
from pre.contract.cosmos_contracts import CosmosContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.crypto.umbral_crypto import UmbralCrypto
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto
from pre.ledger.cosmos.ledger import CosmosLedger
from pre.storage.base_storage import AbstractStorage
from pre.storage.ipfs_storage import IpfsStorage


class AppConf:
    ctx_key = "app_config"

    class Params(Enum):
        LEDGER_CONFIG = "ledger_config"
        STORAGE_CONFIG = "ipfs_config"
        LEDGER_PRIVATE_KEY = "ledger_private_key"
        ENCRYPTION_PRIVATE_KEY = "encryption_private_key"
        CONTRACT_ADDRESS = "contract_address"
        DO_FUND = "fund"
        THRESHOLD = "threshold"

    CRYPTO_CLASS = UmbralCrypto
    STORAGE_CLASS = IpfsStorage
    LEDGER_CLASS = CosmosLedger
    CONTRACT_CLASS = CosmosContract

    def __init__(self, options: Dict, enabled_options: List[str]):
        self.options = {}
        for o in enabled_options:
            self.options[o] = options.pop(o, None)

    @classmethod
    def opt_ledger_config(cls):
        return (
            config_option(
                "--ledger-config", cls.LEDGER_CLASS.CONFIG_CLASS, required=False
            ),
            cls.Params.LEDGER_CONFIG.value,
        )

    @classmethod
    def opt_storage_config(cls):
        return (
            config_option(
                "--ipfs-config", cls.STORAGE_CLASS.CONFIG_CLASS, required=False
            ),
            cls.Params.STORAGE_CONFIG.value,
        )
        return (
            click.option(
                "--ipfs-config",
                cls.Params.STORAGE_CONFIG.value,
                expose_value=True,
                type=click.Path(
                    exists=True,
                    file_okay=True,
                    dir_okay=False,
                    readable=True,
                    path_type=Path,
                ),
            ),
            cls.Params.STORAGE_CONFIG.value,
        )

    @classmethod
    def opt_ledger_private_key(cls):
        return (
            private_key_file_option(
                "--ledger-private-key", "--lpk", cls.Params.LEDGER_PRIVATE_KEY.value
            ),
            cls.Params.LEDGER_PRIVATE_KEY.value,
        )

    @classmethod
    def opt_encryption_private_key(cls):
        return (
            private_key_file_option(
                "--encryption-private-key",
                "--epk",
                cls.Params.ENCRYPTION_PRIVATE_KEY.value,
            ),
            cls.Params.ENCRYPTION_PRIVATE_KEY.value,
        )

    @classmethod
    def opt_contract_address(cls):
        return (
            click.option(
                "--contract-address",
                cls.Params.CONTRACT_ADDRESS.value,
                type=str,
                required=True,
            ),
            cls.Params.CONTRACT_ADDRESS.value,
        )

    @classmethod
    def opt_do_fund(cls):
        return (
            click.option(
                "--fund",
                cls.Params.DO_FUND.value,
                is_flag=True,
            ),
            cls.Params.DO_FUND.value,
        )

    @classmethod
    def opt_threshold(cls):
        return (
            click.option(
                "--threshold",
                cls.Params.THRESHOLD.value,
                type=int,
                required=False,
            ),
            cls.Params.THRESHOLD.value,
        )

    @classmethod
    def deco(cls, *options, expose_app_config=False):
        def deco_(fn):
            enabled_options = []
            for opt_callable in options:
                option, name = opt_callable()
                enabled_options.append(name)
                fn = option(fn)

            def wrapper(*args, **kwargs):
                app_config = cls(kwargs, enabled_options)
                return fn(*args, app_config=app_config, **kwargs)

            if expose_app_config:
                return update_wrapper(cast(Callable, wrapper), fn)

            return fn

        return deco_

    def _get_option(self, opt_name, check_none=True):
        value = self.options.get(opt_name)
        if value is None:
            raise ValueError(f"{opt_name} was not specified")
        return value

    @property
    def ledger_config(self):
        return self._get_option(self.Params.LEDGER_CONFIG.value)

    @property
    def contract_address(self):
        return self._get_option(self.Params.CONTRACT_ADDRESS.value)

    @property
    def do_fund(self):
        return self._get_option(self.Params.DO_FUND.value)

    @property
    def threshold(self):
        return self._get_option(self.Params.THRESHOLD.value)

    @property
    def storage_config(self):
        return self._get_option(self.Params.STORAGE_CONFIG.value)

    @property
    def encryption_private_key(self):
        return self._get_option(self.Params.ENCRYPTION_PRIVATE_KEY.value)

    @property
    def ledger_private_key(self):
        return self._get_option(self.Params.LEDGER_PRIVATE_KEY.value)

    def get_ledger_instance(self) -> AbstractLedger:
        return self.LEDGER_CLASS(**self.ledger_config)

    def get_storage_instance(self) -> AbstractStorage:
        storage = self.STORAGE_CLASS(**self.storage_config)
        storage.connect()
        return storage

    def get_ledger_crypto(self) -> AbstractLedgerCrypto:
        ledger = self.get_ledger_instance()
        return ledger.load_crypto_from_file(self.ledger_private_key)

    def get_cryto_key(self) -> PrivateKey:
        return self.CRYPTO_CLASS.load_key(
            Path(self.encryption_private_key).read_bytes()
        )

    def get_crypto_instance(self) -> AbstractCrypto:
        return self.CRYPTO_CLASS()

    def _get_contract(self, contract_cls):
        return contract_cls(
            ledger=self.get_ledger_instance(), contract_address=self.contract_address
        )

    def get_admin_contract(self) -> AbstractAdminContract:
        return self._get_contract(self.CONTRACT_CLASS.ADMIN_CONTRACT)

    def get_proxy_contract(self) -> AbstractProxyContract:
        return self._get_contract(self.CONTRACT_CLASS.PROXY_CONTRACT)

    def get_owner_contract(self) -> AbstractDelegatorContract:
        return self._get_contract(self.CONTRACT_CLASS.DELEGATOR_CONTRACT)

    def get_query_contract(self) -> AbstractContractQueries:
        return self._get_contract(self.CONTRACT_CLASS.QUERIES_CONTRACT)


PROG_NAME = "keys"


@click.group(name=PROG_NAME)
def cli():
    """Generate private keys for ledger and encryption."""


@cli.command(name="generate-ledger-key")
@AppConf.deco(AppConf.opt_ledger_config, expose_app_config=True)
def generate_ledger_key(ledger_config: Dict, app_config):
    """Generate private key for ledger."""
    print(app_config.options)


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
