# Staging deployment liveness check
export CONT_ADDR=$( curl https://pre-relayer.s3.us-west-2.amazonaws.com/contract/latest.json | jq -r .contract_address) && python3 apps/proxy.py --fund --contract-address ${CONT_ADDR} --ledger-config ${LEDGER_CONFIG} --ledger-private-key ${PROXY_LEDGER} --encryption-private-key ${PROXY_ENCRYPTION} check-liveness



# PR-test liveness check
# export CONT_ADDR=$( curl https://pre-relayer.s3.us-west-2.amazonaws.com/contract/contract-test-${TAG}.json | jq -r .contract_address) 
# python3 ../apps/proxy.py --fund --contract-address ${CONT_ADDR} --ledger-config ${LEDGER_CONFIG} --ledger-private-key lk-${TAG}.key --encryption-private-key pk-${TAG}.key 