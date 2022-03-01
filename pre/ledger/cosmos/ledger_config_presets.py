NODE_CONFIG_PRESETS = {
    "capricorn": dict(
        chain_id="capricorn-1",
        node_address="https://rest-capricorn.fetch.ai:443",
        faucet_url="https://faucet-capricorn.t-v2-london-c.fetch-ai.com",
        prefix="fetch",
        denom="atestfet",
        validator_pk="0ba1db680226f19d4a2ea64a1c0ea40d1ffa3cb98532a9fa366994bb689a34ae",
    ),
    "local_net": dict(
        chain_id="testing",
        node_address="http://127.0.0.1:1317",
        prefix="fetch",
        denom="stake",
        validator_pk="bbaef7511f275dc15f47436d14d6d3c92d4d01befea073d23d0c2750a46f6cb3",
        minimum_gas_price_amount=0,
    ),
}
