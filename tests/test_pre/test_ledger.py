from unittest.mock import patch

import pytest

from pre.ledger.base_ledger import LedgerServerNotAvailable
from pre.ledger.cosmos.ledger import CosmosLedger


def test_crypto():
    ledger = CosmosLedger(**CosmosLedger.CONFIG_CLASS.make_default())
    crypto = ledger.make_new_crypto()

    with patch("pathlib.Path.write_text") as m:
        crypto.save_key_to_file("/root")

    m.assert_called_once_with(crypto.as_str())

    crypto.get_address()
    crypto.get_pubkey_as_bytes()
    crypto.get_pubkey_as_str()
    bytes(crypto)


def test_server_not_avaiable():
    conf = CosmosLedger.CONFIG_CLASS.make_default()
    conf["node_address"] = "http://127.0.0.1:55317"
    ledger = CosmosLedger(**conf)
    with pytest.raises(LedgerServerNotAvailable):
        ledger.check_availability()
