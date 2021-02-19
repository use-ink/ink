#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path accumulator/Cargo.toml
cargo +nightly contract build --manifest-path adder/Cargo.toml
cargo +nightly contract build --manifest-path subber/Cargo.toml
cargo +nightly contract build
