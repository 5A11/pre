#!/usr/bin/env bash
export VALIDATOR_KEY_NAME=validator
export VALIDATOR_MNEMONIC="gap bomb bulk border original scare assault pelican resemble found laptop skin gesture height inflict clinic reject giggle hurdle bubble soldier hurt moon hint"
export PASSWORD="12345678"
export CHAIN_ID=dorado-1
export MONIKER=test-node
export DENOM=atestfet
( echo "$VALIDATOR_MNEMONIC"; echo "$PASSWORD"; echo "$PASSWORD"; ) |fetchd keys add $VALIDATOR_KEY_NAME --recover
fetchd init --chain-id=$CHAIN_ID $MONIKER
echo "$PASSWORD" |fetchd add-genesis-account $(fetchd keys show $VALIDATOR_KEY_NAME -a) 100000000000000000000000$DENOM
echo "$PASSWORD" |fetchd gentx $VALIDATOR_KEY_NAME 10000000000000000000000$DENOM --chain-id $CHAIN_ID
fetchd collect-gentxs
sed -i "s/stake/atestfet/" ~/.fetchd/config/genesis.json
sed -i "s/swagger = false/swagger = true/" ~/.fetchd/config/app.toml
sed -i "s/swagger = false/swagger = true/" ~/.fetchd/config/app.toml
sed -i "s/3s/1s/" ~/.fetchd/config/config.toml
sed -i "s/5s/1s/" ~/.fetchd/config/config.toml
fetchd start