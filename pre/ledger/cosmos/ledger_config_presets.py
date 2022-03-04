NODE_CONFIG_PRESETS = {
    "capricorn": dict(
        chain_id="capricorn-1",
        node_address="https://rest-capricorn.fetch.ai:443",
        faucet_url="https://faucet-capricorn.t-v2-london-c.fetch-ai.com",
        prefix="fetch",
        denom="atestfet",
    ),
    "local_net": dict(
        chain_id="testing",
        node_address="http://127.0.0.1:1317",
        prefix="fetch",
        denom="stake",
        minimum_gas_price_amount=0,
    ),
}
