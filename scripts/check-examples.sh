#!/usr/bin/env bash

# Run this script from the workspace root!
#
# This script iterates through all examples and runs the most important
# actions performed on an ink! smart contract:
#
# - compile to a Wasm blob using `no_std` environment
# - run offchain tests
# - generate metadata JSON
#
# Afterwards the script prints out a summary report.
#
# Exits with `0` if all tests completed successfully or `1` otherwise.

declare -A results_test
declare -A results_wasm
declare -A results_metadata

all_checks_passed=0

build() {
    example=$1
    pushd $example

    cargo contract build
    result_wasm=$?
    let "all_checks_passed |= $result_wasm"
    if [ $result_wasm -eq 0 ]
    then
        results_wasm[$example]="ok"
    else
        results_wasm[$example]="ERROR"
    fi
    popd
}

run_tests() {
    example=$1
    pushd $example

    cargo test --verbose
    result_test=$?
    let "all_checks_passed |= $result_test"
    if [ $result_test -eq 0 ]
    then
        results_test[$example]="ok"
    else
        results_test[$example]="ERROR"
    fi

    popd
}

metadata() {
    example=$1
    pushd $example

    cargo contract generate-metadata
    result_metadata=$?
    let "all_checks_passed |= $result_metadata"
    if [ $result_metadata -eq 0 ]
    then
        results_metadata[$example]="ok"
    else
        results_metadata[$example]="ERROR"
    fi

    popd
}

for example in $(ls -d examples/*/ | grep -v delegator); do
    build $example
    run_tests $example
    metadata $example
done

# the delegator is a special case, we need to build it's sub-contracts first
for example in $(ls -d examples/delegator/{accumulator,adder,subber}/); do
    build $example
    run_tests $example
done
build examples/delegator/
run_tests examples/delegator/
metadata examples/delegator/

banner="---------------"
echo "Example Results"
echo "$banner"
for entry in ${!results_wasm[@]}; do
    echo "- $entry (wasm):     ${results_wasm[$entry]}"
    echo "- $entry (test):     ${results_test[$entry]}"
    echo "- $entry (metadata): ${results_metadata[$entry]}"
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
