#!/bin/bash

# `find` will recursively list all the `Cargo.toml` manifests in the supplied dir.
# The first -exec command will attempt to build using the discovered Cargo.toml file. If it fails the second -exec
# command will not be run. This allows us to filter out any manifests for in the tree for non-contract crates.

find ./integration-tests/ -name "Cargo.toml" \
  -exec sh -c 'cargo contract build --manifest-path "$1" --release --quiet 2>/dev/null' _ {} \; \
  -exec sh -c 'cargo contract build --manifest-path "$1" --release --output-json | jq .dest_wasm' _ {} \; \
  -exec sh -c 'cargo contract build --manifest-path "$1" --release --output-json | jq .dest_wasm | xargs stat -c %s' _ {} \;
