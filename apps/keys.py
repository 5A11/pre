from pathlib import Path

import click

from apps.conf import AppConf
from apps.utils import file_argument_with_rewrite, private_key_file_argument


PROG_NAME = "keys"


@click.group(name=PROG_NAME)
def cli():
    """Generate private keys for ledger and encryption."""


@cli.command(name="generate-ledger-key")
@AppConf.deco(AppConf.opt_ledger_config, expose_app_config=True)
@file_argument_with_rewrite("private-key-file")
def generate_ledger_key(private_key_file: Path, app_config: AppConf):
    """Generate private key for ledger."""
    ledger = app_config.get_ledger_instance(check_availability=False)
    pkey = ledger.make_new_crypto()
    private_key_file.write_text(pkey.as_str())
    click.echo(f"Private key written to `{private_key_file}`")


@cli.command(name="generate-crypto-key")
@file_argument_with_rewrite("private-key-file")
def generate_crypto_key(private_key_file: Path):
    """Generate private key for encryption."""
    pkey = AppConf.CRYPTO_CLASS.make_new_key()
    private_key_file.write_bytes(bytes(pkey))
    click.echo(f"Private key written to `{private_key_file}`")


@cli.command(name="get-ledger-address")
@private_key_file_argument("ledger-private-key", required=True)
@AppConf.deco(AppConf.opt_ledger_config, expose_app_config=True)
def get_ledger_address(
    ledger_private_key,
    app_config: AppConf,
):
    # TODO: proxy address validation
    ledger = app_config.get_ledger_instance(check_availability=False)
    ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)
    click.echo(
        f"Ledger address for key {ledger_private_key} is {ledger_crypto.get_address()}"
    )


@cli.command(name="get-encryption-pubkey")
@private_key_file_argument("encryption-private-key", required=True)
def get_public_key(
    encryption_private_key,
):
    # TODO: proxy address validation
    pkey = AppConf.CRYPTO_CLASS.load_key(encryption_private_key.read_bytes())
    pubkey_hex = pkey.public_key.__bytes__().hex()
    click.echo(f"Public key hex for {encryption_private_key} is {pubkey_hex}")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg
        prog_name=PROG_NAME
    )  # pragma: no cover
