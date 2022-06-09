from base64 import b64decode
from unittest.case import TestCase

import pytest

from pre.common import Coin, Delegation, DelegationState, ReencryptionRequestState
from pre.contract.base_contract import (
    BadContractAddress,
    ContractInstantiateFailure,
    ContractNotTerminated,
    ContractQueryError,
    ContractTerminated,
    DataAlreadyExist,
    DataEntryDoesNotExist,
    DelegationAlreadyExist,
    NotAdminError,
    NotEnoughProxies,
    NotEnoughStakeToWithdraw,
    ProxiesAreTooBusy,
    ProxyAlreadyExist,
    ProxyAlreadyRegistered,
    ProxyNotRegistered,
    QueryDataEntryDoesNotExist,
    ReencryptedCapsuleFragAlreadyProvided,
    ReencryptionAlreadyRequested,
    ReencryptionNotPermitted,
    UnknownProxy,
    UnkownReencryptionRequest,
)
from pre.contract.cosmos_contracts import (
    AdminContract,
    ContractQueries,
    CosmosContract,
    DelegatorContract,
    ProxyContract,
)
from pre.crypto.umbral_crypto import UmbralCrypto
from pre.ledger.cosmos.ledger import CosmosLedger
from pre.storage.ipfs_storage import IpfsStorage

from tests.constants import (
    DEFAULT_DENOMINATION,
    DEFAULT_FETCH_CHAIN_ID,
    DEFAULT_STAKE_DENOMINATION,
    DEFAULT_TESTS_FUNDS_AMOUNT,
    FETCHD_LOCAL_URL,
    FUNDED_FETCHAI_PRIVATE_KEY_1,
    PREFIX,
)
from tests.utils import local_ledger_and_storage


LOCAL_LEDGER_CONFIG = dict(
    denom=DEFAULT_DENOMINATION,
    stake_denom=DEFAULT_STAKE_DENOMINATION,
    chain_id=DEFAULT_FETCH_CHAIN_ID,
    prefix=PREFIX,
    node_address=FETCHD_LOCAL_URL,
    validator_pk=FUNDED_FETCHAI_PRIVATE_KEY_1,
)


