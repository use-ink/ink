# ink! - Parity's ink to write smart contracts

| Linux              | Codecov              | Coveralls              | LoC              | Docs (Core)       | Docs (Model)      |
|:------------------:|:--------------------:|:----------------------:|:----------------:|:-----------------:|:-----------------:|
| [![linux][A1]][A2] | [![codecov][C1]][C2] | [![coveralls][D1]][D2] | [![loc][E1]][E2] | [![docs][F1]][F2] | [![docs][G1]][G2] |

[A1]: https://travis-ci.org/paritytech/ink.svg?branch=master
[A2]: https://travis-ci.org/paritytech/ink

[C1]: https://codecov.io/gh/paritytech/ink/branch/master/graph/badge.svg
[C2]: https://codecov.io/gh/paritytech/ink/branch/master

[D1]: https://coveralls.io/repos/github/paritytech/ink/badge.svg?branch=master
[D2]: https://coveralls.io/github/paritytech/ink?branch=master

[E1]: https://tokei.rs/b1/github/paritytech/ink?category=code
[E2]: https://github.com/Aaronepower/tokei#badges

[F1]: https://img.shields.io/badge/docs-nightly-black.svg
[F2]: https://paritytech.github.io/ink/ink_core/index.html

[G1]: https://img.shields.io/badge/docs-nightly-purple.svg
[G2]: https://paritytech.github.io/ink/ink_model/index.html

**IMPORTANT NOTE:** WORK IN PROGRESS! Do not expect this to be working.

ink is an [eDSL](https://wiki.haskell.org/Embedded_domain_specific_language) to write WebAssembly based smart contracts using the Rust programming language.

## Example

Below is an example using the eDSL demonstrating a simple Flipper smart contract
that has a boolean state that can be flipped or returned.

```rust
contract! {
    /// Specify concrete implementation of contract environment types
    #![env = ink_core::env::DefaultSrmlTypes]
 
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
            *self.value = !*self.value;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            *self.value
        }
    }
}
```

## Documentation

- User
    - Wiki: [link](https://github.com/paritytech/ink/wiki)
- Developer
    - [`core`][F2]: Developer documentation for the core abstractions
        - Storage allocators, SRML environment definitions
        - Offchain test environment
        - Utilities for smart contracts like collections
    - [`model`][G2]: Developer documentation for the model abstractions
        - Virtual model of a smart contract
        - Contains smart contract ABI dispatch
        - Used to build an actual smart contract eDSL on

## Goals

### Core Goals

| | |
|:-:|:-|
| **Ecosystem** | Easy integration with the Rust ecosystem. |
| **Tooling** | Rust tooling works out-of-the-box for smart contract code. This includes auto-completion, syntax highlighting, code coverage for tests, go-to definitions and other IDE goodies. |
| **Testing** | Easy to build, test, deploy and run. |
| **Development** | Development can be done entirely off-chain to speed up the process. |

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
| `cli` | A minimalist tool to setup a smart contract project easily. |
| `core` | The core utilities used to write smart contracts. |
| `model` | Medium-level abstractions to write smart contracts heavily inspired by [Fleetwood](https://github.com/paritytech/fleetwood). |
| `lang` | The actual eDSL based on `ink_core` and `ink_model` to provide a user friendly interface to writing smart contract code. |
| `examples` | Features some smart contracts written for clarity with focus on teaching users how to use pDSL to write their own contracts. |

## Contribution

Visit our [contribution guidelines](CONTRIBUTING.md) for more information.

## License

The entire code within this repository is licensed under the [GPLv3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
 of our products.
