# Chain Extension Example

## What is this example about?

It demonstrates how to call a custom Substrate function from ink!.

See [this chapter](https://paritytech.github.io/ink-docs/macros-attributes/chain-extension)
in our ink! documentation for more details.

There are two parts to this example:

* Defining and calling the extension in ink!.
* Defining the extension in Substrate.

## Chain-side Integration

To integrate this example into Substrate you need to do two things:

* Use the code in [`chain-extension-example.rs`](runtime/chain-extension-example.rs)
  as an implementation for the trait `ChainExtension` in Substrate.
  You can just copy/paste the content of that file into e.g. your `runtime/src/lib.rs`.

* Use the implementation as the associated type `ChainExtension` of the trait
  `pallet_contracts::Config`:
  ```rust
  impl pallet_contracts::Config for Runtime {
    …
    type ChainExtension = FetchRandomExtension;
    …
  }
  ```

## ink! Integration

See the example contract in [`lib.rs`](lib.rs).
