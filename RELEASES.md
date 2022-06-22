# [Unreleased]

# Version 3.3.0

This release restores SemVer compatibility in the `v3.x` series of releases, as well as
compatibility with the [`v0.13.0`](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.13.0)
release of the `substrate-contracts-node`.

## Compatibility
This version will work fine with *substrate-contracts-node* versions from
[0.13.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.13.0) up
to [0.16.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.16.0).

## Changed
*Context: user-reported issues on our SE unveiled backward incompatibility introduced in 3.1.0 release.*
1. [CodeRejected when using ink! v3.1.0](https://substrate.stackexchange.com/questions/2721/cargo-contract-3-0-1)
1. [Incompatibility between ink! v3.0.1 and v3.2.0 ](https://substrate.stackexchange.com/questions/2870/cargo-contract-throws-error-about-supplied-arguments-in-inkconstructor-f)

The following has been done to restore backward compatibility:
- Reverted backward-incompatible piece of [#1224](https://github.com/paritytech/ink/pull/1224).
    - The return signature of `ink_env::set_contract_storage()` was changed to return an
      `Option<u32>`. This could have broken existing code, so this should've been done in
      a `MAJOR` release.
    - Under the hood the PR also changed `Mapping::insert()` to use a new SEAL API
    (`[seal1] seal_set_storage`), which resulted in `CodeRejected` errors in nodes which
    did not have this API (e.g `substrate-contracts-node@0.13.0`).
- Reverted "Optimise deny_payment. Use everywhere semantic of deny ([#1267](https://github.com/paritytech/ink/pull/1267))"
  - This one is to restore compatibility between minor versions of ink! crates; see
    @HCastano's SE [answer](https://substrate.stackexchange.com/a/3000/472) in this
    regard.
- Reverted backward-incompatible piece of [#1233](https://github.com/paritytech/ink/pull/1233).
    - The removal of the `eth_compatibility` crate should have been done in a `MAJOR`
      release.

All these breaking changes are subjects to the upcoming MAJOR *ink!* release 4.0.0.

# Version 3.2.0

## Compatibility
We recommend using a version of the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [c0ee2ad](https://github.com/paritytech/substrate/tree/c0ee2adaa54b22ee0df5d1592cd0430961afd95c)
(May 23, 2022) in your node.

This is the case in the latest release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node)
[v0.16.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.16.0).

## Added
- Contract size optimization in case contract doesn't accept payment ‒ [#1267](https://github.com/paritytech/ink/pull/1267) (thanks [@xgreenx](https://github.com/xgreenx)).

## Changed
- Two functions have been stabilized: [`ink_env::ecdsa_recover`](https://paritytech.github.io/ink/ink_env/fn.ecdsa_recover.html) and [`ink_env::ecdsa_to_eth_address`](https://paritytech.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html) ‒ [#1270](https://github.com/paritytech/ink/pull/1270) [#1273](https://github.com/paritytech/ink/pull/1273)

## Fixed
- Fixed bug with recent Rust and `cargo test` ‒ [#1272](https://github.com/paritytech/ink/pull/1272) (thanks [@xgreenx](https://github.com/xgreenx)).

# Version 3.1.0

## Compatibility
We recommend using a version of the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
later than [7d233c2](https://github.com/paritytech/substrate/tree/7d233c2446b5a60662400a0a4bcfb78bb3b79ff7)
(May 13, 2022) in your node.

This is the case in the latest release of the [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node)
[v0.15.1](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.15.1) and
the latest Polkadot release [v0.9.22](https://github.com/paritytech/polkadot/releases/tag/v0.9.22).

## Breaking Changes
There are two breaking changes in this release:

* As part of [#1235](https://github.com/paritytech/ink/pull/1235) the message selectors of
  your contract may change. A change of selectors would affect your client, frontend, Dapp, etc..
* As part of [#1233](https://github.com/paritytech/ink/pull/1235) we removed the `eth_compatibility`
  crate.<br><br>
  Its recovery functionality has been moved to `ink_env` now: [`ink_env::ecdsa_to_eth_address`](https://paritytech.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html).
  The reason for this change is that we moved the gas-expensive crypto operations into `pallet-contracts`.<br><br>
  The crates `to_default_account_id` function has been removed; the reason for this change is that ink!
  doesn't have knowledge about the specific Substrate types on the underlying chain.
  If you want to retain the function in your contract and are just using standard Substrate types
  you should add the prior functionality to your contract ‒ it was a simple
  `<Blake2x256 as CryptoHash>::hash(&ecdsa_pubkey[u8; 33])`.

## New API functions
We added two new `Mapping` API functions:
[`Mapping::contains`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html#method.contains) and
[`Mapping::insert_return_size`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html#method.insert_return_size) ‒ [#1224](https://github.com/paritytech/ink/pull/1224).
These are more gas-efficient than whatever you were using previously.

Additionaly there are a couple new `ink_env` functions now:
* [`ink_env::set_code_hash`](https://paritytech.github.io/ink/ink_env/fn.set_code_hash.html)
* [`ink_env::own_code_hash`](https://paritytech.github.io/ink/ink_env/fn.own_code_hash.html)
* [`ink_env::code_hash`](https://paritytech.github.io/ink/ink_env/fn.code_hash.html)
* [`ink_env::ecdsa_to_eth_address`](https://paritytech.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html)

## New Upgradeable Contract Example

We've added a new example: [`upgradeable-contracts/set-code-hash`](https://github.com/paritytech/ink/tree/master/examples/upgradeable-contracts#set-code-hash).

It illustrates how the newly added [`ink_env::set_code_hash`](https://paritytech.github.io/ink/ink_env/fn.set_code_hash.html)
can be used to implement an upgradeable contract that replaces its own code.

## Added
- Implement `seal_code_hash` and `seal_own_code_hash` ‒ [#1205](https://github.com/paritytech/ink/pull/1205)
- Add `set_code_hash` function and example ‒ [#1203](https://github.com/paritytech/ink/pull/1203)
- Implement [`ink_env::ecdsa_to_eth_address`](https://paritytech.github.io/ink/ink_env/fn.ecdsa_to_eth_address.html) ‒ [#1233](https://github.com/paritytech/ink/pull/1233)
- Add [`Mapping::contains(key)`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html#method.contains) and [`Mapping::insert_return_size(key, val)`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html#method.insert_return_size) ‒ [#1224](https://github.com/paritytech/ink/pull/1224)

## Fixed
- Fix ordering of message ids if the trait is implemented before the inherent section ‒ [#1235](https://github.com/paritytech/ink/pull/1235)

## Removed
- Removed `eth_compatibility` crate and moved its functionality partly into `ink_env` ‒ [#1233](https://github.com/paritytech/ink/pull/1233)

# Version 3.0.1

## Changed
- Improve upgradeable examples folder structure, explain differences ‒ [#1188](https://github.com/paritytech/ink/pull/1188)

## Fixed
- Update codegen after SCALE v3.1.2 release ‒ [#1189](https://github.com/paritytech/ink/pull/1189)
- Stop using `CallData` in `multisig` example doc test ‒ [#1202](https://github.com/paritytech/ink/pull/1202)

# Version 3.0.0

This is the stable release for ink! 3.0.

It took us a while to get here and going forward we want to do
smaller releases more often.

*Please note that ink! has not been audited.*

## Compatibility
We recommend using a version of the `contracts` pallet later than
[cc282f84ba53ed2a08374d2a655dc8f08cbc5e86](https://github.com/paritytech/substrate/tree/cc282f84ba53ed2a08374d2a655dc8f08cbc5e86)
(March 15, 2022) in your node.

This is the case in the latest release of the `substrate-contracts-node`:
[v0.10.0](https://github.com/paritytech/substrate-contracts-node/releases/tag/v0.10.0).

## Breaking Changes
### We replaced the default off-chain testing environment
The off-chain testing environment can be used to write unit tests
for your smart contract with a simulated chain.
We've now replaced the existing off-chain environment with a new
one, which has a bit of a different API.

The major changes are that there is no longer any notion of "execution
context" ‒ so no more `push_execution_context` or `pop_execution_context`.
You can achieve all the same things with the new API, see [here](https://paritytech.github.io/ink/ink_env/test/index.html)
for the API documentation.

We've also switched all our examples to this new environment, you
can find more "template use-cases" there (e.g. for
[chain extension testing](https://github.com/paritytech/ink/tree/master/examples/rand-extension))

### We removed the dynamic storage allocator
More details on the reasoning behind this can be found in [#1148](https://github.com/paritytech/ink/pull/1148).

### `CallBuilder` API changed to support `delegate` calls
The `CallBuilder` API changed to now support two types of calls:

* `Call`: a cross-contract call.<br/>
   This was the default until this new API change.
* `DelegateCall`: a delegated call.<br/>
  This enables writing upgradeable contracts using
  the `delegate` pattern. An example has been added to demonstrate this:
  [`delegate-calls`](https://github.com/paritytech/ink/tree/master/examples/upgradeable-contracts/delegate-calls).

This is a breaking change, users must now specify the `call_type` to the builder manually.

_If you want to keep existing behavior you just need to specify the type `Call` now._

More details on this change can be found in [#1133](https://github.com/paritytech/ink/pull/1133).

### Unify `ink_env::{eval_contract, invoke_contract}`

The API for `eval_contract` and `invoke_contract` changed. You can read more
about the change in [#1165](https://github.com/paritytech/ink/pull/1165).

## Added
- Added `keep_attr` to `#[ink::contract]` and `#[ink::trait_definition]` ‒ [#1145](https://github.com/paritytech/ink/pull/1145) (thanks [@xgreenx](https://github.com/xgreenx))..
- Implemented the `seal_is_contract` and `seal_caller_is_origin` API ‒ [#1129](https://github.com/paritytech/ink/pull/1129) [#1166](https://github.com/paritytech/ink/pull/1166).
- Add tests in experimental off-chain env for `trait-erc20` ‒ [#1158](https://github.com/paritytech/ink/pull/1158).
- Add tests in experimental off-chain env for `erc721` ‒ [#1157](https://github.com/paritytech/ink/pull/1157).
- Add tests in experimental off-chain env for `multisig` ‒ [#1159](https://github.com/paritytech/ink/pull/1159).
- Add tests in experimental off-chain env for `dns` ‒ [#1156](https://github.com/paritytech/ink/pull/1156).
- Implemented chain extension testing in experimental off-chain env ‒ [#1152](https://github.com/paritytech/ink/pull/1152).

## Changed
- Replaced default off-chain testing engine with experimental one ‒ [#1144](https://github.com/paritytech/ink/pull/1144).
- Changed `CallBuilder` API to now support delegate calls ‒ [#1133](https://github.com/paritytech/ink/pull/1133) (thanks [@VargSupercolony](https://github.com/VargSupercolony) and [@xgreenx](https://github.com/xgreenx)).
- Unify `ink_env::{eval_contract, invoke_contract}` ‒ [#1165](https://github.com/paritytech/ink/pull/1165).

## Removed
- Removed the dynamic storage allocator ‒ [#1148](https://github.com/paritytech/ink/pull/1148).
- Removed `compile_as_dependency` config option ‒ [#1168](https://github.com/paritytech/ink/pull/1168).

# Version 3.0-rc9

This is the 9th release candidate for ink! 3.0.

## Breaking Changes
### We removed all data structures other than `Mapping` from the public ink! API

This is a drastic breaking change; it was no easy decision for us.
It affects `Lazy` and `Memory` as well. The rationale behind this change,
as well as some notes on migrating existing contracts, are explained
in [#1111](https://github.com/paritytech/ink/pull/1111) and
[#1137](https://github.com/paritytech/ink/pull/1137).

If you used `Memory` in your contract, you can achieve the same functionality
by passing this data via arguments. If you think there's a case to be
made for bringing it back, please get in contact with us.

If you use [`ink_storage::Mapping`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html)
in your contract, you need to initialize the data structure using the helper function
[`ink_lang::utils::initialize_contract(…)`](https://paritytech.github.io/ink/ink_lang/utils/fn.initialize_contract.html).
For more code examples you can take a look at our examples, e.g.
[`erc20`](https://github.com/paritytech/ink/blob/master/examples/erc20/lib.rs).

### Please upgrade `scale-info` and `parity-scale-codec` in your contract's dependencies

In this release candidate we upgraded `scale-info` and `parity-scale-codec`. You have to use a compatible
version in your contract's `Cargo.toml` as well; `cargo-contract` will throw an error otherwise.

The `Cargo.toml` should contain
```
scale-info = { version = "2", default-features = false, features = ["derive"], optional = true }
scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive", "full"] }
```

## Added
- Export `ink_lang::utils::initialize_contract(…)` - [#1077](https://github.com/paritytech/ink/pull/1077).
- Add `get_owner()` function to `dns` example contract - [#1118](https://github.com/paritytech/ink/pull/1118) (thanks [@agryaznov](https://github.com/agryaznov)).
- Improved usage documentation of `ink_storage::Mapping` - [#1138](https://github.com/paritytech/ink/pull/1138).

## Changed
- Updated to `parity-scale-codec = "3"` and `scale-info = "2"` - [#1132](https://github.com/paritytech/ink/pull/1132).

## Removed
- Remove `collection` and `lazy` modules from public ink! API - [#1111](https://github.com/paritytech/ink/pull/1111).
- Remove `Memory` from public ink! API - [#1137](https://github.com/paritytech/ink/pull/1137).

## Fixed
- Fix bug with referencing two external trait definitions - [#1141](https://github.com/paritytech/ink/pull/1141).
- Explicitly specify trait in dispatching - [#1131](https://github.com/paritytech/ink/pull/1131) (thanks [@xgreenx](https://github.com/xgreenx)).
- Make `rust-analyzer` expand ink! macros without warning - [#1107](https://github.com/paritytech/ink/pull/1107).

# Version 3.0-rc8

This is the 8th release candidate for ink! 3.0.

## Compatibility

You need to upgrade to a new version of the `contracts` pallet in order to use this
ink! release. Specifically we recommend you upgrade to at least the pallet version
[249dbbb](https://github.com/paritytech/substrate/commit/249dbbba6a1a277a3098c2a5b302645da16451ad)
(or newer than Jan 17, 2022).

For `substrate-contracts-node` we just issued a new release `v0.5.0`, which has been synchronized
to the latest Substrate `master` and hence is compatible with this ink! release candidate.

Also, the metadata changed with this release, so you need to have a recent version of
the user interface you are using. For `polkadot-js/api` and `polkadot-js/apps` the metadata
changes are supported since Jan 19, 2022.

## Metadata Changes
There are two metadata changes in this release candidate:

### For messages and constructors: `name: Array` ➔ `label: String`

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

The ink! PR which implemented this is [#923](https://github.com/paritytech/ink/pull/923),
for the `polkadot-js/api` it was [polkadot-js/api#4255](https://github.com/polkadot-js/api/pull/4255).

### Constructors got a new field: `payable: bool`
Constructors now have to be specified `payable` if it's intended for them to receive value
(i.e. tokens). This is a breaking change, beforehand they were by default payable,
now they are by default non-payable.

The ink! PR which implemented this is [#1065](https://github.com/paritytech/ink/pull/1065), the PR that updated the ink!
metadata to V3 is [#1100](https://github.com/paritytech/ink/pull/1100), and for the `polkadot-js/api` it was [polkadot-js/api#4432](https://github.com/polkadot-js/api/pull/4432).

## Changed
- Update metadata to support payable constructors - [#1100](https://github.com/paritytech/ink/pull/1100).
- Make constructors non-payable by default, require specifying `payable` explicitly - [#1065](https://github.com/paritytech/ink/pull/1065).
- Renamed the error code `EcdsaRecoverFailed` to `EcdsaRecoveryFailed` ‒ [#1064](https://github.com/paritytech/ink/pull/1064).
- Renamed the `ink_env` function `transferred_balance()` to `transferred_value()` ‒ [#1063](https://github.com/paritytech/ink/pull/1063).
- Removed the error codes `BelowSubsistenceThreshold` and `NewContractNotFunded` ‒ [#1062](https://github.com/paritytech/ink/pull/1062).
- Updated ink! to use the most recent `contracts` pallet API ‒ [#1053](https://github.com/paritytech/ink/pull/1053).
- Explicitly link against `rlibc` to get non-buggy version of `memcpy` ‒ [#1049](https://github.com/paritytech/ink/pull/1049).
- Changed the metadata field `name` to `label` for messages and constructors ‒ [#923](https://github.com/paritytech/ink/pull/923) (thanks [@xgreenx](https://github.com/xgreenx)).

## Added
- Added a `remove` method to the `Mapping` data structure ‒ [#1023](https://github.com/paritytech/ink/pull/1023).

# Version 3.0-rc7

This is the 7th release candidate for ink! 3.0.

Since our last release candidate we implemented a number of contract size improvements.
With those improvements the size of our `erc20` example has reduced significantly:

|          |             | Release Build with `cargo-contract` |
|:---------|:------------|:------------------------------------|
| `erc20`  | `3.0.0-rc6` | 29.3 K                              |
| `erc20`  | `3.0.0-rc7` | 10.4 K                              |

The savings apply partly to our other examples; for `erc20` they are most
significant since it has been migrated to use a new [`Mapping`](https://paritytech.github.io/ink/ink_storage/lazy/struct.Mapping.html)
data structure, which we introduce with this release candidate.
The other examples will be migrated to this new data structure as a next step.

## Compatibility

You need to upgrade to a new version of the `contracts` pallet in order to use
this ink! release. This is because we upgraded to a newer `seal_call` API.
Specifically you need to upgrade to at least the pallet version
[e9fca0f](https://github.com/paritytech/substrate/commit/e9fca0f4d7a65998206f29baf0133a7931452104)
(or newer than Nov 24).

## Removed
- Removed the state rent API ‒ [#1036](https://github.com/paritytech/ink/pull/1036).

## Added
- Added support for wildcard selectors ‒ [#1020](https://github.com/paritytech/ink/pull/1020).
    - This enables writing upgradeable smart contracts using the proxy/forward pattern.
      We added a new example to illustrate this ‒ the [proxy](https://github.com/paritytech/ink/tree/master/examples/proxy) example.
    - Annotating a wildcard selector in traits is not supported.
- The ink! codegen now heavily relies on static type information based on traits defined in `ink_lang` ‒ [#665](https://github.com/paritytech/ink/pull/665).
    - Some of those traits and their carried information can be used for static reflection of ink!
      smart contracts. Those types and traits reside in the new `ink_lang::reflect` module and is
      publicly usable by ink! smart contract authors.

## Changed
- Upgraded to the `seal_call` v1 API ‒ [#960](https://github.com/paritytech/ink/pull/960).
  - This API now enables control over the behavior of cross-contract calls, e.g. to forward/clone input,
    enable tail calls and control reentrancy.
    The crate documentation contains more details on the [`CallFlags`](https://paritytech.github.io/ink/ink_env/struct.CallFlags.html).
  - **Note:** The default behavior of cross-contract calls now disallows reentering the calling contract.
- ink! contract definitions via `#[ink::contract]` ‒ [#665](https://github.com/paritytech/ink/pull/665).<br/>
  For ink! smart contracts we now generate two contract types. Given `MyContract`:
    - `MyContract` will still be the storage struct.
      However, it can now additionally be used as static dependency in other smart contracts.
      Static dependencies can be envisioned as being directly embedded into a smart contract.
    - `MyContractRef` is pretty much the same of what we had gotten with the old `ink-as-dependency`.
      It is a typed thin-wrapper around an `AccountId` that is mirroring the ink! smart contract's API
      and implemented traits.
- ink! trait definitions via `#[ink::trait_definition]` ‒ [#665](https://github.com/paritytech/ink/pull/665).
    - ink! trait definitions no longer can define trait constructors.
    - ink! trait implementations now inherit `selector` and `payable` properties for trait messages.
        - Now explicitly setting `selector` or `payable` property for an implemented ink! trait method
          will only act as a guard that the set property is in fact the same as defined by the ink!
          trait definition.
- Improved some ink! specific compile errors ‒ [#665](https://github.com/paritytech/ink/pull/665).
    - For example, when using ink! messages and constructors which have inputs (or
      outputs) that cannot be encoded (or decoded) using the SCALE codec.
- Simplified selector computation for ink! trait methods ‒ [#665](https://github.com/paritytech/ink/pull/665).
    - Now selectors are encoded as `blake2b({namespace}::{trait_identifier}::{message_identifier})[0..4]`.
      If no `namespace` is set for the ink! trait definition then the formula is
      `blake2b({trait_identifier}::{message_identifier})[0..4]`.
      Where `trait_identifier` and `message_identifier` both refer to the identifiers of the ink! trait
      definition and ink! trait message respectively.
- We switched to Rust edition 2021 ‒ [#977](https://github.com/paritytech/ink/pull/977).
- Update chain extension example to show argument passing ‒ [#1029](https://github.com/paritytech/ink/pull/1029).

## Fixed
- Contracts now revert the transaction if an ink! message returns `Result::Err` ‒ [#975](https://github.com/paritytech/ink/pull/975), [#998](https://github.com/paritytech/ink/pull/998).
    - It is still possible to match against a `Result` return type for a called dependency contract
      ‒ i.e. a sub-contract specified in the contract's `Cargo.toml`.
- We implemented a number of Wasm contract size improvements:
    - Simple Mapping Storage Primitive ‒ [#946](https://github.com/paritytech/ink/pull/946).
    - Remove `always` from `inline` to allow compiler decide that to do ‒ [#1012](https://github.com/paritytech/ink/pull/1012) (thanks [@xgreenx](https://github.com/xgreenx)).
    - Add a way to allocate a storage facility using spread (and packed) layouts ‒ [#978](https://github.com/paritytech/ink/pull/978).
    - Extract non-generic part of `push_topic` to reduce code size ‒ [#1026](https://github.com/paritytech/ink/pull/1026).

# Version 3.0-rc6

This is the 6th release candidate for ink! 3.0.

## Compatibility

### Please upgrade `cargo-contract`

You need to update to the latest `cargo-contract` in order to use this release:
```
cargo install cargo-contract --vers ^0.15 --force --locked
```

If you build contracts from this release candidate with an older `cargo-contract`,
the UI's won't display all contract-relevant fields.

### Please upgrade `scale-info` in your contract's dependencies

In this release candidate we upgraded `scale-info`. You have to use a compatible
version in your contract's `Cargo.toml` as well; `cargo-contract` will throw
an error otherwise.

The `Cargo.toml` should contain
```
scale-info = { version = "1.0", default-features = false, features = ["derive"], optional = true }
scale = { package = "parity-scale-codec", version = "2", default-features = false, features = ["derive", "full"] }
```

### New metadata format

There are breaking changes to the metadata format in this release.

- Removes top level `metadataVersion` field from the contract metadata (https://github.com/paritytech/cargo-contract/pull/342/files).
- Introduces new top level versioned metadata [enum](https://github.com/paritytech/ink/blob/master/crates/metadata/src/lib.rs#L68).
- Upgrades to `scale-info` version `1.0` (https://github.com/paritytech/ink/pull/845).
  - The previous supported version was `0.6`, so check release notes for all changes since then: https://github.com/paritytech/ink/pull/845
  - One of the main changes to be aware of is the change to 0 based type lookup ids: https://github.com/paritytech/scale-info/pull/90

## Added
- Added an Ethereum-compatibility function to recover a public key from an ECDSA signature and message hash - [#914](https://github.com/paritytech/ink/pull/914) (thanks [@xgreenx](https://github.com/xgreenx)).
- Added new utility proc. macros to `ink_lang` crate - [#947](https://github.com/paritytech/ink/pull/947):
    - `blake2!`: Compute the BLAKE2b 256-bit hash of the given input literal string.
    - `selector_bytes!`: Compute the ink! selector of the given input literal string and return it as `[u8; 4]`.
    - `selector_id!`: Compute the ink! selector of the given input literal string and return it as `u32`.

## Changed
- Update to `scale-info` 1.0 - [#845](https://github.com/paritytech/ink/pull/845).
- Message and constructor selectors no longer take their inputs as string, but as `u32` decodable integer - [#928](https://github.com/paritytech/ink/pull/928).<br/>
  For example:
    - It is no longer possible to specify a selector as `#[ink(selector = "0xC0DECAFE")]`.
    - The newly allowed formats are `#[ink(selector = 0xC0DECAFE)]` and `#[ink(selector = 42)]`.
    - Smart contract authors are required to update their smart contracts for this change.
- Improved the `multisig` example - [#962](https://github.com/paritytech/ink/pull/962).
- Changed the link to our beginner's workshop to the migrated workshop on `substrate.io` - [#957](https://github.com/paritytech/ink/pull/957).

## Fixed
- Fixed a mistake in the `ink_env::block_timestamp()` documentation - [#937](https://github.com/paritytech/ink/pull/937).

# Version 3.0-rc5 (2021-09-08)

This is the 5th release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

## Compatibility

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

## Added
- Added example for mocking chain extensions in off-chain tests ‒ [#882](https://github.com/paritytech/ink/pull/882).
- Panic messages are now printed to debug buffer ‒ [#894](https://github.com/paritytech/ink/pull/894).

## Changed
- Unlicensed smart contract examples ‒ [#888](https://github.com/paritytech/ink/pull/888).
- Stabilized `seal_debug_message` ‒ [#902](https://github.com/paritytech/ink/pull/902).

# Version 3.0-rc4 (2021-07-19)

This is the 4th release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

## Compatibility

ink! 3.0-rc4 is compatible with

- The "ink! CLI" [`cargo-contract`](https://github.com/paritytech/cargo-contract)
  version `0.13.0` or newer.
    - Install the newest version using `cargo install --force cargo-contract`.
- Substrate version `4.0.0-dev` including the `contracts-pallet` version `4.0.0-dev`.
- [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) version `0.1.0` or newer.
    - Install the newest version using `cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --force`.

The documentation on our [Documentation Portal](https://ink.substrate.io)
is up-to-date with this release candidate. Since the last release candidate we notably
added a number of [Frequently Asked Questions](https://ink.substrate.io/faq)
there.

## Quality Assurance

In order to ensure a continuously high quality of our codebase we implemented a number
of key improvements to our testing setup:

- We've put an emphasis on automated testing of the usage examples in our crate documentation.
  Those are now tested in the context of a complete ink! contract. In the past this was not
  always the case, sometimes usage examples were just isolated code snippets.
- We started our [`ink-waterfall`](https://github.com/paritytech/ink-waterfall) project,
  which runs End-to-End tests through our entire stack.
  All our examples are continuously built using the latest `cargo-contract`. They are
  subsequently deployed on the latest `substrate-contracts-node` by emulating browser interactions with
  both the [`canvas-ui`](https://paritytech.github.io/canvas-ui/#/) and the
  [`polkadot-js`](https://polkadot.js.org/apps/#/) UI.
  This testing setup enables us to detect bugs which only appear in the context of using
  multiple components together early on.
- To improve the readability of our documentation we introduced automated grammar and spell
  checking into our Continuous Integration environment.

## Added
- Added support for the new `seal_random` API ‒ [#734](https://github.com/paritytech/ink/pull/734).
- Added missing documentation for the `ink_storage_derive` procedural macros ‒ [#711](https://github.com/paritytech/ink/pull/711).
- Implemented the (unstable) `seal_rent_params` API ‒ [#755](https://github.com/paritytech/ink/pull/755).
- Implemented the (unstable) `seal_rent_status` API ‒ [#798](https://github.com/paritytech/ink/pull/798).
- Implemented the (unstable) `seal_debug_message` API ‒ [#792](https://github.com/paritytech/ink/pull/792).
    - Printing debug messages can now be achieved via `ink_env::debug_println!(…)`.
    - See [our documentation](https://ink.substrate.io/faq#how-do-i-print-something-to-the-console-from-the-runtime)
      for more information.
    - The examples have been updated to reflect this new way of printing debug messages.
- Added usage comments with code examples to the `ink_env` API ‒ [#797](https://github.com/paritytech/ink/pull/797).
    - The [published crate documentation](https://paritytech.github.io/ink/ink_lang/struct.EnvAccess.html) now contains
      much more code examples for the methods behind `self.env()` and `Self::env()`.
- Added an example implementation for ERC-1155, a multi-token standard ‒ [#800](https://github.com/paritytech/ink/pull/800).
- Implemented binary search for `collections::Vec` ‒ [#836](https://github.com/paritytech/ink/pull/836).
- Added the ability of submitting payable transactions to the `multisig` example ‒ [#820](https://github.com/paritytech/ink/pull/820).
- Implemented `Decode` for `Error` types in the examples, enabling building them as dependencies ‒ [#761](https://github.com/paritytech/ink/pull/761).
- We started working on a new off-chain environment testing engine ‒ [#712](https://github.com/paritytech/ink/pull/712).
    - The old testing environment has a number of limitations, which we are well aware of.
      We're confident that with the new testing engine we will be able to conduct much more
      elaborate testing in an emulated chain environment.
    - For the moment, the new engine is unstable and only available behind a feature flag.
      A number of examples have already been converted to support the new testing engine.

## Changed
- To reduce a contract's space footprint we switched the default allocator to a bump allocator implementation ‒ [#831](https://github.com/paritytech/ink/pull/831).
- A couple of readme's have been reworked:
    - Our main ink! readme ‒ [#774](https://github.com/paritytech/ink/pull/774).
    - The `rand-extension` example readme ‒ [#793](https://github.com/paritytech/ink/pull/793).
    - The `delegator` example readme ‒ [#766](https://github.com/paritytech/ink/pull/766).
- With the stabilization of Rust 1.51 we ware able to remove the `ink-unstable` feature, making
  `collections::SmallVec` and `lazy::LazyArray` available by default ‒ [#746](https://github.com/paritytech/ink/pull/746).
- To resolve confusion, we migrated all usages of `#[test]` in our examples to `#[ink::test]` ‒ [#746](https://github.com/paritytech/ink/pull/746).
    - The difference is that `#[ink::test]` spawns an emulated chain environment (an "off-chain" environment)
      and hence comes with a bit of overhead. It was not always clear to users when they require
      an off-chain environment, we decided to mitigate this confusion by using an emulated chain
      environment for all our example tests.
- With the stabilization of Rust's `min_const_generics` we were able to replace the fixed
  size implementations of `SpreadLayout` and `PackedLayout` for Arrays. These traits are
  now implemented for all Arrays of size `usize` ‒ [#754](https://github.com/paritytech/ink/pull/754).
- We were able to remove the pinned `funty` dependency ‒ [#711](https://github.com/paritytech/ink/pull/711).
- The `contract-transfer` example has been improved for better UI support ‒ [#789](https://github.com/paritytech/ink/pull/789).
- The `contract-transfer` example has been improved for better error handling ‒ [#790](https://github.com/paritytech/ink/pull/790).

## Fixed
- Catch illegal `struct` destructuring pattern in ink! message arguments ‒ [#846](https://github.com/paritytech/ink/pull/846).
- Removed an erroneous `Salt` type in code generation for cross-contract calls ‒ [#842](https://github.com/paritytech/ink/pull/842).
- Do not generate metadata if compiled as dependency ‒ [#811](https://github.com/paritytech/ink/pull/811).
- Fix execution context parameters in DNS example tests ‒ [#723](https://github.com/paritytech/ink/pull/723).
- Fixed the `Greeter` contract example from our doc comments ‒ [#773](https://github.com/paritytech/ink/pull/773).

# Version 3.0-rc3 (2021-03-02)

This is the 3rd release candidate for ink! 3.0.

The list below shows the additions, changes and fixes that are visible to users of ink!.

## Compatibility

ink! 3.0-rc3 is compatible with

- The `cargo-contract` CLI tool version `0.9.1` or newer.
    - Install newest version using `cargo install --force cargo-contract`.
- Substrate version `3.0` including the `contracts-pallet` version `3.0`.

## Added

- Implemented chain extensions feature for ink!.
- ink!'s official documentation portal: https://ink.substrate.io/
- It is now possible to pass a `salt` argument to contract instantiations.
- Implemented fuzz testing for the ink! codebase.

## Changed

- Migrate `ink_storage::SmallVec` and `ink_storage::lazy::SmallLazyArray` to use `min_const_generics`.
    - The `min_const_generics` feature is going to be stabilized in Rust 1.51. For now it was put behind
      the `ink-unstable` crate feature of the `ink_storage` crate.
- Improve error reporting for conflicting ink! attributes.
- Improve error reporting for invalid constructor or message selector. (https://github.com/paritytech/ink/pull/561)
- Remove `iter_mut` for `ink_storage::BinaryHeap` data structure.
- Add documented demonstration how to properly mock `transferred_balance` calls: https://github.com/paritytech/ink/pull/555
- Add contract example which uses `ext_transfer` and `ext_terminate`: https://github.com/paritytech/ink/pull/554
- Improve documentation of `transfer` and `minimum_balance` APIs: https://github.com/paritytech/ink/pull/540

## Fixed

- The Delegator example contract now compiles properly using the `build-all.sh` bash script.
- Update crate dependencies:
    - `scale-info 0.6`
    - `parity-scale-codec 2.0`
    - `rand 0.8`
    - `itertools 0.10`
- Remove unused `tiny-keccak` dependency from `ink_primitives`.
- Changed the default `BlockNumber` type to `u32`. This is a fix since it now properly mirrors Substrate's default `BlockNumber` type.
- Ensure topics are unique: https://github.com/paritytech/ink/pull/594
- Several fixes for `ink_storage` data structures, including:
    - `Drop` implementation for `Pack` now works properly. (https://github.com/paritytech/ink/pull/600)
    - `Drop` implementation for `Lazy` now always properly clean up storage. (https://github.com/paritytech/ink/pull/597)
    - Nested `Lazy` now properly clears storage data. (https://github.com/paritytech/ink/pull/583)
    - `Option` fields now properly clean up nested storage data. (https://github.com/paritytech/ink/pull/570)

# Version 3.0-rc2 (2020-10-22)

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

# Version 3.0-rc1 (2020-10-09)

Be prepared for the ink! 3.0 release notes because the whole version was basically a rewrite of
all the major components that make up ink!. With our experience gained from previous releases
of ink! we were able to detect weak spots of the design and provided ink! with more tools,
more features and more efficiency as ever. Read more below …

## Just. Be. Rust. 3.0

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

## Split of ink_core

The `ink_core` crate no longer exists. It has been split into the new `ink_env` and `ink_storage` crates.

Everything that was previously accessed through `ink_core::env` now lives in `ink_env` and everything
that was previously accessed through `ink_core::storage` now lives in `ink_storage`. Both crates keep
the responsibilities of their former originating `ink_core` modules.

## New Storage Module

The storage module has been reworked entirely.
Also it no longer lives in the `ink_core` crate but instead is defined as its own `ink_storage` crate.

In a sense it acts as the standard storage library for ink! smart contracts in that it provides all the
necessary tools and data structures to organize and operate the contract's storage intuitively and efficiently.

### Lazy

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

### HashMap

In the follow we explore the differences between the high-level `ink_storage::collections::HashMap`
and the low-level `ink_storage::lazy::LazyHashMap`. Both provide very similar functionality in that they map some generic key to some storage entity.

However, their APIs look very different. Whereas the `HashMap` provides a rich and high-level API that is comparable to that of Rust's very own `HashMap`, the `LazyHashMap` provides only a fraction of the API and also operates on `Option<T>` values types instead of `T` directly. It is more similar Solidity mappings than to Rust's `HashMap`.

The fundamental difference of both data structures is that `HashMap` is aware of the keys that have been stored in it and thus can reconstruct exactly which elements and storage regions apply to it. This enables it to provide iteration and automated deletion as well as efficient way to defragment its underlying storage to free some storage space again. This goes very well in the vein of Substrate's storage rent model where contracts have to pay for the storage they are using.

| Data Structure | level of abstraction | caching | lazy | element type | container |
|:--|:-:|:-:|:-:|:-:|:-:|
| `T` | - | yes | no | `T` | primitive value |
| `Lazy<T>` | high-level | yes | yes | `T` | single element container |
| `LazyCell<T>` | low-level | yes | yes | `Option<T>` | single element, no container |
| `Vec<T>` | high-level | yes | yes | `T` | Rust vector-like container |
| `LazyIndexMap<T>` | low-level | yes | yes | `Option<T>` | similar to Solidity mapping |
| `HashMap<K, V>` | high-level | yes | yes | `V` (key type `K`) | Rust map-like container |
| `LazyHashMap<K, V>` | low-level | yes | yes | `Option<V>` (key type `K`) | similar to Solidity mapping |

There are many more! For more information about the specifics please take a look into [the `ink_storage` crate documentation](https://paritytech.github.io/ink/ink_storage/).

### Spread & Packed Modes

Storing or loading complex data structures to and from contract storage can be done in many different ways. You could store all information into a single storage cell or you could try to store all information into as many different cells as possible. Both strategies have pros and cons under different conditions.

For example it might be a very good idea to store all the information under the same cell if all the information is very compact. For example when we are dealing with a byte vector that is expected to never be larger than approx a thousand elements it would probably be more efficient if we store all those thousand bytes in the same cell and especially if we often access many of those (or all) in our contract messages.

On the other hand spreading information across as many cells as possible might be much more efficient if we are dealing with big data structures, a lot of information that is not compact, or when messages that operate on the data always only need a small fraction of the whole data.
An example for this use case is if you have a vector of user accounts where each account stores potentially a lot of information, e.g. a 32-byte hash etc and where our messages only every operate on only a few of those at a time.

The `ink_storage` crate provides the user full control over the strategy or a mix of these two root strategies through some fundamental abstractions that we are briefly presenting to you.

### Default: Spreading Mode

By default ink! spreads information to as many cells as possible. For example if you have the following `#[ink(storage)]` struct every field will live in its own single storage cell. Note that for `c` all 32 bytes will share the same cell!

```rust
#[ink(storage)]
pub struct Spreaded {
    a: i32,
    b: ink_storage::Lazy<i32>,
    c: [u8; 32],
}
```

### Packing Storage

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

### Spreading Array Cells

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

### Opting-out of Storage

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

### Dynamic Storage Allocator

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

### Custom Data Sturctures

While the `ink_storage` crate provides tons of useful utilities and data structures to organize and manipulate the contract's storage contract authors are not limited by its capabilities. By implementing the core `SpreadLayout` and `PackedLayout` traits users are able to define their very own custom storage data structures with their own set of requirement and features that work along the `ink_storage` data structures as long as they fulfill the mere requirements stated by those two traits.

In the future we plan on providing some more ink! workshops and tutorials guiding the approach to design and implement a custom storage data structure.

### In Summary

The new `ink_storage` crate provides everything you need to operate on your contract's storage.
There are low-level and high-level data structures depending on your need of control.
All provided data structures operate lazily on the contract's storage and cache their reads and writes for a more gas efficient storage access.
Users should prefer high-level data structures found in the `collections` module over the low-level data structures found in the `lazy` module.
For a list of all the new storage data structure visit [`ink_storage`'s documentation](https://paritytech.github.io/ink/ink_storage/).

## ink! Attributes

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

| Attribute | Where Applicable | Description |
|:--|:--|:--|
| `#[ink(storage)]` | On `struct` definitions. | Defines the ink! storage struct. There can only be one ink! storage definition per contract. |
| `#[ink(event)]` | On `struct` definitions. | Defines an ink! event. A contract can define multiple such ink! events. |
| `#[ink(anonymous)]` **new** | Applicable to ink! events. | Tells the ink! codegen to treat the ink! event as anonymous which omits the event signature as topic upon emitting. Very similar to anonymous events in Solidity. |
| `#[ink(topic)]` | Applicate on ink! event field. | Tells the ink! codegen to provide a topic hash for the given field. Every ink! event can only have a limited number of such topic field. Similar semantics as to indexed event arguments in Solidity. |
| `#[ink(message)]` | Applicable to methods. | Flags a method for the ink! storage struct as message making it available to the API for calling the contract. |
| `#[ink(constructor)]` | Applicable to method. | Flags a method for the ink! storage struct as constructor making it available to the API for instantiating the contract. |
| `#[ink(payable)]` **new** | Applicable to ink! messages. | Allows receiving value as part of the call of the ink! message. ink! constructors are implicitly payable. |
| `#[ink(selector = "..")]` **new** | Applicable to ink! messages and ink! constructors. | Specifies a concrete dispatch selector for the flagged entity. This allows a contract author to precisely control the selectors of their APIs making it possible to rename their API without breakage. |
| `#[ink(namespace = "..")]` **new** | Applicable to ink! trait implementation blocks. | Changes the resulting selectors of all the ink! messages and ink! constructors within the trait implementation. Allows to disambiguate between trait implementations with overlapping message or constructor names. Use only with great care and consideration! |
| `#[ink(impl)]` **new** | Applicable to ink! implementation blocks. | Tells the ink! codegen that some implementation block shall be granted access to ink! internals even without it containing any ink! messages or ink! constructors. |

## Merging of ink! Attributes

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

## Trait Support

One of the most anticipated features of ink! 3.0 is its Rust trait support.
Through the new `#[ink::trait_definition]` procedural macro it is now possible to define your very own trait definitions that are then implementable by ink! smart contracts.

This allows to define shared smart contract interfaces to different concrete implementations.
Note that this ink! trait definition can be defined anywhere, even in another crate!

### Example

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

Calling the above `Erc20` explicitely through its trait implementation can be done just as if it was normal Rust code:

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

# Version 2.1 (2020-03-25)

- Add built-in support for cryptographic hashes:
    - Blake2 with 128-bit and 256-bit
    - Sha2 with 256-bit
    - Keccak with 256-bit
- Add `ink_core::hash` module for high-level API to the new built-in hashes.
- Update `runtime-storage` example ink! smart contract to demonstrate the new built-in hashes.

# Version 2.0 (2019-12-03)

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

## Update the ink! CLI

Install the latest ink! CLI using the following command:

```bash
cargo install --git https://github.com/paritytech/cargo-contract cargo-contract --force
```

There is a new contract metadata format you need to use. You can generate the metadata using:

```bash
cargo contract generate-metadata
```

This will generate a file `metadata.json` you should upload when deploying or interacting with a
contract.

## Declaring a Contract

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

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## ink! Contract Tag

The ink! contract tag can be extended to provide other configuration information about your
contract.

### Defining Custom Types

We used to define types using a special `#![env = DefaultSrmlTypes]` tag.

Now we simply include the type definition in the `#[ink::contract(...)]` tag:

```rust
#[ink::contract(version = "0.1.0", env = MyCustomTypes)]
```

By default, we use `DefaultSrmlTypes`, so you don't need to define anything unless you plan to use
custom types.

### Dynamic Allocation

It is possible to enable the dynamic environment that allows for dynamic allocations by specifying
`dynamic_allocations = true` in the parameters of the ink! header. This is disabled by default.

```rust
#[ink::contract(version = "0.1.0", dynamic_allocations = true)]
```

> Note: The dynamic environment is still under research and not yet stable.

## Declaring Storage

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

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Declaring Events

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

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Environment Handler

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

## Message Functions

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

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Defining a Constructor

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

See the [ERC20 example](https://github.com/paritytech/ink/blob/master/examples/erc20/src/lib.rs).

## Cross Contract Calls

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

See the [delegator example](https://github.com/paritytech/ink/blob/master/examples/delegator/lib.rs).

## Factory Contracts

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

See the [delegator example](https://github.com/paritytech/ink/blob/master/examples/delegator/lib.rs).

## Contract Tests

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

See the [flipper example](https://github.com/paritytech/ink/blob/master/examples/flipper/src/lib.rs).

**The off-chain test environment has lost a bit of power compared to the old ink! language.**

It is not currently possible to query and set special test data about the environment (such as the
caller of a function or amount of value sent), but these will be added back in the near future.

## ink!-less Implementations

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
