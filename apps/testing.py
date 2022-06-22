import argparse
import json
import multiprocessing
import random
import string
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, IO, Optional, Tuple, Union

import requests
import yaml

from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.common import HashID
from pre.contract.cosmos_contracts import ContractQueries, DelegatorContract
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey
from pre.ledger.cosmos.crypto import CosmosCrypto
from pre.ledger.cosmos.ledger import CosmosLedger, DEFAULT_FUNDS_AMOUNT
from pre.storage.ipfs_storage import IpfsStorage


FetchAddr = str

DEFAULT_KEY_FOLDER = "/test_data/keys/"
DEFAULT_KEY_PREFIX = "testing"
DEFAULT_DELEGATORS_COUNT = 1

DEFAULT_FAUCET_URL = "https://faucet-dorado.fetch.ai"
MAX_REENCRYPTION_WAIT_TIME = 10 * 60  # 10 mins
MINIMUM_FUNDS_AMOUNT = DEFAULT_FUNDS_AMOUNT / 10
DEFAULT_DATA_SIZE = 1000

_key_counter = multiprocessing.Value("i", 0)


@dataclass
class TestingConfig:
    contract_address: FetchAddr
    storage: IpfsStorage
    ledger: CosmosLedger
    owners_count: int
    proc_count: int
    scenario_default: int
    keys_path_config: Tuple[str, str]

    def check_availability(self):
        self.ledger.check_availability()
        self.storage.connect()

    @classmethod
    def from_cli_args(cls, args) -> "TestingConfig":
        ipfs_config = yaml.safe_load(
            Path(args.ipfs_config_file).read_text(encoding="utf-8")
        )
        contract_config_file = requests.get(args.contract_url, allow_redirects=True)
        contract_config = json.loads(contract_config_file.content)

        ipfs_instance = IpfsStorage(**ipfs_config)
        ledger_instance = CosmosLedger(
            **contract_config["network"],
            faucet_url=DEFAULT_FAUCET_URL,
            secure_channel=True,
        )

        return cls(
            contract_config["contract_address"],
            ipfs_instance,
            ledger_instance,
            args.owners_count,
            args.proc_count,
            args.scenario_default if args.scenario_default is not None else 0,
            (args.keys_folder, args.key_file_prefix),
        )


def parse_commandline() -> Tuple[TestingConfig, Any]:
    parser = argparse.ArgumentParser(
        "Testing utility for DabbaFlow contract and proxies deployment"
    )

    parser.add_argument(
        "--ipfs-config-file",
        required=True,
        help="Ipfs node address and port, in yaml format",
    )
    parser.add_argument(
        "--contract-url", required=True, help="Url to deployed contract config file"
    )
    parser.add_argument(
        "--keys-folder",
        type=str,
        default=DEFAULT_KEY_FOLDER,
        help="Folder where to store key file for reuse",
    )
    parser.add_argument(
        "--key-file-prefix",
        type=str,
        default=DEFAULT_KEY_PREFIX,
        help="Prefix to use when saving keys to file, suffix is an index",
    )
    parser.add_argument(
        "--owners-count",
        type=int,
        default=DEFAULT_DELEGATORS_COUNT,
        help="Number of owners to use for each selected scenario",
    )
    parser.add_argument(
        "--proc-count",
        type=int,
        default=multiprocessing.cpu_count(),
        help="Number of processes to create, default is number of cores",
    )
    parser.add_argument(
        "--scenario-default",
        type=int,
        default=1,
        help="Number of runs of the default scenario per owner. "
        "Default scenario is (1) owner register data (2) owner request reencryption (3) reader get data",
    )

    args = parser.parse_args()
    config = TestingConfig.from_cli_args(args)

    config.check_availability()
    return config, args


def new_ledger_key(
    ledger: CosmosLedger, keys_path_config: Tuple[str, str], do_fund: bool = True
) -> CosmosCrypto:
    with _key_counter.get_lock():
        key_file = Path(
            f"{keys_path_config[0]}/{keys_path_config[1]}_{str(_key_counter.value)}"
        )
        _key_counter.value += 1
    if key_file.exists():
        key = ledger.load_crypto_from_file(key_file)
        print(f"[D] Loaded key  {key.get_address()} to {str(key_file)}")
    else:
        key = ledger.make_new_crypto()
        key_file.write_text(key.as_str())
        print(f"[D] Saved key {key.get_address()} to {str(key_file)}")
    if do_fund:
        addr = key.get_address()
        if ledger.get_balance(addr) < MINIMUM_FUNDS_AMOUNT:
            start = time.time()
            ledger.ensure_funds([addr])
            end = time.time()
            print(f"[D] {addr} funded in {end-start}")
    return key