class BaseContractTestCase(TestCase):
    FAKE_CONTRACT_ADDR = "fetch18vd9fpwxzck93qlwghaj6arh4p7c5n890l3amr"
    THRESHOLD = 1
    NUM_PROXIES = 10
    STAKE_DENOM = "atestlearn"

    @classmethod
    def setUpClass(self):
        self.node_confs = local_ledger_and_storage()
        self.ledger_config, self.ipfs_config = self.node_confs.__enter__()

        ipfs_storage = IpfsStorage(**self.ipfs_config)
        ipfs_storage.connect()

        self.crypto = UmbralCrypto()

        self.ledger = CosmosLedger(**self.ledger_config)
        self.validator = self.ledger.load_crypto_from_str(FUNDED_FETCHAI_PRIVATE_KEY_1)
        self.ledger_crypto = self.ledger.make_new_crypto()
        self.some_crypto = self.ledger.make_new_crypto()
        self.ledger.send_funds(
            self.validator,
            self.ledger_crypto.get_address(),
            amount=DEFAULT_TESTS_FUNDS_AMOUNT,
        )
        self.ledger.send_funds(
            self.validator,
            self.ledger_crypto.get_address(),
            amount=DEFAULT_TESTS_FUNDS_AMOUNT,
            denom=DEFAULT_STAKE_DENOMINATION,
        )
        self.ledger.send_funds(
            self.validator,
            self.some_crypto.get_address(),
            amount=DEFAULT_TESTS_FUNDS_AMOUNT,
        )
        self.ledger.send_funds(
            self.validator,
            self.some_crypto.get_address(),
            amount=DEFAULT_TESTS_FUNDS_AMOUNT,
            denom=DEFAULT_STAKE_DENOMINATION,
        )
        self.contract_addr = self._setup_a_contract()
        self.proxy_addr = self.ledger_crypto.get_address()
        self.proxy_pub_key = b"proxy pub key"

        self.proxy_pub_key2 = b"proxy pub key"
        self.proxy_crypto2 = self.ledger.make_new_crypto()
        self.proxy_addr2 = self.proxy_crypto2.get_address()

        self.delegator_pub_key = b64decode(
            "ApEPhAeq+TAL5aKiRkIpdoJ2pD+6qSt1RqHxGthT+XRY"
        )
        self.delegatee_pub_key = b64decode(
            "A5CYTfwD0EocpW4gCKtnP1lIFkMveO55v5+nbJaLqmLX"
        )
        self.hash_id = "hash_id"
        self.capsule = b64decode(
            "Ax83HFfEW1e+DW3KlikFLELPOVqYnlS39baHHC+/vsB4AmV+m1r9eZ6nCV9KXv7dSH+bSdWFbsqWFTxfF5qsjwObLtgsZUVSt8iv8UtkP0bLJs2sguElu4Syek6Seh3ZTj4="
        )
        self.degator_addr = self.ledger_crypto.get_address()
        self.fragment_bytes = b64decode(
            "Agn8MTBWSKzz277FLeNKvhOwa3juw7HBciLmyA/3kZ2hAtQv0l/B+Ej2vQLxZDx+MHDr5uevth9PzntoIz6gbPI1xJk3dVwZohs3YgdaXJsBXpAambF1FpOGrola7KcwjtQDOL6tYr3e6dlMgsW9GnONyZUWk15ixjxdrAIZfp8qWAMCbOd9fCO820cnEqBeQHpit75l8gxb6Al3s28p4uMFeq4Dzsh5SbQgRk7KjI9LEq2a9YzQ2ts3O5KEx3SuZoCOE0UDns625ayBRPD5BHdYwGaCGo/w6oJ5PvRp7rEpMSvxpOACu5HXcj2KNZnzAc2QGNrHmrAIxxS4pUbp7ffoPjSK/eGOs3Yh2IaeLQMzj2FNpUCYii6D3KJMT5sWqdKQV+5Aw6ebgujLY0o4Gs2aJ3toE3GuNuSfwFKzySmpq5CfSGaJJftZDYt72g7t8cRKVFXT6D8ugCXfMVL6GRE7adJEkYU="
        )

    @classmethod
    def _setup_a_contract(self):
        contract_address = AdminContract.instantiate_contract(
            ledger=self.ledger,
            admin_private_key=self.ledger_crypto,
            admin_addr=self.ledger_crypto.get_address(),
            stake_denom=self.STAKE_DENOM,
            per_task_slash_stake_amount="100",
            minimum_proxy_stake_amount="100",
            per_proxy_task_reward_amount="100",
            threshold=self.THRESHOLD,
            proxy_whitelisting=True,
        )
        assert contract_address
        return contract_address

    @classmethod
    def tearDownClass(self):
        self.node_confs.__exit__(None, None, None)  # type: ignore

    @property
    def proxy_contract(self):
        return ProxyContract(self.ledger, self.contract_addr)

    @property
    def admin_contract(self):
        return AdminContract(self.ledger, self.contract_addr)

    @property
    def delegator_contract(self):
        return DelegatorContract(self.ledger, self.contract_addr)

    @property
    def contract_queries(self):
        return ContractQueries(self.ledger, self.contract_addr)


