#!/usr/bin/env

pushd accumulator && cargo contract build && popd &&
pushd adder && cargo contract build && popd &&
pushd subber && cargo contract build && popd &&
cargo contract build
