#!/bin/bash

script_name="${BASH_SOURCE[0]}"
scripts_path=$( cd "$(dirname "$script_name")" || exit; pwd -P )
manifest_path=$1

function usage {
  cat << EOF
Usage: ${script_name} manifest_path

Build and print the contract name and size for the given manifest path, if it is a valid ink! contract project.

Use with '${scripts_path}/for_all_contracts_exec.sh' (see EXAMPLES) to print the contract name and size for all ink! contracts in a directory.

manifest_path
  Path to the Cargo.toml manifest file for a contract project

EXAMPLES
  ${script_name} ./Cargo.toml
  ${scripts_path}/for_all_contracts_exec.sh --path ./integration-tests/ --quiet -- ${script_name} {} \;

EOF
}

if [ -z "$manifest_path" ]; then
  usage
  exit 1
fi

build_result=$(cargo contract build --manifest-path "$manifest_path" --release --quiet --output-json)

if [ $? -eq 0 ]; then
  # only print the contract name and size if the build was successful
  dest_wasm=$(echo "$build_result" | jq -r .dest_wasm)
  contract_dir=$(dirname "$manifest_path")
  contract_size=$(stat -c %s "$dest_wasm")

  echo "$contract_dir" "$contract_size"
  exit 0
else
  echo "Failed to build contract at $manifest_path"
  exit 1
fi
