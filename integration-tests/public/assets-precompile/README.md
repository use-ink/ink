# Assets Precompile Integration

This contract demonstrates how to interact with ERC-20 asset precompiles in ink! using the `ink_precompiles` crate.

## Overview

This example shows how to:
- Use the `ink_precompiles` crate for precompile interfaces
- Test precompile interactions using the runtime-only e2e test framework
- Work with `pallet-assets` through the ERC-20 precompile
- Set up accounts and assets for e2e tests
- Debug precompile errors with `extract_error()`
