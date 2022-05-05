NODE_CONFIG_PRESETS = {
    "dorado": dict(
        chain_id="dorado-1",
        node_address="grpc-dorado.fetch.ai:443",
        faucet_url="https://faucet-dorado.fetch.ai",
        prefix="fetch",
        denom="atestfet",
        minimum_gas_price_amount=500000000000,
        secure_channel=True,
    ),
    "local_net": dict(
        chain_id="testing",
        node_address="localhost:9090",
        prefix="fetch",
        denom="stake",
        minimum_gas_price_amount=0,
        secure_channel=False,
    ),
}
