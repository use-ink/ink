#!/usr/bin/env bash

# switch-e2e-backend - Toggle E2E test backend between sandboxed and non-sandboxed modes
# Usage: switch-e2e-backend --manifest-path <PATH_TO_CONTRACT_CARGO_TOML>

set -euo pipefail

MANIFEST_PATH=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat << EOF
Usage: $0 --manifest-path <PATH>

Toggle E2E test backend between sandboxed and non-sandboxed modes for a contract.

Arguments:
    --manifest-path PATH    Path to the contract's Cargo.toml file

Examples:
    $0 --manifest-path ./integration-tests/my_contract/Cargo.toml
    $0 --manifest-path /path/to/contract/Cargo.toml

EOF
}

error() {
    >&2 echo "Error: $1"
    exit 1
}

info() {
    echo "Info: $1"
}

warn() {
    >&2 echo "Warning: $1"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --manifest-path)
            MANIFEST_PATH="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1. Use --help for usage information."
            ;;
    esac
done

if [[ -z "$MANIFEST_PATH" ]]; then
    error "--manifest-path is required"
fi

if [[ ! -f "$MANIFEST_PATH" ]]; then
    error "Manifest file not found: $MANIFEST_PATH"
fi

MANIFEST_PATH="$(realpath "$MANIFEST_PATH")"
CONTRACT_DIR="$(dirname "$MANIFEST_PATH")"

info "Processing contract at: $CONTRACT_DIR"

is_sandboxed() {
    local cargo_toml="$1"

    # Primary check: look for ink_sandbox::test attributes or backend(runtime_only) in test files
    local test_files
    test_files=$(find "$CONTRACT_DIR" -name "*.rs" -type f)

    while IFS= read -r file; do
        if grep -q '#\[ink_sandbox::test\]' "$file"; then
            return 0
        fi
        if grep -q '#\[.*::test.*backend.*runtime_only\]' "$file"; then
            return 0
        fi
    done <<< "$test_files"

    # Secondary check: look for sandbox feature in Cargo.toml
    if grep -q 'ink_e2e.*features.*sandbox' "$cargo_toml"; then
        return 0
    fi

    return 1
}

add_sandbox_feature() {
    local cargo_toml="$1"

    info "Adding sandbox feature to Cargo.toml"

    if grep -q 'ink_e2e\s*=' "$cargo_toml"; then
        if grep -q 'features.*=' "$cargo_toml"; then
            # Add sandbox to existing features
            sed -i.bak 's/\(features\s*=\s*\[[^]]*\)/\1, "sandbox"]/' "$cargo_toml"
        else
            # Add features array with sandbox
            sed -i.bak 's/\(ink_e2e\s*=\s*{[^}]*\)/\1, features = ["sandbox"]}/' "$cargo_toml"
        fi
    else
        warn "ink_e2e dependency not found in Cargo.toml"
    fi
}

remove_sandbox_feature() {
    local cargo_toml="$1"

    info "Removing sandbox feature from Cargo.toml"

    # Remove sandbox feature from ink_e2e dependency
    sed -i.bak -E 's/,\s*"sandbox"//g; s/"sandbox",\s*//g' "$cargo_toml"

    # Clean up empty features arrays
    sed -i.bak -E 's/features\s*=\s*\[\s*\]/features = []/g; s/,\s*features\s*=\s*\[\]//g' "$cargo_toml"
}

make_tests_sandboxed() {
    local test_files
    test_files=$(find "$CONTRACT_DIR" -name "*.rs" -type f)

    info "Converting tests to sandboxed mode"

    while IFS= read -r file; do
        if [[ ! -f "$file" ]]; then
            continue
        fi

        # Create backup
        cp "$file" "$file.bak"

        # Convert #[ink_e2e::test] to #[ink_sandbox::test(backend(runtime_only(...)))]
        # First handle simple tests
        sed -i.bak -E 's/#\[ink_e2e::test\]/#[ink_sandbox::test(backend(runtime_only(sandbox = ink_sandbox::DefaultSandbox, client = ink_sandbox::SandboxClient)))]/g' "$file"

        # Handle tests with other attributes
        sed -i.bak -E 's/#\[ink_e2e::test\(([^)]+)\)\]/#[ink_sandbox::test(\1, backend(runtime_only(sandbox = ink_sandbox::DefaultSandbox, client = ink_sandbox::SandboxClient)))]/g' "$file"

        # Remove backup if changes were made
        if ! diff -q "$file" "$file.bak" > /dev/null; then
            rm "$file.bak"
            info "Modified: $file"
        else
            rm "$file.bak"
        fi
    done <<< "$test_files"
}

make_tests_non_sandboxed() {
    local test_files
    test_files=$(find "$CONTRACT_DIR" -name "*.rs" -type f)

    info "Converting tests to non-sandboxed mode"

    while IFS= read -r file; do
        if [[ ! -f "$file" ]]; then
            continue
        fi

        # Create backup
        cp "$file" "$file.bak"

        # Remove backend(runtime_only(...)) from test attributes
        sed -i.bak -E 's/backend\s*\(\s*runtime_only\s*\([^)]*\)\)//g' "$file"

        # Remove trailing commas and spaces
        sed -i.bak -E 's/,\s*\)/)/g' "$file"

        # Convert ink_sandbox::test to ink_e2e::test
        sed -i.bak -E 's/#\[ink_sandbox::test/#\[ink_e2e::test/g' "$file"

        # Remove empty parentheses from test attributes
        sed -i.bak -E 's/#\[ink_e2e::test\(\s*\)\]/#[ink_e2e::test]/g' "$file"

        # Remove backup if changes were made
        if ! diff -q "$file" "$file.bak" > /dev/null; then
            rm "$file.bak"
            info "Modified: $file"
        else
            rm "$file.bak"
        fi
    done <<< "$test_files"
}

# Main logic
if is_sandboxed "$MANIFEST_PATH"; then
    info "Current mode: SANDBOXED"
    info "Switching to non-sandboxed mode..."

    remove_sandbox_feature "$MANIFEST_PATH"
    make_tests_non_sandboxed

    info "Successfully switched to non-sandboxed mode"
else
    info "Current mode: NON-SANDBOXED"
    info "Switching to sandboxed mode..."

    add_sandbox_feature "$MANIFEST_PATH"
    make_tests_sandboxed

    info "Successfully switched to sandboxed mode"
fi

# Clean up backup files
find "$CONTRACT_DIR" -name "*.bak" -delete 2>/dev/null || true

info "E2E backend switching completed!"