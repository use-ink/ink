#!/usr/bin/env bash

set -eu

cargo +stable contract build --manifest-path accumulator/Cargo.toml
cargo +stable contract build --manifest-path adder/Cargo.toml
cargo +stable contract build --manifest-path subber/Cargo.toml
cargo +stable contract build
