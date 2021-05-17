# Chain-side Extension

To integrate this example into Substrate you need to do two things:

* Use the code in [`chain-extension-example.rs`](./chain-extension-example.rs) as an implementation for
  the trait `ChainExtension` in Substrate.
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
