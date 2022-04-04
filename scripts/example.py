from pre.contract.cosmos_pre_contract import ProxyReencryptionContract
from pre.ledger.cosmos_endpoint import CosmosEndpoint, NodeConfigPreset


contract_filename = "cw_proxy_reencryption.wasm"

cosmos_endpoint = CosmosEndpoint(
    NodeConfigPreset.local_net,
    validator_pk="0ba1db680226f19d4a2ea64a1c0ea40d1ffa3cb98532a9fa366994bb689a34ae",
)

# For dorado use this instead of local_net one:
# cosmos_endpoint = CosmosEndpoint(NodeConfigPreset.dorado)

delegator_crypto = cosmos_endpoint.make_crypto("delegator.key")
proxy1_crypto = cosmos_endpoint.make_crypto("proxy1.key")
proxy2_crypto = cosmos_endpoint.make_crypto("proxy2.key")

cosmos_endpoint.ensure_funds(
    [
        delegator_crypto.get_address(),
        proxy1_crypto.get_address(),
        proxy2_crypto.get_address(),
    ]
)
pre_contract = ProxyReencryptionContract.instantiate_contract(
    cosmos_endpoint, delegator_crypto, contract_filename
)

res = pre_contract.get_proxy_tasks(proxy1_crypto.get_address())
print(res)

res, err_code = pre_contract.add_proxy(delegator_crypto, proxy1_crypto.get_address())
print(err_code)
print(res)
