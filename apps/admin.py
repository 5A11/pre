import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Optional

import click

from apps.conf import AppConf
from pre.api.admin import AdminAPI


PROG_NAME = "admin"


@dataclass
class DeployedContract:
    contract_address: str


@click.group(name=PROG_NAME)
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_do_fund,
    expose_app_config=True,
)
@click.pass_context
def cli(ctx, app_config: AppConf):
    ctx.ensure_object(dict)
    ctx.obj[AppConf.ctx_key] = app_config


@cli.command("instantiate-contract")
@click.option("--admin-address", type=str, required=False)
@click.option("--threshold", type=int, required=False, default=1)
@click.option("--proxies", type=str, required=False)
@click.option(
    "--output-file",
    type=str,
    required=False,
    help="Path to file containing deployed contract address",
)
@click.pass_context
def instantiate_contract(
    ctx,
    threshold,
    admin_address: Optional[str] = None,
    stake_denom: str = "atestfet",
    proxies: str = "",
    output_file: Optional[str] = None,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    ledger = app_config.get_ledger_instance()
    ledger_crypto = app_config.get_ledger_crypto()

    if not admin_address:
        admin_address = ledger_crypto.get_address()

    app_config.validate_address(admin_address)

    proxies_list = []
    if proxies:
        proxies_list = proxies.split(",")

    for proxy_addr in proxies_list:
        app_config.validate_address(proxy_addr)

    kwargs = dict(
        admin_address=admin_address,
        stake_denom=stake_denom,
        threshold=threshold,
        proxies=proxies_list,
    )

    click.echo("instantiate contract with options:")
    for key, value in kwargs.items():
        click.echo(f" * {key}: {value}")

    if app_config.do_fund:
        ledger = app_config.get_ledger_instance()
        if app_config.fund_if_needed():
            click.echo(f"{app_config.get_ledger_crypto().get_address()} was funded.")

    contract_addr = AdminAPI.instantiate_contract(
        ledger_crypto,
        ledger,
        contract_cls=app_config.CONTRACT_CLASS.ADMIN_CONTRACT,
        **kwargs,  # type: ignore
    )

    if output_file is not None:
        contract = DeployedContract(contract_addr)
        Path(output_file).write_text(json.dumps(asdict(contract)))

    click.echo()
    click.echo(f"Contract was set succesfully. Contract address is {contract_addr}")


@cli.command("add-proxy")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("proxy-address", type=str, required=True)
def add_proxy(
    app_config: AppConf,
    proxy_address: str,
):
    app_config.validate_address(proxy_address)
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.add_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} added")


@cli.command("remove-proxy")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("proxy-address", type=str, required=True)
def remove_proxy(
    app_config: AppConf,
    proxy_address: str,
):
    app_config.validate_address(proxy_address)
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.remove_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} removed")


@cli.command("terminate-contract")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
def terminate_contract(
    app_config: AppConf,
):
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.terminate_contract()
    click.echo("Contract was terminated")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
