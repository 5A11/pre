from typing import Dict

import pytest

from tests.constants import (
    LOCAL_IPFS_CONFIG,
    LOCAL_LEDGER_CONFIG,
    TESTNET_IPFS_CONFIG,
    TESTNET_LEDGER_CONFIG,
)


class NodeConf:
    confs: Dict[str, Dict] = dict(
        local={"ledger": LOCAL_LEDGER_CONFIG, "storage": LOCAL_IPFS_CONFIG},
        testnet={"ledger": TESTNET_LEDGER_CONFIG, "storage": TESTNET_IPFS_CONFIG},
    )
    conf_selected: str = "local"

    @classmethod
    def set(cls, name):
        if name not in cls.confs:
            raise ValueError(
                f"Node conf `{name}` is not supported. Valid are {', '.join(cls.conf.keys())}"
            )
        cls.conf_selected = name

    @classmethod
    def get_conf_name(cls) -> str:
        return cls.conf_selected

    @classmethod
    def get_ledger_conf(cls) -> Dict:
        return cls.confs[cls.conf_selected]["ledger"]

    @classmethod
    def get_storage_conf(cls) -> Dict:
        return cls.confs[cls.conf_selected]["storage"]


@pytest.fixture(scope="session", autouse=True)
def node_conf(request) -> None:
    """Set nodes config for ledger and storage."""
    node_conf_option = request.config.getoption("--node-conf") or "local"
    NodeConf.set(node_conf_option)


def pytest_addoption(parser) -> None:
    """Add --node-conf option."""
    parser.addoption(
        "--node-conf",
        action="store",
        default="local",
        help="ledger and storage config options set",
    )
