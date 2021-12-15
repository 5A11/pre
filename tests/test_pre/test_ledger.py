from unittest.mock import patch

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
