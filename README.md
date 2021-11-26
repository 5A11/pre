# pre

Fetch Network Proxy Reencryption Service

## demo

```bash
git checkout demo
cd docker
```

### build images
```bash
docker-compose build admin owner proxy reader
```

### (optional, when running demo locally) start local ledger and ipfs nodes
```bash
docker-compose up --build
```

### deploy contract
```bash
docker-compose run admin
```
Take note of `$CONTRACT_ADDR`

### add data
```bash
echo "Some data to keep secret" > f.txt
docker-compose run owner --contract-address $CONTRACT_ADDR add-data /data/f.txt
```
Take note of `$DATA_ID`
### run proxies
```bash
# First proxy, only registers
docker-compose run proxy --contract-address  $CONTRACT_ADDR register
# second proxy, run as a daemon
PROXY_INDEX=2 docker-compose run proxy --contract-address $CONTRACT_ADDR run
```

### check data availability for reader
```bash
docker-compose run reader --contract-address $CONTRACT_ADDR get-data-status $DATA_ID
```
Take note of `$READER_PUBKEY`

### grant access to data
```bash
docker-compose run owner --contract-address $CONTRACT_ADDR grant-access $DATA_ID $READER_PUBKEY
```
Take note of `$OWNER_PUBKEY`

### reencrypt data by proxies
First proxy
```bash
docker-compose run proxy --contract-address $CONTRACT_ADDR run --run-once-and-exit
```
Second proxy will automatically process reencryption request.


### get data
```bash
docker-compose run reader --contract-address $CONTRACT_ADDR get-data-status $DATA_ID
docker-compose run reader --contract-address $CONTRACT_ADDR get-data $DATA_ID $OWNER_PUBKEY ff.txt
cat ff.txt
```
## testing

```bash
# deploy local fetch and ipfs nodes
docker-compose -f docker/docker-compose.yml up
# run tests (on another terminal)
docker-compose -f docker/docker-compose.yml run tests
# stop all containers
docker-compose -f docker/docker-compose.yml down
```


## development

### linting and code checks

Run `make lint` to format code, arrange imports, perform mypy, pylint, flake8 checks