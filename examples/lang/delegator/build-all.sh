#!/bin/bash

pushd accumulator && ./build.sh && popd &&
pushd adder && ./build.sh && popd &&
pushd subber && ./build.sh && popd &&
./build.sh
