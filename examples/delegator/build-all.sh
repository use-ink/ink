#!/usr/bin/env bash

set -eu

cargo +stable contract build --manifest-path accumulator/Cargo.toml --skip-linting
cargo +stable contract build --manifest-path adder/Cargo.toml --skip-linting
cargo +stable contract build --manifest-path subber/Cargo.toml --skip-linting
cargo +stable contract build --skip-linting
