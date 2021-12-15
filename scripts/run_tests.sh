#!/bin/bash

pip3 install pytest
pytest /workdir/pre/tests/crypto/test_umbral_crypto.py
pytest --node-conf testnet tests/test_api.py