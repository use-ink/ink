# ink! linting rules

This crate uses [`dylint`](https://github.com/trailofbits/dylint) to define custom
linting rules for [ink!](https://github.com/paritytech/ink);

It is not part of the workspace because it needs a custom linker to be built.

The lints are written against a fixed toolchain version because they are using unstable
APIs. This is why we have a toolchain file here.

You can use it by running `cargo dylint` after adding this to your `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/paritytech/ink.git", pattern = "linting/" },
]
```
