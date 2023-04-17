#!/usr/bin/env bash

./build-all.sh

set -x

ACCUMULATOR=$(cargo contract instantiate --manifest-path=accumulator/Cargo.toml --suri //Alice --skip-confirm --args 10 --output-json | jq -r .contract)
ADDER=$(cargo contract instantiate --manifest-path=adder/Cargo.toml --suri //Alice --skip-confirm --args $ACCUMULATOR --output-json | jq -r .contract)
SUBBER=$(cargo contract instantiate --manifest-path=subber/Cargo.toml --suri //Alice --skip-confirm --args $ACCUMULATOR --output-json | jq -r .contract)
DELEGATOR=$(cargo contract instantiate --manifest-path=./Cargo.toml --suri //Alice --skip-confirm --args $ACCUMULATOR $ADDER $SUBBER --output-json | jq -r .contract)

# Calling `delegator` contract and adding 100
cargo contract call --contract $DELEGATOR --message change --args 100 --suri //Alice --skip-confirm
cargo contract call --contract $DELEGATOR --message get --dry-run --suri //Alice --skip-confirm

# Calling `delegator` contract and substracting 10
cargo contract call --contract $DELEGATOR --message switch --suri //Alice --skip-confirm
cargo contract call --contract $DELEGATOR --message change --args 10 --suri //Alice --skip-confirm
cargo contract call --contract $DELEGATOR --message get --dry-run --suri //Alice --skip-confirm

# Calling `accumulator` contract `get` message to compare with `delegator` contract `get` message
cargo contract call --manifest-path=accumulator/Cargo.toml --contract $ACCUMULATOR --message get --dry-run --suri //Alice --skip-confirm
