from pathlib import Path
from typing import Optional, cast

import click

from apps.conf import AppConf
from apps.utils import file_argument_with_rewrite
from pre.api.delegatee import DelegateeAPI


PROG_NAME = "reader"


@click.group(name=PROG_NAME)
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    AppConf.opt_do_fund,
    expose_app_config=True,
)
@click.pass_context
def cli(ctx, app_config: AppConf):
    ctx.ensure_object(dict)
    ctx.obj[AppConf.ctx_key] = app_config


@cli.command(name="get-data-status")
@click.argument("hash_id", type=str, required=True)
@click.pass_context
def get_data_status(
    ctx: click.Context,
    hash_id: str,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    encryption_private_key = app_config.get_crypto_key()
    delegatee_api = DelegateeAPI(
        encryption_private_key=encryption_private_key,
        contract=app_config.get_query_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    click.echo(f"reader public key: {bytes(encryption_private_key.public_key).hex()}")
    is_ready, *_ = delegatee_api.is_data_ready(hash_id)
    if is_ready:
        click.echo(f"Data {hash_id} is ready!")
        ctx.exit(0)
    else:
        click.echo(f"Data {hash_id} is NOT ready!")
        ctx.exit(0)


@cli.command(name="get-data")
@click.argument("hash_id", type=str, required=True)
@file_argument_with_rewrite("output", required=False)
@click.pass_context
def get_data(
    ctx: click.Context,
    hash_id: str,
    output: Optional[Path],
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    data_file_name = output or Path(hash_id)

    query_contract = app_config.get_query_contract()
    delegatee_api = DelegateeAPI(
        encryption_private_key=app_config.get_crypto_key(),
        contract=query_contract,
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )

    data_entry = query_contract.get_data_entry(hash_id)
    if not data_entry:
        raise ValueError("Couldn't query data entry of data id from contract")

    data = delegatee_api.read_data(hash_id, data_entry.pubkey)
    data_file_name.write_bytes(cast(bytes, data))
    click.echo(f"Data {hash_id} decrypted and stored at {data_file_name}")


@cli.command(name="check-liveness")
@click.pass_context
def check_liveness(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]

    # Check keys
    encryption_private_key = app_config.get_crypto_key()
    assert encryption_private_key, "encryption_private_key not available"

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

    click.echo("Reader is alive")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
