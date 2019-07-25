#!/bin/bash

# Deprecated Note:
# The wasm-build executable that is used to tree-shake the wasm binary
# resulting from the cargo build in the first step expects to find it
# under target/release/wasm32-unknown-unknown/ in the cwd.

PROJNAME=erc20
#cargo clean
#rm Cargo.lock
CARGO_INCREMENTAL=0 cargo +nightly build --no-default-features --release --target=wasm32-unknown-unknown --verbose
wasm2wat -o target/$PROJNAME.wat target/wasm32-unknown-unknown/release/$PROJNAME.wasm
cat target/$PROJNAME.wat | sed "s/(import \"env\" \"memory\" (memory (;0;) 2))/(import \"env\" \"memory\" (memory (;0;) 2 16))/" > target/$PROJNAME-fixed.wat
wat2wasm -o target/$PROJNAME.wasm target/$PROJNAME-fixed.wat

#wasm-build target enyzme --target-runtime=substrate --final=adder --save-raw=./target/enzyme-deployed.wasm --target wasm32-unknown-unknown
