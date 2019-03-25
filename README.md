# pDSL - Parity's DSL for Smart Contracts

| Linux              | Codecov              | Coveralls              | LoC              | Docs (Core)       | Docs (Model)      |
|:------------------:|:--------------------:|:----------------------:|:----------------:|:-----------------:|:-----------------:|
| [![linux][A1]][A2] | [![codecov][C1]][C2] | [![coveralls][D1]][D2] | [![loc][E1]][E2] | [![docs][F1]][F2] | [![docs][G1]][G2] |

[A1]: https://travis-ci.org/Robbepop/pdsl.svg?branch=master
[A2]: https://travis-ci.org/Robbepop/pdsl

[C1]: https://codecov.io/gh/Robbepop/pdsl/branch/master/graph/badge.svg
[C2]: https://codecov.io/gh/Robbepop/pdsl/branch/master

[D1]: https://coveralls.io/repos/github/Robbepop/pdsl/badge.svg?branch=master
[D2]: https://coveralls.io/github/Robbepop/pdsl?branch=master

[E1]: https://tokei.rs/b1/github/Robbepop/pdsl?category=code
[E2]: https://github.com/Aaronepower/tokei#badges

[F1]: https://img.shields.io/badge/docs-nightly-black.svg
[F2]: https://robbepop.github.io/pdsl/pdsl_core/index.html

[G1]: https://img.shields.io/badge/docs-nightly-purple.svg
[G2]: https://robbepop.github.io/pdsl/pdsl_model/index.html

**IMPORTANT NOTE:** WORK IN PROGRESS! Do not expect this to be working.

An [eDSL](https://wiki.haskell.org/Embedded_domain_specific_language) to write WebAssembly based smart contracts in the Rust programming language.

## Example

Below is an example using the eDSL demonstrating a simple Flipper smart contract
that has a boolean state that can be flipped or returned.

```rust
contract! {
    /// Flips its state between `true` and `false`.
    struct Flipper {
        /// The current state of our flag.
        value: storage::Value<bool>,
    }

    impl Deploy for Flipper {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false)
        }
    }

    impl Flipper {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {
            if *self.value {
                *self.value = false
            } else {
                *self.value = true
            }
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            *self.value
        }
    }
}
```

## Goals

### Core Goals

| | |
|:-:|:-|
| **Ecosystem** | Allow for easy integration with the current Rust library ecosystem. |
| **Tooling** | Make the great Rust tooling work out-of-the-box for smart contract code. This includes auto-completion, syntax highlighting, code coverage for tests, go-to definitions and other IDE goodies. These should all work without any custom configuration. |
| **Testing** | Make smart contract code as easy to test as using `cargo test`, and make most testing be possible without a blockchain environment at all. |
| **Building** | Make building of smart contract code as easy as using `cargo build`. This does not necessarily mean that you will be able to just type `cargo build`. It might need a separate subcommand. |

### Key Attributes

| | |
|:-:|:-|
| **Efficient** | Compile smart contract code to machine code that is _at least_ as efficient as if you used the low-level function calls directly. |
| **Robust** | Make it as simple as possible to write code that just does what is expected and as difficult as possible to write incorrect or exploitable code. |
| **Simple** | Smart contract code should be as easy-to-read as possible. |
| **Accessible** | Make it accessible to users by providing excellent documentation and teaching materials. |

## Structure

| Module | Description |
|:-------|:------------|
| `core` | The core utilities used to write smart contracts. |
| `model` | Medium-level abstractions to write smart contracts heavily inspired by [Fleetwood](https://github.com/paritytech/fleetwood). |
| `lang` | The actual eDSL based on `pdsl_core` and `pdsl_model` to provide a user friendly interface to writing smart contract code. |
| `examples` | Features some smart contracts written for clarity with focus on teaching users how to use pDSL to write their own contracts. |

## Contribution

Visit our [contribution guidelines](CONTRIBUTING.md) for more information.

## License

The entire code within this repository is licensed under the [GLP-v3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
