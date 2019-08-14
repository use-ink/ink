#!/bin/bash

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

echo "Workspace Results"
echo "-----------------"
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
    exit 0
else
    echo "workspace: Some checks failed"
    exit 1
fi
echo "-----------------"
