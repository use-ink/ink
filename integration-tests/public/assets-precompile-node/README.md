# Assets Precompile Integration (Node Backend)

This contract demonstrates how to interact with ERC-20 asset precompiles in ink! using the `ink_precompiles` crate, tested with a full node backend.

## Overview

This example shows how to:
- Use the `ink_precompiles` crate for precompile interfaces
- Test precompile interactions using the **node backend** e2e test framework
- Work with `pallet-assets` through the ERC-20 precompile
- Set up accounts and assets for e2e tests with a real node
- Debug precompile errors with `extract_error()`

## Performance Note

These tests use the `node` backend, which spawns a fresh `ink-node` for each test.
This provides perfect test isolation but is slower than runtime-only tests.

**Expected runtime**: ~5-7 seconds per test (~40 seconds total)

### Speed Up During Development

Run against a persistent node:
```bash
ink-node --dev --rpc-port 9944
CONTRACTS_NODE_URL=ws://127.0.0.1:9944 cargo test --package assets_precompile_node --features e2e-tests
```
