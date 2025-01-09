# Static buffer configuration demo

This is a dummy contract illustrating how the [static buffer](/ARCHITECTURE.md#communication-with-the-pallet)
can be configured using the environmental variables.

Simply, run:
```bash
cargo clean
INK_STATIC_BUFFER_SIZE=32 cargo test -F e2e-tests
```

This will configure the buffer to have enough space to instantiate the contract,
but not enough space to retrieve two times the caller's address as it is of 40 bytes,
but we only allocated 32 bytes to the contract.

## Note
You must run `cargo clean` every time you want to modify the buffer size
because the value is baked into the binaries.
