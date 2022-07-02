# ink! Metadata

The ink! metadata is used to describe a contracts properties in a platform independent
way. To learn more about it see the section in the ink! docs.

## Metadata JSON Schema

The ink! metadata is broken down into two main parts. The outer metadata and the ink!
metadata (`MetadataVersioned`).

We won't go into details about the differences between the two here, but if you're
curious you can read about it in the ink! documentation portal.

In order to validate that a contract's metadata conforms to the format expected by
external tooling we have created a set of [JSON Schemas](https://json-schema.org/). These
schemas describe the properties of the metadata and allows us to validate a contract's
metadata for correctness.

In order to validate the metadata we will be using
[`jsonschema`](https://github.com/Stranger6667/jsonschema-rs),
but an online validator like https://www.jsonschemavalidator.net/
will also work.

First, let's install `jsonschema`:

```bash
cargo install jsonschema
```

Next, we'll build our contract's metadata:

```bash
# At the top level of the ink! repo
cargo +nightly contract build --manifest-path ./examples/flipper/Cargo.toml
```

The generated metadata will be in: `$PATH_TO_INK_REPO/examples/flipper/target/ink/metadata.json`.

Now, to validate our metadata we'll need the schemas. These schemas can be found in this
folder: [`outer-schema.json`](./outer-schema.json) and [`ink-v3-schema.json`](ink-v3-schema.json).

We can then use it to validate our metadata against our schema:

```bash
jsonschema outer-schema.json --instance $PATH_TO_INK_REPO/examples/flipper/target/ink/metadata.json
```

If `metadata.json` is respects our schema we should see the following:

```
metadata.json - VALID
```

Otherwise we will see:

```
metadata.json - INVALID. Errors: ...
```

alongside a list of the errors.

We can do a similar thing for the ink! versioned metadata.

```bash
jq '. | {V3}' $PATH_TO_INK_REPO/examples/flipper/target/ink/metadata.json > ink-v3-metadata.json
jsonschema ink-v3-schema.json --instance ink-v3-metadata.json
```

### Metadata Schema Generation
Right now the schemas are generated using a set of cobbled branches across `scale-info`,
`ink`, and `cargo-contract`.

1. Pull the three repos
2. Checkout the `hc-derive-jsonschema` branch across all of them
3. Change the local paths for `scale-info` and `ink` to match your local folder structure

Now, in your patched version of `cargo-contract` run the following:

```
cargo +nightly run -- contract build --manifest-path $PATH_TO_INK_REPO/examples/flipper/Cargo.toml > schema.json
```

In `src/cmd/metadata.rs` you can change the schema being printed depending on what struct
you pass into the `schemars::schema_for` macro (this is called in the `execute` function).

```rust
let schema = schemars::schema_for!(ContractMetadata);
println!("{}", serde_json::to_string_pretty(&schema).unwrap());

OR

let schema = schemars::schema_for!(ink_metadata::MetadataVersioned);
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```
