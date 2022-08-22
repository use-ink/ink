# PSP22 Chain Extension Example

## What is this example about?

It is an example implementation of the
[PSP22 Fungible Token Standard](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md)
as a chain extension, supporting a multi-token system provided by the
[FRAME assets pallet](https://docs.substrate.io/rustdocs/latest/pallet_assets/index.html).
It effectively allows ink! contracts (L2) to interact with native assets (L1) from the
chain runtime in a standardized way.

See [this chapter](https://paritytech.github.io/ink-docs/macros-attributes/chain-extension)
in our ink! documentation for more details about chain extensions.

There are two parts to this example:

* Defining and calling the extension in ink!.
* Defining the extension in Substrate.

## Chain-side Integration

To integrate this example into Substrate you need to do two things:

* In your runtime, use the code in
  [`psp22-extension-example.rs`](runtime/psp22-extension-example.rs)
  as an implementation for the trait `ChainExtension` in Substrate.
  You can just copy/paste that file as a new module, e.g. `runtime/src/chain_extension.rs`.

* In your runtime, use the implementation as the associated type `ChainExtension` of the
  trait `pallet_contracts::Config`:
  ```rust
  impl pallet_contracts::Config for Runtime {
    …
    type ChainExtension = Psp22Extension;
    …
  }
  ```

## ink! Integration

See the example contract in [`lib.rs`](lib.rs).

## Disclaimer

:warning: This is not a feature-complete or production-ready PSP22 implementation. This
example currently lacks proper error management, precise weight accounting, tests (these
all  might be added at a later point).

:warning: `decrease_allowance` function is currently not implemented due to limitations
of the `assets` pallet.
