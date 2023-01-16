# Reentrancy example

This example shows how reentrancy works in ink! smart contracts.
It consists of two smart contracts:

- Main contract, which we initially call
- Fallback contract, which will reenter the main contract

## Workflow

The mechanism of work is the following:

- You call the `inc` method in the main contract and increases the `value` by 1
- It makes a call to `fallback` method in fallback contract, as it has no selector
- `fallback` makes a call back to `inc` method in main contract
- `inc` increases the `value` by 1 again and returns 2 as a result

## Testing

You can run unit-test for this example by running the following command:

```bash
cargo test
```