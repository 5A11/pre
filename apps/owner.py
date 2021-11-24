from pathlib import Path
from typing import Dict, cast

import click

from apps.defaults import CONTRACT_CLASS, LEDGER_CLASS, STORAGE_CLASS
from apps.utils import (
    _make_api_instance,
    config_option,
    file_exists_type,
    private_key_file_option,
)
from pre.api.delegator import DelegatorAPI


PROG_NAME = "owner"


@click.group(name=PROG_NAME)
def cli():
    pass


def common_options(func):
    func = private_key_file_option("--ledger-private-key", "--lpk", required=True)(func)
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


@cli.command(name="add-data")
@common_options
@click.argument("data_file", type=file_exists_type, required=True)
def add_data(
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
    data_file: Path,
):
    delegator_api = cast(
        DelegatorAPI,
        _make_api_instance(
            DelegatorAPI,
            CONTRACT_CLASS.DELEGATOR_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )
    data = data_file.read_bytes()
    hash_id = delegator_api.add_data(data)
    click.echo(f"Data was settled: hash_id is {hash_id}")


@cli.command(name="grant-access")
@common_options
@click.option("--threshold", type=int, required=False, default=1)
@click.option("--proxies", type=str, required=False, default="")
@click.argument("hash_id", type=str, required=True)
@click.argument("reader-publickey", type=str, required=True)
def grant_access(
    ledger_private_key: Path,
    encryption_private_key: Path,
    ipfs_config: Dict,
    ledger_config: Dict,
    contract_address: str,
    threshold: int,
    proxies: str,
    hash_id: str,
    reader_publickey: str,
):
    delegator_api = cast(
        DelegatorAPI,
        _make_api_instance(
            DelegatorAPI,
            CONTRACT_CLASS.DELEGATOR_CONTRACT,
            ledger_private_key,
            encryption_private_key,
            ipfs_config,
            ledger_config,
            contract_address,
        ),
    )

    delegatee_pubkey_bytes = bytes.fromhex(reader_publickey)
    proxies_list = [bytes.fromhex(i) for i in proxies.split(",") if i]

    delegator_api.grant_access(
        hash_id=hash_id,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        threshold=threshold,
        proxies_list=proxies_list,
    )

    click.echo(f"Access to hash_id {hash_id} granted to {reader_publickey}")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
