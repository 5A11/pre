from unittest.mock import ANY, Mock, patch

import pytest

from pre.api.admin import AdminAPI
from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.api.proxy import ProxyAPI
from pre.common import (
    Coin,
    ContractState,
    DelegationState,
    DelegationStatus,
    EncryptedData,
    GetFragmentsResponse,
    ProxyAvailability,
    ProxyTask,
    ReencryptionRequestState,
    StakingConfig,
)
from pre.crypto.base_crypto import AbstractCrypto
from pre.storage.base_storage import AbstractStorage


def test_delegator_api():
    contract = Mock()
    storage: AbstractStorage = Mock()
    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()
    ledger_crypto = Mock()
    delegator_api = DelegatorAPI(
        encryption_private_key, ledger_crypto, contract, storage, crypto
    )

    delegatee_pubkey_bytes = b"reader pub key"
    threshold = 2
    n_max_proxies = 10
    hash_id = "hash_id"
    data = b"bytes"
    crypto.encrypt.return_value = EncryptedData(data), b"capsule"
    storage.store_encrypted_data.return_value = hash_id
    encryption_private_key.public_key = b"pubkey"

    assert delegator_api.add_data(data) == hash_id
    crypto.encrypt.assert_called_once_with(
        crypto.encrypt.return_value[0].data, encryption_private_key.public_key
    )
    storage.store_encrypted_data.assert_called_once_with(crypto.encrypt.return_value[0])

    total_request_reward_amount = Coin(denom="atestfet", amount=str(100))

    contract.get_delegation_status.return_value = DelegationStatus(
        delegation_state=DelegationState.non_existing,
        total_request_reward_amount=total_request_reward_amount,
    )
    contract.get_available_proxies.return_value = []

    with pytest.raises(ValueError, match="proxies_list can not be empty"):
        delegator_api.grant_access(
            hash_id, delegatee_pubkey_bytes, threshold, n_max_proxies
        )

    contract.get_available_proxies.return_value = [
        ProxyAvailability(
            proxy_address="proxy", proxy_pubkey=b"proxy_pub_key", stake_amount="123"
        )
    ] * threshold
    delegator_api.grant_access(
        hash_id, delegatee_pubkey_bytes, threshold, n_max_proxies
    )


def test_delegatee_api():
    contract = Mock()
    storage: AbstractStorage = Mock()
    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()

    threshold = 2
    hash_id = "hash_id"
    data = b"bytes"
    capsule = b"capsule"
    delegator_pubkey_bytes = b"some"

    delegatee_api = DelegateeAPI(encryption_private_key, contract, storage, crypto)

    encryption_private_key.public_key = b"pubkey"

    contract.get_fragments_response.return_value = GetFragmentsResponse(
        threshold=threshold,
        fragments=[b""],
        capsule=capsule,
        reencryption_request_state=ReencryptionRequestState.inaccessible,
    )
    assert not delegatee_api.is_data_ready(hash_id)[0]
    contract.get_fragments_response.assert_called_once_with(
        hash_id=hash_id, delegatee_pubkey_bytes=encryption_private_key.public_key
    )

    contract.get_fragments_response.return_value = GetFragmentsResponse(
        threshold=threshold,
        fragments=[b""] * threshold,
        capsule=capsule,
        reencryption_request_state=ReencryptionRequestState.granted,
    )
    assert delegatee_api.is_data_ready(hash_id)[0]

    crypto.decrypt.return_value = data
    storage.get_encrypted_part.return_value = b"some part"
    storage.get_encrypted_data.return_value = EncryptedData(data)

    assert delegatee_api.read_data(hash_id, delegator_pubkey_bytes) == data
    storage.get_encrypted_part.call_count == threshold
    storage.get_encrypted_data.assert_called_once_with(hash_id)
    crypto.decrypt.assert_called_once_with(
        encrypted_data=storage.get_encrypted_data.return_value,
        encrypted_data_fragments_bytes=[b""] * threshold,
        capsule=capsule,
        delegatee_private_key=ANY,
        delegator_pubkey_bytes=delegator_pubkey_bytes,
    )


