from pre.common import Coin, Delegation
from pre.contract.cosmos_contracts import AdminContract, DelegatorContract, ProxyContract
from pre.ledger.cosmos.ledger import CosmosLedger, NodeConfigPreset


contract_filename = "../contract/cw_proxy_reencryption.wasm"

"""cosmos_endpoint = CosmosLedger(
    chain_id="test",
    prefix="fetch",
    node_address="localhost:9090",
    denom="atestfet",
    validator_pk="bbaef7511f275dc15f47436d14d6d3c92d4d01befea073d23d0c2750a46f6cb3",
)"""


cosmos_endpoint = CosmosLedger(
        chain_id="dorado-1",
        prefix="fetch",
        node_address="grpc-dorado.fetch.ai:443",
        denom = "atestfet",
        faucet_url="https://faucet-dorado.fetch.ai",
        secure_channel=True

    )

# For dorado use this instead of local_net one:
# cosmos_endpoint = CosmosEndpoint(NodeConfigPreset.dorado)

admin_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/admin_ledger.key")
proxy1_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_1_ledger.key")
proxy2_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/proxy_2_ledger.key")
delegator_crypto = cosmos_endpoint.load_crypto_from_file("../test_data/keys/owner_ledger.key")


print(admin_crypto.get_address())

cosmos_endpoint.ensure_funds(
    [
        admin_crypto.get_address(),
        proxy1_crypto.get_address(),
        proxy2_crypto.get_address(),
        delegator_crypto.get_address(),
    ]
)

#code_id, res = cosmos_endpoint.deploy_contract(admin_crypto, contract_filename)

#print(code_id)

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
res = delegator_contract.add_data(delegator_crypto,b"delegator1","data1",b"capsule")
print("Add data1 gas: ", res["txResponse"]["gasUsed"])
res = delegator_contract.add_data(delegator_crypto,b"delegator1","data2",b"capsule")
print("Add data2 gas: ", res["txResponse"]["gasUsed"])
res = delegator_contract.add_data(delegator_crypto,b"delegator1","data3",b"capsule")
print("Add data3 gas: ", res["txResponse"]["gasUsed"])
res = delegator_contract.add_data(delegator_crypto,b"delegator1","data4",b"capsule")
print("Add data4 gas: ", res["txResponse"]["gasUsed"])



# Register proxies
res = proxy_contract.proxy_register(proxy1_crypto,b"proxy1",minimum_registration_stake)
print("Register proxy1 gas: ", res["txResponse"]["gasUsed"])

proxy_contract.proxy_register(proxy2_crypto,b"proxy2",minimum_registration_stake)
print("Register proxy2 gas: ", res["txResponse"]["gasUsed"])


# Add delegations
delegations = [Delegation(proxy_pub_key=b"proxy1", delegation_string=b"delestr"), Delegation(proxy_pub_key=b"proxy2", delegation_string=b"delestr")]
res = delegator_contract.add_delegations(delegator_crypto,b"delegator1",b"delegatee1",delegations)
print("Add delegation gas: ", res["txResponse"]["gasUsed"])


# Request_reencryption

# Update state to get correct total_request_reward_amount
delegation_state_response = delegator_contract.get_delegation_status(
    delegator_pubkey_bytes=b"delegator1",
    delegatee_pubkey_bytes=b"delegatee1",
)

res = delegator_contract.request_reencryption(
    delegator_private_key=delegator_crypto,
    delegator_pubkey_bytes=b"delegator1",
    hash_id=b"data1",
    delegatee_pubkey_bytes=b"delgatee1",
    stake_amount=delegation_state_response.total_request_reward_amount,
)
print("Request reencryption1 gas: ", res["txResponse"]["gasUsed"])


tasks = proxy_contract.get_proxy_tasks(b"proxy1")
print("Proxy 1 tasks: ", len(tasks))