class TestAdminContract(BaseContractTestCase):
    def test_contract_address_set_new(self):
        addr1 = self._setup_a_contract()
        addr2 = self._setup_a_contract()
        assert addr1 != addr2

    def test_bad_calls_for_bad_contract_addr(self):
        fake_addr = self.ledger.make_new_crypto().get_address()
        admin_contract = AdminContract(self.ledger, contract_address=fake_addr)
        with pytest.raises(
            BadContractAddress,
        ):
            admin_contract.add_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)
        with pytest.raises(
            BadContractAddress,
        ):
            admin_contract.remove_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)

    def test_add_proxy_remove_proxy(self):
        self.admin_contract.add_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)

        with pytest.raises(ProxyAlreadyExist):
            self.admin_contract.add_proxy(
                self.ledger_crypto, proxy_addr=self.proxy_addr
            )

        with pytest.raises(NotAdminError):
            self.admin_contract.add_proxy(
                self.some_crypto, proxy_addr=self.some_crypto.get_address()
            )
        self.admin_contract.remove_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)

        with pytest.raises(UnknownProxy):
            self.admin_contract.remove_proxy(
                self.ledger_crypto, proxy_addr=self.proxy_addr
            )

    def test_terminate_contract(self):
        assert not self.contract_queries.get_contract_state().terminated

        with pytest.raises(NotAdminError):
            self.admin_contract.terminate_contract(self.some_crypto)

        self.admin_contract.terminate_contract(self.ledger_crypto)

        assert self.contract_queries.get_contract_state().terminated

        with pytest.raises(ContractTerminated):
            self.admin_contract.terminate_contract(self.ledger_crypto)

    def test_withdraw_contract(self):
        # Need a new contract because it was terminated in previous test
        self.contract_addr = self._setup_a_contract()

        recipient_addr = self.ledger.make_new_crypto().get_address()

        with pytest.raises(ContractNotTerminated):
            self.admin_contract.withdraw_contract(self.ledger_crypto, recipient_addr)

        self.admin_contract.terminate_contract(self.ledger_crypto)

        with pytest.raises(NotEnoughStakeToWithdraw):
            self.admin_contract.withdraw_contract(self.ledger_crypto, recipient_addr)

    def test_get_contract_state(self):
        contract_state = self.contract_queries.get_contract_state()
        assert contract_state.threshold == self.THRESHOLD
        assert not contract_state.terminated

    def test_bad_set_contract(self):
        with pytest.raises(
            ContractInstantiateFailure,
            match="Error parsing into type cw_proxy_reencryption::msg::InstantiateMsg: Invalid number",
        ):
            AdminContract.instantiate_contract(
                self.ledger,
                self.ledger_crypto,
                self.ledger_crypto.get_address(),
                self.STAKE_DENOM,
                None,
                None,
                None,
                -1,
            )

    def test_bad_address_queries(self):
        fake_addr = self.ledger.make_new_crypto().get_address()
        contract_queries = ContractQueries(self.ledger, contract_address=fake_addr)
        with pytest.raises(ContractQueryError):
            contract_queries.get_contract_state()


