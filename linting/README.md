# ink! linting rules

This crate uses [`dylint`](https://github.com/trailofbits/dylint) to define custom
linting rules for [ink!](https://github.com/paritytech/ink).

It is not part of the workspace because it needs a custom linker to be built.

The lints are written against a fixed toolchain version because they are using unstable
APIs. This is why we have [a toolchain file](./rust-toolchain.toml) here.

This crate contains two libraries:
* [`mandatory`](./mandatory) lints are integrated into the ink! smart contracts' build process, adding custom compilation errors to `cargo-build`.
* [`extra`](./extra) lints are designed to check for secure coding style in smart contracts and highlight potential issues. These are optional and intended for use by the contract developer to improve the security properties of their project.

You can use them by running `cargo dylint` after adding this to your `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/paritytech/ink.git", pattern = "linting/mandatory" },
    { git = "https://github.com/paritytech/ink.git", pattern = "linting/extra" },
]
```
