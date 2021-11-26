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
    delegatee_api = DelegateeAPI(
        encryption_private_key=app_config.get_cryto_key(),
        contract=app_config.get_query_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    click.echo(f"reader public key: {bytes(delegatee_api._encryption_private_key.public_key).hex()}")
    is_ready, _, _ = delegatee_api.is_data_ready(hash_id)
    if is_ready:
        click.echo(f"Data {hash_id} is ready!")
        ctx.exit(0)
    else:
        click.echo(f"Data {hash_id} is NOT ready!")
        ctx.exit(1)


@cli.command(name="get-data")
@click.argument("hash_id", type=str, required=True)
@click.argument("owner-public-key", type=str, required=True)
@file_argument_with_rewrite("output", required=False)
@click.pass_context
def get_data(
    ctx: click.Context,
    hash_id: str,
    owner_public_key: str,
    output: Optional[Path],
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    data_file_name = output or Path(hash_id)

    delegatee_api = DelegateeAPI(
        encryption_private_key=app_config.get_cryto_key(),
        contract=app_config.get_query_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )

    delegator_pubkey_bytes = bytes.fromhex(owner_public_key)
    data = delegatee_api.read_data(hash_id, delegator_pubkey_bytes)
    data_file_name.write_bytes(cast(bytes, data))
    click.echo(f"Data {hash_id} decrypted and stored at {data_file_name}")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
