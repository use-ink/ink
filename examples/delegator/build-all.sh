#!/usr/bin/env bash

pushd accumulator && cargo +nightly contract build --generate code-only && popd &&
pushd adder && cargo +nightly contract build --generate code-only && popd &&
pushd subber && cargo +nightly contract build --generate code-only && popd &&
cargo +nightly contract build
