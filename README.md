# pDSL - Parity's DSL for Smart Contracts

**IMPORTANT NOTE:** WORK IN PROGRESS! Do not expect this to be working.

Write WebAssembly based smart contracts in the Rust programming language.

- [`pdsl-core` API documentation](https://robbepop.github.io/pdsl/pdsl_core/index.html).

## Goals

| Core Goals | |
|:-:|:-|
| **Ecosystem** | Allow for easy integration with the current Rust library ecosystem. |
| **Tooling** | Make the great Rust tooling work out-of-the-box for smart contract code. This includes auto-completion, syntax highlighting, code coverage for tests, go-to definitions and other IDE goodies. These should all work without any custom configuration. |
| **Testing** | Make smart contract code as easy to test as using `cargo test`, and make most testing be possible without a blockchain environment at all. |
| **Building** | Make building of smart contract code as easy as using `cargo build`. This does not necessarily mean that you will be able to just type `cargo build`. It might need a separate subcommand. |

| Key Attributes | |
|:-:|:-|
| **Efficient** | Compile smart contract code to machine code that is _at least_ as efficient as if you used the low-level function calls directly. |
| **Robust** | Make it as simple as possible to write code that just does what is expected and as difficult as possible to write incorrect or exploitable code. |
| **Simple** | Smart contract code should be as easy-to-read as possible. |
| **Accessible** | Make it accessible to users by providing excellent documentation and teaching materials. |

## Structure

| Module | Description |
|:-------|:------------|
| `pdsl_core` | The core utilities used to write smart contracts. |
| `pdsl_derive` | A future eDSL based on `pdsl_core` to provide a user friendly interface to writing smart contract code. |
| `pdsl_tests` | Test smart contracts written using pDSL that act as an acceptance test harnish. |
| `examples` | Featuring some smart contracts written for clarity with focus on teaching users how to use pDSL to write their own contracts. |

## Current State

The core abstractions and foundational work has been done in `pdsl_core`.
Even though it is still missing lots of its planned functionality it can already be used to simplify writing smart contracts compared to the bare bones approach.

Using it is still coupled with writing lots of boilerplate code.
This problem is going to be solved in the future by the upcoming `pdsl_derive` module.

## Contribution

The project is currently in experimental phase. We will provide contribution guidelines as soon as we feel that the time for external contributions has come. We are eager to work towards that.

## License

The entire code within this repository is licensed under the [GLP-v3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
