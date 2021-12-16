from typing import Optional

import click

from apps.conf import AppConf
from pre.api.admin import AdminAPI


PROG_NAME = "admin"


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
@click.option("--n-max-proxies", type=int, required=False, default=10)
@click.option("--proxies", type=str, required=False)
@click.pass_context
def instantiate_contract(
    ctx,
    threshold,
    n_max_proxies,
    admin_address: Optional[str] = None,
    stake_denom: str = "atestfet",
    proxies: str = "",
):
    # TODO: admin address validation
    # TODO: proxy address validation
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    ledger = app_config.get_ledger_instance()
    ledger_crypto = app_config.get_ledger_crypto()

    if not admin_address:
        admin_address = ledger_crypto.get_address()

    proxies_list = []
    if proxies:
        proxies_list = proxies.split(",")

    kwargs = dict(
        admin_address=admin_address,
        stake_denom=stake_denom,
        threshold=threshold,
        n_max_proxies=n_max_proxies,
        proxies=proxies_list,
    )

    click.echo("instantiate contract with options:")
    for k, v in kwargs.items():
        click.echo(f" * {k}: {v}")

    if app_config.do_fund:
        ledger = app_config.get_ledger_instance()
        if not ledger.get_balance(admin_address):
            click.echo(f"funding {admin_address}")
            ledger.ensure_funds([admin_address])

    contract_addr = AdminAPI.instantiate_contract(
        ledger_crypto,
        ledger,
        contract_cls=app_config.CONTRACT_CLASS.ADMIN_CONTRACT,
        **kwargs,
    )  # type: ignore
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
    # TODO: proxy address validation
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
    # TODO: proxy address validation
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.remove_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} removed")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
