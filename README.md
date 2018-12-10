# pDSL - Parity's DSL for Smart Contracts

**IMPORTANT NOTE:** THIS IS WORK IN PROGRESS! Do not expect this to be working of final in any way.

Write WebAssembly based smart contracts in Rust operating on Substrate.

## Structure

This repository currently exists of three different sub modules.

- `pdsl_core`: Defines the core utilities and abstractions to declare, implement, test and execute smart contracts.
- `pdsl_derive`: Utilities to simplify writing smart contract code.
- `pdsl_test`: Test framework for the above.

While users can use only `pdsl_core` to write entire smart contracts it is recommended to use this library via `pdsl_derive`.

## Design Goals

- `pdsl_core`
	- Safe and unsafe abstractions
	- Library-only solution
	- Stand alone for writing smart contracts
	- Facility to test and even benchmark contracts off-chain
	- No tricky-intransparent abstractions

- `pdsl_derive`
	- eDSL that makes writing smart contracts using `pdsl_core` easier
	- Based only on `pdsl_core`
	- Empower your smart contracts by Rust's
		- safety guarantees
		- performance characteristics 

## License

The entire code within this repository is licensed under the [GLP-v3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
