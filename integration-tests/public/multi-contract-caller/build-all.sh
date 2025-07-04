#!/usr/bin/env bash

set -eu

cargo-contract-nightly contract build --manifest-path accumulator/Cargo.toml
cargo-contract-nightly contract build --manifest-path adder/Cargo.toml
cargo-contract-nightly contract build --manifest-path subber/Cargo.toml
cargo-contract-nightly contract build
