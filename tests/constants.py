DEFAULT_FETCH_DOCKER_IMAGE_TAG = "fetchai/fetchd:0.8.4"
DEFAULT_FETCH_LEDGER_ADDR = "http://127.0.0.1"
DEFAULT_FETCH_LEDGER_RPC_PORT = 26657
DEFAULT_FETCH_LEDGER_REST_PORT = 1317
DEFAULT_FETCH_ADDR_REMOTE = "https://rest-stargateworld.fetch.ai:443"
DEFAULT_FETCH_MNEMONIC = "gap bomb bulk border original scare assault pelican resemble found laptop skin gesture height inflict clinic reject giggle hurdle bubble soldier hurt moon hint"
DEFAULT_MONIKER = "test-node"
DEFAULT_FETCH_CHAIN_ID = "stargateworld-3"
DEFAULT_GENESIS_ACCOUNT = "validator"
DEFAULT_DENOMINATION = "atestfet"
FETCHD_INITIAL_TX_SLEEP = 6

PREFIX = "fetch"
FETCHD_CONFIGURATION = dict(
    mnemonic=DEFAULT_FETCH_MNEMONIC,
    moniker=DEFAULT_MONIKER,
    chain_id=DEFAULT_FETCH_CHAIN_ID,
    genesis_account=DEFAULT_GENESIS_ACCOUNT,
    denom=DEFAULT_DENOMINATION,
)
FUNDED_FETCHAI_PRIVATE_KEY_1 = (
    "bbaef7511f275dc15f47436d14d6d3c92d4d01befea073d23d0c2750a46f6cb3"
)

FETCHD_LOCAL_URL = "http://127.0.0.1:1317"
FETCHD_DC_URL = "http://fetchd:1317"
FETCHD_URL = FETCHD_DC_URL
FETCHD_DC_CHAIN_ID = "test"
FETCHD_CHAIN_ID = FETCHD_DC_CHAIN_ID

IPFS_DC_HOST = "ipfs"
IPFS_HOST = IPFS_DC_HOST
IPFS_PORT = 5001