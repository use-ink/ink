# ink! Smart Contract Examples & Integration Tests

Welcome to the collection of example smart contracts and integration tests for [ink!](https://github.com/use-ink/ink), the relentless smart contract language for Polkadot and Substrate.

This directory contains a curated set of examples demonstrates everything from basic "Hello World" contracts to advanced upgradeability patterns and cross-chain messaging (XCM).

## Directory Structure

The examples are organized into categories to help you navigate from foundational concepts to complex implementations.

### Basics
*Fundamental building blocks for learning ink!.*

- **[`flipper`](basics/flipper)**: The classic "Hello World" of smart contracts. A simple boolean value you can flip.
- **[`incrementer`](basics/incrementer)**: Demonstrates simple state mutation with integers.
- **[`events`](basics/events)**: Examples of defining and emitting events.

### 🪙 Tokens
*Implementations of standard token interfaces.*

- **[`erc20`](tokens/erc20)**: Complete implementation of the ERC-20 fungible token standard.
- **[`erc721`](tokens/erc721)**: Non-Fungible Token (NFT) standard implementation.
- **[`erc1155`](tokens/erc1155)**: Multi-Token standard implementation.

### Traits
*Defining shared behavior using Rust traits.*

- **[`erc20`](traits/erc20)**: ERC-20 implemented using ink! trait definitions.
- **[`flipper`](traits/flipper)**: The flipper contract, but defined via a trait.
- **[`incrementer`](traits/incrementer)**: Trait-based incrementer.
- **[`dyn-cross-contract`](traits/dyn-cross-contract)**: How to make cross-contract calls using dynamic trait dispatch.

### Cross-Contract Interactions
*Calling other contracts from your contract.*

- **[`basic`](cross-contract/basic)**: Simple example of one contract calling another.
- **[`advanced`](cross-contract/advanced)**: Using call builders to set gas limits and storage deposits.
- **[`multi-caller`](cross-contract/multi-caller)**: A contract that can switch between calling different target contracts.
- **[`transfer`](cross-contract/transfer)**: Sending value (tokens) to another contract.
- **[`invocation`](cross-contract/invocation)**: Instantiating new contracts from within a contract.

### Storage
*Advanced storage layouts and data structures.*

- **[`basic`](storage/basic)**: Overview of available storage types.
- **[`complex`](storage/complex)**: Nested structs and Enums in storage.
- **[`lazyvec`](storage/lazyvec)**: Using `Lazy` vector types for gas optimization.
- **[`allocator`](storage/allocator)**: Using a custom heap allocator (e.g., `bumpalo`).

### Runtime & Chain
*Interacting with the Substrate chain and runtime.*

- **[`call-contract`](runtime/call-contract)**: The runtime calling into a contract.
- **[`e2e-call`](runtime/e2e-call)**: A contract calling a runtime dispatchable (pallet function).
- **[`xcm`](runtime/xcm)**: Cross-Consensus Messaging examples.
- **[`precompile`](runtime/precompile)**: interacting with runtime precompiles.
- **[`assets-precompile`](runtime/assets-precompile)**: Interacting with the `pallet-assets` precompile.

### Advanced
*Complex patterns and niche features.*

- **[`custom-env`](advanced/custom-env)**: Defining custom chain extensions and environment types.

### Use Cases
*Real-world scenarios and common patterns.*

- **[`dns`](use-cases/dns)**: Shows how to use the storage `Mapping` type.

### Operations
*Contract maintenance and operational tasks.*

- **[`terminator`](operations/terminator)**: How to remove a contract from storage using `terminate`.
- **[`upgradeable`](operations/upgradeable)**: Contracts that can upgrade their own code (`set_code_hash`).

### Testing
*Tools and techniques for testing contracts.*

- **[`fuzzing`](testing/fuzzing)**: Setup for property-based fuzz testing.
- **[`debugging`](testing/debugging)**: Techniques for interpreting debug buffers.

## Running the Tests

Most examples in this directory are standard Rust crates. You can run their tests using Cargo.

```bash
cd basics/flipper
cargo test
```

For End-to-End (E2E) tests that require a running Substrate node (like `substrate-contracts-node`), look for `e2e_tests.rs` files and ensure your environment is set up correctly with the `PINK_Runtime` or a local node.

## Contributing

If you are adding a new example, please place it in the most appropriate category directory. Ensure it includes a `README.md` and basic tests.
