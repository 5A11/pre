# This contract is heavily under construction

# A proxy-reencryption contract

# How to setup wasm, deploy contract and interact with it

Before getting started install rust using the official [setup guide](https://doc.rust-lang.org/book/ch01-01-installation.html). 
You will also need to run the following commands to set up rust:
```
rustup default stable
cargo version
# If this is lower than 1.52.0+, update
rustup update stable

rustup target list --installed
rustup target add wasm32-unknown-unknown
```

Install Fetchd
```
git clone https://github.com/fetchai/fetchd.git
cd fetchd
git checkout v0.8.7
make install

# Check if wasmcli is properly installed
fetchd version
# Version should be 0.8.7
```

Compile contract
```
RUSTFLAGS='-C link-arg=-s' cargo wasm
# Compiled .wasm binary will be in /target/wasm32-unknown-unknown/release
```


