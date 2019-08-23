#!/bin/bash

pushd accumulator && ./build.sh && popd &&
pushd adder && ./build.sh && popd &&
pushd subber && ./build.sh && popd &&
pushd delegator && cargo build --release --features ink-generate-abi && ./build.sh && popd
