#!/usr/bin/env bash

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

all_crates=("core" "alloc" "utils" "model" "lang" "lang" "lang/macro" "cli")
wasm_crates=("core" "alloc" "utils" "model" "lang" "lang" "lang/macro")

results["check_all_features"]=true
for crate in "${all_crates[@]}"; do
    cargo check --verbose --all-features --manifest-path $crate/Cargo.toml
    let "results['check_all_features'] |= $?"
    cargo check --verbose --no-default-features --manifest-path $crate/Cargo.toml
    let "results['check_no_defaults'] |= $?"
done

for crate in "${wasm_crates[@]}"; do
    cargo build --verbose --manifest-path $crate/Cargo.toml --no-default-features --release --target=wasm32-unknown-unknown
    let "results['build_wasm'] |= $?"
done

cargo fmt --verbose --all -- --check
results["fmt"]=$?

cargo clippy --verbose --all --all-features -- -D warnings
results["clippy_all_features"]=$?

cargo clippy --verbose --all --no-default-features -- -D warnings
results["clippy_no_defaults"]=$?

cargo test --verbose --all --all-features --release
results["test_all_features"]=$?

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
    exit 1
fi
