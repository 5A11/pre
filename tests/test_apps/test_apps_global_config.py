import os
import re
import shutil
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.case import TestCase

import prometheus_client
import yaml
from click.testing import CliRunner

from apps.admin import cli as admin_cli
from apps.keys import cli as keys_cli
from apps.owner import cli as owner_cli
from apps.proxy import cli as proxy_cli
from apps.reader import cli as reader_cli
from pre.ledger.cosmos.ledger import CosmosLedger, DEFAULT_FUNDS_AMOUNT

from tests.constants import (
    DEFAULT_DENOMINATION,
    DEFAULT_FETCH_CHAIN_ID,
    FETCHD_CONFIGURATION,
    FETCHD_LOCAL_URL,
    FUNDED_FETCHAI_PRIVATE_KEY_1,
    IPFS_PORT,
    PREFIX,
)
from tests.utils import IPFSDaemon, _fetchd_context


class TestApps(TestCase):
    ledger_config = dict(
        denom=DEFAULT_DENOMINATION,
        chain_id=DEFAULT_FETCH_CHAIN_ID,
        prefix=PREFIX,
        node_address=FETCHD_LOCAL_URL,
        validator_pk=FUNDED_FETCHAI_PRIVATE_KEY_1,
    )

    ipfs_config = dict(
        addr="localhost",
        port=IPFS_PORT,
    )

    THRESHOLD = 1
    runner = CliRunner()

    def setUp(self):
        self.tdir = Path(TemporaryDirectory().__enter__())
        os.mkdir(self.tdir)
        self.fetchd = _fetchd_context(FETCHD_CONFIGURATION)
        self.fetchd.__enter__()  # type: ignore
        self.ipfs = IPFSDaemon()
        self.ipfs.__enter__()  # type: ignore

        self.ipfs_config_file = self.tdir / "ipfs_config.yaml"
        self.ipfs_config_file.write_text(yaml.dump(self.ipfs_config))

        self.ledger_config_file = self.tdir / "ledger_config.yaml"
        self.ledger_config_file.write_text(yaml.dump(self.ledger_config))

        self.global_config = {
            "ledger_config": self.ledger_config,
            "ipfs_config": self.ipfs_config,
            "threshold": self.THRESHOLD,
        }
        self.global_config_file = self.tdir / "global_config.yaml"
        self.global_config_file.write_text(yaml.dump(self.global_config))

        self.admin_ledger_key = str(self.tdir / "admin_ledger.key")
        self.delegator_ledger_key = str(self.tdir / "delegator_ledger.key")
        self.proxy_ledger_key = str(self.tdir / "proxy_ledger.key")

        self.admin_encryption_key = str(self.tdir / "admin_encryption.key")
        self.delegator_encryption_key = str(self.tdir / "delegator_encryption.key")
        self.proxy_encryption_key = str(self.tdir / "proxy_encryption.key")
        self.delegatee_encryption_key = str(self.tdir / "delegatee_encryption.key")

    def make_ledger_key(self, filename):
        result = self.runner.invoke(
            keys_cli,
            [
                "generate-ledger-key",
                "--ledger-config",
                self.ledger_config_file,
                filename,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0
        assert re.search("Private key written", result.output)

    def make_encryption_key(self, filename):
        result = self.runner.invoke(
            keys_cli,
            [
                "generate-crypto-key",
                filename,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result
        assert re.search("Private key written", result.output), result

    def get_address_for_ledger_key(self, filename):
        result = self.runner.invoke(
            keys_cli,
            [
                "get-ledger-address",
                "-c",
                self.global_config_file,
                filename,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert re.match("Ledger address for key", result.output)
        return result.output.rstrip().split(" ")[-1]

    def get_pubkey_for_encryption_key(self, filename):
        result = self.runner.invoke(
            keys_cli,
            [
                "get-encryption-pubkey",
                filename,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0
        assert re.match("Public key hex for ", result.output)
        return result.output.rstrip().split(" ")[-1]

    def fund(self, address, amount=DEFAULT_FUNDS_AMOUNT):
        ledger = CosmosLedger(**self.ledger_config)
        funded_ledger_crypto = ledger.load_crypto_from_str(FUNDED_FETCHAI_PRIVATE_KEY_1)
        ledger.send_funds(funded_ledger_crypto, address, amount)

    def test_apps(self):
        self.make_encryption_key(self.admin_encryption_key)
        self.make_encryption_key(self.delegator_encryption_key)
        self.make_encryption_key(self.proxy_encryption_key)
        self.make_encryption_key(self.delegatee_encryption_key)

        self.make_ledger_key(self.admin_ledger_key)
        self.make_ledger_key(self.delegator_ledger_key)
        self.make_ledger_key(self.proxy_ledger_key)

        admin_address = self.get_address_for_ledger_key(self.admin_ledger_key)
        proxy_address = self.get_address_for_ledger_key(self.proxy_ledger_key)
        delegator_address = self.get_address_for_ledger_key(self.delegator_ledger_key)

        self.fund(admin_address)
        self.fund(proxy_address)
        self.fund(delegator_address)

        result = self.runner.invoke(
            admin_cli,
            [
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                self.admin_ledger_key,
                "instantiate-contract",
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0
        last_line = result.output.splitlines()[-1]
        assert re.match(
            "Contract was set successfully. Contract address is ", last_line
        )
        contract_address = last_line.rstrip().split(" ")[-1]

        self.global_config["contract_address"] = contract_address
        self.global_config_file.write_text(yaml.dump(self.global_config))

        result = self.runner.invoke(
            admin_cli,
            [
                "add-proxy",
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                self.admin_ledger_key,
                proxy_address,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert re.match("Proxy .* added", result.output)

        result = self.runner.invoke(
            proxy_cli,
            [
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                self.proxy_ledger_key,
                "--encryption-private-key",
                self.proxy_encryption_key,
                "register",
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert re.match("Proxy was registered", result.output)

        data = b"some random bytes"
        data_file = self.tdir / "data.file"
        data_file.write_bytes(data)

        result = self.runner.invoke(
            owner_cli,
            [
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                str(self.delegator_ledger_key),
                "--encryption-private-key",
                str(self.delegator_encryption_key),
                "add-data",
                str(data_file),
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert re.match("Data was settled: hash_id is", result.output)
        hash_id = result.output.rstrip().split(" ")[-1]

        delegatee_pubkey = self.get_pubkey_for_encryption_key(
            self.delegatee_encryption_key
        )
        result = self.runner.invoke(
            owner_cli,
            [
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                self.delegator_ledger_key,
                "--encryption-private-key",
                self.delegator_encryption_key,
                "grant-access",
                hash_id,
                delegatee_pubkey,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert re.match("Access to .* granted to ", result.output)

        # reset the underlying Prometheus registry
        prometheus_client.REGISTRY = prometheus_client.CollectorRegistry(
            auto_describe=True
        )

        result = self.runner.invoke(
            proxy_cli,
            [
                "-c",
                self.global_config_file,
                "--ledger-private-key",
                self.proxy_ledger_key,
                "--encryption-private-key",
                self.proxy_encryption_key,
                "run",
                "--run-once-and-exit",
                "--disable-metrics",
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert "Reencryption task processed" in result.output

        result = self.runner.invoke(
            reader_cli,
            [
                "-c",
                self.global_config_file,
                "--encryption-private-key",
                self.delegatee_encryption_key,
                "get-data-status",
                hash_id,
            ],
            catch_exceptions=False,
        )

        assert result.exit_code == 0, result.output
        assert re.search(
            f"Data {hash_id} is ready!", result.output, re.MULTILINE
        ), result.output

        result_data_file = self.tdir / "decrypted.data"
        result = self.runner.invoke(
            reader_cli,
            [
                "-c",
                self.global_config_file,
                "--encryption-private-key",
                self.delegatee_encryption_key,
                "get-data",
                hash_id,
                str(result_data_file),
            ],
            catch_exceptions=False,
        )

        assert result.exit_code == 0, result.output
        assert re.match(
            "Data .* decrypted and stored at .*", result.output
        ), result.output

        assert data == result_data_file.read_bytes()

    def tearDown(self):
        self.fetchd.__exit__(None, None, None)  # type: ignore
        self.ipfs.__exit__(None, None, None)  # type: ignore
        shutil.rmtree(self.tdir)
