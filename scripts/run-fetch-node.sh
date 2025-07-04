#!/usr/bin/env bash
set -e

fetchd init test-node --chain-id test
sed -i 's/stake/atestfet/' ~/.fetchd/config/genesis.json

fetchd config keyring-backend test
echo $FETCHMNEMONIC | fetchd keys add validator --recover
fetchd add-genesis-account $(fetchd keys show validator -a) 1152997575000000000000000000atestfet,100000000000000000000atestlearn
fetchd gentx validator 100000000000000000000atestfet --keyring-backend test --chain-id test
fetchd collect-gentxs

fetchd start --rpc.laddr tcp://0.0.0.0:26657