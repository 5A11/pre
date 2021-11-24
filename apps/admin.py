from typing import Optional

import click

from apps.defaults import CONTRACT_CLASS, LEDGER_CLASS
from apps.utils import config_option, private_key_file_option
from pre.api.admin import AdminAPI


PROG_NAME = "admin"


@click.group(name=PROG_NAME)
def cli():
    pass


@cli.command("instantiate-contract")
@config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)
@private_key_file_option("--ledger-private-key", "--pk", required=True)
@click.option("--admin-address", type=str, required=False)
@click.option("--threshold", type=int, required=False, default=1)
@click.option("--n-max-proxies", type=int, required=False, default=10)
@click.option("--proxies", type=str, required=False)
def instantiate_contract(
    ledger_config,
    ledger_private_key,
    threshold,
    n_max_proxies,
    admin_address: Optional[str] = None,
    proxies: str = "",
):
    # TODO: admin address validation
    # TODO: proxy address validation
    ledger = LEDGER_CLASS(**ledger_config)
    ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)

    if not admin_address:
        admin_address = ledger_crypto.get_address()

    proxies_list = []
    if proxies:
        proxies_list = proxies.split(",")

    kwargs = dict(
        admin_address=admin_address,
        threshold=threshold,
        n_max_proxies=n_max_proxies,
        proxies=proxies_list,
    )

    click.echo("instantiate contract with options:")
    for k, v in kwargs.items():
        click.echo(f" * {k}: {v}")

    contract_addr = AdminAPI.instantiate_contract(ledger_crypto, ledger, **kwargs)  # type: ignore
    click.echo()
    click.echo(f"Contract was set succesfully. Contract address is {contract_addr}")


@cli.command("add-proxy")
@config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)
@private_key_file_option("--ledger-private-key", "--pk", required=True)
@click.option("--contract-address", type=str, required=True)
@click.argument("proxy-address", type=str, required=True)
def add_proxy(
    ledger_config,
    ledger_private_key,
    contract_address: str,
    proxy_address: str,
):
    # TODO: proxy address validation
    ledger = LEDGER_CLASS(**ledger_config)
    ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)
    contract = CONTRACT_CLASS.ADMIN__CONTRACT(
        ledger=ledger, contract_address=contract_address
    )
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.add_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} added")


@cli.command("remove-proxy")
@config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)
@private_key_file_option("--ledger-private-key", "--pk", required=True)
@click.option("--contract-address", type=str, required=True)
@click.option("--proxy-address", type=str, required=True)
def remove_proxy(
    ledger_config,
    ledger_private_key,
    contract_address: str,
    proxy_address: str,
):
    # TODO: proxy address validation
    ledger = LEDGER_CLASS(**ledger_config)
    ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)
    contract = CONTRACT_CLASS.ADMIN__CONTRACT(
        ledger=ledger, contract_address=contract_address
    )
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.remove_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} removed")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
