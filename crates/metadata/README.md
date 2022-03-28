# ink-metadata

Contract metadata is required in order for UIs and clients to interact with contracts. This crate defines the ABI for 
a contract's constructors and messages, as well as the layout of its storage.

Contracts written in `ink!` are typically built using [`cargo-contract`](https://github.com/paritytech/cargo-contract)
which will produce a `metadata.json` and `<contract-name>.contract` file. Both contain a json representation 
of the metadata defined here, the latter additionally contains the Wasm code itself.

The top level structure of the serialized metadata


The `spec` defines the ABI of a contract's 

