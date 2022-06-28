import json
from dataclasses import asdict, dataclass
from decimal import Decimal
from pathlib import Path
from typing import Optional, Tuple

import click

from apps.conf import AppConf
from pre.api.admin import AdminAPI


PROG_NAME = "admin"


class Coin:
    ATOM_PREFIX = "a"
    ATOM_FACTOR = 10 ** 18

    def __init__(self, coin: str) -> None:
        amount, denom = self._parse_coin(coin)
        if denom[0] != self.ATOM_PREFIX:
            denom = self.ATOM_PREFIX + denom
            amount = amount * self.ATOM_FACTOR
        self.amount = int(amount)
        self.denom = denom

    def __mul__(self, other_integer: int) -> "Coin":
        return Coin(f"{self.amount*other_integer}{self.denom}")

    def _parse_coin(self, coin: str) -> Tuple[Decimal, str]:
        for i, c in enumerate(coin):
            if str.isalpha(c) and i > 0:
                return Decimal(coin[0:i]), coin[i:]
        raise ValueError(f"Couldn't parse coin {coin}")


@dataclass
class LedgerNetworkConfig:
    node_address: str
    chain_id: str
    prefix: str
    denom: str

    @classmethod
    def from_app_config(cls, app_conf: AppConf) -> "LedgerNetworkConfig":
        lconf = app_conf.ledger_config
        return cls(
            lconf["node_address"], lconf["chain_id"], lconf["prefix"], lconf["denom"]
        )


@dataclass
class DeployedContract:
    contract_address: str
    network: LedgerNetworkConfig


@click.group(name=PROG_NAME)
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_do_fund,
    expose_app_config=True,
)
@click.pass_context
def cli(ctx, app_config: AppConf):
    ctx.ensure_object(dict)
    ctx.obj[AppConf.ctx_key] = app_config


@cli.command("instantiate-contract")
@click.option("--admin-address", type=str, required=False)
@click.option("--threshold", type=int, required=False, default=1)
@click.option("--proxies", type=str, required=False)
@click.option("--stake_denom", type=str, required=False, default="atestfet")
@click.option("--proxy-reward", type=str, required=False, default="100atestfet")
@click.option(
    "--output-file",
    type=str,
    required=False,
    help="Path to file containing deployed contract address",
)
@click.option("--proxy-whitelisting", is_flag=True, help="Enable proxy whitelisting")
@click.pass_context
def instantiate_contract(
    ctx,
    threshold,
    admin_address: Optional[str] = None,
    stake_denom: str = "atestfet",
    proxies: str = "",
    proxy_reward: str = "100atestfet",
    output_file: Optional[str] = None,
    proxy_whitelisting: Optional[bool] = False,
):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]
    ledger = app_config.get_ledger_instance()
    ledger_crypto = app_config.get_ledger_crypto()

    if not admin_address:
        admin_address = ledger_crypto.get_address()

    app_config.validate_address(admin_address)

    proxies_list = []
    if proxies:
        proxies_list = proxies.split(",")

    for proxy_addr in proxies_list:
        app_config.validate_address(proxy_addr)

    proxy_reward_coin = Coin(proxy_reward)
    if proxy_reward_coin.denom != stake_denom:
        raise ValueError(f"Proxy reward should be in {stake_denom}")

    kwargs = dict(
        admin_address=admin_address,
        stake_denom=stake_denom,
        threshold=threshold,
        proxies=proxies_list,
        proxy_whitelisting=proxy_whitelisting,
        per_proxy_task_reward_amount=proxy_reward_coin.amount,
        per_task_slash_stake_amount=proxy_reward_coin.amount,
        minimum_proxy_stake_amount=proxy_reward_coin.amount * 10,
    )

    click.echo("instantiate contract with options:")
    for key, value in kwargs.items():
        click.echo(f" * {key}: {value}")

    if app_config.do_fund:
        ledger = app_config.get_ledger_instance()
        if app_config.fund_if_needed():
            click.echo(f"{app_config.get_ledger_crypto().get_address()} was funded.")

    contract_addr = AdminAPI.instantiate_contract(
        ledger_crypto,
        ledger,
        contract_cls=app_config.CONTRACT_CLASS.ADMIN_CONTRACT,
        **kwargs,  # type: ignore
    )

    if output_file is not None:
        contract = DeployedContract(
            contract_addr, LedgerNetworkConfig.from_app_config(app_config)
        )
        Path(output_file).write_text(json.dumps(asdict(contract), indent=4))

    click.echo()
    click.echo(f"Contract was set successfully. Contract address is {contract_addr}")


@cli.command("add-proxy")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("proxy-address", type=str, required=True)
def add_proxy(
    app_config: AppConf,
    proxy_address: str,
):
    app_config.validate_address(proxy_address)
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.add_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} added")


@cli.command("remove-proxy")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
@click.argument("proxy-address", type=str, required=True)
def remove_proxy(
    app_config: AppConf,
    proxy_address: str,
):
    app_config.validate_address(proxy_address)
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.remove_proxy(proxy_address)
    click.echo(f"Proxy {proxy_address} removed")


@cli.command("terminate-contract")
@AppConf.deco(
    AppConf.opt_ledger_private_key,
    AppConf.opt_ledger_config,
    AppConf.opt_contract_address,
    expose_app_config=True,
)
def terminate_contract(
    app_config: AppConf,
):
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.terminate_contract()
    click.echo("Contract was terminated")


@click.argument("recipient-address", type=str, required=True)
def withdraw_contract(
    app_config: AppConf,
    recipient_address: str,
):
    app_config.validate_address(recipient_address)
    ledger_crypto = app_config.get_ledger_crypto()
    contract = app_config.get_admin_contract()
    api = AdminAPI(ledger_crypto=ledger_crypto, contract=contract)
    api.withdraw_contract(recipient_address)
    click.echo(f"Contract of balance was withdrawn to {recipient_address}")


@cli.command(name="check-liveness")
@click.pass_context
def check_liveness(ctx):
    app_config: AppConf = ctx.obj[AppConf.ctx_key]

    # Check ledger
    ledger = app_config.get_ledger_instance()
    assert ledger, "ledger not available"

    ledger.check_availability()

    # Check keys
    ledger_crypto = app_config.get_ledger_crypto()
    assert ledger_crypto, "ledger_crypto not available"

    click.echo("Admin is alive")


if __name__ == "__main__":
    cli(  # pylint: disable=unexpected-keyword-arg,no-value-for-parameter
        prog_name=PROG_NAME
    )  # pragma: no cover
