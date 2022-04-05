#!/bin/bash
#
# Script for publishing the various ink! crates to `crates.io`.
#
# It is assumed that version numbers have already been bumped. It also doesn't do any Git
# or Changelog related work.

set -eux

# This give us the "best" order in which to release our crates
to_release=( $(cargo unleash to-release) )
crates=()

for element in "${to_release[@]}"
do
    # `to-release` also gives us the version number, we only care about the crate names
    if [[ $element == *"ink_"* ]]; then
       echo "${element}"
       crates[${#crates[@]}]="${element}"
    fi
done

for crate in "${crates[@]}"
do
    # Wait a little (15 sec) between publishing steps to allow the registry to catch up
    sleep 15

    # Need `--allow-dirty` here since `cargo unleash to-release` removes
    # dev-dependencies when it runs
    #
    # NOTE: When you're ready to actually publish get rid of the `--dry-run` flag
    cargo publish -p $crate --allow-dirty --dry-run
done
