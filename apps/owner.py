from pathlib import Path

import click

from apps.conf import AppConf
from apps.utils import file_exists_type
from pre.api.delegator import DelegatorAPI


PROG_NAME = "owner"


@click.group(name=PROG_NAME)
def cli():
    pass


@cli.command(name="add-data")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("data_file", type=file_exists_type, required=True)
def add_data(
    app_config: AppConf,
    data_file: Path,
):
    delegator_api = DelegatorAPI(
        encryption_private_key=app_config.get_cryto_key(),
        ledger_crypto=app_config.get_ledger_crypto(),
        contract=app_config.get_owner_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
    )

    data = data_file.read_bytes()
    hash_id = delegator_api.add_data(data)
    click.echo(f"Data was settled: hash_id is {hash_id}")


@cli.command(name="grant-access")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_encryption_private_key,
    AppConf.opt_storage_config,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.option("--threshold", type=int, required=False, default=1)
@click.option("--proxies", type=str, required=False, default="")
@click.argument("hash_id", type=str, required=True)
@click.argument("reader-publickey", type=str, required=True)
def grant_access(
    app_config: AppConf,
    threshold: int,
    proxies: str,
    hash_id: str,
    reader_publickey: str,
):
    delegator_api = DelegatorAPI(
        encryption_private_key=app_config.get_cryto_key(),
        ledger_crypto=app_config.get_ledger_crypto(),
        contract=app_config.get_owner_contract(),
        storage=app_config.get_storage_instance(),
        crypto=app_config.get_crypto_instance(),
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
