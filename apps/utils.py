from pathlib import Path
from typing import Dict

import click
import yaml

from apps.defaults import CRYPTO_CLASS, LEDGER_CLASS, STORAGE_CLASS
from pre.common import AbstractConfig


ledger_config_file_option = click.option()
ledger_private_key_option = click.option()
crypto_private_key_option = click.option()


def _make_api_instance(
    api_cls,
    contract_class,
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
):

    storage = STORAGE_CLASS(**ipfs_config)
    storage.connect()

    ledger = LEDGER_CLASS(**ledger_config)
    if ledger_private_key is None:
        ledger_crypto = None
    else:
        ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)

    contract = contract_class(ledger=ledger, contract_address=contract_address)

    epk = CRYPTO_CLASS.load_key(encryption_private_key.read_bytes())
    api_instance = api_cls(
        epk,
        ledger_crypto=ledger_crypto,
        contract=contract,
        storage=storage,
        crypto=CRYPTO_CLASS(),
    )
    return api_instance


def write_private_key_file_argument(func):
    def check_rewrite(ctx: click.Context, param, value):
        rewrite = ctx.params.pop("rewrite")
        if Path(value).exists() and not rewrite:
            click.echo(
                f"File `{value}` exists, please use --rewrite option to allow file rewrite"
            )
            ctx.exit(1)
        return value

    func = click.option("--rewrite", is_flag=True, is_eager=True, expose_value=True)(
        func
    )
    func = click.argument(
        "private-key-file",
        type=click.Path(file_okay=True, dir_okay=False, writable=True, path_type=Path),
        required=True,
        expose_value=True,
        callback=check_rewrite,
    )(func)
    return func


file_exists_type = click.Path(
    file_okay=True,
    dir_okay=False,
    readable=True,
    exists=True,
    path_type=Path,
)


def private_key_file_argument(name: str, *args, **kwargs):
    def deco(func):
        func = click.argument(
            name,
            *args,
            type=file_exists_type,
            **kwargs,
        )(func)
        return func

    return deco


def private_key_file_option(name: str, *args, **kwargs):
    def deco(func):
        func = click.option(
            name,
            *args,
            type=file_exists_type,
            **kwargs,
        )(func)
        return func

    return deco


def config_option(option_name: str, config_class: AbstractConfig, **options):
    def _load_and_check_config(ctx: click.Context, param, value):
        del ctx
        del param
        if not value:
            return config_class.make_default()
        data = yaml.safe_load(Path(value).read_text())
        return config_class.validate(data)

    def deco(func):
        func = click.option(
            option_name,
            expose_value=True,
            callback=_load_and_check_config,
            type=click.Path(
                exists=True,
                file_okay=True,
                dir_okay=False,
                readable=True,
                path_type=Path,
            ),
            **options,
        )(func)

        return func

    return deco