class TestDelegatorContract(BaseContractTestCase):
    def test_add_delegation_add_reencryption_request(self):
        self.delegator_contract.add_data(
            self.ledger_crypto, self.delegator_pub_key, self.hash_id, self.capsule
        )

        with pytest.raises(DataAlreadyExist):
            self.delegator_contract.add_data(
                self.ledger_crypto, self.delegator_pub_key, self.hash_id, self.capsule
            )

        with pytest.raises(NotEnoughProxies):
            self.delegator_contract.add_delegations(
                self.ledger_crypto,
                self.delegator_pub_key,
                self.delegatee_pub_key,
                delegations=[],
            )
        assert (
            self.delegator_contract.get_delegation_status(
                delegator_pubkey_bytes=self.delegator_pub_key,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
            ).delegation_state
            == DelegationState.non_existing
        )

        staking_config = self.proxy_contract.get_staking_config()
        minimum_registration_stake = Coin(
            denom=staking_config.stake_denom,
            amount=str(staking_config.minimum_proxy_stake_amount),
        )

        with pytest.raises(UnknownProxy):
            self.proxy_contract.proxy_register(
                self.ledger_crypto,
                proxy_pubkey_bytes=self.proxy_pub_key,
                stake_amount=minimum_registration_stake,
            )

        self.admin_contract.add_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)

        with pytest.raises(ProxyAlreadyExist):
            self.admin_contract.add_proxy(
                self.ledger_crypto, proxy_addr=self.proxy_addr
            )

        assert not self.delegator_contract.get_available_proxies()

        self.proxy_contract.proxy_register(
            self.ledger_crypto,
            proxy_pubkey_bytes=self.proxy_pub_key,
            stake_amount=minimum_registration_stake,
        )
        # same proxy
        with pytest.raises(ProxyAlreadyRegistered):
            self.proxy_contract.proxy_register(
                self.ledger_crypto,
                proxy_pubkey_bytes=self.proxy_pub_key,
                stake_amount=minimum_registration_stake,
            )
        # different proxy addr, same pubkey
        with pytest.raises(UnknownProxy):
            self.proxy_contract.proxy_register(
                self.some_crypto,
                proxy_pubkey_bytes=self.proxy_pub_key,
                stake_amount=minimum_registration_stake,
            )

        self.admin_contract.add_proxy(self.ledger_crypto, proxy_addr=self.proxy_addr)
        with pytest.raises(ProxyAlreadyRegistered):
            self.proxy_contract.proxy_register(
                self.some_crypto,
                proxy_pubkey_bytes=self.proxy_pub_key,
                stake_amount=minimum_registration_stake,
            )

        assert self.delegator_contract.get_available_proxies()

        self.delegator_contract.add_delegations(
            self.ledger_crypto,
            self.delegator_pub_key,
            self.delegatee_pub_key,
            delegations=[
                Delegation(
                    proxy_pub_key=self.proxy_pub_key, delegation_string=b"somedata"
                )
            ],
        )

        with pytest.raises(DelegationAlreadyExist):
            self.delegator_contract.add_delegations(
                self.ledger_crypto,
                self.delegator_pub_key,
                self.delegatee_pub_key,
                delegations=[
                    Delegation(
                        proxy_pub_key=self.proxy_pub_key, delegation_string=b"somedata"
                    )
                ],
            )

        assert (
            self.delegator_contract.get_delegation_status(
                delegator_pubkey_bytes=self.delegator_pub_key,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
            ).delegation_state
            == DelegationState.active
        )

        assert not self.proxy_contract.get_proxy_tasks(
            proxy_address=self.ledger_crypto.get_address()
        )

        assert self.proxy_contract.get_contract_state()

        delegation_state_response = self.delegator_contract.get_delegation_status(
            delegator_pubkey_bytes=self.delegator_pub_key,
            delegatee_pubkey_bytes=self.delegatee_pub_key,
        )

        self.delegator_contract.request_reencryption(
            delegator_private_key=self.ledger_crypto,
            delegator_pubkey_bytes=self.delegator_pub_key,
            hash_id=self.hash_id,
            delegatee_pubkey_bytes=self.delegatee_pub_key,
            stake_amount=delegation_state_response.total_request_reward_amount,
        )

        with pytest.raises(DataEntryDoesNotExist):
            self.delegator_contract.request_reencryption(
                delegator_private_key=self.ledger_crypto,
                delegator_pubkey_bytes=self.delegator_pub_key,
                hash_id="Q",
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                stake_amount=delegation_state_response.total_request_reward_amount,
            )

        with pytest.raises(ReencryptionNotPermitted):
            self.delegator_contract.request_reencryption(
                delegator_private_key=self.validator,
                delegator_pubkey_bytes=self.delegator_pub_key,
                hash_id=self.hash_id,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                stake_amount=delegation_state_response.total_request_reward_amount,
            )

        with pytest.raises(ProxiesAreTooBusy):
            self.delegator_contract.request_reencryption(
                delegator_private_key=self.ledger_crypto,
                delegator_pubkey_bytes=self.delegator_pub_key,
                hash_id=self.hash_id,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                stake_amount=delegation_state_response.total_request_reward_amount,
            )

        staking_config = self.proxy_contract.get_staking_config()

        self.proxy_contract.add_stake(
            proxy_private_key=self.ledger_crypto,
            stake_amount=Coin(
                denom=staking_config.stake_denom,
                amount=str(staking_config.minimum_proxy_stake_amount),
            ),
        )

        proxy_status = self.proxy_contract.get_proxy_status(
            self.ledger_crypto.get_address()
        )
        assert proxy_status

        proxy_status = self.proxy_contract.get_proxy_status("some bad addr")
        assert not proxy_status

        proxy_task = self.proxy_contract.get_proxy_tasks(
            proxy_address=self.ledger_crypto.get_address()
        )
        assert proxy_task

        assert (
            self.contract_queries.get_fragments_response(
                self.hash_id, delegatee_pubkey_bytes=self.delegatee_pub_key
            ).reencryption_request_state
            == ReencryptionRequestState.ready
        )

        self.proxy_contract.provide_reencrypted_fragment(
            proxy_private_key=self.ledger_crypto,
            hash_id=self.hash_id,
            delegatee_pubkey_bytes=self.delegatee_pub_key,
            fragment_bytes=self.fragment_bytes,
        )

        with pytest.raises(ReencryptionAlreadyRequested):
            self.delegator_contract.request_reencryption(
                delegator_private_key=self.ledger_crypto,
                delegator_pubkey_bytes=self.delegator_pub_key,
                hash_id=self.hash_id,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                stake_amount=delegation_state_response.total_request_reward_amount,
            )

        assert (
            self.delegator_contract.get_delegation_status(
                delegator_pubkey_bytes=self.delegator_pub_key,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
            ).delegation_state
            == DelegationState.active
        )

        with pytest.raises(ReencryptedCapsuleFragAlreadyProvided):
            self.proxy_contract.provide_reencrypted_fragment(
                proxy_private_key=self.ledger_crypto,
                hash_id=self.hash_id,
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                fragment_bytes=self.fragment_bytes,
            )

        with pytest.raises(UnkownReencryptionRequest):
            self.proxy_contract.provide_reencrypted_fragment(
                proxy_private_key=self.ledger_crypto,
                hash_id="another hash id",
                delegatee_pubkey_bytes=self.delegatee_pub_key,
                fragment_bytes=self.fragment_bytes,
            )

        assert (
            self.fragment_bytes
            in self.contract_queries.get_fragments_response(
                self.hash_id, delegatee_pubkey_bytes=self.delegatee_pub_key
            ).fragments
        )

        # Data entry doesn't exist
        with pytest.raises(QueryDataEntryDoesNotExist):
            self.contract_queries.get_fragments_response(
                "bad hash id", delegatee_pubkey_bytes=self.delegatee_pub_key
            )

        # no errors
        self.contract_queries.get_fragments_response(
            self.hash_id, delegatee_pubkey_bytes=self.delegatee_pub_key + b"bad_pubkey"
        )

        self.proxy_contract.proxy_unregister(self.ledger_crypto)

        with pytest.raises(ProxyNotRegistered):
            self.proxy_contract.proxy_unregister(self.ledger_crypto)
        assert not self.delegator_contract.get_available_proxies()

        # test get data entry

        data_entry = self.contract_queries.get_data_entry(self.hash_id)
        assert data_entry
        # need a fix
        # assert data_entry.addr == self.degator_addr
        assert data_entry.pubkey == self.delegator_pub_key

        data_entry = self.contract_queries.get_data_entry("hashid not exists")
        assert not data_entry

        with pytest.raises(NotEnoughStakeToWithdraw):
            self.proxy_contract.withdraw_stake(self.ledger_crypto)

        with pytest.raises(NotEnoughStakeToWithdraw):
            self.proxy_contract.withdraw_stake(self.ledger_crypto, "1000")


def test_contract_validate_contract_address():
    CosmosContract.validate_contract_address(
        "fetch14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9szlkpka"
    )

    with pytest.raises(ValueError):
        CosmosContract.validate_contract_address("fetch18v")
