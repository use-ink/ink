# Static buffer configuration demo

This is a dummy contract illustrating how the [static buffer](/ARCHITECTURE.md#communication-with-the-pallet)
can be be configured using the crate features.

Right now we offer these set of sizes and their corresponding features:
```toml
# Configurable sizes of the static buffer
2GB-buffer = []
1GB-buffer = []
512MB-buffer = []
128MB-buffer = []
1MB-buffer = []
512kB-buffer = []
128kB-buffer = []
```

You can configure the buffer size by using one of the features:
```toml
ink = { path = "../../crates/ink", default-features = false, features = ["512kB-buffer"]}
```

Otherwise, the default size of 16 kB is used.
