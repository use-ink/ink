# ink! code examples

This folder contains a set of example contracts for ink!.

Have a look at the different examples to better understand how to use ink! to build your own Substrate smart contracts.

Besides teaching you how to write smart contracts, this folder is also meant for testing purposes. If you want to contribute to Parity ink!, please run the `check-examples.sh` script in the `./scripts` folder before submitting your PR.

## Preparation

For building the example smart contracts found in this folder you will need to have `cargo-contract` installed.

```
cargo install --git https://github.com/paritytech/ink cargo-contract
```

Add `--force` option to update to the most recent `cargo-contract` version.

## Build example contract and generate the contracts ABI

To build a single example and generate the contracts Wasm file, navigate to the root of the smart contract and run the following command:

`cargo contract build`

To generate the Application Binary Interface (ABI) of the contract, run the following command:

`cargo contract generate-abi`

You should now have a optimized `<contract-name>.wasm` file and a `abi.json` file in the `./target` folder of your contract.

## License

The entire code within this repository is licensed under the [GPLv3](LICENSE). Please [contact us](https://www.parity.io/contact/) if you have questions about the licensing of our products.
