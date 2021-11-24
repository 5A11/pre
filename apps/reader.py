from pathlib import Path
from typing import Dict, Optional, cast

import click

from apps.defaults import CONTRACT_CLASS, LEDGER_CLASS, STORAGE_CLASS
from apps.utils import _make_api_instance, config_option, private_key_file_option
from pre.api.delegatee import DelegateeAPI


PROG_NAME = "reader"


@click.group(name=PROG_NAME)
def cli():
    pass


def common_options(func):
    func = config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)(
        func
    )
    func = config_option("--ipfs-config", STORAGE_CLASS.CONFIG_CLASS, required=False)(
        func
    )
    func = private_key_file_option("--encryption-private-key", "--epk", required=True)(
        func
    )
    func = click.option("--contract-address", type=str, required=True)(func)
    return func


@cli.command(name="get-data-status")
@common_options
@click.argument("hash_id", type=str, required=True)
@click.pass_context
def get_data_status(
    ctx: click.Context,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
    hash_id: str,
):
    ledger_private_key = None
    delegatee_api = cast(
        DelegateeAPI,
        _make_api_instance(
            DelegateeAPI,
            CONTRACT_CLASS.QUERIES_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    is_ready, _, _ = delegatee_api.is_data_ready(hash_id)
    if is_ready:
        click.echo(f"Data {hash_id} is ready!")
        ctx.exit(0)
    else:
        click.echo(f"Data {hash_id} is NOT ready!")
        ctx.exit(1)


@cli.command(name="get-data")
@common_options
@click.argument("hash_id", type=str, required=True)
@click.argument("owner-publickey", type=str, required=True)
@click.argument(
    "data_file_name",
    type=click.Path(file_okay=True, dir_okay=False, path_type=Path),
    required=False,
)
@click.option("--rewrite", is_flag=True)
@click.pass_context
def get_data(
    ctx: click.Context,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
    hash_id: str,
    owner_publickey: str,
    data_file_name: Optional[Path],
    rewrite: bool,
):
    ledger_private_key = None
    data_file_name = data_file_name or Path(hash_id)

    if data_file_name.exists() and not rewrite:
        raise ValueError(f"{data_file_name} file is exist! please use --rewrite option")

    delegatee_api = cast(
        DelegateeAPI,
        _make_api_instance(
            DelegateeAPI,
            CONTRACT_CLASS.QUERIES_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    delegator_pubkey_bytes = bytes.fromhex(owner_publickey)
    data = delegatee_api.read_data(hash_id, delegator_pubkey_bytes)
    data_file_name.write_bytes(cast(bytes, data))
    click.echo(f"Data {hash_id} decrypted and stored at {data_file_name}")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
