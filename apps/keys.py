from pathlib import Path
from typing import Dict

import click

from apps.defaults import CRYPTO_CLASS, LEDGER_CLASS
from apps.utils import (
    config_option,
    private_key_file_argument,
    write_private_key_file_argument,
)


PROG_NAME = "keys"


@click.group(name=PROG_NAME)
def cli():
    """Generate private keys for ledger and encryption."""


@cli.command(name="generate-ledger-key")
@config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)
@write_private_key_file_argument
def generate_ledger_key(private_key_file: Path, ledger_config: Dict):
    """Generate private key for ledger."""
    ledger = LEDGER_CLASS(**ledger_config)
    pkey = ledger.make_new_crypto()
    private_key_file.write_text(pkey.as_str())
    click.echo(f"Private key written to `{private_key_file}`")


@cli.command(name="generate-crypto-key")
@write_private_key_file_argument
def generate_crypto_key(private_key_file: Path):
    """Generate private key for encryption."""
    pkey = CRYPTO_CLASS.make_new_key()
    private_key_file.write_bytes(bytes(pkey))
    click.echo(f"Private key written to `{private_key_file}`")


@cli.command(name="get-ledger-address")
@private_key_file_argument("ledger-private-key", required=True)
@config_option("--ledger-config", LEDGER_CLASS.CONFIG_CLASS, required=False)
def get_ledger_private_key(
    ledger_config,
    ledger_private_key,
):
    # TODO: proxy address validation
    ledger = LEDGER_CLASS(**ledger_config)
    ledger_crypto = ledger.load_crypto_from_file(ledger_private_key)
    click.echo(
        f"Ledger address for key {ledger_private_key} is {ledger_crypto.get_address()}"
    )


@cli.command(name="get-encryption-pubkey")
@private_key_file_argument("encryption-ledger-private-key", required=True)
def get_public_key(
    encryption_ledger_private_key,
):
    # TODO: proxy address validation
    pkey = CRYPTO_CLASS.load_key(encryption_ledger_private_key.read_bytes())
    pubkey_hex = pkey.public_key.__bytes__().hex()
    click.echo(f"Public key hex for {encryption_ledger_private_key} is {pubkey_hex}")


if __name__ == "__main__":
    cli(
        prog_name=PROG_NAME
    )  # pragma: no cover  # pylint: disable=unexpected-keyword-arg
