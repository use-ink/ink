# `call-runtime` example

## What is this example about?

It demonstrates how to call a runtime dispatchable from an ink! contract.

## Chain-side configuration

To integrate this example into Substrate you need to adjust pallet contracts configuration in your runtime:
  `pallet_contracts::Config`:
  ```rust
  impl pallet_contracts::Config for Runtime {
    …
    // `Everything` or anything that will allow for Balances::transfer extrinsic
    type CallFilter = frame_support::traits::Everything; 
    type UnsafeUnstableInterface = ConstBool<true>;
    …
  }
  ```
