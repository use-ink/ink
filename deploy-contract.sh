#!/bin/bash

set -ex

F="${1:-pass}"

cargo contract build --manifest-path examples/flipper/Cargo.toml
cargo contract instantiate \
    --manifest-path examples/flipper/Cargo.toml \
    --suri //Alice --output-json \
    --constructor default \
    --salt $(date +%s) \
    --skip-confirm > /tmp/deployment-output.json

FLIPPER_ADDRESS=$(tail -n +2 /tmp/deployment-output.json | jq --raw-output .contract)
echo $FLIPPER_ADDRESS

cargo contract build --manifest-path examples/cross_chain_test/Cargo.toml
cargo contract instantiate \
     --manifest-path examples/cross_chain_test/Cargo.toml \
     --suri //Alice --output-json \
     --salt $(date +%s) \
     --skip-confirm > /tmp/deployment-output.json

 CROSS_ADDRESS=$(tail -n +2 /tmp/deployment-output.json | jq --raw-output .contract)
 echo $CROSS_ADDRESS

 ./call-contract.sh $F $CROSS_ADDRESS $FLIPPER_ADDRESS
