#!/bin/bash

function usage {
  cat << EOF
Usage: ${script_name} [OPTION] --path PATH -- COMMAND [INITIAL-ARGS]

Execute the supplied COMMAND with INITIAL-ARGS for all ink! contracts (recursively) found in the given path.
The manifest path (full path to the Cargo.toml file) is passed either:
  - in place of the "{}" placeholder in the command, if present
  - as the last argument to the command, if "{}" is not present

Returns 0 (success) if the command succeeds against *all* contract projects, if any fail returns 1 (failure).

OPTIONS
  -i, --ignore
  Path to ignore when recursively finding contract projects

  -p, --path
  Path to recursively find contract projects for which to execute the supplied command

EXAMPLES
   ${script_name} -path integration-tests -- cargo check --manifest-path
   ${script_name} -path integration-tests -- cargo contract build --manifest-path {} --release

EOF
}

# enable recursive globs
shopt -s globstar

script_name="${BASH_SOURCE[0]}"
scripts_path=$( cd "$(dirname "$script_name")" || exit; pwd -P )
command=( "${@:2}" )

options=$(getopt -o p:i: --long path:,ignore: -- "$@")
[ $? -eq 0 ] || {
    echo "Incorrect option provided"
    usage
    exit 1
}
eval set -- "$options"
ignore=()
while true; do
    case "$1" in
    -p|--path)
        shift; # The arg is next in position args
        path="$1"
        ;;
    -i|--ignore)
        shift; # The arg is next in position args
        ignore+=("$1")
        ;;
    --)
        shift
        break
        ;;
    esac
    shift
done

command=("${@}")

echo command "${command[@]}"

if [ -z "$path" ] || [ "${#command[@]}" -le 0 ]; then
  usage
  exit 1
fi

successes=()
failures=()

# default to adding the argument as the last argument to the command
arg_index=${#command[@]}
# find the index of the argument placeholder "{}", if present
for i in "${!command[@]}"; do
  if [ "${command[$i]}" = "{}" ]; then
    arg_index=$i
    break
  fi
done

for manifest_path in "$path"/**/Cargo.toml;
  do if "$scripts_path"/is_contract.sh "$manifest_path"; then
    command[$arg_index]="$manifest_path"
    echo Running: "${command[@]}"
    "${command[@]}"

    if [ $? -eq 0 ]; then
      successes+=("$manifest_path")
    else
      failures+=("$manifest_path")
    fi
  fi
done

GREEN='\033[1;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

printf "\nSucceeded: %s\n" ${#successes[@]}
for success in "${successes[@]}"; do
  printf "  ${GREEN}\u2713${NC} %s \n" "$success"
done

printf "\nFailed: %s\n" ${#failures[@]}
for failure in "${failures[@]}"; do
  printf "  ${RED}\u2717${NC} %s \n" "$failure"
done

if [ ${#failures[@]} -gt 0 ]; then
  exit 1
else
  exit 0
fi
