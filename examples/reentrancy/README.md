# Reentrancy example

This example shows how reentrancy works in ink! smart contracts.
It consists of two smart contracts:

- Main contract, which we initially call
- Fallback contract, which will reenter the main contract

## Workflow

The mechanism of work is the following:

- The `inc` method in the main contract is called
- `inc` method increases the `value` by 1
- `inc` method makes a call to `fallback` method in fallback contract, as it has no selector
- `fallback` makes a call back to `inc` method in main contract
- `inc` increases the `value` by 1 again and returns 2 as a result

## Testing

### Unit testing
You can run unit-test for this example by running the following command:

```bash
cargo test
```
### On-chain testing

If you want to test example locally on your node, first you should run `substrate-contracts-node`:

```bash
substrate-contracts-node --dev
```

Then build both contracts using `cargo-contract`:

```bash
cargo contract build --release
```

Then you can use `contracts-ui` to upload and instantiate the contracts:

- Firstly deploy `main_contract`
- Take it's code hash(you can get it from `main_contract.contract` file in `target/ink` folder of main contract)
- Then deploy `fallback_contract` using the code hash of `main_contract` as a constructor argument, it will instantiate `main_contract` inside.
- Also if you want to test both contracts, you need to get address of main contract from fallback contract, and instantiate main_contract with it and
it's metadata, which you can find in `metadata.json` file in `target/ink` folder of main contract.