# `custom-environment` example

## What is this example about?

It demonstrates how to use custom environment, both in the contract and in the E2E tests.

## Chain-side configuration

To integrate this example into Substrate you need to adjust pallet contracts configuration in your runtime:

```rust
// In your node's runtime configuration file (runtime.rs)
parameter_types! {
  pub Schedule: pallet_contracts::Schedule<Runtime> = pallet_contracts::Schedule::<Runtime> {
    limits: pallet_contracts::Limits {
      event_topics: 6,
      ..Default::default()
    },
    ..Default::default()
  };
}

impl pallet_contracts::Config for Runtime {
  …
  type Schedule = Schedule;
  …
}
 ```
