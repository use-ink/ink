#!/bin/bash

# Clean all Rust contracts in subdirectories
# Runs `cargo clean` and removes Cargo.lock for each contract

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Cleaning all Rust contracts in $SCRIPT_DIR"
echo "============================================"

# Find all directories containing Cargo.toml and process them
find "$SCRIPT_DIR" -name "Cargo.toml" -type f | while read -r cargo_file; do
    dir="$(dirname "$cargo_file")"

    # Skip any target directories (these are build artifacts)
    if [[ "$dir" == *"/target/"* ]]; then
        continue
    fi

    echo ""
    echo "Cleaning: $dir"

    # Run cargo clean
    (cd "$dir" && cargo clean 2>/dev/null) || echo "  Warning: cargo clean failed in $dir"

    # Remove Cargo.lock if it exists
    if [ -f "$dir/Cargo.lock" ]; then
        rm "$dir/Cargo.lock"
        echo "  Removed Cargo.lock"
    fi
done

echo ""
echo "============================================"
echo "Done! All contracts cleaned."