def new_encryption_key() -> UmbralPrivateKey:
    key = UmbralCrypto.make_new_key()
    return key


def new_delegator(
    contract_address: FetchAddr,
    ledger: CosmosLedger,
    storage: IpfsStorage,
    keys_path_config: Tuple[str, str],
    enc_key_maybe: Optional[UmbralPrivateKey] = None,
    ledger_key_maybe: Optional[CosmosCrypto] = None,
) -> DelegatorAPI:
    enc_key = enc_key_maybe if enc_key_maybe is not None else new_encryption_key()
    ledger_key = (
        ledger_key_maybe
        if ledger_key_maybe is not None
        else new_ledger_key(ledger, keys_path_config)
    )
    contract_api = DelegatorContract(ledger, contract_address)
    return DelegatorAPI(enc_key, ledger_key, contract_api, storage, UmbralCrypto())


def new_delegatee(
    contract_address: FetchAddr,
    ledger: CosmosLedger,
    storage: IpfsStorage,
    enc_key_maybe: Optional[UmbralPrivateKey] = None,
) -> DelegateeAPI:
    enc_key = enc_key_maybe if enc_key_maybe is not None else new_encryption_key()
    contract_api = ContractQueries(ledger, contract_address)
    return DelegateeAPI(enc_key, contract_api, storage, UmbralCrypto())


def register_data(user: DelegatorAPI, data_maybe: Optional[bytes] = None) -> HashID:
    data = (
        data_maybe
        if data_maybe is not None
        else "".join(
            random.choice(string.ascii_uppercase + string.digits)
            for _ in range(DEFAULT_DATA_SIZE)
        ).encode()
    )
    return user.add_data(data)


def grant_access(delegator: DelegatorAPI, delegatee: DelegateeAPI, data_id: HashID):
    threshold = (
        ContractQueries(
            delegator._contract.ledger, delegator._contract.contract_address
        )
        .get_contract_state()
        .threshold
    )
    delegator.grant_access(
        data_id,
        bytes(delegatee._encryption_private_key.public_key),
        threshold,
        sys.maxsize,
    )


def read_data(delegatee: DelegateeAPI, data_id: HashID) -> Optional[Union[bytes, IO]]:
    waited_for = 0
    while not delegatee.is_data_ready(data_id)[0]:
        if waited_for > MAX_REENCRYPTION_WAIT_TIME:
            print(
                f"[E] Data is not ready    : {bytes(delegatee._encryption_private_key.public_key).hex()} {data_id}",
                file=sys.stderr,
            )
            return None
        time.sleep(30)
        waited_for += 30

    data_entry = delegatee._contract.get_data_entry(data_id)
    assert data_entry is not None
    delegator_pubkey = data_entry.pubkey
    data = delegatee.read_data(data_id, delegator_pubkey)
    print(
        f"[I] Data read successfully: {bytes(delegatee._encryption_private_key.public_key).hex()} {data_id}"
    )
    return data


def scenario_default(args):
    config = TestingConfig.from_cli_args(args)
    config.storage.connect()

    delegator = new_delegator(
        config.contract_address, config.ledger, config.storage, config.keys_path_config
    )

    for _ in range(config.scenario_default):
        delegatee = new_delegatee(
            config.contract_address, config.ledger, config.storage
        )

        addr = delegator._ledger_crypto.get_address()
        print(f"[D] Running end to end scenario for delegator {addr}")

        print(f"[D] Registering data {addr}")
        data_id = register_data(delegator)
        print(f"[D] Granting access to data {data_id} {addr}")
        grant_access(delegator, delegatee, data_id)
        print(f"[D] Reading data {data_id} from {addr}")
        read_data(delegatee, data_id)


def main():
    config, args = parse_commandline()

    delegator_count = [args for _ in range(config.owners_count)]

    proc_count = min(config.proc_count, config.owners_count * config.scenario_default)
    with multiprocessing.Pool(processes=proc_count) as pool:
        if config.scenario_default:
            pool.map(scenario_default, delegator_count)


if __name__ == "__main__":
    main()
