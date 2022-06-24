from pathlib import Path

import click

from apps.conf import AppConf
from apps.utils import file_exists_type
from pre.api.delegator import DelegatorAPI


PROG_NAME = "owner"


@click.group(name=PROG_NAME)
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    AppConf.opt_do_fund,
    AppConf.opt_threshold,
    expose_app_config=True,
)
@click.pass_context
def cli(ctx, app_config: AppConf):
    ctx.ensure_object(dict)
    ctx.obj[AppConf.ctx_key] = app_config


@cli.command(name="add-data")
@click.argument("data_file", type=file_exists_type, required=True)
@click.pass_context
def add_data(
    ctx,
    data_file: Path,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    delegator_api = DelegatorAPI(
        encryption_private_key=app_config.get_crypto_key(),
        ledger_crypto=app_config.get_ledger_crypto(),
        contract=app_config.get_owner_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )

    if app_config.fund_if_needed():
        click.echo(
            f"{app_config.get_ledger_crypto()} was funded with default gas fees funds"
        )

    data = data_file.read_bytes()
    hash_id = delegator_api.add_data(data)
    click.echo(f"Data was settled: hash_id is {hash_id}")


@cli.command(name="grant-access")
@click.argument("hash_id", type=str, required=True)
@click.argument("reader-public-key", type=str, required=True)
@click.option("--n-max-proxies", type=int, required=False, default=10)
@click.pass_context
def grant_access(
    ctx,
    hash_id: str,
    reader_public_key: str,
    n_max_proxies: int,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    delegator_api = DelegatorAPI(
        encryption_private_key=app_config.get_crypto_key(),
        ledger_crypto=app_config.get_ledger_crypto(),
        contract=app_config.get_owner_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    # click.echo(f"owner public key: {bytes(delegator_api._encryption_private_key.public_key).hex()}")

    delegatee_pubkey_bytes = bytes.fromhex(reader_public_key)

    if app_config.fund_if_needed(staking=True):
        click.echo(
            f"{app_config.get_ledger_crypto()} was funded with default gas fees and stake funds"
        )

    delegator_api.grant_access(
        hash_id=hash_id,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        threshold=app_config.threshold,
        n_max_proxies=n_max_proxies,
    )

    click.echo(f"Access to hash_id {hash_id} granted to {reader_public_key}")


@cli.command(name="check-liveness")
@click.pass_context
def check_liveness(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]

    # Check keys
    encryption_private_key = app_config.get_crypto_key()
    assert encryption_private_key, "encryption_private_key not available"

    ledger_crypto = app_config.get_ledger_crypto()
    assert ledger_crypto, "ledger_crypto not available"

    crypto = app_config.get_crypto_instance()
    assert crypto, "crypto not available"

    # Check contract
    query_contract = app_config.get_query_contract()
    assert query_contract, "contract not available"

    contract_state = query_contract.get_contract_state()
    assert contract_state, "Failed to query contract state"

    # Check storage
    storage = app_config.get_storage_instance()
    assert storage, "storage not available"

    storage.connect()
    storage.disconnect()

    click.echo("Owner is alive")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
