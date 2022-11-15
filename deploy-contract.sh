#!/bin/bash

set -ex

F="${1:-pass}"

cargo contract build --manifest-path examples/flipper/Cargo.toml --offline
cargo contract instantiate \
    --manifest-path examples/flipper/Cargo.toml \
    --suri //Alice --output-json \
    --constructor default \
    --salt $(date +%s) \
    --skip-confirm > /tmp/deployment-output.json

FLIPPER_ADDRESS=$(tail -n +2 /tmp/deployment-output.json | jq --raw-output .contract)
echo $FLIPPER_ADDRESS

cargo contract build --manifest-path examples/lang_err_integration_tests/Cargo.toml --offline
cargo contract instantiate \
     --manifest-path examples/lang_err_integration_tests/Cargo.toml \
     --suri //Alice --output-json \
     --salt $(date +%s) \
     --skip-confirm > /tmp/deployment-output.json

 INTEGRATION_ADDRESS=$(tail -n +2 /tmp/deployment-output.json | jq --raw-output .contract)
 echo $INTEGRATION_ADDRESS

 ./call-contract.sh $F $INTEGRATION_ADDRESS $FLIPPER_ADDRESS
