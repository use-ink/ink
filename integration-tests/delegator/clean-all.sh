#!/usr/bin/env bash

set -eu

cargo clean --manifest-path accumulator/Cargo.toml
cargo clean --manifest-path adder/Cargo.toml
cargo clean --manifest-path subber/Cargo.toml
cargo clean
