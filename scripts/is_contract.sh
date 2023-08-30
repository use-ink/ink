#!/bin/bash

SCRIPT_NAME="${BASH_SOURCE[0]}"
MANIFEST_PATH=$1

function usage {
  cat << EOF
Usage: ${SCRIPT_NAME} MANIFEST_PATH

Succeeds if the crate at MANIFEST_PATH is a contract crate, fails otherwise.

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

if grep -q '^#\[ink::contract\]' $SOURCE_PATH; then
    exit 0
else
    exit 1
fi