def test_admin_api():
    contract = Mock()
    storage: AbstractStorage = Mock()
    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()
    ledger_crypto = Mock()
    ledger = Mock()
    admin_api = AdminAPI(ledger_crypto, contract)

    threshold = 2
    hash_id = "hash_id"
    data = b"bytes"
    crypto.encrypt.return_value = EncryptedData(data), b"capsule"
    storage.store_encrypted_data.return_value = hash_id
    encryption_private_key.public_key = b"pubkey"
    proxies = ["some_proxy"]
    admin_address = "admin_addr"
    label = "PRE"
    proxy_addr = "proxy_addr"
    stake_denom = "atestfet"

    with patch.object(AdminAPI.CONTRACT_CLASS, "instantiate_contract") as mock:
        admin_api.instantiate_contract(
            ledger_crypto,
            ledger,
            admin_address,
            stake_denom,
            threshold,
            None,
            None,
            None,
            proxies,
            None,
            None,
            label,
        )
        mock.assert_called_once_with(
            ledger,
            ledger_crypto,
            admin_address,
            stake_denom,
            threshold,
            None,
            None,
            None,
            proxies,
            None,
            None,
            label,
        )

    admin_api.add_proxy(proxy_addr)
    contract.add_proxy.assert_called_once_with(ledger_crypto, proxy_addr)

    admin_api.remove_proxy(proxy_addr)
    contract.remove_proxy.assert_called_once_with(ledger_crypto, proxy_addr)


def test_proxy_api():
    contract = Mock()
    contract.get_contract_state.return_value = ContractState(
        admin="admin", threshold=1, terminated=False
    )

    contract.get_staking_config.return_value = StakingConfig(
        stake_denom="atestlearn",
        minimum_proxy_stake_amount="1000",
        per_proxy_task_reward_amount="100",
    )

    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()
    ledger_crypto = Mock()

    hash_id = "hash_id"
    delegator_pubkey_bytes = b"delegator_pubkey"
    delegatee_pubkey_bytes = b"degelatee_pubkey"
    encryption_private_key.public_key = b"pubkey"
    delegation_string = b"delegation string"
    proxy_api = ProxyAPI(encryption_private_key, ledger_crypto, contract, crypto)
    capsule = b"capsule"
    minimum_registration_stake = Coin(denom="atestlearn", amount=str(1000))

    proxy_api.register()
    contract.proxy_register.assert_called_once_with(
        proxy_private_key=ledger_crypto,
        proxy_pubkey_bytes=encryption_private_key.public_key,
        stake_amount=minimum_registration_stake,
    )

    proxy_api.unregister()
    contract.proxy_unregister.assert_called_once_with(ledger_crypto)

    proxy_task = ProxyTask(
        hash_id=hash_id,
        capsule=capsule,
        delegatee_pubkey=delegatee_pubkey_bytes,
        delegator_pubkey=delegator_pubkey_bytes,
        delegation_string=delegation_string,
    )

    contract.get_proxy_tasks.return_value = [proxy_task]
    assert proxy_api.get_reencryption_requests()[0] == proxy_task
    contract.get_proxy_tasks.assert_called_once_with(encryption_private_key.public_key)

    proxy_api.process_reencryption_request(proxy_task)

    crypto.reencrypt.assert_called_once_with(
        capsule_bytes=capsule,
        delegation_bytes=delegation_string,
        proxy_private_key=encryption_private_key,
        delegator_pubkey_bytes=delegator_pubkey_bytes,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
    )

    contract.provide_reencrypted_fragment.assert_called_once_with(
        proxy_private_key=ledger_crypto,
        hash_id=hash_id,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        fragment_bytes=ANY,
    )
