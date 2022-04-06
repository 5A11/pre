import time
from typing import Optional

import click
from prometheus_client import start_http_server

from apps.conf import AppConf
from apps.metrics import ProxyMetrics
from pre.api.proxy import ProxyAPI
from pre.contract.base_contract import ContractExecutionError, ContractQueryError
from pre.contract.cosmos_contracts import encode_bytes
from pre.crypto.base_crypto import DecryptionError


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
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )

    if app_config.fund_if_needed():
        click.echo(f"{app_config.get_ledger_crypto()} was funded")

    proxy_api.register()
    click.echo("Proxy was registered")


@cli.command(name="unregister")
@click.pass_context
def unregister(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.unregister()
    click.echo("Proxy was unregistered")


@cli.command(name="deactivate")
@click.pass_context
def deactivate(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.deactivate()
    click.echo("Proxy was deactivated")


@cli.command(name="reactivate")
@click.pass_context
def reactivate(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.reactivate()
    click.echo("Proxy was reactivated")


@cli.command(name="withdraw_stake")
@click.option("--stake_amount", type=int, required=False)
@click.pass_context
def withdraw_stake(ctx, stake_amount: Optional[int] = None):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )
    proxy_api.withdraw_stake(stake_amount)
    click.echo("Stake was withdrawn")


@cli.command(name="run")
@click.pass_context
@click.option(
    "--run-once-and-exit", is_flag=True, hidden=True, help="for test purposes"
)
@click.option(
    "--only-deactivate-on-exit",
    is_flag=True,
    hidden=False,
    help="Do not unregister when existing, only deactivate",
)
@click.option(
    "--metrics-port",
    is_flag=False,
    hidden=False,
    type=int,
    default=9090,
    help="Prometheus metrics server port",
)
@click.option(
    "--disable-metrics",
    is_flag=True,
    hidden=False,
    help="Do not run metrics server",
)
@click.option(
    "--auto-withdrawal",
    is_flag=True,
    required=False,
    default=True,
    help="Enable/disable automatic stake withdrawing",
)
def run(
    ctx,
    run_once_and_exit: bool,
    only_deactivate_on_exit: bool,
    metrics_port: int,
    disable_metrics: bool,
    auto_withdrawal: bool,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    proxy_api = ProxyAPI(
        app_config.get_crypto_key(),
        app_config.get_ledger_crypto(),
        contract=app_config.get_proxy_contract(),
        crypto=app_config.get_crypto_instance(),
    )

    metrics = ProxyMetrics(disable_metrics)
    if not disable_metrics:
        click.echo(f"Starting metrics server on {metrics_port}")
        start_http_server(metrics_port)

    click.echo(f"Proxy address {app_config.get_ledger_crypto().get_address()}")
    click.echo(f"Proxy pubkey  {encode_bytes(proxy_api._pub_key_as_bytes())}")

    if app_config.fund_if_needed():
        click.echo(f"{app_config.get_ledger_crypto().get_address()} was funded")

    try:
        if not proxy_api.registered():
            proxy_api.register()
            click.echo(
                "Proxy registered (or reactivated) successfully"
            )  # pragma: nocover
        else:
            click.echo("Proxy was already registered. skip registration")
    except ContractExecutionError:
        metrics.report_contract_execution_failure()
        raise
    except ContractQueryError:
        metrics.report_contract_query_failure()
        raise

    logged = False
    try:
        while True:
            try:
                with metrics.time_query_tasks.time():
                    if not logged:
                        click.echo("Querying reencryption tasks from contract...")
                        logged = True
                    task = None
                    tasks = proxy_api.get_reencryption_requests()
                    if len(tasks) > 0:
                        task = tasks[0]
            except ContractQueryError as e:
                click.echo(f"Warning: failed to query contract: {str(e)}")
                metrics.report_contract_query_failure()
                task = None

            task_processing_failed = False
            if task is not None:
                logged = False
                metrics.report_pending_tasks_count(len(tasks))
                click.echo(f"Got a reencryption task: {task}")
                try:
                    with metrics.time_process_task.time():
                        proxy_api.process_reencryption_request(task)
                    click.echo(f"Reencryption task processed: {task}")
                    metrics.report_task_succeeded()
                    if auto_withdrawal:
                        proxy_api.withdraw_stake()
                        click.echo("Stake withdrawn.")
                except (DecryptionError, ContractExecutionError) as e:
                    click.echo(
                        f"Error: failed to process reencryption request {task.hash_id} : {str(e)}"
                    )
                    if isinstance(e, DecryptionError):
                        metrics.report_umbral_reencryption_failure()
                    elif isinstance(e, ContractExecutionError):
                        metrics.report_contract_execution_failure()
                        task_processing_failed = True
                    metrics.report_task_failed()
            else:  # pragma: nocover
                time.sleep(DEFAULT_SLEEP_TIME)

            if task_processing_failed:
                try:
                    proxy_api.skip_task(task.hash_id)
                except ContractExecutionError as e:
                    click.echo(f"Error: failed to skip task {task.hash_id}, {e}")
                    metrics.report_contract_execution_failure()

            if run_once_and_exit:  # pragma: nocover
                break
    finally:
        try:
            click.echo(
                f"Unregistering Proxy (deactivate only? {only_deactivate_on_exit})..."
            )
            proxy_api.unregister(only_deactivate_on_exit)
        except ContractExecutionError:
            metrics.report_contract_execution_failure()
            raise

        if only_deactivate_on_exit:
            click.echo("Proxy successfully deactivated")
        else:
            click.echo("Proxy successfully unregistered")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
