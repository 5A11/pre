import time
from pathlib import Path
from typing import Dict, cast

import click

from apps.defaults import CONTRACT_CLASS, LEDGER_CLASS, STORAGE_CLASS
from apps.utils import _make_api_instance, config_option, private_key_file_option
from pre.api.proxy import ProxyAPI


PROG_NAME = "proxy"


DEFAULT_SLEEP_TIME = 5


@click.group(name=PROG_NAME)
def cli():
    pass


def common_options(func):
    func = private_key_file_option("--ledger-private-key", "--lpk", required=True)(func)
    func = config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)(
        func
    )
    func = config_option("--ipfs-config", STORAGE_CLASS.CONFIG_CLASS, required=False)(
        func
    )
    func = private_key_file_option("--encryption-private-key", "--epk", required=True)(
        func
    )
    func = click.option("--contract-address", type=str, required=True)(func)
    return func


@cli.command(name="register")
@common_options
def register(
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
):
    proxy_api = cast(
        ProxyAPI,
        _make_api_instance(
            ProxyAPI,
            CONTRACT_CLASS.PROXY_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    proxy_api.register()
    click.echo("Proxy was registered")


@cli.command(name="unregister")
@common_options
def unregister(
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
):
    proxy_api = cast(
        ProxyAPI,
        _make_api_instance(
            ProxyAPI,
            CONTRACT_CLASS.PROXY_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    proxy_api.unregister()
    click.echo("Proxy was unregistered")


@cli.command(name="run")
@common_options
@click.option(
    "--run-once-and-exit", is_flag=True, hidden=True, help="for test purposes"
)
def run(
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
    run_once_and_exit: bool,
):
    proxy_api = cast(
        ProxyAPI,
        _make_api_instance(
            ProxyAPI,
            CONTRACT_CLASS.PROXY_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    try:
        try:
            proxy_api.register()
            click.echo("Proxy was registered")
        except ValueError as exc:
            if "Generic error: Proxy already registered" in str(exc):
                click.echo("Proxy was registered already")
            else:
                raise
        while True:
            task = proxy_api.get_next_reencryption_request()
            if not task:
                time.sleep(DEFAULT_SLEEP_TIME)
                continue
            click.echo(f"Got a reencryption task: {task}")
            proxy_api.process_reencryption_request(task)
            click.echo(f"Reencryption task processed: {task}")

            if run_once_and_exit:  # pragma: nocover
                break

    finally:
        proxy_api.unregister()
        click.echo("Proxy was unregistered")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
