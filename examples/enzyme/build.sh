#!/bin/bash

# The wasm-build executable that is used to tree-shake the wasm binary
# resulting from the cargo build in the first step expects to find it
# under target/release/wasm32-unknown-unknown/ in the cwd.

cargo +nightly build --release --target=wasm32-unknown-unknown --verbose
wasm-build target enyzme --target-runtime=substrate --final=adder --save-raw=./target/enzyme-deployed.wasm --target wasm32-unknown-unknown
