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
SUCCESSES=()
FAILURES=()

for manifest_path in "$FIND_PATH"/**/Cargo.toml;
  do if "$SCRIPTS_PATH"/is_contract.sh "$manifest_path"; then
    echo Running: "${@:2}" "$manifest_path";
    "${@:2}" "$manifest_path";

    if [ $? -eq 0 ]; then
      SUCCESSES+=("$manifest_path")
    else
      FAILURES+=("$manifest_path")
    fi
  fi
done

echo ""
echo "Succeeded:" ${#SUCCESSES[@]}
for success in "${SUCCESSES[@]}"; do
  echo "$success"
done

echo ""
echo "Failed:" ${#FAILURES[@]}
for failure in "${FAILURES[@]}"; do
  echo "$failure"
done

if [ ${#FAILURES[@]} -gt 0 ]; then
  exit 1
else
  exit 0
fi
