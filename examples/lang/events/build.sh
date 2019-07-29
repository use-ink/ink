#!/bin/bash

PROJNAME=events

# cargo clean
# rm Cargo.lock

CARGO_INCREMENTAL=0 cargo +nightly build --no-default-features --release --features generate-api-description --target=wasm32-unknown-unknown --verbose
wasm2wat -o target/$PROJNAME.wat target/wasm32-unknown-unknown/release/$PROJNAME.wasm
cat target/$PROJNAME.wat | sed "s/(import \"env\" \"memory\" (memory (;0;) 2))/(import \"env\" \"memory\" (memory (;0;) 2 16))/" > target/$PROJNAME-fixed.wat
wat2wasm -o target/$PROJNAME.wasm target/$PROJNAME-fixed.wat
wasm-prune --exports call,deploy target/$PROJNAME.wasm target/$PROJNAME-pruned.wasm
