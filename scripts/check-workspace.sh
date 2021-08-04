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

purely_std_crates=("lang/codegen" "metadata")
also_wasm_crates=("env" "storage" "storage/derive" "allocator" "prelude" "primitives" "lang" "lang/macro" "lang/ir")
all_crates=("${also_wasm_crates[@]}" "${purely_std_crates[@]}")

results["check_all_features"]=true
for crate in "${all_crates[@]}"; do
    cargo check --verbose --all-features --manifest-path crates/$crate/Cargo.toml
    let "results['check_all_features'] |= $?"

    cargo build --verbose --all-features --release --manifest-path crates/${crate}/Cargo.toml
    let "results['build_std'] |= $?"
done

for crate in "${also_wasm_crates[@]}"; do
    cargo build --verbose --no-default-features --release --target wasm32-unknown-unknown --manifest-path crates/${crate}/Cargo.toml
    let "results['build_wasm'] |= $?"
done

cargo fmt --verbose --all -- --check
results["fmt"]=$?

cargo clippy --verbose --all --all-features -- -D warnings
results["clippy_all_features"]=$?

cargo test --verbose --all-features --no-fail-fast --workspace --release
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
