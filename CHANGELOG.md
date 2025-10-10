# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[Unreleased]

### Added
- Implements the API for the `pallet-revive` host function `to_account_id` - [#2578](https://github.com/use-ink/ink/pull/2578)
- Add `#[ink::contract_ref]` attribute - [#2648](https://github.com/use-ink/ink/pull/2648)
- Add `ink_revive_types` (and remove `pallet-revive` dependency from `ink_e2e`) - [#2657](https://github.com/use-ink/ink/pull/2657)
- non-allocating Solidity ABI encoder - [#2655](https://github.com/use-ink/ink/pull/2655)

### Changed
- Marks the `pallet-revive` host function `account_id` stable - [#2578](https://github.com/use-ink/ink/pull/2578)
- Stabilize `is_contract` - [#2654](https://github.com/use-ink/ink/pull/2654)
- Extract `sandbox` from `ink_e2e` into a new `ink_sandbox` crate - [#2659](https://github.com/use-ink/ink/pull/2659)
- Synchronize with `polkadot-sdk/1b1cef306d9ceebf963fd15a04b5c79ee2618bce` ‒ [2675](https://github.com/use-ink/ink/pull/2675)

### Fixed
- Fix decoding of `HostFn::minimum_balance` return value - [#2656](https://github.com/use-ink/ink/pull/2656)
- Fix handling of `HostFn::code_hash` and `HostFn::weight_to_fee` - [#2672](https://github.com/use-ink/ink/pull/2672)

### Fixed
- `name` override fixes for message id computation and trait definitions - [#2649](https://github.com/use-ink/ink/pull/2649)

## Version 6.0.0-alpha.4

### Added
- Add integration test for arithmetic overflow checks - [#2631](https://github.com/use-ink/ink/pull/2631)
- E2E: Misc quality of life improvements, new API functions, better debuggability ‒ [2634](https://github.com/use-ink/ink/pull/2634)

### Changed
- Error on message and constructor `selector` overrides in Solidity ABI mode - [#2638](https://github.com/use-ink/ink/pull/2638)
- Improve abstractions for Solidity ABI encoding `Result` types - [#2635](https://github.com/use-ink/ink/pull/2635)
- Feature gate `xcm` - [#2641](https://github.com/use-ink/ink/pull/2641)
- Refactor multi ABI interfaces for event emission via `ink_env` - [#2643](https://github.com/use-ink/ink/pull/2643)

### Fixed
- Bring intended panic handler behavior back ‒ [2636](https://github.com/use-ink/ink/pull/2636)
- Support `name` attribute in trait definitions - [#2644](https://github.com/use-ink/ink/pull/2644)

## Version 6.0.0-alpha.3

Compatibility of this release:

* Rust >= 1.88
* [`cargo-contract` `v6.0.0-alpha.3`](https://github.com/use-ink/cargo-contract/releases/tag/v6.0.0-alpha.3)
* [`ink-node` `v0.45.1`](https://github.com/use-ink/ink-node/releases/tag/v0.45.1)
* [`polkadot-sdk` from `use-ink/polkadot-sdk/a71ec19a94702ea71767ba5ac97603ea6c6305c1`](https://github.com/use-ink/polkadot-sdk/tree/pallet-revive-with-system-and-storage-precompiles)

We have to use a slight fork of `polkadot-sdk` for the moment. It's just `polkadot-sdk/master` plus two
commits on top with pre-compiles. Those two commits are PRs to `polkadot-sdk`. but haven't been merged yet.

### Added
- Support functions of the `Storage` and `System` pre-compiles ‒ [2619](https://github.com/use-ink/ink/pull/2619)

### Changed
- Synchronize with `polkadot-sdk/c40b36c3a7c208f9a6837b80812473af3d9ba7f7` ‒ [2589](https://github.com/use-ink/ink/pull/2589)
- Synchronize with `polkadot-sdk/a71ec19a94702ea71767ba5ac97603ea6c6305c1` ‒ [2619](https://github.com/use-ink/ink/pull/2619)
- Refactor multi ABI interfaces - [#2618](https://github.com/use-ink/ink/pull/2618)
- Upgrade to Rust edition 2024 - [#2624](https://github.com/use-ink/ink/pull/2624)

### Removed
- Removed functionalities around calling into the runtime and chain extensions ‒ [2621](https://github.com/use-ink/ink/pull/2621)
- Remove `Environment::MAX_EVENT_TOPICS` and remove `Environment` generic type from event abstractions - [#2622](https://github.com/use-ink/ink/pull/2622)

### Fixed
- E2E: Fixes around correct handling of storage deposit limit ‒ [#2589](https://github.com/use-ink/ink/pull/2589)
- Make `NativeToEthRatio` part of the `Environment` ‒ [#2604](https://github.com/use-ink/ink/pull/2604)
- E2E: Fix `ink_sandbox` gating - [#2626](https://github.com/use-ink/ink/pull/2626)

## Version 6.0.0-alpha.1

### Added
- Support ABI `cfg` flag in codegen - [#2501](https://github.com/use-ink/ink/pull/2501)
- Generate Solidity ABI compatibility metadata - [#2510](https://github.com/use-ink/ink/pull/2510)
- Improve Solidity ABI support in `codegen`, `ink_env` and `ink_e2e` - [#2517](https://github.com/use-ink/ink/pull/2517)
- Support Solidity ABI encoded constructor dispatch - [#2525](https://github.com/use-ink/ink/pull/2525)
- Export `Weight` with Solidity encoding - [#2540](https://github.com/use-ink/ink/pull/2540)
- Implement `SolEncode` and `SolDecode` for generated contract refs, call and message builders - [#2539](https://github.com/use-ink/ink/pull/2539)
- Abstractions for mapping Rust/ink! `Result` and error types to/from Solidity ABI error and result representations - [#2543](https://github.com/use-ink/ink/pull/2543)
- `Derive` macros for implementing `SolEncode` and `SolDecode` for arbitrary types - [#2549](https://github.com/use-ink/ink/pull/2549)
- Improve handling of Solidity constructor return and revert data - [#2552](https://github.com/use-ink/ink/pull/2552)
- Implement `SolEncode` and `SolDecode` for `Option<T>` - [#2545](https://github.com/use-ink/ink/pull/2545)
- Allow writing E2E fuzz tests for contracts - [#2570](https://github.com/use-ink/ink/pull/2570)
- Item name/identifier overrides for overloading, selector computation and metadata - [#2577](https://github.com/use-ink/ink/pull/2577)
- Add custom errors to Solidity compatible metadata - [#2583](https://github.com/use-ink/ink/pull/2583)
- Efficient conversions and representations for byte sequence references for Solidity ABI encoding/decoding - [#2590](https://github.com/use-ink/ink/pull/2590)
- Add `#[ink::error]` attribute macro - [#2585](https://github.com/use-ink/ink/pull/2585)

### Changed
- Use marker trait for finding ink! storage `struct` during code analysis - [2499](https://github.com/use-ink/ink/pull/2499)
- Solidity ABI compatibility metadata improvements - [#2511](https://github.com/use-ink/ink/pull/2511)
- Share intermediate build artifacts across all contract builds in e2e tests - [#2531](https://github.com/use-ink/ink/pull/2531)
- Refactor Solidity bytes wrapper(s) - [#2569](https://github.com/use-ink/ink/pull/2569)
- Refactor events for `pallet-revive` and multiple ABI support - [#2580](https://github.com/use-ink/ink/pull/2580)

### Fixed
- Update metadata version to version 6 ‒ [#2507](https://github.com/use-ink/ink/pull/2507)
- Ensure immutable messages are not payable - [#2535](https://github.com/use-ink/ink/pull/2535)

## Version 6.0.0-alpha

This is our first alpha release for ink! v6. We release it together
with `cargo-contract` `v6.0.0-alpha`.

The biggest change is that we are in the process of migrating from `pallet-contracts` +
WebAssembly (executed in `wasmi`) to [`pallet-revive`](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive) +
RISC-V (executed in [PolkaVM](https://github.com/paritytech/polkavm/)).
_This is a major breaking change, ink! v6 will only be compatible with `cargo-contract` >= v6
and chains that include `pallet-revive`._

We did a detailed write-up of the background to this development and the reasoning
[here](https://use.ink/6.x/current-state). We also updated the [`ink/ARCHITECTURE.md`](https://github.com/use-ink/ink/blob/master/ARCHITECTURE.md)
to reflect the new setup.

Compatibility of this release:
* Rust >= 1.86
* [`cargo-contract` `v6.0.0-alpha`](https://github.com/use-ink/cargo-contract/releases/tag/v6.0.0-alpha)
* [`ink-node/5f93093`](https://github.com/use-ink/ink-node/commit/5f93093dcffbbd2c2e44bfd2e457dc418c844623)
* [`polkadot-sdk/stable2503`](https://github.com/paritytech/polkadot-sdk/tree/stable2503)

In the following we'll describe some breaking changes on a high-level. The
context to understand them is that the `pallet-revive` team has Ethereum/Solidity
support as the number one priority. All their design decisions derive from that,
they don't want to maintain code that is unnecessary for that objective.

_🚧 This is an alpha release, changes will still happen and there are rough edges. 🚧_

### Cross-contract calling Solidity contracts
We are introducing a new attribute `abi` for the `#[ink::contract]` macro.
These are the values it takes:

```
#[ink::contract(abi = "all")]
#[ink::contract(abi = "sol")]
#[ink::contract(abi = "ink")]
```

The default currently is `abi = "ink"`, but we might change this before a production
release.

The implication of supporting Solidity ABI encoding is that there is a restriction on
the types you can use as constructor/message arguments or return types.
You won't be able to use Rust types for which no mapping to a Solidity type exists.
An error about a missing trait implementation for this type will be thrown.

Please note that your contract sizes will get larger if you support both the ink!
and Solidity ABI.

### Types

#### Contract Balance: `U256`
For the type of a contract's balance, `pallet-revive` uses depending on the context
* either the configured `pallet_revive::Config::Currency` type (which corresponds
  to the `ink::Environment::Balance` type.
* or a hardcoded `U256` (which corresponds to what Ethereum uses).
  In this alpha release we just adhere to requiring the types that `pallet-revive` uses.
  In an upcoming beta release this could be simplified to reduce UX friction by just
  using one type everywhere and converting to the `pallet-revive` one.

#### Contract Address: `Address` / `H160`
For a contract's account, `pallet-revive` is using either the configured `AccountId` type
of the `polkadot-sdk` runtime, or `H160`.

`Address` is a more semantically named type alias for `H160` defined in `ink_primitives`,
and re-exported in the `ink` crate.

Finding the `Address`/`H160` for an `AccountId` is done via an address derivation scheme
derived in [#7662](https://github.com/paritytech/polkadot-sdk/pull/7662).
After instantiating a contract, the address is no longer returned by `pallet-revive`.
Instead one has to derive it from given parameters (see the linked PR). `cargo-contract`
does that automatically.

For contract instantiations and contract calls the pallet requires that a 1-to-1 mapping
of an `AccountId` to an `Address`/`H160` has been created. This can be done via the
`map_account`/`unmap_account` API.
The PR [#6096](https://github.com/paritytech/polkadot-sdk/pull/6096) contains more
information.

Besides the publicly exposed crate functions, we've introduced a new subcommand
`cargo contract account` that allows resolving the `H160` contract address to the
Substrate `AccountId` which it is mapped to.

#### Contract Hash: `H256`
For a contract's hash value, `pallet-revive` uses a fixed `H256`, Previously,
the `ink::Environment::Hash` type referenced the hash type being used for the
contract's hash. Now it's just a fixed `H256`.

### Contract delegates can no longer be done by code
In `pallet-contracts` (and hence up until ink! v5), a pattern for upgradeable
contracts was to delegate the contract execution to a different code, e.g. to
a new version of the contract's code.

This distinction of contract code that was uploaded to a chain vs. an instantiated
contract from this code no longer exists in `pallet-revive`. If you want to
delegate the execution, you will have to specify another contract's address
to which code you want to delegate to. This other contract needs to be instantiated
on-chain.

For the execution, the context of the contract that delegates will continue
to be used (storage, caller, value).

Specifically the delegate API changed like this:

```
/// ink! v5
#[derive(Clone)]
pub struct DelegateCall<E: Environment> {
    code_hash: E::Hash,
    call_flags: CallFlags,
}

/// ink! v6
#[derive(Clone)]
pub struct DelegateCall {
    address: H160,
    flags: CallFlags,
    ref_time_limit: u64,
    proof_size_limit: u64,
    deposit_limit: Option<[u8; 32]>,
}
```

### Feature `ink/unstable-hostfn`
In `pallet-revive` a number of functions can only be called by smart contracts
if the chain that the pallet is running on has enabled the feature
`pallet-revive/unstable-hostfn`.
This feature is not enabled on Kusama or Westend!

It is enabled for the `substrate-contracts-node` version that we linked above.

### New debugging workflow
Previously `pallet-contracts` returned a `debug_message` field with contract
instantiations and dry-runs.
Whenever `ink::env::debug_println` was invoked in a contract, ink! wrote debugging
info to this field. This functionality has been removed. Instead `pallet-revive` now
supports other means of debugging.

The most relevant new debugging workflow is the tracing API. There are a number
of PRs that implemented it, so we won't link a specific one here. A good starting
point to look deeper into it is the [`tracing.rs`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/revive/src/tracing.rs).

We have implemented barebones support for this tracing API in the 6.0.0-alpha
versions of ink! + `cargo-contract`. But it's really barebones and should
certainly be improved before a production release.

Please see [our developer documentation](https://use.ink/docs/v6/contract-debugging)
for more details.
We've also added a contract example to illustrate these new debugging strategies:
[`debugging-strategies`](https://github.com/use-ink/ink/tree/master/integration-tests/public/debugging-strategies).

### Restrictions which `cfg` attributes can be used

This change was done as a recommendation from the ink! 5.x audit.
In a nutshell it prevents developers from hiding functionality in a contract,
that would not be visible in the metadata (so e.g. on a block explorer).
The relevant PR is [#2313](https://github.com/use-ink/ink/pull/2313).

From ink! 6.0 on only these attributes are allowed in `#[cfg(…)]`:
- `test`
- `feature` (without `std`)
- `any`
- `not`
- `all`

### Metadata Changes

The field `source.wasm` was renamed to `source.contract_binary`.

### `no_main`

Previously ink! contracts started with this line:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
```

This line instructs the Rust compiler to not link the Rust
standard library with your contract.
If you want to know about why:
we have an entry
["Why is Rust's standard library (stdlib) not available in ink!?"](./faq.md#why-no_std)
Please see [our developer documentation](https://use.ink/docs/v6/faq/#why-no_std)
in our FAQ.

With ink! v6, an additional crate-level attribute needs to be set:

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]
```

It instructs the compiler not to use the default `fn main() {}` function as the
entry point for your smart contract. This is needed because PolkaVM uses a different
entry point (the `deploy` function).

### `substrate-contracts-node` can no longer be used
The `substrate-contracts-node` is still maintained by Parity for ink! v5 and
`pallet-contracts`, but it does not support `pallet-revive`.

We've set up a new project in its place: [`ink-node`](https://github.com/use-ink/ink-node).
As before, it functions as a simple local development node.
It contains `pallet-revive` in a default configuration.
You can find binary releases of the node [here](https://github.com/use-ink/ink-node/releases).

### Changed
- Restrict which `cfg` attributes can be used ‒ [#2313](https://github.com/use-ink/ink/pull/2313)
- More idiomatic return types for metadata getters - [#2398](https://github.com/use-ink/ink/pull/2398)
- Define static distributed events slice in `ink` crate - [#2487](https://github.com/use-ink/ink/pull/2487)
- [E2E] Update `subxt` dependencies ‒ [#2504](https://github.com/use-ink/ink/pull/2504)

### Added
- Support for `caller_is_root` - [#2332](https://github.com/use-ink/ink/pull/2332)
- Allow setting features for contract build in E2E tests - [#2460](https://github.com/use-ink/ink/pull/2460)
- Improve support for Solidity ABI calling conventions - [#2411](https://github.com/use-ink/ink/pull/2411)
- Implement contract invocation in off-chain environment engine - [#1957](https://github.com/paritytech/ink/pull/1988)
- Abstractions for mapping arbitrary Rust types to Solidity ABI compatible types - [#2441](https://github.com/use-ink/ink/pull/2441)
- Documentation for contract abi arg and provided Rust/ink! to Solidity type mappings - [2463](https://github.com/use-ink/ink/pull/2463)
- Implement `SolDecode`, `SolTypeDecode` and support `SolBytes` for boxed slices - [2476](https://github.com/use-ink/ink/pull/2476)

### Fixed
- [E2E] Have port parsing handle comma-separated list ‒ [#2336](https://github.com/use-ink/ink/pull/2336)
- Always use ink! ABI/ SCALE codec for constructor and instantiation related builders and utilities - [#2474](https://github.com/use-ink/ink/pull/2474)
- Get rid of "extrinsic for call failed: Pallet error: Revive::AccountAlreadyMapped" - [2483](https://github.com/use-ink/ink/pull/2483)
- CI disk usage via standardised toolchains: `stable` 1.86, `nightly` 2025-02-20 - [#2484](https://github.com/use-ink/ink/pull/2484)
- CI contract size submission - [#2490](https://github.com/use-ink/ink/pull/2490)
- CI relax criteria for `measurements-master` artifact lookup - [#2491](https://github.com/use-ink/ink/pull/2491)

## Version 5.1.0

This is the first ink! release outside of Parity. ink! was started at Parity and
during this year became a community project maintained by the ink! Alliance, a
loose group of former Parity employees and teams who want ink! to ensure a bright
future for ink!.

You can find more details about the community handover in
[this X post](https://x.com/ink_lang/status/1783877356819783916).
Generally, the only thing changing on the user-facing side is that the repositories
have been moved from `paritytech` to the new GitHub organization `use-ink`.

❣ _We want to say a big thank you to our Polkadot community, which recently decided on
funding the continued maintenance and development of ink! with
[a Polkadot Treasury grant](https://polkadot.polkassembly.io/referenda/1123)._

### Highlights

This version of ink! comes with three highlights plus some minor fixes.

#### (1) XCM Support

ink! 5.1 supports the usage of XCM in contracts, developers are no longer limited
to cross-contract calls, but can now execute cross-parachain calls.

We added a contract example that demonstrates the usage:
[`contract-xcm`](https://github.com/use-ink/ink-examples/tree/main/contract-xcm)

We also added a new page on our documentation website:
[https://use.ink/basics/xcm](https://use.ink/basics/xcm).

You can view the Rust docs of the two functions here:

* [`xcm_send`](https://docs.rs/ink_env/5.1.0/ink_env/fn.xcm_send.html)
* [`xcm_execute`](https://docs.rs/ink_env/5.1.0/ink_env/fn.xcm_execute.html)

#### (2) Call an ink! contract from a `polkadot-sdk` runtime

ink! 5.1 comes with basic support for calling contracts from a Polkadot runtime.
We've added [this example](https://github.com/use-ink/ink-examples/tree/main/runtime-call-contract)
that demonstrates how to call `flipper` from a `polkadot-sdk` runtime pallet.

Calling a contract from the runtime is an interesting application for parachains,
as they can put logic into a contract instead of their runtime. Contracts have
a number of advantages, as they are easier to upgrade and allow for
faster development iteration cycles.

The limitations currently are:

* Contract calls can only be made to trait messages. This makes sense in the
  `pallet-contracts` context, as it is better to depend on a trait rather
  than a contract impl, since you are working against an interface.
* Only contract messages can be called currently, no constructors.
* The API could be nicer.

#### (3) E2E Testing

We replaced our `drink` sandbox dependency with an internal ink! crate.
In case you use [DRink!](https://use.ink/basics/contract-testing/drink):

First, you should upgrade your `drink` dependency to `version = "0.18.0"`.
Second, these are the two changes you have to make:

```diff
- #[ink_e2e::test(backend(runtime_only(sandbox = ink_e2e::MinimalSandbox)))]
+ #[ink_e2e::test(backend(runtime_only(sandbox = ink_e2e::DefaultSandbox)))]
```

```diff
- ink_e2e = { version = "5", features = ["drink"] }
+ ink_e2e = { version = "5", features = ["sandbox"] }
```

### Compatibility

The compatibility changes a bit to ink! 5.0:

- Rust: `>= 1.81`
- `cargo-contract`: `>= 5.0.0`
- `polkadot-sdk`: [>= v1.12.0](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-v1.12.0)
  (this release stabilized the `pallet-contracts` XCM functions that ink! uses)
- `substrate-contracts-node`: `>= 0.42.0`
- [DRink!](https://github.com/inkdevhub/drink): `>= 0.18.0`

For the linter in `cargo-contract` the Rust toolchain version changed.
To upgrade:

```
export TOOLCHAIN_VERSION=nightly-2024-09-05
rustup install $TOOLCHAIN_VERSION
rustup component add rust-src --toolchain $TOOLCHAIN_VERSION
rustup run $TOOLCHAIN_VERSION cargo install cargo-dylint dylint-link
```

### Added
- [Runtime-to-Contract Calls] Environment agnostic contract invocation API, for calling contracts from runtime ‒ [#2219](https://github.com/use-ink/ink/pull/2219)
- [Runtime-to-Contract Calls] Add `no-panic-handler` feature ‒ [#2164](https://github.com/paritytech/ink/pull/2164)
- [Runtime-to-Contract Calls] Add example for calling a contract from a runtime pallet ‒ [#2189](https://github.com/paritytech/ink/pull/2189)
- [XCM] Add `xcm_execute` and `xcm_send` support ‒ [#1912](https://github.com/use-ink/ink/pull/1912)
- [Linter] Add links to detailed lint description ‒ [#2170](https://github.com/use-ink/ink/pull/2170)
- [E2E] Adds a message to SandboxErr to add context for easier debugging ‒ [#2218](https://github.com/use-ink/ink/pull/2218)
- [E2E] Add ability to take and restore snapshots ‒ [#2261](https://github.com/paritytech/ink/pull/2261) (thanks [@0xLucca](https://github.com/0xLucca)!)
- [E2E] Demonstrate usage of seeds for secret URIs in E2E test for chain snapshots ‒ [#2163](https://github.com/paritytech/ink/pull/2163)

### Changed
- Update repository URLs & references from `paritytech` GitHub organization to new `use-ink` one ‒ [#2220](https://github.com/use-ink/ink/pull/2220) and [#2248](https://github.com/use-ink/ink/pull/2248)
- [E2E] Update `subxt` and `polkadot-sdk` dependencies ‒ [#2174](https://github.com/use-ink/ink/pull/2174)
- [Drink backend] Replace `drink` sandbox with internal `ink_sandbox` ‒ [#2158](https://github.com/use-ink/ink/pull/2158)

### Fixed
- [XCM] Fix XCM-support to single encode the XCM message ‒ [#2278](https://github.com/use-ink/ink/pull/2278)
- [Examples] ERC-721: `burn()` clears token approval ‒ [#2099](https://github.com/paritytech/ink/pull/2099)
- [E2E] Fix outdated docs for `[ink_e2e::test]` ‒ [#2162](https://github.com/use-ink/ink/pull/2162)
- [E2E] Build contracts before initializing node rpc ‒ [#2168](https://github.com/use-ink/ink/pull/2162)
- [E2E] `set_account_balance` now can't set balance below existential deposit ‒ [#1983](https://github.com/paritytech/ink/pull/1983) (thanks [@0xLucca](https://github.com/0xLucca)!)
- [E2E] Fix outdated docs for `[ink_e2e::test]` ‒ [#2162](https://github.com/paritytech/ink/pull/2162)

## Version 5.0.0

ℹ️ _We've created a migration guide from ink! 4 to ink! 5. It also contains an
overview over all breaking changes and newly added features._

👉 _You can view it [here](https://use.ink/faq/migrating-from-ink-4-to-5)._

### Summary
This release addresses the rest of the severities described in the [OpenZeppelin security review](https://blog.openzeppelin.com/security-review-ink-cargo-contract) of ink! and `cargo-contract`.
One of the notable addressed issues is the proxy selector clashing attack.
As of this release, ink! only allows exactly one other message with a well-known reserved selector to be defined.
You can read more about the change in the [#1827](https://github.com/use-ink/ink/pull/1827) and [#2031](https://github.com/use-ink/ink/pull/2031).

ink! 5.0.0 features a significant number of new features:
- We have introduced a new API based on the calculated or specified selectors for the event definition. This allows events to be defined in separate files and modules, and be shared across multiple ink! contracts - [#1827](https://github.com/use-ink/ink/pull/1827) and [#2031](https://github.com/use-ink/ink/pull/2031).
- [@pmikolajczyk41](https://github.com/pmikolajczyk41) has introduced an alternative E2E testing framework, [DRink!](https://github.com/inkdevhub/drink?tab=readme-ov-file#as-an-alternative-backend-to-inks-e2e-testing-framework), that support quasi-testing model, it allows the test simulate a running node as part of the E2E test while improving debugging experience such as allowing to set breakpoint and step through each stage of execution cycle.
- Following improvements in E2E, we have added a call builder API that allows to easily build calls while significantly reducing boilerplate code - [#1917](https://github.com/use-ink/ink/pull/1917) and [#2075](https://github.com/use-ink/ink/pull/2075)
- Another notable introduction in 5.0.0 release is the support for multiple chain extensions that empower developers
to build even more sophisticated and advanced contracts for supported chains - [#1958](https://github.com/use-ink/ink/pull/1958).
- To further address our consideration of the intrinsic security of ink! smart contracts,
we have disallowed unchecked arithmetic expressions. `cargo-contract` will fail to compile the contract with the raw arithmetic operation - [#1831](https://github.com/use-ink/ink/pull/1831).

These are the main features we have introduced in this release. We also encourage developers
to have a look at more detailed changelog entries to find out about any breaking changes that may affect
the development of new ink! contracts.

### Compatibility

See [the compatibility section](https://use.ink/faq/migrating-from-ink-4-to-5/#compatibility) of our migration guide for a detailed description. On a high level:

- Rust: `>= 1.70`
- `cargo-contract`: `>= 4.0.0`
- polkadot-sdk: [>= 0.9.3](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-v0.9.3). But if using the new functions introduced in [#2123](https://github.com/use-ink/ink/pull/2123) and [#2077](https://github.com/use-ink/ink/pull/2077) [>= 1.8.0](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-v1.8.0) and if using the new functions introduced in [#2076](https://github.com/use-ink/ink/pull/2076) [>= 1.9.0](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-v1.8.0).
- `polkadot-js/api` and `polkadot-js/api-contract`: `>= 10.12.1`
- `substrate-contracts-node`: `>= 0.39.0`

### Changelog

#### Added
- Add Hash trait to Selector struct - [#2149](https://github.com/use-ink/ink/pull/2149)
- `instantiate_v2` with additional limit parameters [#2123](https://github.com/use-ink/ink/pull/2123)
- Custom signature topic in Events - [#2031](https://github.com/use-ink/ink/pull/2031)
- [Linter] `non_fallible_api` lint - [#2004](https://github.com/use-ink/ink/pull/2004)
- [Linter] Publish the linting crates on crates.io - [#2060](https://github.com/use-ink/ink/pull/2060)
- [E2E] Added `create_call_builder` for testing existing contracts - [#2075](https://github.com/use-ink/ink/pull/2075)
- `call_v2` cross-contract calls with additional limit parameters - [#2077](https://github.com/use-ink/ink/pull/2077)
- `delegate_dependency` api calls - [#2076](https://github.com/use-ink/ink/pull/2076)
- Allow mutable parameters in messages - [#2004](https://github.com/use-ink/ink/pull/2004)
- Clean E2E configuration parsing - [#1922](https://github.com/use-ink/ink/pull/1922)
- Make `set_code_hash` generic - [#1906](https://github.com/use-ink/ink/pull/1906)
- Provide a `StorageVec` datastructure built on top of `Lazy` - [#1995](https://github.com/use-ink/ink/pull/1995)
- Add fallible methods for `Mapping` and `Lazy` - [#1910](https://github.com/use-ink/ink/pull/1910)
- [E2E] Allow testing with live-chain state - [#1949](https://github.com/use-ink/ink/pull/1949)
- [E2E] Call builders and extra gas margin option - [#1917](https://github.com/use-ink/ink/pull/1917)
- [Linter] `storage_never_freed` lint - [#1932](https://github.com/use-ink/ink/pull/1932)
- [Linter] `strict_balance_equality` lint - [#1914](https://github.com/use-ink/ink/pull/1914)
- [Linter] `no_main` lint - [#2001](https://github.com/use-ink/ink/pull/2001)
- Reexport `scale` dependencies, introduce `#[ink::scale_derive]` - [#1890](https://github.com/use-ink/ink/pull/1890)
- Upgradeable contracts example - [#1889](https://github.com/use-ink/ink/pull/1889)
- Persist static buffer size in metadata - [#1880](https://github.com/use-ink/ink/pull/1880)
- Modify static buffer size via environmental variables - [#1869](https://github.com/use-ink/ink/pull/1869)
- Added `sr25519_verify` function to `ink_env` [#1840](https://github.com/use-ink/ink/pull/1840)
- Events `2.0` - [#1827](https://github.com/use-ink/ink/pull/1827)
- Add `set_block_number` to off-chain test api `Engine` - [#1806](https://github.com/use-ink/ink/pull/1806)
- Stabilize `call_runtime` ‒ [#1749](https://github.com/use-ink/ink/pull/1749)
- Schema generation - [#1765](https://github.com/use-ink/ink/pull/1765)
- Restrict wildcard selectors to have exactly one other message - [#1708](https://github.com/use-ink/ink/pull/1708)
- [Linter] Warn when primitive number is annotated as event topic - [#1837](https://github.com/use-ink/ink/pull/1837)
- [Drink backend] allow for arbitrary runtime - [#1892](https://github.com/use-ink/ink/pull/1892)
- [Drink backend] support runtime call - [#1891](https://github.com/use-ink/ink/pull/1891)
- [Drink backend] Make tests generic `E2EBackend` trait - [#1867](https://github.com/use-ink/ink/pull/1867)
- [Drink backend] Backend choice  ‒ [#1864](https://github.com/use-ink/ink/pull/1864)
- [Drink backend] Backend traits - [#1857](https://github.com/use-ink/ink/pull/1857)
- [Drink backend] Abstract error and result structs - [#1844](https://github.com/use-ink/ink/pull/1844)

#### Changed
- Use name-only syntax for `anonymous` ink! event item configuration argument - [#2140](https://github.com/use-ink/ink/pull/2140)
- Restrict syntax for setting default ink! e2e test runtime-only emulator - [#2143](https://github.com/use-ink/ink/pull/2143)
- Restrict syntax for setting ink! e2e test node to auto - [#2146](https://github.com/use-ink/ink/pull/2146)
- Bump Substrate crates - [#2141](https://github.com/use-ink/ink/pull/2141)
- Minor fixes - [#2144](https://github.com/use-ink/ink/pull/2144),
[#2137](https://github.com/use-ink/ink/pull/2137), [#2132](https://github.com/use-ink/ink/pull/2132)
- Bump metadata version to 5 [#2126](https://github.com/use-ink/ink/pull/2126)
- Use `MaxEncodedLen` for output buffer size [#2128](https://github.com/use-ink/ink/pull/2128)
- `Mapping`: Reflect all possible failure cases in comments ‒ [#2079](https://github.com/use-ink/ink/pull/2079)
- [E2E] Rename `.call` to `.call_builder` ‒ [#2078](https://github.com/use-ink/ink/pull/2078)
- Improve syntax for ink! e2e `runtime_only` attribute argument - [#2083](https://github.com/use-ink/ink/pull/2083)
- [E2E] Remove `additional_contracts` parameter [#2098](https://github.com/use-ink/ink/pull/2098)
- [E2E] change node url backend config - [#2101](https://github.com/use-ink/ink/pull/2101)
- Messages return `TypeSpec` directly - [#1999](https://github.com/use-ink/ink/pull/1999)
- Fail when decoding from storage and not all bytes consumed - [#1897](https://github.com/use-ink/ink/pull/1897)
- Support multiple chain extensions - [#1958](https://github.com/use-ink/ink/pull/1958)
  - New example of how to use multiple chain extensions in one contract.
  - Affects the usage of the `#[ink::chain_extension]` macro and the definition of the chain extension.
- Split up `ink_linting` to mandatory and extra libraries - [#2032](https://github.com/use-ink/ink/pull/2032)
- [E2E] resolve DispatchError error details for dry-runs - [#1994](https://github.com/use-ink/ink/pull/1994)
- [E2E] update to new `drink` API - [#2005](https://github.com/use-ink/ink/pull/2005)
- Reexport `scale` dependencies, introduce `#[ink::scale_derive]` ‒ [#1890](https://github.com/use-ink/ink/pull/1890)
- Use of workspace dependencies and properties - [#1835](https://github.com/use-ink/ink/pull/1835)
- Remove of unchecked arithmetic - [#1831](https://github.com/use-ink/ink/pull/1831)
- Use `decode_all` for decoding cross contract call result - [#1810](https://github.com/use-ink/ink/pull/1810)
- [E2E] build contracts at runtime instead of during codegen - [#1881](https://github.com/use-ink/ink/pull/1881)
- [E2E] crate refactoring - [#1830](https://github.com/use-ink/ink/pull/1830)
- [E2E] improve call API, remove `build_message` + callback - [#1782](https://github.com/use-ink/ink/pull/1782)

#### Fixed
- Fix alignment in allocator [#2100](https://github.com/use-ink/ink/pull/2100)
- Fix the `StorageVec` type by excluding the `len_cached` field from its type info - [#2052](https://github.com/use-ink/ink/pull/2052)
- Fix panic in `approve_for` in the ERC-721 example - [#2092](https://github.com/use-ink/ink/pull/2092)
- ERC-721: `transfer_token_from` now ensures the token owner is correct - [#2093](https://github.com/use-ink/ink/pull/2093)
- `RootLayout::new()` is generic again to allow using `ink_metadata` in pure `PortableForm` contexts - [#1989](https://github.com/use-ink/ink/pull/1989)

## Version 5.0.0-rc.3

### Changed
- Use name-only syntax for `anonymous` ink! event item configuration argument - [#2140](https://github.com/use-ink/ink/pull/2140)
- Restrict syntax for setting default ink! e2e test runtime-only emulator - [#2143](https://github.com/use-ink/ink/pull/2143)
- Restrict syntax for setting ink! e2e test node to auto - [#2146](https://github.com/use-ink/ink/pull/2146)
- Bump Substrate crates - [#2141](https://github.com/use-ink/ink/pull/2141)
- Minor fixes - [#2144](https://github.com/use-ink/ink/pull/2144),
[#2137](https://github.com/use-ink/ink/pull/2137), [#2132](https://github.com/use-ink/ink/pull/2132)

## Version 5.0.0-rc.2

### Added
- `instantiate_v2` with additional limit parameters [#2123](https://github.com/use-ink/ink/pull/2123)

### Changed
- Bump metadata version to 5 [#2126](https://github.com/use-ink/ink/pull/2126)
- Use `MaxEncodedLen` for output buffer size [#2128](https://github.com/use-ink/ink/pull/2128)

### Fixed
- Fix alignment in allocator [#2100](https://github.com/use-ink/ink/pull/2100)

## Version 5.0.0-rc.1

### Added
- Custom signature topic in Events - [#2031](https://github.com/use-ink/ink/pull/2031)
- [Linter] `non_fallible_api` lint - [#2004](https://github.com/use-ink/ink/pull/2004)
- [Linter] Publish the linting crates on crates.io - [#2060](https://github.com/use-ink/ink/pull/2060)
- [E2E] Added `create_call_builder` for testing existing contracts - [#2075](https://github.com/use-ink/ink/pull/2075)
- `call_v2` cross-contract calls with additional limit parameters - [#2077](https://github.com/use-ink/ink/pull/2077)
- `instantiate_v2` with additional limit parameters - [#2123](https://github.com/use-ink/ink/pull/2123)
- `delegate_dependency` api calls - [#2076](https://github.com/use-ink/ink/pull/2076)

### Changed
- `Mapping`: Reflect all possible failure cases in comments ‒ [#2079](https://github.com/use-ink/ink/pull/2079)
- [E2E] Rename `.call` to `.call_builder` ‒ [#2078](https://github.com/use-ink/ink/pull/2078)
- Improve syntax for ink! e2e `runtime_only` attribute argument - [#2083](https://github.com/use-ink/ink/pull/2083)
- [E2E] Remove `additional_contracts` parameter [#2098](https://github.com/use-ink/ink/pull/2098)
- [E2E] change node url backend config - [#2101](https://github.com/use-ink/ink/pull/2101)

### Fixed
- Fix the `StorageVec` type by excluding the `len_cached` field from its type info - [#2052](https://github.com/use-ink/ink/pull/2052)
- Fix panic in `approve_for` in the ERC-721 example - [#2092](https://github.com/use-ink/ink/pull/2092)
- ERC-721: `transfer_token_from` now ensures the token owner is correct - [#2093](https://github.com/use-ink/ink/pull/2093)

## Version 5.0.0-rc

### Added
- Allow mutable parameters in messages - [#2004](https://github.com/use-ink/ink/pull/2004)
- Clean E2E configuration parsing - [#1922](https://github.com/use-ink/ink/pull/1922)
- Make `set_code_hash` generic - [#1906](https://github.com/use-ink/ink/pull/1906)
- Provide a `StorageVec` datastructure built on top of `Lazy` - [#1995](https://github.com/use-ink/ink/pull/1995)
- Add fallible methods for `Mapping` and `Lazy` - [#1910](https://github.com/use-ink/ink/pull/1910)
- [E2E] Allow testing with live-chain state - [#1949](https://github.com/use-ink/ink/pull/1949)
- [E2E] Call builders and extra gas margin option - [#1917](https://github.com/use-ink/ink/pull/1917)
- [Linter] `storage_never_freed` lint - [#1932](https://github.com/use-ink/ink/pull/1932)
- [Linter] `strict_balance_equality` lint - [#1914](https://github.com/use-ink/ink/pull/1914)
- [Linter] `no_main` lint - [#2001](https://github.com/use-ink/ink/pull/2001)

### Changed
- Messages return `TypeSpec` directly - [#1999](https://github.com/use-ink/ink/pull/1999)
- Fail when decoding from storage and not all bytes consumed - [#1897](https://github.com/use-ink/ink/pull/1897)
- Support multiple chain extensions - [#1958](https://github.com/use-ink/ink/pull/1958)
  - New example of how to use multiple chain extensions in one contract.
  - Affects the usage of the `#[ink::chain_extension]` macro and the definition of the chain extension.
- Split up `ink_linting` to mandatory and extra libraries - [#2032](https://github.com/use-ink/ink/pull/2032)
- [E2E] resolve DispatchError error details for dry-runs - [#1994](https://github.com/use-ink/ink/pull/1994)
- [E2E] update to new `drink` API - [#2005](https://github.com/use-ink/ink/pull/2005)


## Version 5.0.0-alpha

The preview release of the ink! 5.0.0 release.
This release addresses the majority of issues raised in the OpenZeppelin audit
in particular we addressed the proxy selector clashing attack.
As of this release, ink! only allows exactly one other message with a well-known reserved selector to be defined.
You can read more about the change in the [PR](https://github.com/use-ink/ink/pull/1708).

Other notable changes:

- Rework of event definitions - [#1827](https://github.com/use-ink/ink/pull/1827).
- Updated upgradeable contract example illustrating `delegate_call` - [#1889](https://github.com/use-ink/ink/pull/1889).
- Removal of unchecked arithmetic. `cargo-contract` will fail compiling the contract with raw arithmetic operations - [#1831](https://github.com/use-ink/ink/pull/1831).
- Introduction of an alternative off-chain E2E testing backend, [DRink!](https://github.com/inkdevhub/drink?tab=readme-ov-file#as-an-alternative-backend-to-inks-e2e-testing-framework).
**Big thanks to [@pmikolajczyk41](https://github.com/pmikolajczyk41) for this massive contribution!**

You can see a more detailed log of changes below:

### Added
- Reexport `scale` dependencies, introduce `#[ink::scale_derive]` - [#1890](https://github.com/use-ink/ink/pull/1890)
- Upgradeable contracts example - [#1889](https://github.com/use-ink/ink/pull/1889)
- Persist static buffer size in metadata - [#1880](https://github.com/use-ink/ink/pull/1880)
- Modify static buffer size via environmental variables - [#1869](https://github.com/use-ink/ink/pull/1869)
- Added `sr25519_verify` function to `ink_env` [#1840](https://github.com/use-ink/ink/pull/1840)
- Events `2.0` - [#1827](https://github.com/use-ink/ink/pull/1827)
- Add `set_block_number` to off-chain test api `Engine` - [#1806](https://github.com/use-ink/ink/pull/1806)
- Stabilize `call_runtime` ‒ [#1749](https://github.com/use-ink/ink/pull/1749)
- Schema generation - [#1765](https://github.com/use-ink/ink/pull/1765)
- Restrict wildcard selectors to have exactly one other message - [#1708](https://github.com/use-ink/ink/pull/1708)
- [Linter] Warn when primitive number is annotated as event topic - [#1837](https://github.com/use-ink/ink/pull/1837)
- [Drink backend] allow for arbitrary runtime - [#1892](https://github.com/use-ink/ink/pull/1892)
- [Drink backend] support runtime call - [#1891](https://github.com/use-ink/ink/pull/1891)
- [Drink backend] Make tests generic `E2EBackend` trait - [#1867](https://github.com/use-ink/ink/pull/1867)
- [Drink backend] Backend choice  ‒ [#1864](https://github.com/use-ink/ink/pull/1864)
- [Drink backend] Backend traits - [#1857](https://github.com/use-ink/ink/pull/1857)
- [Drink backend] Abstract error and result structs - [#1844](https://github.com/use-ink/ink/pull/1844)

### Changed
- Reexport `scale` dependencies, introduce `#[ink::scale_derive]` ‒ [#1890](https://github.com/use-ink/ink/pull/1890)
- Use of workspace dependencies and properties - [#1835](https://github.com/use-ink/ink/pull/1835)
- Remove of unchecked arithmetic - [#1831](https://github.com/use-ink/ink/pull/1831)
- Use `decode_all` for decoding cross contract call result - [#1810](https://github.com/use-ink/ink/pull/1810)
- [E2E] build contracts at runtime instead of during codegen - [#1881](https://github.com/use-ink/ink/pull/1881)
- [E2E] crate refactoring - [#1830](https://github.com/use-ink/ink/pull/1830)
- [E2E] improve call API, remove `build_message` + callback - [#1782](https://github.com/use-ink/ink/pull/1782)

### Fixed
- `RootLayout::new()` is generic again to allow using `ink_metadata` in pure `PortableForm` contexts - [#1989](https://github.com/use-ink/ink/pull/1989)

## 4.3.0

### Fixed

- Fix E2E tests for newer rust toolchain & contracts node - #[1884](https://github.com/use-ink/ink/pull/1884)
- Enable Rust >= `1.70`, update `subxt` and `contract-build` - [#1855](https://github.com/use-ink/ink/pull/1855)
- Fix unsupported `--ws-port` argument - [#1850](https://github.com/use-ink/ink/pull/1850)

## Version 4.2.0

### Added
- Persist `Environment` in metadata ‒ [#1741](https://github.com/use-ink/ink/pull/1741)
- Added possibility for `runtime_call` in E2E tests ‒ [#1736](https://github.com/use-ink/ink/pull/1736)
- Added `default` attribute to constructors and messages ‒ [#1724](https://github.com/use-ink/ink/pull/1724)
- Added clarification about `Mapping::size` unit ‒ [#1735](https://github.com/use-ink/ink/pull/1735)

### Changed
- Upgraded `syn` to version `2` ‒ [#1731](https://github.com/use-ink/ink/pull/1731)
- Update `scale-info` requirement to `2.5` ‒ [#1733](https://github.com/use-ink/ink/pull/1733)
- Bump `subxt` to `0.28.0` ‒ [#1750](https://github.com/use-ink/ink/pull/1750)

## Version 4.1.0

### Added
- Basic support for `dyn Trait` to allow cross-contract calls only with trait - [#1673](https://github.com/use-ink/ink/pull/1673)
- E2E: auto detect contracts to be built - [#1691](https://github.com/use-ink/ink/pull/1691)
- Add `set_code_hash` to `EnvAccess` - [#1698](https://github.com/use-ink/ink/pull/1698)
- Add `set_block_timestamp` to off-chain test api `Engine` - [#1721](https://github.com/use-ink/ink/pull/1721)

### Changed
- Support conditional compilation - [#1707](https://github.com/use-ink/ink/pull/1707)

## Version 4.0.1

### Fixed
- Fixing `ManualKey<0>` to act properly - [#1670](https://github.com/use-ink/ink/pull/1670)
- Indicated latest release of `cargo-contract` in `e2e` crate

### Added
- Add `call-runtime` support - [#1641](https://github.com/use-ink/ink/pull/1641)

## Version 4.0.0

The latest stable release of ink! is here 🥳

This version brings a lot of usability improvements, making the language better suited
for the needs of production parachains.

A couple of highlights include:
- Changes to how contract storage works, which significantly reduced the sizes of
  contract binaries
- A new end-to-end testing framework, letting you easily write integration tests
- Changes to the metadata format, which (in part) makes error handling more expressive

There's a lot more to dig through, so take some time to poke around the `CHANGELOG`
(including the `4.0.0-alpha` and `4.0.0-beta` releases).

You may notice there have been a few breaking changes. No need to be scared though, we
wrote up a [migration guide](https://use.ink/faq/migrating-from-ink-3-to-4) covering all
of the breaking changes and how to update your ink! 3.x contract accordingly.

Thanks to everyone that helped make this release possible ❤️

### Compatibility

In order to build contracts which use ink! `v4.0.0` you need to use
`cargo-contract`
[`v2.0.0`](https://github.com/use-ink/cargo-contract/releases/tag/v2.0.0).
You can install it as follows:

`cargo install cargo-contract --force --locked`

You will also need to use a version of [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [polkadot-v0.9.37](https://github.com/paritytech/substrate/tree/polkadot-v0.9.37)
in your node.

The [`v0.24.0`](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.24.0)
release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) is
compatible with the ink! `4.0.0` release.

For full compatibility requirements see the [migration guide](https://use.ink/faq/migrating-from-ink-3-to-4/#compatibility).

- Add `Mapping::contains(key)` and `Mapping::insert_return_size(key, val)` ‒ [#1224](https://github.com/use-ink/ink/pull/1224)
- Add [`payment-channel`](https://github.com/use-ink/ink-examples/tree/main/payment-channel) example ‒ [#1248](https://github.com/use-ink/ink/pull/1248) (thanks [@kanishkatn](https://github.com/kanishkatn)!)
- Add `version` field to ink! metadata ‒ [#1313](https://github.com/use-ink/ink/pull/1313)
- The `rand-extension` example has been adapted to an updated version of the `ChainExtension` API ‒ [#1356](https://github.com/use-ink/ink/pull/1356)
- Add `ink_env::pay_with_call!` helper macro for off-chain emulation of sending payments with contract message calls ‒ [#1379](https://github.com/use-ink/ink/pull/1379)
- Allow using `Result<Self, Error>` as a return type in constructors ‒ [#1446](https://github.com/use-ink/ink/pull/1446)
- Add `Mapping::take()` function allowing to get a value removing it from storage ‒ [#1461](https://github.com/use-ink/ink/pull/1461)
- Add E2E testing framework MVP ‒ [#1395](https://github.com/use-ink/ink/pull/1395)
- Add E2E tests for `Mapping` functions - [#1492](https://github.com/use-ink/ink/pull/1492)
- E2E: expose call dry-run method ‒ [#1624](https://github.com/use-ink/ink/pull/1624)
- Make cross-contract callee non-optional ‒ [#1636](https://github.com/use-ink/ink/pull/1636)
- Support custom environment in E2E tests - [#1645](https://github.com/use-ink/ink/pull/1645) (thanks [@pmikolajczyk41](https://github.com/pmikolajczyk41)!)

### Changed
- Contract size optimization in case contract doesn't accept payment ‒ [#1267](https://github.com/use-ink/ink/pull/1267) (thanks [@xgreenx](https://github.com/xgreenx)!)
- Move ink! linter into `ink` repository ‒ [#1361](https://github.com/use-ink/ink/pull/1267)
- Introduce `ink` entrance crate ‒ [#1223](https://github.com/use-ink/ink/pull/1223)
- Use `XXH32` instead of `sha256` for calculating storage keys ‒ [#1393](https://github.com/use-ink/ink/pull/1393)
- Storage Refactoring ‒ [#1331](https://github.com/use-ink/ink/pull/1331)
- Add support for language level errors (`LangError`) ‒ [#1450](https://github.com/use-ink/ink/pull/1450)
- Return `LangError`s from constructors ‒ [#1504](https://github.com/use-ink/ink/pull/1504)
- Update `scale-info` requirement to `2.3` ‒ [#1467](https://github.com/use-ink/ink/pull/1467)
- Merge `Mapping::insert(key, val)` and `Mapping::insert_return_size(key, val)` into one method - [#1463](https://github.com/use-ink/ink/pull/1463)
- FFI: no more `__unstable__` wasm import module ‒ [#1522](https://github.com/use-ink/ink/pull/1522)
- Clean up CallBuilder `return()` type ‒ [#1525](https://github.com/use-ink/ink/pull/1525)
- Fix trait message return type metadata ‒ [#1531](https://github.com/use-ink/ink/pull/1531)
- Bump Dylint dependencies ‒ [#1551](https://github.com/use-ink/ink/pull/1551)
- Stabilize `take_storage` ‒ [#1568](https://github.com/use-ink/ink/pull/1568)
- Chain Extension: Evaluation of method return type at compile time ‒ [#1569](https://github.com/use-ink/ink/pull/1569)
- Make more functions be const ‒ [#1574](https://github.com/use-ink/ink/pull/1574) (thanks [@yjhmelody](https://github.com/yjhmelody)!)
- Unify fallible and non fallible `instantiate` methods ‒ [#1591](https://github.com/use-ink/ink/pull/1591)
- Make `CallBuilder` and `CreateBuilder` error handling optional ‒ [#1602](https://github.com/use-ink/ink/pull/1602)
- Rename `CallBuilder::fire()` method to `invoke()` ‒ [#1604](https://github.com/use-ink/ink/pull/1604)
- chore: add minimum rust version to the ink crate ‒ [#1609](https://github.com/use-ink/ink/pull/1609) (thanks [@Kurtsley](https://github.com/Kurtsley)!)
- Rename `_checked` codegen call methods with `try_` ‒ [#1621](https://github.com/use-ink/ink/pull/1621)
- Bump Substrate and `subxt` dependencies ‒ [#1549](https://github.com/use-ink/ink/pull/1549)
- E2E: spawn a separate contracts node instance per test ‒ [#1642](https://github.com/use-ink/ink/pull/1642)

### Fixed
- Trim single whitespace prefix in the metadata `docs` field ‒ [#1385](https://github.com/use-ink/ink/pull/1385)
- Allow pay_with_call to take multiple arguments ‒ [#1401](https://github.com/use-ink/ink/pull/1401)
- Add Determinism enum from pallet-contracts ‒ [#1547](https://github.com/use-ink/ink/pull/1547)
- Added missed `WhereClosure` for the generics into `storage_item` ‒ [#1536](https://github.com/use-ink/ink/pull/1536) (thanks [@xgreenx](https://github.com/xgreenx)!)

### Removed
- Implement `ecdsa_to_eth_address()` and remove `eth_compatibility` crate ‒ [#1233](https://github.com/use-ink/ink/pull/1233)
- Remove `wee-alloc` ‒ [#1403](https://github.com/use-ink/ink/pull/1403)
- Remove `ink_env::random` function ‒ [#1442](https://github.com/use-ink/ink/pull/1442)
- Remove `Default` implementation for AccountId ‒ [#1255](https://github.com/use-ink/ink/pull/1255)

## Version 4.0.0-rc

The first release candidate is here! This is the first release which could become the final
`v4.0.0`. Any subsequent release candidates should only contain bug fixes: no API changes,
breaking or otherwise.

### Breaking Changes

1. We've renamed some of the generated message methods on the `ContractRef` struct. They
   have been changed from `_checked` to `try_` ([#1621](https://github.com/use-ink/ink/pull/1621))
1. We have removed the `Default` implementation for `AccountId`s. This is because of
   security concerns around the use of the zero address which has a known private key in
   the `sr25519` and `ed25519` curves ([#1255](https://github.com/use-ink/ink/pull/1255)).
1. We have replaced the `CallBuilder::callee()` method with a `call()` method and also
   added a `delegate()` method. This streamlines the `Call` and `DelegateCall` workflows
   ([#1636](https://github.com/use-ink/ink/pull/1636))

### Added
- E2E: expose call dry-run method ‒ [#1624](https://github.com/use-ink/ink/pull/1624)
- Make cross-contract callee non-optional ‒ [#1636](https://github.com/use-ink/ink/pull/1636)

### Changed
- Rename `_checked` codegen call methods with `try_` ‒ [#1621](https://github.com/use-ink/ink/pull/1621)
- Bump Substrate and `subxt` dependencies ‒ [#1549](https://github.com/use-ink/ink/pull/1549)

### Removed
- Remove `Default` implementation for AccountId ‒ [#1255](https://github.com/use-ink/ink/pull/1255)

## Version 4.0.0-beta.1
The coolest feature included in this release is the first published version of
ink!'s native ["end-to-end" (E2E) testing framework](https://github.com/use-ink/ink/issues/1234).

This enables testing of a contract by deploying and calling it on a Substrate node with
`pallet-contracts`. See the [`erc20` example](./examples/erc20/lib.rs) for usage.

### Breaking Changes
This release includes a couple of breaking changes.

1. The `CallBuilder::returns()` method does not require an extra `MessageResult` anymore
   as the type is now added under the hood [(#1525)](https://github.com/use-ink/ink/pull/1525)
1. The `CallBuilder::invoke()` and `CreateBuilder::instantiate() `methods now unwrap the
   `Result` from `pallet-contracts` under the hood ([#1602](https://github.com/use-ink/ink/pull/1602))
   If you wish to handle the error use the new `try_` variants of those methods instead.
1. The `CallBuilder::fire()` method has been renamed to `invoke()`
   ([#1604](https://github.com/use-ink/ink/pull/1604))
1. The `returns_result` flag has been removed from the `#[ink(extension = …)]` attribute
   ([#1569](https://github.com/use-ink/ink/pull/1569))
   We now infer this information at compile time. If `handle_status` is set to `true`,
   the return type will still be wrapped into `Result` as before.
1. The Minimum Supported Rust Version (MSRV) has been set to `1.63.0`. This was already
   the case, but previously it was enforced by `cargo-contract` instead of ink!
   ([#1609](https://github.com/use-ink/ink/pull/1609))

### Added
- Add E2E testing framework MVP ‒ [#1395](https://github.com/use-ink/ink/pull/1395)
- Add E2E tests for `Mapping` functions - [#1492](https://github.com/use-ink/ink/pull/1492)

### Fixed
- Add Determinism enum from pallet-contracts ‒ [#1547](https://github.com/use-ink/ink/pull/1547)
- Added missed `WhereClosure` for the generics into `storage_item` ‒ [#1536](https://github.com/use-ink/ink/pull/1536) (thanks [@xgreenx](https://github.com/xgreenx))

### Changed
- Handle `LangError` from instantiate ‒ [#1512](https://github.com/use-ink/ink/pull/1512)
- FFI: no more `__unstable__` wasm import module ‒ [#1522](https://github.com/use-ink/ink/pull/1522)
- Clean up CallBuilder `return()` type ‒ [#1525](https://github.com/use-ink/ink/pull/1525)
- Fix trait message return type metadata ‒ [#1531](https://github.com/use-ink/ink/pull/1531)
- Bump Dylint dependencies ‒ [#1551](https://github.com/use-ink/ink/pull/1551)
- Stabilize `take_storage` ‒ [#1568](https://github.com/use-ink/ink/pull/1568)
- Chain Extension: Evaluation of method return type at compile time ‒ [#1569](https://github.com/use-ink/ink/pull/1569)
- Make more functions be const ‒ [#1574](https://github.com/use-ink/ink/pull/1574) (thanks [@yjhmelody](https://github.com/yjhmelody))
- Unify fallible and non fallible `instantiate` methods ‒ [#1591](https://github.com/use-ink/ink/pull/1591)
- Make `CallBuilder` and `CreateBuilder` error handling optional ‒ [#1602](https://github.com/use-ink/ink/pull/1602)
- Rename `CallBuilder::fire()` method to `invoke()` ‒ [#1604](https://github.com/use-ink/ink/pull/1604)
- chore: add minimum rust version to the ink crate ‒ [#1609](https://github.com/use-ink/ink/pull/1609) (thanks [@Kurtsley](https://github.com/Kurtsley))

## Version 4.0.0-beta

The focus of the first `beta` release is to establish the stable ABI for the final `4.0.0`
release. It means that whilst subsequent `beta` releases may contain breaking contract
*code* changes, the ABI will remain the same so that any contract compiled and deployed
with `4.0.0-beta` continue to be compatible with all future `4.0.0` versions.

### Compatibility
In order to build contracts which use ink! `v4.0.0-beta` you need to use
`cargo-contract`
[`v2.0.0-beta`](https://github.com/use-ink/cargo-contract/releases/tag/v2.0.0-beta).
You can install it as follows:

`cargo install cargo-contract --version 2.0.0-beta`

You will also need to use a version of [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [265e3f1](https://github.com/paritytech/substrate/commit/265e3f12a2937fe4f71280b3652471627609d04f)
(Nov 3, 2022) in your node.

The [`v0.22.1`](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.22.1)
release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) is
compatible with the ink! `4.0.0-beta` release.

### Breaking Changes

## Constructors and Messages now return `LangError`s

We have added a way to handle errors that are neither specific to a particular contract,
nor from the underlying execution environment (e.g `pallet-contracts`). Instead these are
errors that may come from the smart contracting language itself.

For example, take the case where a contract message is called using an invalid selector.
This is not something a smart contract author should need to define as failure case, nor
is it something that `pallet-revive` needs to be aware of.

Previously, the contract execution would trap if an invalid selector was used, leaving
callers with no way to handle the error gracefully. This can now be handled with the help
of the newly added `LangError`.

In short, this change means that all ink! messages and constructors now return a
`Result<R, LangError>`, where `R` is the original return type. Contract callers can
choose to handle the `LangError`.

In order to make this error compatible with other languages we have also added a
`lang_error` field to the metadata format. This will be the central registry of all the
different error variants which languages may want to emit in the future.

Related pull-requests:
- https://github.com/use-ink/ink/pull/1450
- https://github.com/use-ink/ink/pull/1504

Related discussions:
- https://github.com/use-ink/ink/issues/1207
- https://github.com/paritytech/substrate/issues/11018
- https://github.com/use-ink/ink/issues/1002

## Random function removed
We had to remove [`ink_env::random`](https://docs.rs/ink_env/3.3.1/ink_env/fn.random.html)
with [#1442](https://github.com/use-ink/ink/pull/1442).
This function allowed contract developers getting random entropy.
There is unfortunately no way how this can be done safely enough
with built-in Substrate primitives on-chain currently. We're
following the recommendation of our auditors to remove it.

The alternative right now is to provide random entropy off-chain to
the contract, to use a random entropy oracle, or to have a chain-extension
that does this, in case the chain has a possibility to do so.

We hope to bring this function back in a future release of ink!, the
best hope right now is that it could come back with [Sassafras](https://wiki.polkadot.network/docs/learn-consensus#badass-babe-sassafras), a block production
protocol for future versions of Polkadot.

### Added
- Allow using `Result<Self, Error>` as a return type in constructors ‒ [#1446](https://github.com/use-ink/ink/pull/1446)
- Add `Mapping::take()` function allowing to get a value removing it from storage ‒ [#1461](https://github.com/use-ink/ink/pull/1461)

### Changed
- Add support for language level errors (`LangError`) ‒ [#1450](https://github.com/use-ink/ink/pull/1450)
- Return `LangError`s from constructors ‒ [#1504](https://github.com/use-ink/ink/pull/1504)
- Update `scale-info` requirement to `2.3` ‒ [#1467](https://github.com/use-ink/ink/pull/1467)
- Merge `Mapping::insert(key, val)` and `Mapping::insert_return_size(key, val)` into one method - [#1463](https://github.com/use-ink/ink/pull/1463)

### Removed
- Remove `ink_env::random` function ‒ [#1442](https://github.com/use-ink/ink/pull/1442)

## Version 4.0.0-alpha.3

### Breaking Changes

#### New `ink` crate
The `ink_lang` crate has been replaced in [#1223](https://github.com/use-ink/ink/pull/1223) by a new top level `ink`
crate. All existing sub-crates are reexported and should be used via the new `ink` crate, so e.g. `ink::env` instead of
`ink_env`. Contract authors should now import the top level `ink` crate instead of the individual crates.

##### Migration
- In `Cargo.toml` Replace all individual `ink_*` crate dependencies with the `ink` crate.
- In the contract source:
  - Remove the commonly used `use ink_lang as ink` idiom.
  - Replace all usages of individual crates with reexports, e.g. `ink_env` ➜ `ink::env`.

#### Storage Rework
[#1331](https://github.com/use-ink/ink/pull/1331) changes the way `ink!` works with contract storage. Storage keys
are generated at compile-time, and user facing abstractions which determine how contract data is laid out in storage
have changed.

##### Migration
- Initialize `Mapping` fields with `Mapping::default()` instead of  `ink_lang::utils::initialize_contract` in
constructors. See [`erc20`](./examples/erc20/lib.rs) and other examples which use a `Mapping`.
- Remove `SpreadAllocate`, `SpreadLayout` and `PackedLayout` implementations.

#### Removal of `wee-alloc` support
ink! uses a bump allocator by default, additionally we supported another allocator (`wee-alloc`)
through a feature flag. `wee-alloc` is no longer maintained and we removed support for it.

### Changed
- Introduce `ink` entrance crate ‒ [#1223](https://github.com/use-ink/ink/pull/1223)
- Use `XXH32` instead of `sha256` for calculating storage keys ‒ [#1393](https://github.com/use-ink/ink/pull/1393)
- Storage Refactoring ‒ [#1331](https://github.com/use-ink/ink/pull/1331)

### Fixed
- Trim single whitespace prefix in the metadata `docs` field ‒ [#1385](https://github.com/use-ink/ink/pull/1385)
- Allow pay_with_call to take multiple arguments ‒ [#1401](https://github.com/use-ink/ink/pull/1401)

### Added
- Add `ink_env::pay_with_call!` helper macro for off-chain emulation of sending payments with contract message calls ‒ [#1379](https://github.com/use-ink/ink/pull/1379)

### Removed
- Remove `wee-alloc` ‒ [#1403](https://github.com/use-ink/ink/pull/1403)

## Version 4.0.0-alpha.2

**This version was skipped due to an error during the release process**

As part of this series of `alpha`s we are looking for ways to streamline our release process,
so small errors like this are learning experiences for us.

## Version 4.0.0-alpha.1

### Compatibility
In order to build contracts which use ink! `v4.0.0-alpha.1` you need to use
`cargo-contract`
[`v2.0.0-alpha.1`](https://github.com/use-ink/cargo-contract/releases/tag/v2.0.0-alpha.1).
You can install it as follows:

`cargo install cargo-contract --version 2.0.0-alpha.1`

We recommend using a version of the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [6b85535](https://github.com/paritytech/substrate/tree/6b8553511112afd5ae7e8e6877dc2f467850f155)
(Aug 12, 2022) in your node.

The compatibility issues will be with `ChainExtension`'s and the functions mentioned above.

### Breaking Changes
This release contains a few breaking changes. These are indicated with the :x: emoji.
Most of these were initially introduced in `v3.1.0` and `v3.2.0` releases but
compatibility was restored in `v3.3.0`.

- As part of [#1224](https://github.com/use-ink/ink/pull/1224) the return type of `ink_env::set_contract_storage()` was changed to
return an `Option<u32>` instead of `()`.
- As part of [#1233](https://github.com/use-ink/ink/pull/1233) the `eth_compatibility` crate was removed. The
  `ecdsa_to_eth_address()` function from it can now be found in the `ink_env` crate.
- As part of [#1267](https://github.com/use-ink/ink/pull/1267) an argument to `ink_lang::codegen::execute_constructor()` (which is
  used internally by the ink! macros) was removed.
- As part of [#1313](https://github.com/use-ink/ink/pull/1313) the ink! ABI was changed so that the version was specified using a
  dedicated `version` key instead of an implicit key which wrapped the entire ABI.

### Added
- :x: Add `Mapping::contains(key)` and `Mapping::insert_return_size(key, val)` ‒ [#1224](https://github.com/use-ink/ink/pull/1224)
- Add [`payment-channel`](https://github.com/use-ink/ink-examples/tree/main/payment-channel) example ‒ [#1248](https://github.com/use-ink/ink/pull/1248)
- :x: Add `version` field to ink! metadata ‒ [#1313](https://github.com/use-ink/ink/pull/1313)
- The `rand-extension` example has been adapted to an updated version of the `ChainExtension` API ‒ [#1356](https://github.com/use-ink/ink/pull/1356)

### Changed
- :x: Contract size optimization in case contract doesn't accept payment ‒ [#1267](https://github.com/use-ink/ink/pull/1267) (thanks [@xgreenx](https://github.com/xgreenx)).
- Move ink! linter into `ink` repository ‒ [#1361](https://github.com/use-ink/ink/pull/1267)

### Removed
- :x: Implement `ecdsa_to_eth_address()` and remove `eth_compatibility` crate ‒ [#1233](https://github.com/use-ink/ink/pull/1233)

## Version 3.3.1

At the moment teams which use both Substrate and ink! in the same codebase are
[experiencing some issues](https://github.com/use-ink/ink/pull/1348#issuecomment-1207477615)
with updating to Substrate's [`polkadot-0.9.27` branch](https://github.com/paritytech/substrate/tree/polkadot-v0.9.27).
This is because that branch uses the `secp256k1@0.24`, which is incompatible with
`secp256k1@0.22`

This release bumps the `secp256k1` version from `v0.22` to `v0.24`.

## Version 3.3.0

This release restores SemVer compatibility in the `v3.x` series of releases, as well as
compatibility with the [`v0.13.0`](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.13.0)
release of the `substrate-contracts-node`.

### Compatibility
This version will work fine with *substrate-contracts-node* versions from
[0.13.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.13.0) up
to [0.16.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.16.0).

### Changed
*Context: user-reported issues on our SE unveiled backward incompatibility introduced in 3.1.0 release.*
1. [CodeRejected when using ink! v3.1.0](https://substrate.stackexchange.com/questions/2721/cargo-contract-3-0-1)
1. [Incompatibility between ink! v3.0.1 and v3.2.0 ](https://substrate.stackexchange.com/questions/2870/cargo-contract-throws-error-about-supplied-arguments-in-inkconstructor-f)

The following has been done to restore backward compatibility:
- Reverted backward-incompatible piece of [#1224](https://github.com/use-ink/ink/pull/1224).
    - The return signature of `ink_env::set_contract_storage()` was changed to return an
      `Option<u32>`. This could have broken existing code, so this should've been done in
      a `MAJOR` release.
    - Under the hood the PR also changed `Mapping::insert()` to use a new SEAL API
    (`[seal1] seal_set_storage`), which resulted in `CodeRejected` errors in nodes which
    did not have this API (e.g `substrate-contracts-node@0.13.0`).
- Reverted "Optimise deny_payment. Use everywhere semantic of deny ([#1267](https://github.com/use-ink/ink/pull/1267))"
  - This one is to restore compatibility between minor versions of ink! crates; see
    @HCastano's SE [answer](https://substrate.stackexchange.com/a/3000/472) in this
    regard.
- Reverted backward-incompatible piece of [#1233](https://github.com/use-ink/ink/pull/1233).
    - The removal of the `eth_compatibility` crate should have been done in a `MAJOR`
      release.

All these breaking changes are subjects to the upcoming MAJOR *ink!* release 4.0.0.

## Version 3.2.0

### Compatibility
We recommend using a version of the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [c0ee2ad](https://github.com/paritytech/substrate/tree/c0ee2adaa54b22ee0df5d1592cd0430961afd95c)
(May 23, 2022) in your node.

This is the case in the latest release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node)
[v0.16.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.16.0).

### Added
- Contract size optimization in case contract doesn't accept payment ‒ [#1267](https://github.com/use-ink/ink/pull/1267) (thanks [@xgreenx](https://github.com/xgreenx)).

### Changed
- Two functions have been stabilized: [`ink_env::ecdsa_recover`](https://use-ink.github.io/ink/ink_env/fn.ecdsa_recover.html) and [`ink_env::ecdsa_to_eth_address`](https://use-ink.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html) ‒ [#1270](https://github.com/use-ink/ink/pull/1270) [#1273](https://github.com/use-ink/ink/pull/1273)

### Fixed
- Fixed bug with recent Rust and `cargo test` ‒ [#1272](https://github.com/use-ink/ink/pull/1272) (thanks [@xgreenx](https://github.com/xgreenx)).

## Version 3.1.0

### Compatibility
We recommend using a version of the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [7d233c2](https://github.com/paritytech/substrate/tree/7d233c2446b5a60662400a0a4bcfb78bb3b79ff7)
(May 13, 2022) in your node.

This is the case in the latest release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node)
[v0.15.1](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.15.1) and
the latest Polkadot release [v0.9.22](https://github.com/paritytech/polkadot/releases/tag/v0.9.22).

### Breaking Changes
There are two breaking changes in this release:

* As part of [#1235](https://github.com/use-ink/ink/pull/1235) the message selectors of
  your contract may change. A change of selectors would affect your client, frontend, Dapp, etc..
* As part of [#1233](https://github.com/use-ink/ink/pull/1235) we removed the `eth_compatibility`
  crate.<br><br>
  Its recovery functionality has been moved to `ink_env` now: [`ink_env::ecdsa_to_eth_address`](https://use-ink.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html).
  The reason for this change is that we moved the gas-expensive crypto operations into `pallet-contracts`.<br><br>
  The crates `to_default_account_id` function has been removed; the reason for this change is that ink!
  doesn't have knowledge about the specific Substrate types on the underlying chain.
  If you want to retain the function in your contract and are just using standard Substrate types
  you should add the prior functionality to your contract ‒ it was a simple
  `<Blake2x256 as CryptoHash>::hash(&ecdsa_pubkey[u8; 33])`.

### New API functions
We added two new `Mapping` API functions:
[`Mapping::contains`](https://use-ink.github.io/ink/ink_storage/struct.Mapping.html#method.contains) and
[`Mapping::insert_return_size`](https://use-ink.github.io/ink/ink_storage/struct.Mapping.html#method.insert_return_size) ‒ [#1224](https://github.com/use-ink/ink/pull/1224).
These are more gas-efficient than whatever you were using previously.

Additionally there are a couple new `ink_env` functions now:
* [`ink_env::set_code_hash`](https://use-ink.github.io/ink/ink_env/fn.set_code_hash.html)
* [`ink_env::own_code_hash`](https://use-ink.github.io/ink/ink_env/fn.own_code_hash.html)
* [`ink_env::code_hash`](https://use-ink.github.io/ink/ink_env/fn.code_hash.html)
* [`ink_env::ecdsa_to_eth_address`](https://use-ink.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html)

### New Upgradeable Contract Example

We've added a new example: [`upgradeable-contracts/set-code-hash`](https://github.com/use-ink/ink-examples/tree/main/upgradeable-contracts#set-code-hash#set-code-hash).

It illustrates how the newly added [`ink_env::set_code_hash`](https://use-ink.github.io/ink/ink_env/fn.set_code_hash.html)
can be used to implement an upgradeable contract that replaces its own code.

### Added
- Implement `seal_code_hash` and `seal_own_code_hash` ‒ [#1205](https://github.com/use-ink/ink/pull/1205)
- Add `set_code_hash` function and example ‒ [#1203](https://github.com/use-ink/ink/pull/1203)
- Implement [`ink_env::ecdsa_to_eth_address`](https://use-ink.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html) ‒ [#1233](https://github.com/use-ink/ink/pull/1233)
- Add [`Mapping::contains(key)`](https://use-ink.github.io/ink/ink_storage/struct.Mapping.html#method.contains) and [`Mapping::insert_return_size(key, val)`](https://use-ink.github.io/ink/ink_storage/struct.Mapping.html#method.insert_return_size) ‒ [#1224](https://github.com/use-ink/ink/pull/1224)

### Fixed
- Fix ordering of message ids if the trait is implemented before the inherent section ‒ [#1235](https://github.com/use-ink/ink/pull/1235)

### Removed
- Removed `eth_compatibility` crate and moved its functionality partly into `ink_env` ‒ [#1233](https://github.com/use-ink/ink/pull/1233)

## Version 3.0.1

### Changed
- Improve upgradeable examples folder structure, explain differences ‒ [#1188](https://github.com/use-ink/ink/pull/1188)

### Fixed
- Update codegen after SCALE v3.1.2 release ‒ [#1189](https://github.com/use-ink/ink/pull/1189)
- Stop using `CallData` in `multisig` example doc test ‒ [#1202](https://github.com/use-ink/ink/pull/1202)

## Version 3.0.0

This is the stable release for ink! 3.0.

It took us a while to get here and going forward we want to do
smaller releases more often.

*Please note that ink! has not been audited.*

### Compatibility
We recommend using a version of the `contracts` pallet later than
[cc282f84ba53ed2a08374d2a655dc8f08cbc5e86](https://github.com/paritytech/substrate/tree/cc282f84ba53ed2a08374d2a655dc8f08cbc5e86)
(March 15, 2022) in your node.

This is the case in the latest release of the `substrate-contracts-node`:
[v0.10.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.10.0).

### Breaking Changes
#### We replaced the default off-chain testing environment
The off-chain testing environment can be used to write unit tests
for your smart contract with a simulated chain.
We've now replaced the existing off-chain environment with a new
one, which has a bit of a different API.

The major changes are that there is no longer any notion of "execution
context" ‒ so no more `push_execution_context` or `pop_execution_context`.
You can achieve all the same things with the new API, see [here](https://use-ink.github.io/ink/ink_env/test/index.html)
for the API documentation.

We've also switched all our examples to this new environment, you
can find more "template use-cases" there (e.g. for
[chain extension testing](https://github.com/use-ink/ink-examples/tree/main/rand-extension))

#### We removed the dynamic storage allocator
More details on the reasoning behind this can be found in [#1148](https://github.com/use-ink/ink/pull/1148).

#### `CallBuilder` API changed to support `delegate` calls
The `CallBuilder` API changed to now support two types of calls:

* `Call`: a cross-contract call.<br/>
   This was the default until this new API change.
* `DelegateCall`: a delegated call.<br/>
  This enables writing upgradeable contracts using
  the `delegate` pattern. An example has been added to demonstrate this:
  [`delegate-calls`](https://github.com/use-ink/ink-examples/tree/main/upgradeable-contracts).

This is a breaking change, users must now specify the `call_type` to the builder manually.

_If you want to keep existing behavior you just need to specify the type `Call` now._

More details on this change can be found in [#1133](https://github.com/use-ink/ink/pull/1133).

#### Unify `ink_env::{eval_contract, invoke_contract}`

The API for `eval_contract` and `invoke_contract` changed. You can read more
about the change in [#1165](https://github.com/use-ink/ink/pull/1165).

### Added
- Added `keep_attr` to `#[ink::contract]` and `#[ink::trait_definition]` ‒ [#1145](https://github.com/use-ink/ink/pull/1145) (thanks [@xgreenx](https://github.com/xgreenx))..
- Implemented the `seal_is_contract` and `seal_caller_is_origin` API ‒ [#1129](https://github.com/use-ink/ink/pull/1129) [#1166](https://github.com/use-ink/ink/pull/1166).
- Add tests in experimental off-chain env for `trait-erc20` ‒ [#1158](https://github.com/use-ink/ink/pull/1158).
- Add tests in experimental off-chain env for `erc721` ‒ [#1157](https://github.com/use-ink/ink/pull/1157).
- Add tests in experimental off-chain env for `multisig` ‒ [#1159](https://github.com/use-ink/ink/pull/1159).
- Add tests in experimental off-chain env for `dns` ‒ [#1156](https://github.com/use-ink/ink/pull/1156).
- Implemented chain extension testing in experimental off-chain env ‒ [#1152](https://github.com/use-ink/ink/pull/1152).

### Changed
- Replaced default off-chain testing engine with experimental one ‒ [#1144](https://github.com/use-ink/ink/pull/1144).
- Changed `CallBuilder` API to now support delegate calls ‒ [#1133](https://github.com/use-ink/ink/pull/1133) (thanks [@VargSupercolony](https://github.com/VargSupercolony) and [@xgreenx](https://github.com/xgreenx)).
- Unify `ink_env::{eval_contract, invoke_contract}` ‒ [#1165](https://github.com/use-ink/ink/pull/1165).

### Removed
- Removed the dynamic storage allocator ‒ [#1148](https://github.com/use-ink/ink/pull/1148).
- Removed `compile_as_dependency` config option ‒ [#1168](https://github.com/use-ink/ink/pull/1168).

## Version 3.0-rc9

This is the 9th release candidate for ink! 3.0.

### Breaking Changes
#### We removed all data structures other than `Mapping` from the public ink! API

This is a drastic breaking change; it was no easy decision for us.
It affects `Lazy` and `Memory` as well. The rationale behind this change,
as well as some notes on migrating existing contracts, are explained
in [#1111](https://github.com/use-ink/ink/pull/1111) and
[#1137](https://github.com/use-ink/ink/pull/1137).

If you used `Memory` in your contract, you can achieve the same functionality
by passing this data via arguments. If you think there's a case to be
made for bringing it back, please get in contact with us.

If you use [`ink_storage::Mapping`](https://use-ink.github.io/ink/ink_storage/struct.Mapping.html)
in your contract, you need to initialize the data structure using the helper function
[`ink_lang::utils::initialize_contract(…)`](https://use-ink.github.io/ink/ink_lang/utils/fn.initialize_contract.html).
For more code examples you can take a look at our examples, e.g.
[`erc20`](https://github.com/use-ink/ink/blob/master/examples/erc20/lib.rs).

#### Please upgrade `scale-info` and `parity-scale-codec` in your contract's dependencies

In this release candidate we upgraded `scale-info` and `parity-scale-codec`. You have to use a compatible
version in your contract's `Cargo.toml` as well; `cargo-contract` will throw an error otherwise.

The `Cargo.toml` should contain
```
scale-info = { version = "2", default-features = false, features = ["derive"], optional = true }
scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive", "full"] }
```

### Added
- Export `ink_lang::utils::initialize_contract(…)` - [#1077](https://github.com/use-ink/ink/pull/1077).
- Add `get_owner()` function to `dns` example contract - [#1118](https://github.com/use-ink/ink/pull/1118) (thanks [@agryaznov](https://github.com/agryaznov)).
- Improved usage documentation of `ink_storage::Mapping` - [#1138](https://github.com/use-ink/ink/pull/1138).

### Changed
- Updated to `parity-scale-codec = "3"` and `scale-info = "2"` - [#1132](https://github.com/use-ink/ink/pull/1132).

### Removed
- Remove `collection` and `lazy` modules from public ink! API - [#1111](https://github.com/use-ink/ink/pull/1111).
- Remove `Memory` from public ink! API - [#1137](https://github.com/use-ink/ink/pull/1137).

### Fixed
- Fix bug with referencing two external trait definitions - [#1141](https://github.com/use-ink/ink/pull/1141).
- Explicitly specify trait in dispatching - [#1131](https://github.com/use-ink/ink/pull/1131) (thanks [@xgreenx](https://github.com/xgreenx)).
- Make `rust-analyzer` expand ink! macros without warning - [#1107](https://github.com/use-ink/ink/pull/1107).

## Version 3.0-rc8

This is the 8th release candidate for ink! 3.0.

### Compatibility

You need to upgrade to a new version of the `contracts` pallet in order to use this
ink! release. Specifically we recommend you upgrade to at least the pallet version
[249dbbb](https://github.com/paritytech/substrate/commit/249dbbba6a1a277a3098c2a5b302645da16451ad)
(or newer than Jan 17, 2022).

For `substrate-contracts-node` we just issued a new release `v0.5.0`, which has been synchronized
to the latest Substrate `master` and hence is compatible with this ink! release candidate.

Also, the metadata changed with this release, so you need to have a recent version of
the user interface you are using. For `polkadot-js/api` and `polkadot-js/apps` the metadata
changes are supported since Jan 19, 2022.

### Metadata Changes
There are two metadata changes in this release candidate:

#### For messages and constructors: `name: Array` ➔ `label: String`

The messages and constructors exposed by a contract were previously denoted with
```json
"name": ["foo", "bar"]
```
Now they are instead denoted with
```json
"label": "foo"
```
For public contract methods that are implemented from a trait, the trait name is prefixed to
the method name with `::` as the separator. So e.g. `trait_name::get_owner`.

The ink! PR which implemented this is [#923](https://github.com/use-ink/ink/pull/923),
for the `polkadot-js/api` it was [polkadot-js/api#4255](https://github.com/polkadot-js/api/pull/4255).

#### Constructors got a new field: `payable: bool`
Constructors now have to be specified `payable` if it's intended for them to receive value
(i.e. tokens). This is a breaking change, beforehand they were by default payable,
now they are by default non-payable.

The ink! PR which implemented this is [#1065](https://github.com/use-ink/ink/pull/1065), the PR that updated the ink!
metadata to V3 is [#1100](https://github.com/use-ink/ink/pull/1100), and for the `polkadot-js/api` it was [polkadot-js/api#4432](https://github.com/polkadot-js/api/pull/4432).

### Changed
- Update metadata to support payable constructors - [#1100](https://github.com/use-ink/ink/pull/1100).
- Make constructors non-payable by default, require specifying `payable` explicitly - [#1065](https://github.com/use-ink/ink/pull/1065).
- Renamed the error code `EcdsaRecoverFailed` to `EcdsaRecoveryFailed` ‒ [#1064](https://github.com/use-ink/ink/pull/1064).
- Renamed the `ink_env` function `transferred_balance()` to `transferred_value()` ‒ [#1063](https://github.com/use-ink/ink/pull/1063).
- Removed the error codes `BelowSubsistenceThreshold` and `NewContractNotFunded` ‒ [#1062](https://github.com/use-ink/ink/pull/1062).
- Updated ink! to use the most recent `contracts` pallet API ‒ [#1053](https://github.com/use-ink/ink/pull/1053).
- Explicitly link against `rlibc` to get non-buggy version of `memcpy` ‒ [#1049](https://github.com/use-ink/ink/pull/1049).
- Changed the metadata field `name` to `label` for messages and constructors ‒ [#923](https://github.com/use-ink/ink/pull/923) (thanks [@xgreenx](https://github.com/xgreenx)).

### Added
- Added a `remove` method to the `Mapping` data structure ‒ [#1023](https://github.com/use-ink/ink/pull/1023).

## Version 3.0-rc7

This is the 7th release candidate for ink! 3.0.

Since our last release candidate we implemented a number of contract size improvements.
With those improvements the size of our `erc20` example has reduced significantly:

|         |             | Release Build with `cargo-contract` |
| :------ | :---------- | :---------------------------------- |
| `erc20` | `3.0.0-rc6` | 29.3 K                              |
| `erc20` | `3.0.0-rc7` | 10.4 K                              |

The savings apply partly to our other examples; for `erc20` they are most
significant since it has been migrated to use a new [`Mapping`](https://use-ink.github.io/ink/ink_storage/lazy/struct.Mapping.html)
data structure, which we introduce with this release candidate.
The other examples will be migrated to this new data structure as a next step.

### Compatibility

You need to upgrade to a new version of the `contracts` pallet in order to use
this ink! release. This is because we upgraded to a newer `seal_call` API.
Specifically you need to upgrade to at least the pallet version
[e9fca0f](https://github.com/paritytech/substrate/commit/e9fca0f4d7a65998206f29baf0133a7931452104)
(or newer than Nov 24).

### Removed
- Removed the state rent API ‒ [#1036](https://github.com/use-ink/ink/pull/1036).

### Added
- Added support for wildcard selectors ‒ [#1020](https://github.com/use-ink/ink/pull/1020).
    - This enables writing upgradeable smart contracts using the proxy/forward pattern.
    - Annotating a wildcard selector in traits is not supported.
- The ink! codegen now heavily relies on static type information based on traits defined in `ink_lang` ‒ [#665](https://github.com/use-ink/ink/pull/665).
    - Some of those traits and their carried information can be used for static reflection of ink!
      smart contracts. Those types and traits reside in the new `ink_lang::reflect` module and is
      publicly usable by ink! smart contract authors.

### Changed
- Upgraded to the `seal_call` v1 API ‒ [#960](https://github.com/use-ink/ink/pull/960).
  - This API now enables control over the behavior of cross-contract calls, e.g. to forward/clone input,
    enable tail calls and control reentrancy.
    The crate documentation contains more details on the [`CallFlags`](https://use-ink.github.io/ink/ink_env/struct.CallFlags.html).
  - **Note:** The default behavior of cross-contract calls now disallows reentering the calling contract.
- ink! contract definitions via `#[ink::contract]` ‒ [#665](https://github.com/use-ink/ink/pull/665).<br/>
  For ink! smart contracts we now generate two contract types. Given `MyContract`:
    - `MyContract` will still be the storage struct.
      However, it can now additionally be used as static dependency in other smart contracts.
      Static dependencies can be envisioned as being directly embedded into a smart contract.
    - `MyContractRef` is pretty much the same of what we had gotten with the old `ink-as-dependency`.
      It is a typed thin-wrapper around an `AccountId` that is mirroring the ink! smart contract's API
      and implemented traits.
- ink! trait definitions via `#[ink::trait_definition]` ‒ [#665](https://github.com/use-ink/ink/pull/665).
    - ink! trait definitions no longer can define trait constructors.
    - ink! trait implementations now inherit `selector` and `payable` properties for trait messages.
        - Now explicitly setting `selector` or `payable` property for an implemented ink! trait method
          will only act as a guard that the set property is in fact the same as defined by the ink!
          trait definition.
- Improved some ink! specific compile errors ‒ [#665](https://github.com/use-ink/ink/pull/665).
    - For example, when using ink! messages and constructors which have inputs (or
      outputs) that cannot be encoded (or decoded) using the SCALE codec.
- Simplified selector computation for ink! trait methods ‒ [#665](https://github.com/use-ink/ink/pull/665).
    - Now selectors are encoded as `blake2b({namespace}::{trait_identifier}::{message_identifier})[0..4]`.
      If no `namespace` is set for the ink! trait definition then the formula is
      `blake2b({trait_identifier}::{message_identifier})[0..4]`.
      Where `trait_identifier` and `message_identifier` both refer to the identifiers of the ink! trait
      definition and ink! trait message respectively.
- We switched to Rust edition 2021 ‒ [#977](https://github.com/use-ink/ink/pull/977).
- Update chain extension example to show argument passing ‒ [#1029](https://github.com/use-ink/ink/pull/1029).

### Fixed
- Contracts now revert the transaction if an ink! message returns `Result::Err` ‒ [#975](https://github.com/use-ink/ink/pull/975), [#998](https://github.com/use-ink/ink/pull/998).
    - It is still possible to match against a `Result` return type for a called dependency contract
      ‒ i.e. a sub-contract specified in the contract's `Cargo.toml`.
- We implemented a number of Wasm contract size improvements:
    - Simple Mapping Storage Primitive ‒ [#946](https://github.com/use-ink/ink/pull/946).
    - Remove `always` from `inline` to allow compiler decide that to do ‒ [#1012](https://github.com/use-ink/ink/pull/1012) (thanks [@xgreenx](https://github.com/xgreenx)).
    - Add a way to allocate a storage facility using spread (and packed) layouts ‒ [#978](https://github.com/use-ink/ink/pull/978).
    - Extract non-generic part of `push_topic` to reduce code size ‒ [#1026](https://github.com/use-ink/ink/pull/1026).

## Version 3.0-rc6

This is the 6th release candidate for ink! 3.0.

### Compatibility

#### Please upgrade `cargo-contract`

You need to update to the latest `cargo-contract` in order to use this release:
```
cargo install cargo-contract --vers ^0.15 --force --locked
```

If you build contracts from this release candidate with an older `cargo-contract`,
the UI's won't display all contract-relevant fields.

#### Please upgrade `scale-info` in your contract's dependencies

In this release candidate we upgraded `scale-info`. You have to use a compatible
version in your contract's `Cargo.toml` as well; `cargo-contract` will throw
an error otherwise.

The `Cargo.toml` should contain
```
scale-info = { version = "1.0", default-features = false, features = ["derive"], optional = true }
scale = { package = "parity-scale-codec", version = "2", default-features = false, features = ["derive", "full"] }
```

#### New metadata format

There are breaking changes to the metadata format in this release.

- Removes top level `metadataVersion` field from the contract metadata (https://github.com/use-ink/cargo-contract/pull/342/files).
- Introduces new top level versioned metadata [enum](https://github.com/use-ink/ink/blob/master/crates/metadata/src/lib.rs#L68).
- Upgrades to `scale-info` version `1.0` (https://github.com/use-ink/ink/pull/845).
  - The previous supported version was `0.6`, so check release notes for all changes since then: https://github.com/use-ink/ink/pull/845
  - One of the main changes to be aware of is the change to 0 based type lookup ids: https://github.com/paritytech/scale-info/pull/90

### Added
- Added an Ethereum-compatibility function to recover a public key from an ECDSA signature and message hash - [#914](https://github.com/use-ink/ink/pull/914) (thanks [@xgreenx](https://github.com/xgreenx)).
- Added new utility proc. macros to `ink_lang` crate - [#947](https://github.com/use-ink/ink/pull/947):
    - `blake2!`: Compute the BLAKE2b 256-bit hash of the given input literal string.
    - `selector_bytes!`: Compute the ink! selector of the given input literal string and return it as `[u8; 4]`.
    - `selector_id!`: Compute the ink! selector of the given input literal string and return it as `u32`.

### Changed
- Update to `scale-info` 1.0 - [#845](https://github.com/use-ink/ink/pull/845).
- Message and constructor selectors no longer take their inputs as string, but as `u32` decodable integer - [#928](https://github.com/use-ink/ink/pull/928).<br/>
  For example:
    - It is no longer possible to specify a selector as `#[ink(selector = "0xC0DECAFE")]`.
    - The newly allowed formats are `#[ink(selector = 0xC0DECAFE)]` and `#[ink(selector = 42)]`.
    - Smart contract authors are required to update their smart contracts for this change.
- Improved the `multisig` example - [#962](https://github.com/use-ink/ink/pull/962).
- Changed the link to our beginner's workshop to the migrated workshop on `substrate.io` - [#957](https://github.com/use-ink/ink/pull/957).

### Fixed
- Fixed a mistake in the `ink_env::block_timestamp()` documentation - [#937](https://github.com/use-ink/ink/pull/937).

## Version 3.0-rc5 (2021-09-08)

This is the 5th release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

### Compatibility

Make sure to use a recent Rust nightly and `cargo-contract` with the current release:
```
cargo install cargo-contract --vers ^0.14 --force --locked && rustup update
```

In the past we recommended using our [`canvas-node`](https://github.com/paritytech/canvas)
for local contract development and testing. We've now migrated this node to be run
as a Parachain. This new setup comes with some additional overhead though (such as requiring
a local Polkadot installation); for local development this is often unnecessary.

We've therefore created a new project, the
[`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node).
It fulfills the same purpose the `canvas-node` did before ‒ it's a standalone node which is
just Substrate's [`node-template`](https://github.com/paritytech/substrate/tree/master/bin/node-template)
modified to include [the `contracts` pallet](https://github.com/paritytech/substrate/tree/master/frame/contracts).
You can install the newest version like this:
```
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --force
```
After you've installed the node it can be run via `substrate-contracts-node --tmp --dev`.

### Added
- Added example for mocking chain extensions in off-chain tests ‒ [#882](https://github.com/use-ink/ink/pull/882).
- Panic messages are now printed to debug buffer ‒ [#894](https://github.com/use-ink/ink/pull/894).

### Changed
- Unlicensed smart contract examples ‒ [#888](https://github.com/use-ink/ink/pull/888).
- Stabilized `seal_debug_message` ‒ [#902](https://github.com/use-ink/ink/pull/902).

## Version 3.0-rc4 (2021-07-19)

This is the 4th release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

### Compatibility

ink! 3.0-rc4 is compatible with

- The "ink! CLI" [`cargo-contract`](https://github.com/use-ink/cargo-contract)
  version `0.13.0` or newer.
    - Install the newest version using `cargo install --force cargo-contract`.
- Substrate version `4.0.0-dev` including the `contracts-pallet` version `4.0.0-dev`.
- [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) version `0.1.0` or newer.
    - Install the newest version using `cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --force`.

The documentation on our [Documentation Portal](https://use.ink)
is up-to-date with this release candidate. Since the last release candidate we notably
added a number of [Frequently Asked Questions](https://use.ink/faq)
there.

### Quality Assurance

In order to ensure a continuously high quality of our codebase we implemented a number
of key improvements to our testing setup:

- We've put an emphasis on automated testing of the usage examples in our crate documentation.
  Those are now tested in the context of a complete ink! contract. In the past this was not
  always the case, sometimes usage examples were just isolated code snippets.
- We started our [`ink-waterfall`](https://github.com/use-ink/ink-waterfall) project,
  which runs End-to-End tests through our entire stack.
  All our examples are continuously built using the latest `cargo-contract`. They are
  subsequently deployed on the latest `substrate-contracts-node` by emulating browser interactions with
  both the [`canvas-ui`](https://use-ink.github.io/canvas-ui/#/) and the
  [`polkadot-js`](https://polkadot.js.org/apps/#/) UI.
  This testing setup enables us to detect bugs which only appear in the context of using
  multiple components together early on.
- To improve the readability of our documentation we introduced automated grammar and spell
  checking into our Continuous Integration environment.

### Added
- Added support for the new `seal_random` API ‒ [#734](https://github.com/use-ink/ink/pull/734).
- Added missing documentation for the `ink_storage_derive` procedural macros ‒ [#711](https://github.com/use-ink/ink/pull/711).
- Implemented the (unstable) `seal_rent_params` API ‒ [#755](https://github.com/use-ink/ink/pull/755).
- Implemented the (unstable) `seal_rent_status` API ‒ [#798](https://github.com/use-ink/ink/pull/798).
- Implemented the (unstable) `seal_debug_message` API ‒ [#792](https://github.com/use-ink/ink/pull/792).
    - Printing debug messages can now be achieved via `ink_env::debug_println!(…)`.
    - See [our documentation](https://use.ink/faq#how-do-i-print-something-to-the-console-from-the-runtime)
      for more information.
    - The examples have been updated to reflect this new way of printing debug messages.
- Added usage comments with code examples to the `ink_env` API ‒ [#797](https://github.com/use-ink/ink/pull/797).
    - The [published crate documentation](https://use-ink.github.io/ink/ink_lang/struct.EnvAccess.html) now contains
      much more code examples for the methods behind `self.env()` and `Self::env()`.
- Added an example implementation for ERC-1155, a multi-token standard ‒ [#800](https://github.com/use-ink/ink/pull/800).
- Implemented binary search for `collections::Vec` ‒ [#836](https://github.com/use-ink/ink/pull/836).
- Added the ability of submitting payable transactions to the `multisig` example ‒ [#820](https://github.com/use-ink/ink/pull/820).
- Implemented `Decode` for `Error` types in the examples, enabling building them as dependencies ‒ [#761](https://github.com/use-ink/ink/pull/761).
- We started working on a new off-chain environment testing engine ‒ [#712](https://github.com/use-ink/ink/pull/712).
    - The old testing environment has a number of limitations, which we are well aware of.
      We're confident that with the new testing engine we will be able to conduct much more
      elaborate testing in an emulated chain environment.
    - For the moment, the new engine is unstable and only available behind a feature flag.
      A number of examples have already been converted to support the new testing engine.

### Changed
- To reduce a contract's space footprint we switched the default allocator to a bump allocator implementation ‒ [#831](https://github.com/use-ink/ink/pull/831).
- A couple of readme's have been reworked:
    - Our main ink! readme ‒ [#774](https://github.com/use-ink/ink/pull/774).
    - The `rand-extension` example readme ‒ [#793](https://github.com/use-ink/ink/pull/793).
    - The `delegator` example readme ‒ [#766](https://github.com/use-ink/ink/pull/766).
- With the stabilization of Rust 1.51 we ware able to remove the `ink-unstable` feature, making
  `collections::SmallVec` and `lazy::LazyArray` available by default ‒ [#746](https://github.com/use-ink/ink/pull/746).
- To resolve confusion, we migrated all usages of `#[test]` in our examples to `#[ink::test]` ‒ [#746](https://github.com/use-ink/ink/pull/746).
    - The difference is that `#[ink::test]` spawns an emulated chain environment (an "off-chain" environment)
      and hence comes with a bit of overhead. It was not always clear to users when they require
      an off-chain environment, we decided to mitigate this confusion by using an emulated chain
      environment for all our example tests.
- With the stabilization of Rust's `min_const_generics` we were able to replace the fixed
  size implementations of `SpreadLayout` and `PackedLayout` for Arrays. These traits are
  now implemented for all Arrays of size `usize` ‒ [#754](https://github.com/use-ink/ink/pull/754).
- We were able to remove the pinned `funty` dependency ‒ [#711](https://github.com/use-ink/ink/pull/711).
- The `contract-transfer` example has been improved for better UI support ‒ [#789](https://github.com/use-ink/ink/pull/789).
- The `contract-transfer` example has been improved for better error handling ‒ [#790](https://github.com/use-ink/ink/pull/790).

### Fixed
- Catch illegal `struct` destructuring pattern in ink! message arguments ‒ [#846](https://github.com/use-ink/ink/pull/846).
- Removed an erroneous `Salt` type in code generation for cross-contract calls ‒ [#842](https://github.com/use-ink/ink/pull/842).
- Do not generate metadata if compiled as dependency ‒ [#811](https://github.com/use-ink/ink/pull/811).
- Fix execution context parameters in DNS example tests ‒ [#723](https://github.com/use-ink/ink/pull/723).
- Fixed the `Greeter` contract example from our doc comments ‒ [#773](https://github.com/use-ink/ink/pull/773).

## Version 3.0-rc3 (2021-03-02)

This is the 3rd release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

### Compatibility

ink! 3.0-rc3 is compatible with

- The `cargo-contract` CLI tool version `0.9.1` or newer.
    - Install newest version using `cargo install --force cargo-contract`.
- Substrate version `3.0` including the `contracts-pallet` version `3.0`.

### Added

- Implemented chain extensions feature for ink!.
- ink!'s official documentation portal: https://use.ink/
- It is now possible to pass a `salt` argument to contract instantiations.
- Implemented fuzz testing for the ink! codebase.

### Changed

- Migrate `ink_storage::SmallVec` and `ink_storage::lazy::SmallLazyArray` to use `min_const_generics`.
    - The `min_const_generics` feature is going to be stabilized in Rust 1.51. For now it was put behind
      the `ink-unstable` crate feature of the `ink_storage` crate.
- Improve error reporting for conflicting ink! attributes.
- Improve error reporting for invalid constructor or message selector. (https://github.com/use-ink/ink/pull/561)
- Remove `iter_mut` for `ink_storage::BinaryHeap` data structure.
- Add documented demonstration how to properly mock `transferred_balance` calls: https://github.com/use-ink/ink/pull/555
- Add contract example which uses `ext_transfer` and `ext_terminate`: https://github.com/use-ink/ink/pull/554
- Improve documentation of `transfer` and `minimum_balance` APIs: https://github.com/use-ink/ink/pull/540

### Fixed

- The Delegator example contract now compiles properly using the `build-all.sh` bash script.
- Update crate dependencies:
    - `scale-info 0.6`
    - `parity-scale-codec 2.0`
    - `rand 0.8`
    - `itertools 0.10`
- Remove unused `tiny-keccak` dependency from `ink_primitives`.
- Changed the default `BlockNumber` type to `u32`. This is a fix since it now properly mirrors Substrate's default `BlockNumber` type.
- Ensure topics are unique: https://github.com/use-ink/ink/pull/594
- Several fixes for `ink_storage` data structures, including:
    - `Drop` implementation for `Pack` now works properly. (https://github.com/use-ink/ink/pull/600)
    - `Drop` implementation for `Lazy` now always properly clean up storage. (https://github.com/use-ink/ink/pull/597)
    - Nested `Lazy` now properly clears storage data. (https://github.com/use-ink/ink/pull/583)
    - `Option` fields now properly clean up nested storage data. (https://github.com/use-ink/ink/pull/570)

## Version 3.0-rc2 (2020-10-22)

This is the 2nd release candidate for ink! 3.0.

On top of the changes introduced in the first release candidate for ink! 3.0 we introduced
the following improvements, new features and bug fixes:

- The `ink_storage` crate now comes with a new `BinaryHeap` data structure
  that has a very similar interface to the well known Rust standard library
  `BinaryHeap`. It features specific optimizations to reduce the storage reads
  and writes required for its operations.
- Fixed a bug with `ink_storage::Lazy` that corrupted the storage of
  other storage data structures if it was unused in a contract execution.
- The `ink_storage::alloc::Box` type now implements `scale_info::TypeInfo` which
  now allows it to be fully used inside other storage data structures such as
  `ink_storage::collections::Vec`. The missing of this implementation was
  considered a bug.
- The `LazyHashMap` low-level storage abstraction is now re-exported from within
  the `ink_storage::lazy` module and docs are inlined.
- Added note about the `ink_core` split into `ink_env` and `ink_storage` crates
  to the release notes of ink! 3.0-rc1.
- The `Cargo.toml` documentation now properly links to the one deployed at docs.rs.
  On top of that crate level documentation for the `ink_allocator` crate has been
  added.
- Add new ERC-20 example contract based on a trait implementation. Also modernized
  the old non-trait based ERC-20 example token contract.

## Version 3.0-rc1 (2020-10-09)

Be prepared for the ink! 3.0 release notes because the whole version was basically a rewrite of
all the major components that make up ink!. With our experience gained from previous releases
of ink! we were able to detect weak spots of the design and provided ink! with more tools,
more features and more efficiency as ever. Read more below …

### Just. Be. Rust. 3.0

In the 3.0 update we further explored the space for ink! to just feel like it was plain Rust.
With this in mind we changed the syntax slightly in order to better map from ink! to the generated
Rust code. So what users see is mostly what will be generated by ink! later.

In this vein `#[ink(storage)]` and `#[ink(event)]` structs as well as `#[ink(message)]` and
`#[ink(constructor)]` methods now need to be specified with public visibility (`pub`).

The `#[ink(constructors)]` syntax also changes and no longer uses a `&mut self` receiver but
now follows the natural Rust constructors scheme. So it is no longer possible to shoot
yourself in the foot by accidentally forgetting to initialize some important data structures.

**Old ink! 2.0:**
```rust
#[ink(constructor)]
fn new_erc20(&mut self, initial_supply: Balance) {
    let caller = self.env().caller();
    self.total_supply.set(initial_supply);
    self.balances.insert(caller, initial_supply);
}
```
**New ink! 3.0:**
```rust
#[ink(constructor)]
pub fn new_erc20(initial_supply: Balance) -> Self {
    let caller = self.env().caller();
    let mut balances = ink_storage::HashMap::new();
    balances.insert(caller, initial_supply);
    Self {
        total_supply: initial_supply,
        balances,
    }
}
```

Also ink! 3.0 no longer requires a mandatory `version` field in the header of the ink! module attribute.

Syntactically this is all it takes to port your current ink! smart contracts over to ink! 3.0 syntax.

### Split of ink_core

The `ink_core` crate no longer exists. It has been split into the new `ink_env` and `ink_storage` crates.

Everything that was previously accessed through `ink_core::env` now lives in `ink_env` and everything
that was previously accessed through `ink_core::storage` now lives in `ink_storage`. Both crates keep
the responsibilities of their former originating `ink_core` modules.

### New Storage Module

The storage module has been reworked entirely.
Also it no longer lives in the `ink_core` crate but instead is defined as its own `ink_storage` crate.

In a sense it acts as the standard storage library for ink! smart contracts in that it provides all the
necessary tools and data structures to organize and operate the contract's storage intuitively and efficiently.

#### Lazy

The most fundamental change in how you should think about data structures provided by the new `ink_storage`
crate is that they are inherently lazy. We will explain what this means below!
The `ink_storage` crate provides high-level and low-level lazy data structures.
The difference between high-level and low-level lies in the distinction in how these data structures are aware
of the elements that they operate on. For high-level data structures they are fully aware about the elements
they contains, do all the clean-up by themselves so the user can concentrate on the business logic.
For low-level data structures the responsibility about the elements lies in the hands of the contract author.
Also they operate on cells (`Option<T>`) instead of entities of type `T`.
But what does that mean exactly?

The new `ink_storage::Lazy` type is what corresponds the most to the old `ink_core::storage::Value` type. Both cache their entities and both act lazily on the storage. This means that a read or write operation is only performed when it really needs to in order to satisfy other inputs.
Data types such as Rust primitives `i32` or Rust's very own `Vec` or data structures can also be used to operate on the contract's storage, however, they will load their contents eagerly which is often not what you want.

An example follows with the below contract storage and a message that operates on either of the two fields.
```rust
#[ink(storage)]
pub struct TwoValues {
    offset: i32,
    a: i32,
    b: i32,
}

impl TwoValues {
    #[ink(message)]
    pub fn set(&mut self, which: bool, new_value: i32) {
        match which {
            true  => { self.a = self.offset + new_value; },
            false => { self.b = self.offset + new_value; },
        }
    }
}
```

Whenever we call `TwoValues::set` always both `a` and `b` are loaded despite the fact the we only operate on one of them at a time. This is very costly since storage accesses are in fact database look-ups.
In order to prevent this eager loading of storage contents we can make use of `ink_storage::Lazy` or other lazy data structures defined in that crate:
```rust
#[ink(storage)]
pub struct TwoValues {
    offset: i32,
    a: ink_storage::Lazy<i32>,
    b: ink_storage::Lazy<i32>,
}

impl TwoValues {
    #[ink(message)]
    pub fn set(&mut self, which: bool, new_value: i32) {
        match which {
            true  => { self.a = offset + new_value; },
            false => { self.b = offset + new_value; },
        }
    }
}
```
Now `a` and `b` are only loaded when the contract really needs their values.
Note that `offset` remained `i32` since it is always needed and could spare the minor overhead of the `ink_storage::Lazy` wrapper.

#### HashMap

In the follow we explore the differences between the high-level `ink_storage::collections::HashMap`
and the low-level `ink_storage::lazy::LazyHashMap`. Both provide very similar functionality in that they map some generic key to some storage entity.

However, their APIs look very different. Whereas the `HashMap` provides a rich and high-level API that is comparable to that of Rust's very own `HashMap`, the `LazyHashMap` provides only a fraction of the API and also operates on `Option<T>` values types instead of `T` directly. It is more similar Solidity mappings than to Rust's `HashMap`.

The fundamental difference of both data structures is that `HashMap` is aware of the keys that have been stored in it and thus can reconstruct exactly which elements and storage regions apply to it. This enables it to provide iteration and automated deletion as well as efficient way to defragment its underlying storage to free some storage space again. This goes very well in the vein of Substrate's storage rent model where contracts have to pay for the storage they are using.

| Data Structure      | level of abstraction | caching | lazy  |        element type        |          container           |
| :------------------ | :------------------: | :-----: | :---: | :------------------------: | :--------------------------: |
| `T`                 |          -           |   yes   |  no   |            `T`             |       primitive value        |
| `Lazy<T>`           |      high-level      |   yes   |  yes  |            `T`             |   single element container   |
| `LazyCell<T>`       |      low-level       |   yes   |  yes  |        `Option<T>`         | single element, no container |
| `Vec<T>`            |      high-level      |   yes   |  yes  |            `T`             |  Rust vector-like container  |
| `LazyIndexMap<T>`   |      low-level       |   yes   |  yes  |        `Option<T>`         | similar to Solidity mapping  |
| `HashMap<K, V>`     |      high-level      |   yes   |  yes  |     `V` (key type `K`)     |   Rust map-like container    |
| `LazyHashMap<K, V>` |      low-level       |   yes   |  yes  | `Option<V>` (key type `K`) | similar to Solidity mapping  |

There are many more! For more information about the specifics please take a look into [the `ink_storage` crate documentation](https://use-ink.github.io/ink/ink_storage/).

#### Spread & Packed Modes

Storing or loading complex data structures to and from contract storage can be done in many different ways. You could store all information into a single storage cell or you could try to store all information into as many different cells as possible. Both strategies have pros and cons under different conditions.

For example it might be a very good idea to store all the information under the same cell if all the information is very compact. For example when we are dealing with a byte vector that is expected to never be larger than approx a thousand elements it would probably be more efficient if we store all those thousand bytes in the same cell and especially if we often access many of those (or all) in our contract messages.

On the other hand spreading information across as many cells as possible might be much more efficient if we are dealing with big data structures, a lot of information that is not compact, or when messages that operate on the data always only need a small fraction of the whole data.
An example for this use case is if you have a vector of user accounts where each account stores potentially a lot of information, e.g. a 32-byte hash etc and where our messages only every operate on only a few of those at a time.

The `ink_storage` crate provides the user full control over the strategy or a mix of these two root strategies through some fundamental abstractions that we are briefly presenting to you.

#### Default: Spreading Mode

By default ink! spreads information to as many cells as possible. For example if you have the following `#[ink(storage)]` struct every field will live in its own single storage cell. Note that for `c` all 32 bytes will share the same cell!

```rust
#[ink(storage)]
pub struct Spreaded {
    a: i32,
    b: ink_storage::Lazy<i32>,
    c: [u8; 32],
}
```

#### Packing Storage

We can alter this behavior by using the `ink_storage::Pack` abstraction:

```rust
pub struct Spreaded {
    a: i32,
    b: ink_storage::Lazy<i32>,
    c: [u8; 32],
}

#[ink(storage)]
pub struct Packed {
    packed: ink_storage::Pack<Spreaded>,
}
```

Now all fields of `Spreaded` will share the same storage cell. This means whenever one of them is stored to or loaded from the contract storage, all of them are stored or loaded. A user has to choose wisely what mode of operation is more suitable for their contract.

These abstractions can be combined in various ways, yielding full control to the users. For example, in the following only `a` and `b` share a common storage cell while `c` lives in its own:

```rust
pub struct Spreaded {
    a: i32,
    b: ink_storage::Lazy<i32>,
}

#[ink(storage)]
pub struct Packed {
    packed: ink_storage::Pack<Spreaded>,
    c: [u8; 32],
}
```

#### Spreading Array Cells

If we prefer to store all bytes of `c` into their own storage cell we can make use of the `SmallVec` data structure. The `SmallVec` is a high-level data structure that allows to efficiently organize a fixed number of elements similar to a Rust array. However, unlike a Rust array it acts lazily upon the storage and spreads its elements into different cells.

```rust
use typenum::U32;

pub struct Spreaded {
    a: i32,
    b: ink_storage::Lazy<i32>,
}

#[ink(storage)]
pub struct Packed {
    packed: ink_storage::Pack<Spreaded>,
    c: SmallVec<u8, U32>,
}
```

#### Opting-out of Storage

If you are in need of storing some temporary information across method and message boundaries ink! will have your back with the `ink_storage::Memory` abstraction. It allows you to simply opt-out of using the storage for the wrapped entity at all and thus is very similar to Solidity's very own `memory` annotation.

An example below:

```rust
#[ink(storage)]
pub struct OptedOut {
    a: i32,
    b: ink_storage::Lazy<i32>,
    c: ink_storage::Memory<i32>,
}
```

The the above example `a` and `b` are normal storage entities, however, `c` on the other hand side will never load from or store to contract storage and will always be reset to the default value of its `i32` type for every contract call.
It can be accessed from all ink! messages or methods via `self.c` but will never manipulate the contract storage and thus acts wonderfully as some shared local information.

#### Dynamic Storage Allocator

In the previous section we have seen how the default mode of operation is to spread information and how we can opt-in to packing information into single cells via `ink_storage::Packed`.

However, what if we wanted to store a vector of a vector of `i32` for example?
Naturally a user would try to construct this as follows:

```rust
use ink_storage::Vec as StorageVec;

#[ink(storage)]
pub struct Matrix {
    values: StorageVec<StorageVec<i32>>,
}
```

However, this will fail compilation with an error indicating that `StorageVec<T>` requires for its `T` to be packed (`T: PackedLayout`) which `StorageVec<T>` itself does not since it always stores all of its elements into different cells. The same applies to many other storage data structures provided by `ink_storage` and is a trade-off the ink! team decided for the case of efficiency of the overall system.
Instead what a user can do in order to get their vector-of-vector to be working is to make use of ink!'s dynamic storage allocator capabilities.

For this the contract author has to first enable the feature via:

```rust
use ink_lang as ink;

#[ink::contract(dynamic_storage_allocator = true)]
mod matrix {
    // contract code ...
}
```

And then we can define our `Matrix` `#[ink(storage)]` as follows:

```rust
use ink_storage::{
    Vec as StorageVec,
    Box as StorageBox,
};

#[ink(storage)]
pub struct Matrix {
    values: StorageVec<StorageBox<StorageVec<i32>>>,
}
```

With `ink_storage::Box<T>` we can use a `T: SpreadLayout` as if it was `T: PackedLayout` since the `ink_storage::Box<T>` itself suffices the requirements and can be put into a single contract storage cell. The whole concept works quite similar to how Rust's `Box` works: by an indirection - contract authors are therefore advised to make use of dynamic storage allocator capabilities only if other ways of dealing with ones problems are not applicable.

#### Custom Data Structures

While the `ink_storage` crate provides tons of useful utilities and data structures to organize and manipulate the contract's storage contract authors are not limited by its capabilities. By implementing the core `SpreadLayout` and `PackedLayout` traits users are able to define their very own custom storage data structures with their own set of requirement and features that work along the `ink_storage` data structures as long as they fulfill the mere requirements stated by those two traits.

In the future we plan on providing some more ink! workshops and tutorials guiding the approach to design and implement a custom storage data structure.

#### In Summary

The new `ink_storage` crate provides everything you need to operate on your contract's storage.
There are low-level and high-level data structures depending on your need of control.
All provided data structures operate lazily on the contract's storage and cache their reads and writes for a more gas efficient storage access.
Users should prefer high-level data structures found in the `collections` module over the low-level data structures found in the `lazy` module.
For a list of all the new storage data structure visit [`ink_storage`'s documentation](https://use-ink.github.io/ink/ink_storage/).

### ink! Attributes

For ink! 3.0 we have added some more useful ink! specific attributes to the table.
All of these ink! attributes are available to specify inside an ink! module.
An ink! module is the module that is flagged by `#[ink::contract]` containing all the ink! definitions:
```rust
use ink_lang as ink;

#[ink::contract]
mod erc20 {
    #[ink(storage)]
    pub struct Erc20 { ... }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self { .. }

        #[ink(constructor)]
        pub fn total_supply(&self) -> Balance { .. }

        // etc. ...
    }
}
```

We won't be going into the details for any of those but will briefly present the entire set of ink! specific attributes below:

| Attribute                          | Where Applicable                                   | Description                                                                                                                                                                                                                                                     |
| :--------------------------------- | :------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `#[ink(storage)]`                  | On `struct` definitions.                           | Defines the ink! storage struct. There can only be one ink! storage definition per contract.                                                                                                                                                                    |
| `#[ink(event)]`                    | On `struct` definitions.                           | Defines an ink! event. A contract can define multiple such ink! events.                                                                                                                                                                                         |
| `#[ink(anonymous)]` **new**        | Applicable to ink! events.                         | Tells the ink! codegen to treat the ink! event as anonymous which omits the event signature as topic upon emitting. Very similar to anonymous events in Solidity.                                                                                               |
| `#[ink(topic)]`                    | Applicate on ink! event field.                     | Tells the ink! codegen to provide a topic hash for the given field. Every ink! event can only have a limited number of such topic field. Similar semantics as to indexed event arguments in Solidity.                                                           |
| `#[ink(message)]`                  | Applicable to methods.                             | Flags a method for the ink! storage struct as message making it available to the API for calling the contract.                                                                                                                                                  |
| `#[ink(constructor)]`              | Applicable to method.                              | Flags a method for the ink! storage struct as constructor making it available to the API for instantiating the contract.                                                                                                                                        |
| `#[ink(payable)]` **new**          | Applicable to ink! messages.                       | Allows receiving value as part of the call of the ink! message. ink! constructors are implicitly payable.                                                                                                                                                       |
| `#[ink(selector = "..")]` **new**  | Applicable to ink! messages and ink! constructors. | Specifies a concrete dispatch selector for the flagged entity. This allows a contract author to precisely control the selectors of their APIs making it possible to rename their API without breakage.                                                          |
| `#[ink(namespace = "..")]` **new** | Applicable to ink! trait implementation blocks.    | Changes the resulting selectors of all the ink! messages and ink! constructors within the trait implementation. Allows to disambiguate between trait implementations with overlapping message or constructor names. Use only with great care and consideration! |
| `#[ink(impl)]` **new**             | Applicable to ink! implementation blocks.          | Tells the ink! codegen that some implementation block shall be granted access to ink! internals even without it containing any ink! messages or ink! constructors.                                                                                              |

### Merging of ink! Attributes

It is possible to merge attributes that share a common flagged entity.
The example below demonstrates this for a payable message with a custom selector.

```rust
#[ink(message)]
#[ink(payable)]
#[ink(selector = "0xCAFEBABE")]
pub fn transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), Error> {
    // actual implementation
}
```
We can also write the above ink! message definition in the following way:
```rust
#[ink(message, payable, selector = "0xCAFEBABE")]
pub fn transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), Error> {
    // actual implementation
}
```

### Trait Support

One of the most anticipated features of ink! 3.0 is its Rust trait support.
Through the new `#[ink::trait_definition]` procedural macro it is now possible to define your very own trait definitions that are then implementable by ink! smart contracts.

This allows to define shared smart contract interfaces to different concrete implementations.
Note that this ink! trait definition can be defined anywhere, even in another crate!

#### Example

Defined in the `base_erc20.rs` module.

```rust
use ink_lang as ink;

#[ink::trait_definition]
pub trait BaseErc20 {
    /// Creates a new ERC-20 contract and initializes it with the initial supply for the instantiator.
    #[ink(constructor)]
    fn new(initial_supply: Balance) -> Self;

    /// Returns the total supply.
    #[ink(message)]
    fn total_supply(&self) -> Balance;

    /// Transfers `amount` from caller to `to`.
    #[ink(message, payable)]
    fn transfer(&mut self, to: AccountId, amount: Balance);
}
```

An ink! smart contract definition can then implement this trait definition as follows:

```rust
use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use base_erc20::BaseErc20;

    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        // more fields ...
    }

    impl BaseErc20 for Erc20 {
        #[ink(constructor)]
        fn new(initial_supply: Balance) -> Self {
            // implementation ...
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            // implementation ...
        }

        #[ink(message, payable)]
        fn transfer(&mut self, to: AccountId, amount: Balance) {
            // implementation ...
        }
    }
}
```

Calling the above `Erc20` explicitly through its trait implementation can be done just as if it was normal Rust code:

```rust
// --- Instantiating the ERC-20 contract:
//
let mut erc20 = <Erc20 as BaseErc20>::new(1000);
// --- Is just the same as:
use base_erc20::BaseErc20;
let mut erc20 = Erc20::new(1000);

// --- Retrieving the total supply:
//
assert_eq!(<Erc20 as BaseErc20>::total_supply(&erc20), 1000);
// --- Is just the same as:
use base_erc20::BaseErc20;
assert_eq!(erc20.total_supply(), 1000);
```

There are still many limitations to ink! trait definitions and trait implementations.
For example it is not possible to define associated constants or types or have default implemented methods.
These limitations exist because of technical intricacies, however, please expect that many of those will be tackled in future ink! releases.

## Version 2.1 (2020-03-25)

- Add built-in support for cryptographic hashes:
    - Blake2 with 128-bit and 256-bit
    - Sha2 with 256-bit
    - Keccak with 256-bit
- Add `ink_core::hash` module for high-level API to the new built-in hashes.
- Update `runtime-storage` example ink! smart contract to demonstrate the new built-in hashes.

## Version 2.0 (2019-12-03)

The ink! version 2.0 syntax has one major philosophy:

> Just. Be. Rust.

To accomplish this, we take advantage of all the standard Rust types and structures and use
[attribute macros](https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros) to
tag these standard structures to be different parts of the ink! language.

Anything that is not tagged with an `#[ink(...)]` attribute tag is just standard Rust, and can be
used in and out of your contract just like standard Rust could be used!

**Every valid ink! contract is required to have at least one `#[ink(constructor)]`, at least one
`#[ink(message)]` and exactly one `#[ink(storage)]` attribute.**

Follow the instructions below to understand how to migrate your ink! 1.0 contracts to this new ink!
2.0 syntax.

### Update the ink! CLI

Install the latest ink! CLI using the following command:

```bash
cargo install --git https://github.com/use-ink/cargo-contract cargo-contract --force
```

There is a new contract metadata format you need to use. You can generate the metadata using:

```bash
cargo contract generate-metadata
```

This will generate a file `metadata.json` you should upload when deploying or interacting with a
contract.

### Declaring a Contract

The fundamental change with the new ink! syntax is how we declare a new contract.

We used to wrap the whole ink! contract into a `contract!` macro. At that point, all syntax within
the macro could be custom, and in our first iteration of the language, we used that in ways that
made our code not really Rust anymore.

Now we wrap the whole contract in a standard Rust module, and include an attribute tag to identify
this object as part of the ink! language. This means that all of our code from this point forward
will be valid Rust!

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
contract! {
    ...
}
```

</td>
<td>

```rust
use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod erc20 {
    ...
}
```

</td>
</tr>
</table>

> Note: we now require a mandatory ink! version in the header. You're welcome.

See the [ERC20 example](https://github.com/use-ink/ink/blob/master/integration-tests/public/upgradeable-contracts/delegator/lib.rs).

### ink! Contract Tag

The ink! contract tag can be extended to provide other configuration information about your
contract.

#### Defining Custom Types

We used to define types using a special `#![env = DefaultSrmlTypes]` tag.

Now we simply include the type definition in the `#[ink::contract(...)]` tag:

```rust
#[ink::contract(version = "0.1.0", env = MyCustomTypes)]
```

By default, we use `DefaultSrmlTypes`, so you don't need to define anything unless you plan to use
custom types.

#### Dynamic Allocation

It is possible to enable the dynamic environment that allows for dynamic allocations by specifying
`dynamic_allocations = true` in the parameters of the ink! header. This is disabled by default.

```rust
#[ink::contract(version = "0.1.0", dynamic_allocations = true)]
```

> Note: The dynamic environment is still under research and not yet stable.

### Declaring Storage

We define storage items just the same as before, but now we need to add the `#[ink(storage)]`
attribute tag.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

</td>
<td>

```rust
#[ink(storage)]
struct Erc20 {
    total_supply: storage::Value<Balance>,
    balances: storage::HashMap<AccountId, Balance>,
    allowances: storage::HashMap<(AccountId, AccountId), Balance>,
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/use-ink/ink/blob/master/examples/erc20/src/lib.rs).

### Declaring Events

To update your events, you need to:

1. Change the old `event` keyword to a standard Rust `struct`.
2. Add the `#[ink(event)]` attribute tag to your `struct`.

If you were previously indexing the items in your event with `#[indexed]`:

3. Add the `#[ink(topic)]` attribute tag to each item in your event.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
event Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    #[indexed]
    value: Balance,
}
```

</td>
<td>

```rust
#[ink(event)]
struct Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    #[ink(topic)]
    value: Balance,
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/use-ink/ink/blob/master/examples/erc20/src/lib.rs).

### Environment Handler

`EnvHandler` is no longer exposed to the user and instead the environment is now always accessed via
`self.env()`.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

**Getting the caller:**

```rust
let caller = env.caller();
```

**Emitting an event:**

```rust
env.emit(...)
```

</td>
<td>

**Getting the caller:**

```rust
let caller = self.env().caller();
```

**Emitting an event:**

```rust
self.env().emit_event(...)
```

</td>
</tr>
</table>

> Note: The name of the function used to emit an event was updated to `emit_event`.

### Message Functions

We used to use `pub(external)` to tag functions that could be called by the outside world.

We now simply add the attribute `#[ink(message)]`.

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
pub(external) fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

</td>
<td>

```rust
#[ink(message)]
fn total_supply(&self) -> Balance {
    *self.total_supply
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/use-ink/ink/blob/master/examples/erc20/src/lib.rs).

### Defining a Constructor

We used to define our constructor by implementing the `Deploy` trait and defining the `deploy`
function.

But now our constructor function is in the same place as the rest of our contract functions, within
the general implementation of the storage struct.

We tag these functions with the `#[ink(constructor)]` attribute. We can create multiple different
constructors by simply creating more functions with the same tag. You can name a constructor
function whatever you want (except starting with `__ink` which is reserved for all functions).

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
impl Deploy for Erc20 {
    fn deploy(&mut self, init_supply: Balance) {
        let caller = env.caller();
        self.total_supply.set(init_value);
        self.balances.insert(caller, init_supply);
        env.emit(Transfer {
            from: None,
            to: Some(env.caller()),
            value: init_value
        });
    }
}
```

</td>
<td>

```rust
impl Erc20 {
    #[ink(constructor)]
    fn new(&mut self, initial_supply: Balance) {
        let caller = self.env().caller();
        self.total_supply.set(initial_supply);
        self.balances.insert(caller, initial_supply);
        self.env().emit_event(Transferred {
            from: None,
            to: Some(caller),
            amount: initial_supply,
        });
    }
}
```

</td>
</tr>
</table>

See the [ERC20 example](https://github.com/use-ink/ink/blob/master/examples/erc20/src/lib.rs).

### Cross Contract Calls

It is now possible to call ink! messages and ink! constructors. So ink! constructors allow
delegation and ink! messages can easily call other ink! messages.

Given another ink! contract like `mod Adder { ... }`, we can call any of its functions:

```rust
use adder::Adder;
//--snip--
#[ink(storage)]
struct Delegator {
    adder: storage::Value<Adder>,
}
//--snip--
let result = self.adder.inc(by);
```

See the [delegator example](https://github.com/use-ink/ink/blob/master/examples/delegator/lib.rs).

### Factory Contracts

Creation of other contracts from a factory contract works pretty much the same way it did in the old
ink! language.

However, users are now required to specify the `code_hash` separately rather than in the
constructor:

```rust
.using_code(code_hash)
```

Also, they need to specify the used ink! environment (most likely `self.env()`):

```rust
create_using(self.env())
```

<table style="width: 100%;">
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```rust
let accumulator = Accumulator::new(accumulator_code_hash, init_value)
    .value(total_balance / 4)
    .create()
    .expect("failed at instantiating the accumulator contract");
```

</td>
<td>

```rust
let accumulator = Accumulator::new(init_value)
    .value(total_balance / 4)
    .gas_limit(12345)
    .using_code(accumulator_code_hash)
    .create_using(self.env())
    .expect("failed at instantiating the `Accumulator` contract");
```

</td>
</tr>
</table>

See the [delegator example](https://github.com/use-ink/ink/blob/master/integration-tests/public/upgradeable-contracts/delegator/lib.rs).

### Contract Tests

Testing contracts off-chain is done by `cargo test` and users can simply use the standard routines
of creating unit test modules within the ink! project:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() { ... }
}
```

Test instances of contracts can be created with something like:

```rust
let contract = MyContract::my_constructor(a, b);
```

Messages can simply be called on the returned instance as if `MyContract::my_constructor` returns a
`Self` instance.

See the [flipper example](https://github.com/use-ink/ink/blob/master/examples/flipper/src/lib.rs).

**The off-chain test environment has lost a bit of power compared to the old ink! language.**

It is not currently possible to query and set special test data about the environment (such as the
caller of a function or amount of value sent), but these will be added back in the near future.

### ink!-less Implementations

It is also possible to annotate an entire `impl` blocks with:

```rust
#[ink(impl)]
impl Contract {
    fn internal_function(&self) {
        self.env().emit_event(EventName);
    }
}.
```

This is useful if the `impl` block itself does not contain any ink! constructors or messages, but you
still need to access some of the "magic" provided by ink!. In the example above, you would not have
access to `emit_event` without `#[ink(impl)]`.
