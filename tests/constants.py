DEFAULT_FETCH_DOCKER_IMAGE_TAG = "fetchai/fetchd:0.10.4"
DEFAULT_FETCH_LEDGER_ADDR = "localhost"
DEFAULT_FETCH_LEDGER_RPC_PORT = 9090
DEFAULT_FETCH_ADDR_REMOTE = "grpc-dorado.fetch.ai:443"
DEFAULT_FETCH_MNEMONIC = "gap bomb bulk border original scare assault pelican resemble found laptop skin gesture height inflict clinic reject giggle hurdle bubble soldier hurt moon hint"
DEFAULT_MONIKER = "test-node"
DEFAULT_FETCH_CHAIN_ID = "dorado-1"
DEFAULT_GENESIS_ACCOUNT = "validator"
DEFAULT_DENOMINATION = "atestfet"
FETCHD_INITIAL_TX_SLEEP = 6
DEFAULT_TESTS_FUNDS_AMOUNT = 20 * 10**18

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

FETCHD_LOCAL_URL = "localhost:9090"
FETCHD_DC_URL = "fetchd:9090"
FETCHD_URL = FETCHD_DC_URL
FETCHD_DC_CHAIN_ID = "test"
FETCHD_CHAIN_ID = FETCHD_DC_CHAIN_ID

IPFS_DC_HOST = "ipfs"
IPFS_HOST = IPFS_DC_HOST
IPFS_PORT = 5001


LOCAL_LEDGER_CONFIG = dict(
    denom=DEFAULT_DENOMINATION,
    chain_id=DEFAULT_FETCH_CHAIN_ID,
    prefix=PREFIX,
    node_address=FETCHD_LOCAL_URL,
    validator_pk=FUNDED_FETCHAI_PRIVATE_KEY_1,
)

LOCAL_IPFS_CONFIG = dict(
    addr="localhost",
    port=IPFS_PORT,
)


TESTNET_IPFS_CONFIG = dict(
    addr=IPFS_DC_HOST,
    port=IPFS_PORT,
)

TESTNET_LEDGER_CONFIG = dict(
    denom=DEFAULT_DENOMINATION,
    chain_id=FETCHD_CHAIN_ID,
    prefix=PREFIX,
    node_address=FETCHD_URL,
    validator_pk=FUNDED_FETCHAI_PRIVATE_KEY_1,
)
