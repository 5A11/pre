#!/bin/bash

pip3 install pytest
pip3 install --upgrade docker
pytest --node-conf testnet tests/test_pre_api_end2end.py