# ink! - Parity's ink to write smart contracts

| Linux              | Codecov              | Coveralls              | LoC              |
|:------------------:|:--------------------:|:----------------------:|:----------------:|
| [![linux][A1]][A2] | [![codecov][C1]][C2] | [![coveralls][D1]][D2] | [![loc][E1]][E2] |

[A1]: https://travis-ci.org/paritytech/ink.svg?branch=master
[A2]: https://travis-ci.org/paritytech/ink

[C1]: https://codecov.io/gh/paritytech/ink/branch/master/graph/badge.svg
[C2]: https://codecov.io/gh/paritytech/ink/branch/master

[D1]: https://coveralls.io/repos/github/paritytech/ink/badge.svg?branch=master
[D2]: https://coveralls.io/github/paritytech/ink?branch=master

[E1]: https://tokei.rs/b1/github/paritytech/ink?category=code
[E2]: https://github.com/Aaronepower/tokei#badges

[F1]: https://img.shields.io/badge/docs-core-blue.svg
[F2]: https://paritytech.github.io/ink/ink_core

[G1]: https://img.shields.io/badge/docs-model-blue.svg
[G2]: https://paritytech.github.io/ink/ink_model

[H1]: https://img.shields.io/badge/docs-abi-blue.svg
[H2]: https://paritytech.github.io/ink/ink_abi

**IMPORTANT NOTE:** WORK IN PROGRESS! Do not expect this to be working.

ink! is an [eDSL](https://wiki.haskell.org/Embedded_domain_specific_language) to write WebAssembly based smart contracts using the Rust programming language targeting Substrate blockchains.

For more information please visit [the ink! tutorial](https://substrate.dev/substrate-contracts-workshop/#/0/building-your-contract).

## Developer Documentation

| `ink_abi` | `ink_core` | `ink_model` |
|-----------|------------|-------------|
| [![][H1]][H2] | [![][F1]][F2] | [![][G1]][G2] |

### Scripts

Use the scripts provided under `scripts` directory in order to run checks on either the workspace or all examples. Please do this before pushing work in a PR.

## Examples

For building the example smart contracts found under `examples` you will need to have `cargo-contract` installed.

```
cargo install --git https://github.com/paritytech/ink cargo-contract
```

Add `--force` option to update to the most recent `cargo-contract` version.

Please visit [the documentation](https://substrate.dev/substrate-contracts-workshop/#/0/building-your-contract)
about building contracts and generating metadata.

### Hello, World! - The Flipper

The `Flipper` contract is a simple contract containing only a single `bool` value
that it can flip from `true` to `false` and vice versa and return the current state.

Below you can see the code using the `ink_lang2` frontend to ink!.

```rust
use ink_core::storage;
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod flipper {
    /// The storage of the flipper contract.
    #[ink(storage)]
    struct Flipper {
        /// The single `bool` value.
        value: storage::Value<bool>,
    }

    impl Flipper {
        /// Instantiates a new Flipper contract and initializes `value` to `init_value`.
        #[ink(constructor)]
        fn new(&mut self, init_value: bool) {
            self.value.set(init_value);
        }

        /// Instantiates a new Flipper contract and initializes `value` to `false` by default.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(false)
        }

        /// Flips `value` from `true` to `false` or vice versa.
        #[ink(message)]
        fn flip(&mut self) {
            *self.value = !self.get();
        }

        /// Returns the current state of `value`.
        #[ink(message)]
        fn get(&self) -> bool {
            *self.value
        }
    }

    /// As in normal Rust code we are able to define tests like below.
    ///
    /// Simply execute `cargo test` in order to test your contract.
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            // Note that `#[ink(constructor)]` functions that above have been
            // defined as `&mut self` can be used as normal Rust constructors
            // in test mode.
            let flipper = Flipper::default();
            assert_eq!(flipper.get(), false);
        }

        #[test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert_eq!(flipper.get(), false);
            flipper.flip();
            assert_eq!(flipper.get(), true);
        }
    }
}
```

## Contribution

Visit our [contribution guidelines](CONTRIBUTING.md) for more information.

## License

The entire code within this repository is licensed under the [GPLv3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
 of our products.
