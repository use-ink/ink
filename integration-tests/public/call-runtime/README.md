# `call-runtime` example

## What is this example about?

It demonstrates how to call a runtime dispatchable from an ink! contract.

## Chain-side configuration

To integrate this example into Substrate you need to adjust pallet contracts configuration in your runtime:
  ```rust
  // In your node's runtime configuration file (runtime.rs)
  impl pallet_contracts::Config for Runtime {
    …
    // `Everything` or anything that will allow for the `Balances::transfer` extrinsic.
    type CallFilter = frame_support::traits::Everything; 
    type UnsafeUnstableInterface = ConstBool<true>;
    …
  }
  ```

## Comparison to `ChainExtension`

Just as a chain extension, `call_runtime` API allows contracts for direct calling to the runtime.
You can trigger any extrinsic that is not forbidden by `pallet_contracts::Config::CallFilter`.
Consider writing a chain extension if you need to perform one of the following tasks:
- Return data.
- Provide functionality **exclusively** to contracts.
- Provide custom weights.
- Avoid the need to keep the `Call` data structure stable.
