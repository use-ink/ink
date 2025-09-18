#!/bin/bash

set -eu

SCRIPT_NAME="${BASH_SOURCE[0]}"
MANIFEST_PATH=$1

function usage {
  cat << EOF
Usage: ${SCRIPT_NAME} MANIFEST_PATH

Succeeds if the crate at MANIFEST_PATH is *probably* contract crate, fails otherwise. The heuristic used is:
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
  exit 1
fi

ROOT_PACKAGE=$(cargo metadata --format-version=1 --manifest-path "$MANIFEST_PATH" |
  jq -r '.resolve.root')
SOURCE_PATH=$(cargo metadata --format-version=1 --manifest-path "$MANIFEST_PATH" |
  jq -r --arg ROOT_PACKAGE "$ROOT_PACKAGE" '
    .packages[]
    | select(.id == $ROOT_PACKAGE).targets[]
    | select(.kind[] | contains("lib")).src_path')

# Check if SOURCE_PATH is empty
if [ -z "$SOURCE_PATH" ]; then
  >&2 echo "Error: Source path is empty for $MANIFEST_PATH."
  # we exit with 0 as this happens e.g. for the `Cargo.toml` that
  # denotes a workspace (or e2e/macro/Cargo.toml, linting/Cargo.toml,
  # ink/macro/Cargo.toml).
  # so it's not an error per se, we just don't consider this a contract then.
  exit 0
fi

# Check for the #[ink::contract] macro in the source file
if grep -qE '^#\[(::)?ink::contract([^]]*)\]' "$SOURCE_PATH"; then
    exit 0
else
    # we exit with a defined error code > 1, to allow a distinction
    # between `is_contract == false` and "error while checking".
    exit 3
fi
