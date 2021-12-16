from unittest.mock import ANY, Mock, patch

import pytest
from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin

from pre.api.admin import AdminAPI
from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.api.proxy import ProxyAPI
from pre.common import (
    DelegationState,
    EncryptedData,
    GetFragmentsResponse,
    ProxyTask,
    ReencryptionRequestState,
    GetDelegationStateResponse, ContractState,
)
from pre.contract.cosmos_contracts import AdminContract
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
    hash_id = "hash_id"
    data = b"bytes"
    crypto.encrypt.return_value = EncryptedData(data, b"capsule")
    storage.store_encrypted_data.return_value = hash_id
    encryption_private_key.public_key = b"pubkey"

    assert delegator_api.add_data(data) == hash_id
    crypto.encrypt.assert_called_once_with(
        crypto.encrypt.return_value.data, encryption_private_key.public_key
    )
    storage.store_encrypted_data.assert_called_once_with(crypto.encrypt.return_value)

    minimum_request_reward = Coin(denom="atestfet",
                                  amount=str(100))

    contract.get_delegation_state.return_value = GetDelegationStateResponse(
        delegation_state=DelegationState.non_existing,
        minimum_request_reward=minimum_request_reward)
    contract.get_selected_proxies_for_delegation.return_value = []
    contract.request_proxies_for_delegation.return_value = []

    with pytest.raises(ValueError, match="proxies_list can not be empty"):
        delegator_api.grant_access(hash_id, delegatee_pubkey_bytes, threshold)

    with pytest.raises(
            ValueError, match="not enought proxies: .* cause threshold is .*"
    ):
        contract.request_proxies_for_delegation.return_value = [b"proxy_pub_key"]
        delegator_api.grant_access(hash_id, delegatee_pubkey_bytes, threshold)

    contract.request_proxies_for_delegation.return_value = [
                                                               b"proxy_pub_key"
                                                           ] * threshold
    delegator_api.grant_access(hash_id, delegatee_pubkey_bytes, threshold)


def test_delegatee_api():
    contract = Mock()
    storage: AbstractStorage = Mock()
    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()

    threshold = 2
    hash_id = "hash_id"
    data = b"bytes"
    delegator_pubkey_bytes = b"some"

    delegatee_api = DelegateeAPI(encryption_private_key, contract, storage, crypto)

    encryption_private_key.public_key = b"pubkey"

    contract.get_fragments_response.return_value = GetFragmentsResponse(
        threshold=threshold,
        fragments=[b""],
        reencryption_request_state=ReencryptionRequestState.inaccessible,
    )
    assert not delegatee_api.is_data_ready(hash_id)[0]
    contract.get_fragments_response.assert_called_once_with(
        hash_id=hash_id, delegatee_pubkey_bytes=encryption_private_key.public_key
    )

    contract.get_fragments_response.return_value = GetFragmentsResponse(
        threshold=threshold,
        fragments=[b""] * threshold,
        reencryption_request_state=ReencryptionRequestState.granted,
    )
    assert delegatee_api.is_data_ready(hash_id)[0]

    crypto.decrypt.return_value = data
    storage.get_encrypted_part.return_value = b"some part"
    storage.get_encrypted_data.return_value = EncryptedData(data, b"capsule")

    assert delegatee_api.read_data(hash_id, delegator_pubkey_bytes) == data
    storage.get_encrypted_part.call_count == threshold
    storage.get_encrypted_data.assert_called_once_with(hash_id)
    crypto.decrypt.assert_called_once_with(
        encrypted_data=storage.get_encrypted_data.return_value,
        encrypted_data_fragments_bytes=[storage.get_encrypted_part.return_value]
                                       * threshold,
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

    delegatee_pubkey_bytes = b"reader pub key"
    threshold = 2
    hash_id = "hash_id"
    data = b"bytes"
    crypto.encrypt.return_value = EncryptedData(data, b"capsule")
    storage.store_encrypted_data.return_value = hash_id
    encryption_private_key.public_key = b"pubkey"
    proxies = ["some_proxy"]
    max_proxies = 5
    admin_address = "admin_addr"
    label = "PRE"
    proxy_addr = "proxy_addr"
    stake_denom = "atestfet"

    with patch.object(AdminAPI.CONTRACT_CLASS, "instantiate_contract") as mock:
        contract_address = admin_api.instantiate_contract(
            ledger_crypto, ledger, admin_address, stake_denom, threshold, None, None, max_proxies, proxies, label
        )
        mock.assert_called_once_with(
            ledger, ledger_crypto, admin_address, stake_denom, threshold, None, None, max_proxies, proxies, label
        )

    admin_api.add_proxy(proxy_addr)
    contract.add_proxy.assert_called_once_with(ledger_crypto, proxy_addr)

    admin_api.remove_proxy(proxy_addr)
    contract.remove_proxy.assert_called_once_with(ledger_crypto, proxy_addr)


def test_proxy_api():
    contract = Mock()
    contract.get_contract_state.return_value = ContractState(admin="admin",
                                                             threshold=1,
                                                             n_max_proxies=1,
                                                             stake_denom="atestfet",
                                                             minimum_proxy_stake_amount="1000",
                                                             minimum_request_reward_amount="100")

    storage: AbstractStorage = Mock()
    encryption_private_key = Mock()
    crypto: AbstractCrypto = Mock()
    ledger_crypto = Mock()

    threshold = 2
    hash_id = "hash_id"
    data = b"bytes"
    delegator_pubkey_bytes = b"delegator_pubkey"
    delegatee_pubkey_bytes = b"degelatee_pubkey"
    encryption_private_key.public_key = b"pubkey"
    delegation_string = b"delegation string"
    proxy_api = ProxyAPI(
        encryption_private_key, ledger_crypto, contract, storage, crypto
    )
    fragment_hash_id = b"fragment hash id"
    capsule = b"capsule"
    minimum_registration_stake = Coin(denom="atestfet",
                                      amount=str(1000))

    proxy_api.register()
    contract.proxy_register.assert_called_once_with(
        proxy_private_key=ledger_crypto,
        proxy_pubkey_bytes=encryption_private_key.public_key,
        stake_amount=minimum_registration_stake
    )

    proxy_api.unregister()
    contract.proxy_unregister.assert_called_once_with(ledger_crypto)

    proxy_task = ProxyTask(
        hash_id=hash_id,
        delegatee_pubkey=delegatee_pubkey_bytes,
        delegator_pubkey=delegator_pubkey_bytes,
        delegation_string=delegation_string,
    )

    contract.get_next_proxy_task.return_value = proxy_task
    assert proxy_api.get_next_reencryption_request() == proxy_task
    contract.get_next_proxy_task.assert_called_once_with(
        encryption_private_key.public_key
    )

    storage.get_capsule.return_value = capsule
    storage.store_encrypted_part.return_value = fragment_hash_id
    proxy_api.process_reencryption_request(proxy_task)
    storage.get_capsule.assert_called_once_with(proxy_task.hash_id)

    crypto.reencrypt.assert_called_once_with(
        capsule_bytes=capsule,
        delegation_bytes=delegation_string,
        proxy_private_key=encryption_private_key,
        delegator_pubkey_bytes=delegator_pubkey_bytes,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
    )

    storage.store_encrypted_part.assert_called_once_with(ANY)
    contract.provide_reencrypted_fragment.assert_called_once_with(
        proxy_private_key=ledger_crypto,
        hash_id=hash_id,
        delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        fragment_hash_id=fragment_hash_id,
    )
