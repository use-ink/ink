#!/bin/bash

SCRIPT_NAME="${BASH_SOURCE[0]}"
MANIFEST_PATH=$1

function usage {
  cat << EOF
Usage: ${SCRIPT_NAME} MANIFEST_PATH

Build and print the contract name and size for the given manifest path, if it is a valid ink! contract project.
Use with `find` (see EXAMPLES) to print the contract name and size for all ink! contracts in a directory.

MANIFEST_PATH
  Path to the Cargo.toml manifest file for a contract project

EXAMPLES
  ${SCRIPT_NAME} ./Cargo.toml
  find ./integration-tests/ -name "Cargo.toml" -exec ${SCRIPT_NAME} {} \;

EOF
}

if [ -z "$MANIFEST_PATH" ]; then
  usage
  exit 1
fi

BUILD_RESULT=$(cargo contract build --manifest-path "$MANIFEST_PATH" --release --quiet --output-json 2>/dev/null)

if [ $? -eq 0 ]; then
    # only print the contract name and size if the build was successful
    DEST_WASM=$(echo "$BUILD_RESULT" | jq -r .dest_wasm)
    CONTRACT_NAME=$(basename "$DEST_WASM" .wasm)
    CONTRACT_SIZE=$(stat -c %s "$DEST_WASM")

    echo "$CONTRACT_NAME" "$CONTRACT_SIZE"
fi
