# Function `instantiate_contract`

```rust
pub fn instantiate_contract<E, ContractRef, Args, Salt, R>(
    params: &CreateParams<E, ContractRef, Args, Salt, R>
) -> Result<ConstructorResult<<R as ConstructorReturnType<ContractRef>>::Output>>
where
    E: Environment,
    ContractRef: FromAccountId<E>,
    Args: Encode,
    Salt: AsRef<[u8]>,
    R: ConstructorReturnType<ContractRef>,
```

## Description

`instantiate_contract` is a low level way to instantiate another smart contract.

## Related ink! functions

- [instantiate_contract](https://paritytech.github.io/ink/ink_env/fn.instantiate_contract.html)

## Test case (original status)

### Comparison Integration vs E2E

The End-to-End test works correctly since it invokes successfully the call to the second contract. In Integration did not work since it's [unimplemented](https://github.com/paritytech/ink/blob/c2af39883aab48c71dc09dac5d06583f2e84dc54/crates/env/src/engine/off_chain/impls.rs#L464).

| \#  | Test                                                            | Integration | E2E |
| --- | --------------------------------------------------------------- | :---------: | :-: |
| 1   | Attempts to instantiate a contract.                             |     ❌      | ✅  |

### Result

The implementation of instantiate_contract() is somewhat tied to that of code_hash and own_code_hash(). The strategy picked for one will condition the solution for the other. The simpler of the two may take up to roughly 15 days. There are some things to work out, such as how to call the required function, that add some uncertainty to the estimate.

## Status after implementation

| \#  | Test                                                            | Integration | E2E |
| --- | --------------------------------------------------------------- | :---------: | :-: |
| 1   | Attempts to instantiate a contract.                             |     ✅      | ✅  |
