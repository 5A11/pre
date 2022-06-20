from time import sleep

from pre.common import Coin, Delegation
from pre.contract.cosmos_contracts import AdminContract, DelegatorContract, ProxyContract
from pre.ledger.cosmos.ledger import CosmosLedger

cosmos_endpoint = CosmosLedger(
    chain_id="dorado-1",
    prefix="fetch",
    node_address="grpc-dorado.fetch.ai:443",
    denom="atestfet",
    faucet_url="https://faucet-dorado.fetch.ai",
    secure_channel=True

)

admin_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/admin_ledger.key")
proxy1_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_1_ledger.key")
proxy2_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_2_ledger.key")
proxy3_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_3_ledger.key")
proxy4_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_4_ledger.key")
proxy5_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_5_ledger.key")
delegator_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/owner_ledger.key")

print(admin_crypto.get_address())

cosmos_endpoint.ensure_funds(
    [
        admin_crypto.get_address(),
        proxy1_crypto.get_address(),
        proxy2_crypto.get_address(),
        proxy3_crypto.get_address(),
        proxy4_crypto.get_address(),
        proxy5_crypto.get_address(),
        delegator_crypto.get_address(),
    ]
)

contract_address = AdminContract.instantiate_contract(
    ledger=cosmos_endpoint,
    admin_private_key=admin_crypto,
    admin_addr=admin_crypto.get_address(),
    stake_denom="atestfet",
    threshold=1,
    timeout_height=10,
)
print(contract_address)

admin_contract = AdminContract(cosmos_endpoint, contract_address)
delegator_contract = DelegatorContract(cosmos_endpoint, contract_address)
proxy_contract = ProxyContract(cosmos_endpoint, contract_address)

staking_config = proxy_contract.get_staking_config()
minimum_registration_stake = Coin(
    denom=staking_config.stake_denom,
    amount=str(staking_config.minimum_proxy_stake_amount),
)

# Add data
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data1", b"capsule")
print("Add data1 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data2", b"capsule")
print("Add data2 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data3", b"capsule")
print("Add data3 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data4", b"capsule")
print("Add data4 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data5", b"capsule")
print("Add data5 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data6", b"capsule")
print("Add data6 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

# Register proxies
res = proxy_contract.proxy_register(proxy1_crypto, b"proxy1", minimum_registration_stake)
print("Register proxy1 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = proxy_contract.proxy_register(proxy2_crypto, b"proxy2", minimum_registration_stake)
print("Register proxy2 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = proxy_contract.proxy_register(proxy3_crypto, b"proxy3", minimum_registration_stake)
print("Register proxy3 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = proxy_contract.proxy_register(proxy4_crypto, b"proxy4", minimum_registration_stake)
print("Register proxy4 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
res = proxy_contract.proxy_register(proxy5_crypto, b"proxy5", minimum_registration_stake)
print("Register proxy5 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

# Add delegations
delegations = [Delegation(proxy_pub_key=b"proxy1", delegation_string=b"delestr1"),
               Delegation(proxy_pub_key=b"proxy2", delegation_string=b"delestr2"),
               Delegation(proxy_pub_key=b"proxy3", delegation_string=b"delestr3"),
               Delegation(proxy_pub_key=b"proxy4", delegation_string=b"delestr4"),
               Delegation(proxy_pub_key=b"proxy5", delegation_string=b"delestr5")
               ]

res = delegator_contract.add_delegations(delegator_crypto, b"delegator1", b"delegatee1", delegations)
print("Add delegation gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

# Request_reencryption

# Update state to get correct total_request_reward_amount
delegation_state_response = delegator_contract.get_delegation_status(
    delegator_pubkey_bytes=b"delegator1",
    delegatee_pubkey_bytes=b"delegatee1",
)

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data1",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption1 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data2",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption2 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data3",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption3 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data4",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption4 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data5",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption5 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id="data6",
    delegatee_pubkey_bytes=b"delegatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption6 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])


tasks = proxy_contract.get_proxy_tasks(b"proxy1")
print("Proxy 1 tasks: ", len(tasks))

print("Waiting for tasks to time out")
while len(tasks) > 0:
    sleep(10)
    tasks = proxy_contract.get_proxy_tasks(b"proxy1")
    print("Proxy1 tasks: ", len(tasks))


res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data7", b"capsule")
print("Add data7 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])

res = delegator_contract.add_data(delegator_crypto, b"delegator1", "data8", b"capsule")
print("Add data8 gas: ", res["txResponse"]["gasUsed"], "/", res["txResponse"]["gasWanted"])
