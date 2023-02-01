# ink!ternals

This document describes the architecture of ink!. The information
here targets those who want to understand or modify the inner
workings of this project.

In general we treat documentation as a first-class citizen.
All crates mentioned below should be documented really well.
You can find the crate documentation on docs.rs or for our
`master` branch under GitHub pages. So for `ink` e.g.:

* [https://docs.rs/ink/latest/ink](https://docs.rs/ink/latest/ink) (latest published release)
* [https://paritytech.github.io/ink/ink](https://paritytech.github.io/ink/ink) (`master`)

ink! is composed of a number of crates that are all found in the
`crates/` folder. On a high-level those can be grouped as:

* [`ink`](https://github.com/paritytech/ink/tree/master/crates/ink):
  The ink! language itself.
* [`allocator`](https://github.com/paritytech/ink/tree/master/crates/allocator):
  The allocator used for dynamic memory allocation in a contract.
* [`env`](https://github.com/paritytech/ink/tree/master/crates/env):
  Serves two roles:
  * Exposes environmental functions, like information about the caller
    of a contract call or e.g. self-terminating the contract.
  * Provides the connection to the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts),
    so anything that calls into the underlying execution engine of the smart contract.
    This includes getting and setting a smart contracts storage, as well
    as the mentioned environmental functions.
* [`metadata`](https://github.com/paritytech/ink/tree/master/crates/metadata):
  Describes the contract in a platform agnostic way, i.e. its interface
  and the types, its storage layout, etc.
* [`prelude`](https://github.com/paritytech/ink/tree/master/crates/prelude):
  Provides an interface to typical standard library types and
  functionality (like `vec` or `string`). Since contracts are run in a
  `no_std` environment we provide this crate as an entrypoint for accessing
  functionality of the standard library.
* [`primitives`](https://github.com/paritytech/ink/tree/master/crates/primitives):
  Utilities that are used internally by multiple ink! crates.
* [`storage`](https://github.com/paritytech/ink/tree/master/crates/prelude):
  The collections that are available for contract developers to put in
  a smart contracts storage.
* [`engine`](https://github.com/paritytech/ink/tree/master/crates/engine):
  An off-chain testing engine, it simulates a blockchain environment and allows
  mocking specified conditions.
* [`e2e`](https://github.com/paritytech/ink/tree/master/crates/e2e):
  An end-to-end testing framework for ink! contracts. It requires a Substrate node
  which includes `pallet-contracts` running in the background. The crate provides a
  macro which can be used
  to write an idiomatic Rust test that will in the background create transactions,
  submit it to the Substrate chain and return the state changes, gas costs, etc.

An important thing to note is that the crates are primarily run in
a `no_std` environment.
Exceptions are `metadata` and `engine`, which cover use-cases that
are only relevant off-chain.

ink! contracts are compiled for a WebAssembly (Wasm) target architecture,
i.e. they are executed in a Wasm sandbox execution environment on the
blockchain itself ‒ hence a `no_std` environment.
More specifically they are executed by the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts),
a module of the Substrate blockchain framework. This module takes ink!
smart contracts and runs them in a sandbox environment.

## Overview

<img src="./.images/inkternals.png" alt="ink!ternals" width="800" />

The above diagram shows the main components of the ink! language
and how they interact. This pipeline is run once you execute
`cargo build` on an ink! smart contract.

The central delegating crate for the ink! eDSL is `ink`.

In the `crates/ink/` folder you'll find three separate
crates on which `ink` relies heavily:

* `ink_macro`: The procedural macros, they take code annotated with e.g.
   `[ink::contract]` and forwards it to `ink_ir`.
* `ink_ir`: Defines everything the procedural macro needs in order to
   parse, analyze and generate code for ink! smart contracts.
* `ink_codegen`: Generates Rust code from the ink! IR.

## Building ink! contracts

While you can build an ink! smart contract with just `cargo build`, we
recommend using our build tool [`cargo-contract`](https://github.com/paritytech/cargo-contract).
It automatically compiles for the correct WebAssembly target
architecture and uses an optimal set of compiler flags.

## Allocator

ink! smart contracts use a very simple bump allocator for dynamic
allocations. You can find it in `crates/allocator/`.

This allocator never frees allocated space, in case it runs out of
a defined limit of space to allocate it crashes.
This was done with the intention of reducing its complexity, which
would have resulted in higher costs for the user (due to increased
gas costs) and a lower transaction throughput. Freeing memory is
irrelevant for our use-case anyway, as the entire memory instance
is set up fresh for each individual contract call anyway.

## Unstable Rust features in ink!

We would like to get away from unstable features of Rust in ink!, so
that users can just use stable Rust for building their contracts.
At the moment we're still stuck with one nightly feature though:
[alloc_error_handler](https://github.com/rust-lang/rust/issues/51540).
It's needed because we use a specialized memory allocation handler,
the `ink_allocator` crate.
It's unclear when or if this feature will ever make it to stable.

We had a lot of issues when requiring users to use Rust nightly. Mostly
because there were regularly bugs in the nightly Rust compiler that
often took days to be fixed.
As a consequence we decided on having `cargo-contract` `v2.0.0` run
`cargo +stable build` with `RUSTC_BOOTSTRAP=1`. This is kind of a hack,
the env variable enables unstable features in the stable Rust toolchain.
But it enabled us to switch tutorials/guides to Rust stable.

One advantage is that users don't deal with an ever-changing nightly
compiler. It's easier for us to support. If you build a contract without
`cargo-contract` you will have to set this env variable too or use nightly.

## Interaction with `pallet-contracts`

The Wasm blob to which an ink! contract is compiled is executed in
an execution environment named [`pallet-contracts`](https://github.com/paritytech/substrate/commits/master/frame/contracts)
on-chain.
This `pallet-contracts` is the smart contracts module of
[the Substrate blockchain framework](http://substrate.io/).

The relationship is as depicted in this diagram:

<img src="./.images/pallet-contracts.png" alt="pallet-contracts Interaction" width="800" />

### Communication with the pallet
ink! uses a static buffer for interacting with `pallet-contracts`, i.e.
to move data between the pallet and a smart contract.
The advantage of a static buffer is that no gas-expensive heap allocations
are necessary, all allocations are done using simple pointer arithmetic.
The implementation of this static buffer is found in
[`ink_env/src/engine/on_chain/buffer.rs`](https://github.com/paritytech/ink/blob/master/crates/env/src/engine/on_chain/buffer.rs).

The methods for communicating with the pallet are found in [`ink_env/src/engine/on_chain/impls.rs`](https://github.com/paritytech/ink/blob/master/crates/env/src/engine/on_chain/impls.rs).
If you look at the implementations you'll see a common pattern of

* SCALE-encoding values on the ink! side in order to pass them as a slice
  of bytes to the `pallet-contracts`.
* SCALE-decoding values that come from the `pallet-contracts` side in order
  to convert them into the proper types on the ink! side, making them available
  for contract developers.

### The pallet API
The function signature of the `pallet-contracts` API functions is defined in
[`ink_env/src/engine/on_chain/ext.rs`](https://github.com/paritytech/ink/blob/master/crates/env/src/engine/on_chain/ext.rs).
You'll see that we import different versions of API functions, something
like the following excerpt:

```rust
#[link(wasm_import_module = "seal0")]
extern "C" {
    pub fn get_storage(
        key_ptr: Ptr32<[u8]>,
        output_ptr: Ptr32Mut<[u8]>,
        output_len_ptr: Ptr32Mut<u32>,
    ) -> ReturnCode;
}

#[link(wasm_import_module = "seal1")]
extern "C" {
    pub fn set_storage(
        key_ptr: Ptr32<[u8]>,
        value_ptr: Ptr32<[u8]>,
        value_len: u32,
    ) -> ReturnCode;
}
```

Smart contracts are immutable, thus the `pallet-contracts` can never change or remove
old API functions ‒ otherwise smart contracts that are deployed on-chain would break.

Hence there is this version mechanism. Functions start out at version `seal0` and for
each new released iteration of the function there is a new version of it introduced.
In the example above you can see that we changed the function `set_storage` at
one point.

The prefix `seal` here is for historic reasons. There is some analogy to sealing a
contract. And we found seals to be a cute animal as well ‒ like squids!

## `Environment` Trait

You can use ink! on any blockchain that was built with the [Substrate](https://substrate.io)
framework and includes the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts)
module.
Substrate does not define specific types for a blockchain, it uses
generic types throughout.
Chains built on Substrate can decide on their own which types they want
to use for e.g. the chain's block number or account id's. For example,
chains that intend to be compatible to Ethereum typically use the same
type as Ethereum for their `AccountId`.

The `Environment` trait is how ink! knows the concretes types of the chain
to which the contract will be deployed to.
Specifically, our `ink_env` crate defines a trait [`Environment`](https://paritytech.github.io/ink/ink_env/trait.Environment.html)
which specifies the types.
By default, ink! uses the default Substrate types, the `ink_env` crate
exports an implementation of the `Environment` trait for that:
[`DefaultEnvironment`](https://paritytech.github.io/ink/ink_env/enum.DefaultEnvironment.html).

If you are developing for a chain that uses different types than the
Substrate default types you can configure a different environment in
the contract macro ([documentation here](https://paritytech.github.io/ink/ink/attr.contract.html#header-arguments)):

```rust
#[ink::contract(env = MyCustomTypes)]
```

__Important:__ If a developer writes a contract for a chain that deviates
from the default Substrate types, they have to make sure to use that
chain's `Environment`.
