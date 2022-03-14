import time

import click

from apps.conf import AppConf
from pre.api.proxy import ProxyAPI
from pre.contract.base_contract import ProxyAlreadyRegistered


PROG_NAME = "proxy"


DEFAULT_SLEEP_TIME = 5


@click.group(name=PROG_NAME)
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    AppConf.opt_do_fund,
    expose_app_config=True,
)
@click.pass_context
def cli(ctx, app_config: AppConf):
    ctx.ensure_object(dict)
    ctx.obj[AppConf.ctx_key] = app_config


@cli.command(name="register")
@click.pass_context
def register(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )

    if app_config.fund_if_needed():
        click.echo(f"{app_config.pp_config.get_ledger_crypto()} was funded")

    proxy_api.register()
    click.echo("Proxy was registered")


@cli.command(name="unregister")
@click.pass_context
def unregister(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.unregister()
    click.echo("Proxy was unregistered")


@cli.command(name="run")
@click.pass_context
@click.option(
    "--run-once-and-exit", is_flag=True, hidden=True, help="for test purposes"
)
def run(ctx, run_once_and_exit: bool):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )

    if app_config.fund_if_needed():
        click.echo(f"{app_config.pp_config.get_ledger_crypto()} was funded")

    try:
        proxy_api.register()
        click.echo("Proxy was registered")  # pragma: nocover
    except ProxyAlreadyRegistered:
        click.echo("Proxy was already registered. skip registration")

    try:
        while True:
            task = proxy_api.get_next_reencryption_request()
            if task is not None:
                click.echo(f"Got a reencryption task: {task}")
                proxy_api.process_reencryption_request(task)
                click.echo(f"Reencryption task processed: {task}")
            else:  # pragma: nocover
                time.sleep(DEFAULT_SLEEP_TIME)

            if run_once_and_exit:  # pragma: nocover
                break
    finally:
        proxy_api.unregister()
        click.echo("Proxy was unregistered")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
