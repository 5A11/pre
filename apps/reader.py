from pathlib import Path
from typing import Optional, cast

import click

from apps.conf import AppConf
from apps.utils import file_argument_with_rewrite
from pre.api.delegatee import DelegateeAPI


PROG_NAME = "reader"


@click.group(name=PROG_NAME)
def cli():
    pass


@cli.command(name="get-data-status")
@AppConf.deco(
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("hash_id", type=str, required=True)
@click.pass_context
def get_data_status(
    ctx: click.Context,
    app_config: AppConf,
    hash_id: str,
):
    delegatee_api = DelegateeAPI(
        encryption_private_key=app_config.get_cryto_key(),
        contract=app_config.get_query_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )
    is_ready, _, _ = delegatee_api.is_data_ready(hash_id)
    if is_ready:
        click.echo(f"Data {hash_id} is ready!")
        ctx.exit(0)
    else:
        click.echo(f"Data {hash_id} is NOT ready!")
        ctx.exit(1)


@cli.command(name="get-data")
@AppConf.deco(
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("hash_id", type=str, required=True)
@click.argument("owner-publickey", type=str, required=True)
@file_argument_with_rewrite("data-file-name", required=False)
def get_data(
    app_config: AppConf,
    hash_id: str,
    owner_publickey: str,
    data_file_name: Optional[Path],
):
    data_file_name = data_file_name or Path(hash_id)

    delegatee_api = DelegateeAPI(
        encryption_private_key=app_config.get_cryto_key(),
        contract=app_config.get_query_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )

    delegator_pubkey_bytes = bytes.fromhex(owner_publickey)
    data = delegatee_api.read_data(hash_id, delegator_pubkey_bytes)
    data_file_name.write_bytes(cast(bytes, data))
    click.echo(f"Data {hash_id} decrypted and stored at {data_file_name}")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
