#!/usr/bin/env bash

set -eu

cargo contract build --manifest-path accumulator/Cargo.toml
cargo contract build --manifest-path adder/Cargo.toml
cargo contract build --manifest-path subber/Cargo.toml
cargo contract build
