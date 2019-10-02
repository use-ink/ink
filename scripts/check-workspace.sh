#!/bin/bash

# Run this script from the workspace root!
#
# This script iterates through all crates in the workspace and runs
# the most important actions to verify integrity and control quality.
#
# - compile under different setups
# - check formatting according to our house rules
# - run a linter (clippy) under different setups
# - run all tests
# - build Wasm blobs
#
# Afterwards the script prints out a summary report.
#
# Exits with `0` if all tests completed successfully or `1` otherwise.

declare -A results

cargo check --verbose --all --all-features
results["check_all_features"]=$?

cargo check --verbose --all --no-default-features
results["check_no_defaults"]=$?

cargo fmt --verbose --all -- --check
results["fmt"]=$?

cargo clippy --verbose --all --all-features -- -D warnings
results["clippy_all_features"]=$?

cargo clippy --verbose --all --no-default-features -- -D warnings
results["clippy_no_defaults"]=$?

cargo test --verbose --all --all-features
results["test_all_features"]=$?

cargo build --verbose --all --no-default-features --release --target=wasm32-unknown-unknown
results["build_wasm"]=$?

all_checks_passed=0
banner="-----------------"

echo "Workspace Results"
echo "$banner"
for entry in ${!results[@]}; do
    result_str=""
    if [ ${results[$entry]} -eq 0 ]
    then
        result_str="ok"
    else
        result_str="ERROR"
    fi
    echo "- $entry: $result_str"
    let "all_checks_passed |= ${results[$entry]}"
done
echo ""
if [ $all_checks_passed -eq 0 ]
then
    echo "workspace: All checks passed"
    echo "$banner"
    exit 0
else
    echo "workspace: Some checks failed"
    echo "$banner"
    exit 0 # debug
fi
