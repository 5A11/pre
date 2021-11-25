import time

import click

from apps.conf import AppConf
from pre.api.proxy import ProxyAPI


PROG_NAME = "proxy"


DEFAULT_SLEEP_TIME = 5


@click.group(name=PROG_NAME)
def cli():
    pass


@cli.command(name="register")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
def register(app_config: AppConf):
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.register()
    click.echo("Proxy was registered")


@cli.command(name="unregister")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
def unregister(app_config: AppConf):
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.unregister()
    click.echo("Proxy was unregistered")


@cli.command(name="run")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.option(
    "--run-once-and-exit", is_flag=True, hidden=True, help="for test purposes"
)
def run(run_once_and_exit: bool, app_config: AppConf):
    proxy_api = ProxyAPI(
        app_config.get_cryto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
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
