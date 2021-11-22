set -e

contract_root=$1
cwd=$(pwd)

cd $contract_root
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp target/wasm32-unknown-unknown/release/*.wasm $cwd
for contract in $(ls $cwd/*.wasm); do
    echo "Optimizing size of ${contract}"
    wasm-opt -Oz $contract -o $contract
done
