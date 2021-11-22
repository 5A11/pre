from pre.api.admin import AdminAPI
from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.api.proxy import ProxyAPI
from pre.common import (
    PrivateKey,
)
from pre.contract.cosmos_contracts import (
    AdminContract,
    ContractQueries,
    DelegatorContract,
    ProxyContract,
)
from pre.crypto.umbral_crypto import UmbralCrypto, UmbralPrivateKey
from pre.ledger.cosmos.ledger import CosmosLedger
from pre.storage.ipfs_storage import IpfsStorage

from tests.constants import (
    DEFAULT_DENOMINATION,
    FETCHD_CHAIN_ID,
    FETCHD_URL,
    FUNDED_FETCHAI_PRIVATE_KEY_1,
    IPFS_HOST,
    IPFS_PORT,
    PREFIX,
)

def make_priv_key() -> PrivateKey:
    return UmbralPrivateKey.random()


def test_api():
    # with _fetchd_context(FETCHD_CONFIGURATION), IPFSDaemon():
    if 1:
        THRESHOLD = 1

        ipfs_storage = IpfsStorage(addr=IPFS_HOST, port=IPFS_PORT)
        ipfs_storage.connect()

        umbral_crypto = UmbralCrypto()

        n_max_proxies = 10
        ledger = CosmosLedger(
            denom=DEFAULT_DENOMINATION,
            chain_id=FETCHD_CHAIN_ID,
            prefix=PREFIX,
            node_address=FETCHD_URL,
            validator_pk=FUNDED_FETCHAI_PRIVATE_KEY_1,
        )
        # create crypto. admin is a validator, so ha some funds
        admin_ledger_crypto = ledger.load_crypto_from_str(FUNDED_FETCHAI_PRIVATE_KEY_1)
        delegator_ledger_crypto = ledger.make_new_crypto()
        proxy_ledger_crypto = ledger.make_new_crypto()

        # transfer funds to proxy and delegator
        ledger._send_funds(
            admin_ledger_crypto, delegator_ledger_crypto.get_address(), 10000
        )
        ledger._send_funds(
            admin_ledger_crypto, proxy_ledger_crypto.get_address(), 10000
        )

        contract_address = AdminContract.instantiate_contract(
            ledger=ledger,
            admin_private_key=admin_ledger_crypto,
            admin_addr=admin_ledger_crypto.get_address(),
            threshold=THRESHOLD,
            n_max_proxies=n_max_proxies,
        )
        admin_contract = AdminContract(ledger, contract_address)
        admin_api = AdminAPI(admin_ledger_crypto, admin_contract)

        delegator_priv_key = make_priv_key()
        delegator_contract = DelegatorContract(ledger, contract_address)
        delegator = DelegatorAPI(
            delegator_priv_key,
            delegator_ledger_crypto,
            delegator_contract,
            ipfs_storage,
            umbral_crypto,
        )

        proxy_priv_key = make_priv_key()
        proxy_contract = ProxyContract(ledger, contract_address)
        proxy = ProxyAPI(
            proxy_priv_key,
            proxy_ledger_crypto,
            proxy_contract,
            ipfs_storage,
            umbral_crypto,
        )
        admin_api.add_proxy(proxy_ledger_crypto.get_address())
        proxy.register()

        delegatee_priv_key = make_priv_key()
        delegatee_contract = ContractQueries(ledger, contract_address)
        delegatee = DelegateeAPI(
            delegatee_priv_key, delegatee_contract, ipfs_storage, umbral_crypto
        )

        data = b"some random bytes"
        hash_id = delegator.add_data(data)
        delegator.grant_access(
            hash_id=hash_id,
            delegatee_pubkey_bytes=bytes(delegatee_priv_key.public_key),
            threshold=THRESHOLD,
        )

        proxy_task = proxy.get_next_reencryption_request()
        assert proxy_task

        proxy.process_reencryption_request(proxy_task)

        decrypted_data = delegatee.read_data(
            hash_id, delegator_pubkey_bytes=bytes(delegator_priv_key.public_key)
        )

        assert decrypted_data == data
