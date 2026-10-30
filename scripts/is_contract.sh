#!/bin/bash

SCRIPT_NAME="${BASH_SOURCE[0]}"
MANIFEST_PATH=$1

function usage {
  cat << EOF
Usage: ${SCRIPT_NAME} MANIFEST_PATH

Returns 0 for a probable contract, 1 for another package, and 2 for discovery errors. The heuristic used is:
  - Find the root package of the crate
  - Find the source file of the root package that is a lib
  - Check for the presence of the `#[ink::contract]` attribute macro

MANIFEST_PATH
  Path to the Cargo.toml manifest file for a possible contract project

EXAMPLES
  ${SCRIPT_NAME} ./Cargo.toml

EOF
}

if [ -z "$MANIFEST_PATH" ]; then
  usage
  exit 2
fi

# Discovery does not need the dependency graph. Resolution failures must not
# silently turn a contract into a skipped package.
MANIFEST_PATH=$(realpath "$MANIFEST_PATH") || exit 2
METADATA=$(cargo metadata --no-deps --format-version=1 --manifest-path "$MANIFEST_PATH") || exit 2
SOURCE_PATH=$(jq -r --arg manifest "$MANIFEST_PATH" '
  .packages[]
  | select(.manifest_path == $manifest).targets[]
  | select(any(.kind[]; contains("lib"))).src_path
' <<< "$METADATA") || exit 2

# Cargo packages have at most one library target.
if [ -z "$SOURCE_PATH" ]; then
    exit 1
fi

grep -q '^#\[ink::contract\([^]]*\)\]' "$SOURCE_PATH"
