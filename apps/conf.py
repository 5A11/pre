from enum import Enum
from functools import update_wrapper
from pathlib import Path
from typing import Any, Callable, Dict, List, cast

import click
import yaml

from apps.utils import file_exists_type, private_key_file_option
from pre.common import AbstractConfig, PrivateKey
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
from pre.ledger.cosmos.ledger import CosmosLedger, DEFAULT_FUNDS_AMOUNT
from pre.storage.base_storage import AbstractStorage
from pre.storage.ipfs_storage import IpfsStorage


class ContractAddressConfig(AbstractConfig):
    @classmethod
    def validate(cls, data: Any) -> Any:
        # validation is made in app config class cause bound to contract class
        return data

    @classmethod
    def make_default(cls) -> Any:
        return None


class ThresholdConfig(AbstractConfig):
    @classmethod
    def validate(cls, data: Any) -> int:
        if not isinstance(data, int):
            raise ValueError("Threshold is not int value!")
        if data <= 0:
            raise ValueError("Threshold is not int  positive value!")
        return int(data)

    @classmethod
    def make_default(cls) -> Any:
        return None


class PrivateKeyConfig(AbstractConfig):
    @classmethod
    def validate(cls, data: Any) -> Path:
        path = Path(data)
        if not path.exists():
            raise ValueError(f"File {path} does not exist")
        return path

    @classmethod
    def make_default(cls) -> Any:
        return None


class AppConf:
    ctx_key = "app_config"

    class Params(Enum):
        LEDGER_CONFIG = "ledger_config"
        STORAGE_CONFIG = "ipfs_config"
        LEDGER_PRIVATE_KEY = "ledger_private_key"
        ENCRYPTION_PRIVATE_KEY = "encryption_private_key"
        CONTRACT_ADDRESS = "contract_address"
        GLOBAL_CONFIG = "config"
        DO_FUND = "fund"
        THRESHOLD = "threshold"

    CRYPTO_CLASS = UmbralCrypto
    STORAGE_CLASS = IpfsStorage
    LEDGER_CLASS = CosmosLedger
    CONTRACT_CLASS = CosmosContract

    @classmethod
    def _make_global_config_class(cls):
        class GlobalConfig(AbstractConfig):
            SECTIONS = {
                cls.Params.LEDGER_CONFIG.value: cls.LEDGER_CLASS.CONFIG_CLASS,
                cls.Params.STORAGE_CONFIG.value: cls.STORAGE_CLASS.CONFIG_CLASS,
                cls.Params.CONTRACT_ADDRESS.value: ContractAddressConfig,
                cls.Params.LEDGER_PRIVATE_KEY.value: PrivateKeyConfig,
                cls.Params.ENCRYPTION_PRIVATE_KEY.value: PrivateKeyConfig,
                cls.Params.THRESHOLD.value: ThresholdConfig,
            }

            @classmethod
            def validate(cls, data: Dict) -> Dict:
                result_data = {}
                for section_name, section_conf_class in cls.SECTIONS.items():
                    if section_name not in data:
                        continue
                    result_data[section_name] = section_conf_class.validate(
                        data[section_name]
                    )
                return result_data

            @classmethod
            def make_default(cls) -> Dict:
                result_data = {}
                for section_name, section_conf_class in cls.SECTIONS.items():
                    default = section_conf_class.make_default()
                    if default is not None:
                        result_data[section_name] = default
                return result_data

        return GlobalConfig

    def __init__(self, options: Dict, enabled_options: List[str]):
        self.options = options.pop(self.Params.GLOBAL_CONFIG.value, None) or {}
        for option_name in enabled_options:
            if option_name is self.Params.GLOBAL_CONFIG.value:
                continue

            self.options[option_name] = options.pop(option_name) or self.options.get(
                option_name
            )

    @classmethod
    def config_option(cls, *args, config_class: AbstractConfig, **options):
        def _load_and_check_config(
            ctx: click.Context, param, value
        ):  # pylint: disable=unused-argument
            if not value:
                return value
            data = cls._load_config_from_file(value)
            return config_class.validate(data)

        def deco(func):
            func = click.option(
                *args,
                callback=_load_and_check_config,
                type=file_exists_type,
                **options,
            )(func)
            return func

        return deco

    @classmethod
    def _load_config_from_file(cls, filename: Path) -> Dict:
        return yaml.safe_load(Path(filename).read_text(encoding="utf-8"))

    @classmethod
    def opt_ledger_config(cls):
        return (
            cls.config_option(
                "--ledger-config",
                cls.Params.LEDGER_CONFIG.value,
                config_class=cls.LEDGER_CLASS.CONFIG_CLASS,
                required=False,
            ),
            cls.Params.LEDGER_CONFIG.value,
        )

    @classmethod
    def opt_storage_config(cls):
        return (
            cls.config_option(
                "--ipfs-config",
                cls.Params.STORAGE_CONFIG.value,
                config_class=cls.STORAGE_CLASS.CONFIG_CLASS,
                required=False,
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
    def opt_global_config(cls):
        return (
            cls.config_option(
                "--config",
                "-c",
                cls.Params.GLOBAL_CONFIG.value,
                config_class=cls._make_global_config_class(),
                required=False,
            ),
            cls.Params.GLOBAL_CONFIG.value,
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
                required=False,
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
                required=True,
                default=False,
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
    def deco(cls, *options, expose_app_config=False, global_config=True):
        options = list(options)
        if global_config:
            options.append(cls.opt_global_config)

        def deco_(func):
            enabled_options = []
            for opt_callable in options:
                option, name = opt_callable()
                enabled_options.append(name)
                func = option(func)

            def wrapper(*args, **kwargs):
                app_config = cls(kwargs, enabled_options)
                return func(*args, app_config=app_config, **kwargs)

            if expose_app_config:
                return update_wrapper(cast(Callable, wrapper), func)

            return func

        return deco_

    def _get_option(self, opt_name):
        value = self.options.get(opt_name)
        if value is None:
            raise ValueError(f"{opt_name} was not specified")
        return value

    @property
    def ledger_config(self):
        return self._get_option(self.Params.LEDGER_CONFIG.value)

    @property
    def contract_address(self):
        addr = self._get_option(self.Params.CONTRACT_ADDRESS.value)
        self.CONTRACT_CLASS.validate_contract_address(addr)
        return addr

    @property
    def do_fund(self):
        try:
            return self._get_option(self.Params.DO_FUND.value)
        except ValueError:
            return False

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

    def get_ledger_instance(self, check_availability=True) -> AbstractLedger:
        ledger = self.LEDGER_CLASS(**self.ledger_config)
        if check_availability:
            ledger.check_availability()
        return ledger

    def get_storage_instance(self) -> AbstractStorage:
        storage = self.STORAGE_CLASS(**self.storage_config)
        storage.connect()
        return storage

    def get_ledger_crypto(self) -> AbstractLedgerCrypto:
        ledger = self.get_ledger_instance(check_availability=False)
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

    def fund_if_needed(self) -> bool:
        if self.do_fund:
            ledger = self.get_ledger_instance()
            addr = self.get_ledger_crypto().get_address()
            if ledger.get_balance(addr) < DEFAULT_FUNDS_AMOUNT:
                ledger.ensure_funds([addr])
                return True
        return False

    def validate_address(self, address: str):
        self.LEDGER_CLASS.validate_address(address)
