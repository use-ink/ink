#!/bin/bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
shopt -s globstar nullglob

# Examples are separate workspaces. Seed fresh lockfiles from the tested
# workspace dependencies instead of resolving newer, incompatible releases.
for manifest in "$repo_root"/integration-tests/**/Cargo.toml; do
    lockfile="${manifest%Cargo.toml}Cargo.lock"
    if [ ! -e "$lockfile" ]; then
        cp "$repo_root/Cargo.lock" "$lockfile"
    fi
done
