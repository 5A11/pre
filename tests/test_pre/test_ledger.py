import json
from unittest.mock import Mock, patch

import pytest

from pre.contract.cosmos_contracts import CosmosContract
from pre.ledger.base_ledger import LedgerServerNotAvailable
from pre.ledger.cosmos.ledger import (
    BroadcastException,
    CosmosLedger,
    CosmosLedgerConfig,
)


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


def test_server_not_available():
    conf = CosmosLedger.CONFIG_CLASS.make_default()
    conf["node_address"] = "http://127.0.0.1:55317"
    ledger = CosmosLedger(**conf)
    with pytest.raises(LedgerServerNotAvailable):
        ledger.check_availability()


def test_contract_validate_address():
    CosmosLedger.validate_address("fetch18vd9fpwxzck93qlwghaj6arh4p7c5n890l3amr")

    with pytest.raises(ValueError):
        CosmosLedger.validate_address("bad")


def test_error_handling():
    ledger = CosmosLedger("", "", "")
    with patch.object(
        ledger, "generate_tx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException,
            match="Failed to deploy contract code after multiple attempts",
        ):
            ledger.deploy_contract(
                Mock(),
                contract_filename=CosmosContract.ADMIN_CONTRACT.CONTRACT_WASM_FILE,
            )

    sleep_mock.assert_called()

    with patch.object(
        ledger, "generate_tx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException, match="Failed to init contract after multiple"
        ):
            ledger.send_init_msg(Mock(), code_id=1, init_msg={}, label="")

    sleep_mock.assert_called()

    with patch.object(
        ledger, "generate_tx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException,
            match="Getting contract state failed after multiple attempts",
        ):
            ledger.send_query_msg("", {})

    sleep_mock.assert_called()

    with patch.object(
        ledger, "generate_tx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException, match="Failed to execute contract after multiple"
        ):
            ledger.send_execute_msg(Mock(), "", {})

    sleep_mock.assert_called()

    tx = Mock()
    tx.SerializeToString.return_value = b""
    with patch.object(
        ledger.tx_client, "BroadcastTx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException, match="Broadcasting tx failed after multiple attempts"
        ):
            ledger.broadcast_tx(tx)
    sleep_mock.assert_called()

    response_mock = Mock()
    response_mock.tx_response.code = 1
    with patch.object(
        ledger.tx_client, "BroadcastTx", return_value=response_mock
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(BroadcastException, match="Transaction cannot be broadcast"):
            ledger.broadcast_tx(tx)

    with patch.object(
        ledger.tx_client, "GetTx", side_effect=BroadcastException("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException,
            match="Getting tx response failed after multiple attempts",
        ):
            ledger._make_tx_request("")

    sleep_mock.assert_called()

    account_response = Mock()
    account_response.account.Is.return_value = False
    with patch.object(ledger.auth_client, "Account", return_value=account_response):
        with pytest.raises(TypeError, match="Unexpected account type"):
            ledger._query_account_data("someaddr")

    with patch.object(
        ledger.auth_client, "Account", side_effect=Exception("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException,
            match="Getting account data failed after multiple attempts",
        ):
            ledger._query_account_data("someaddr")

    sleep_mock.assert_called()

    with patch.object(ledger, "get_balance", return_value=1), patch.object(
        ledger, "_send_funds"
    ) as sendfunds_mock, patch.object(ledger, "validator_crypto", Mock()):
        with pytest.raises(
            BroadcastException,
            match="Refilling funds from validator failed after multiple attempts",
        ):
            ledger._refill_wealth_from_validator(["someaddr"], 10000000)

    sendfunds_mock.assert_called()

    with patch.object(ledger, "validator_crypto", None):
        with pytest.raises(RuntimeError, match="Validator is not defined."):
            ledger._refill_wealth_from_validator(["someaddr"], 10000000)

    resp = Mock()
    resp.status_code = 200

    with patch.object(ledger, "get_balance", return_value=10000000), patch.object(
        ledger, "_sleep"
    ) as sleep_mock, patch("pre.ledger.cosmos.ledger.requests.post", return_value=resp):

        ledger._refill_wealth_from_faucet(["someaddr"], 10000000)

    sleep_mock.assert_called()

    with patch.object(ledger, "get_balance", return_value=5000000000), patch.object(
        ledger, "_sleep"
    ) as sleep_mock, patch("requests.post", return_value=resp):

        ledger._refill_wealth_from_faucet(["someaddr"])

    with patch.object(ledger, "get_balance", return_value=1), patch.object(
        ledger, "_sleep"
    ) as sleep_mock, patch(
        "pre.ledger.cosmos.ledger.requests.post", side_effect=Exception("oops")
    ):

        ledger._refill_wealth_from_faucet(["someaddr"], 10000000)

    sleep_mock.assert_called()

    resp.status_code = 400
    with patch.object(ledger, "get_balance", return_value=5000000000), patch.object(
        ledger, "_sleep"
    ) as sleep_mock, patch("requests.post", return_value=resp):
        ledger._refill_wealth_from_faucet(["someaddr"], 10000000)
    sleep_mock.assert_called()

    resp = Mock()
    resp.balance.amount = 100
    with patch.object(ledger.bank_client, "Balance", return_value=resp):
        assert ledger.get_balance("someaddr") == 100

    with patch.object(
        ledger.bank_client, "Balance", side_effect=Exception("oops")
    ), patch.object(ledger, "_sleep") as sleep_mock:
        with pytest.raises(
            BroadcastException, match="Getting balance failed after multiple attempts"
        ):
            ledger.get_balance("someaddr")
    sleep_mock.assert_called()

    with patch.object(ledger, "get_balance", return_value="5000"):
        assert ledger.query_funds("some addr") == "5000"

    with pytest.raises(
        RuntimeError,
        match="Faucet or validator was not specified, cannot refill addresses",
    ):
        ledger.ensure_funds(["someaddr"])

    with patch.object(ledger, "faucet_url", "some"), patch.object(
        ledger, "_refill_wealth_from_faucet"
    ) as mock:
        ledger.ensure_funds(["someaddr"])
    mock.assert_called()

    with patch.object(ledger, "validator_crypto", "some"), patch.object(
        ledger, "_refill_wealth_from_validator"
    ) as mock:
        ledger.ensure_funds(["someaddr"])
    mock.assert_called()


def test_config():
    CosmosLedger(**CosmosLedgerConfig.validate({}))


def test_ledger_load_file():
    ledger = CosmosLedger(**CosmosLedgerConfig.make_default())
    crypto = ledger.make_new_crypto()
    crypto.as_str()
    with patch("pathlib.Path.read_text", return_value=crypto.as_str()):
        ledger.load_crypto_from_file("test")


def test_check_availability():
    ledger = CosmosLedger(**CosmosLedgerConfig.make_default())
    with patch.object(
        ledger.rest_client,
        "get",
        return_value=json.dumps({"node_info": {"network": "some"}}),
    ):
        with pytest.raises(LedgerServerNotAvailable, match="Bad chain id"):
            ledger.check_availability()
