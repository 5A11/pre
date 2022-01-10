#!/bin/bash

pip3 install pytest
pytest --node-conf testnet tests/test_pre_api_end2end.py