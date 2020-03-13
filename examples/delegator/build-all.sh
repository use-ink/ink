#!/usr/bin/env bash

pushd accumulator && cargo +nightly contract build && popd &&
pushd adder && cargo +nightly contract build && popd &&
pushd subber && cargo +nightly contract build && popd &&
cargo +nightly contract build
