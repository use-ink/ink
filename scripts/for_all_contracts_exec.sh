#!/bin/bash

SCRIPT_NAME="${BASH_SOURCE[0]}"
FIND_PATH=$1

function usage {
  cat << EOF
Usage: ${SCRIPT_NAME} cmd

TODO

EOF
}

# enable recursive globs
shopt -s globstar

SCRIPTS_PATH=$( cd "$(dirname "$SCRIPT_NAME")" || exit; pwd -P )

for manifest_path in "$FIND_PATH"/**/Cargo.toml;
  do if "$SCRIPTS_PATH"/is_contract.sh "$manifest_path"; then
    echo Running: "${@:2}" "$manifest_path";
  fi
done
