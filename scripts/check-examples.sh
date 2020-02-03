#!/usr/bin/env bash

# Run this script from the workspace root!
#
# This script iterates through all examples under `examples/lang` folder
# and runs the most important actions performed on an ink! smart contract:
#
# - compile to a Wasm blob using `no_std` environment
# - run offchain tests
# - generate ABI file using `ink-generate-abi`
#
# Afterwards the script prints out a summary report.
#
# Exits with `0` if all tests completed successfully or `1` otherwise.

declare -A results_test
declare -A results_wasm
declare -A results_abi

all_checks_passed=0
for example in $(ls examples/lang); do
    cargo build --release --no-default-features --target=wasm32-unknown-unknown --verbose --manifest-path examples/lang/$example/Cargo.toml
    result_wasm=$?
    let "all_checks_passed |= $result_wasm"
    if [ $result_wasm -eq 0 ]
    then
        results_wasm[$example]="ok"
    else
        results_wasm[$example]="ERROR"
    fi
    cargo test --verbose --manifest-path examples/lang/$example/Cargo.toml
    result_test=$?
    let "all_checks_passed |= $result_test"
    if [ $result_test -eq 0 ]
    then
        results_test[$example]="ok"
    else
        results_test[$example]="ERROR"
    fi
    cargo run --package abi-gen --manifest-path examples/lang/$example/Cargo.toml
    result_abi=$?
    let "all_checks_passed |= $result_abi"
    if [ $result_abi -eq 0 ]
    then
        results_abi[$example]="ok"
    else
        results_abi[$example]="ERROR"
    fi
done

banner="---------------"
echo "Example Results"
echo "$banner"
for entry in ${!results_wasm[@]}; do
    echo "- $entry (wasm): ${results_wasm[$entry]}"
    echo "- $entry (test): ${results_test[$entry]}"
    echo "- $entry (abi) : ${results_abi[$entry]}"
done
echo ""
if [ $all_checks_passed -eq 0 ]
then
    echo "examples: All checks passed"
    echo "$banner"
    exit 0
else
    echo "examples: Some checks failed"
    echo "$banner"
    exit 1
fi
