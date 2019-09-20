#!/bin/bash

pushd accumulator && cargo contract build && popd &&
pushd adder && cargo contract build && popd &&
pushd subber && cargo contract build && popd &&
cargo contract build
