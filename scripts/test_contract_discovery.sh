#!/bin/bash
set -euo pipefail

scripts_path=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT
export CARGO_NET_OFFLINE=true

expect_status() {
    local expected=$1 status=0
    shift
    "$@" > "$test_dir/output" 2>&1 || status=$?
    if [ "$status" -ne "$expected" ]; then
        cat "$test_dir/output" >&2
        echo "Expected exit $expected, got $status: $*" >&2
        exit 1
    fi
}

for package in contract helper; do
    directory="$test_dir/good/$package with spaces"
    mkdir -p "$directory"
    cat > "$directory/Cargo.toml" <<EOF
[package]
name = "$package"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[workspace]
[dependencies]
nonexistent_discovery_probe = "0.0.0"
EOF
done
printf '#[ink::contract]\nmod example {}\n' > "$test_dir/good/contract with spaces/lib.rs"
printf 'pub fn helper() {}\n' > "$test_dir/good/helper with spaces/lib.rs"

expect_status 0 "$scripts_path/is_contract.sh" "$test_dir/good/contract with spaces/Cargo.toml"
expect_status 1 "$scripts_path/is_contract.sh" "$test_dir/good/helper with spaces/Cargo.toml"
export DISCOVERY_OUTPUT="$test_dir/discovered"
expect_status 0 "$scripts_path/for_all_contracts_exec.sh" --path "$test_dir/good" -- \
    bash -c 'printf "%s\n" "$1" >> "$DISCOVERY_OUTPUT"' _ '{}'
[ "$(cat "$DISCOVERY_OUTPUT")" = "$test_dir/good/contract with spaces/Cargo.toml" ]
expect_status 1 "$scripts_path/for_all_contracts_exec.sh" --path "$test_dir/good" -- false

mkdir -p "$test_dir/broken" "$test_dir/empty"
printf 'invalid manifest' > "$test_dir/broken/Cargo.toml"
expect_status 2 "$scripts_path/is_contract.sh" "$test_dir/broken/Cargo.toml"
expect_status 2 "$scripts_path/for_all_contracts_exec.sh" --path "$test_dir/broken" -- true
expect_status 1 "$scripts_path/for_all_contracts_exec.sh" --path "$test_dir/empty" -- true

echo "Contract discovery tests passed"
