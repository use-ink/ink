# Upgradeable Contracts

There are different ways a contract can be upgraded in ink!

This folder illustrates some of the common and best practices to achieve upgradeability in your contracts.

## [`set-code-hash`](set-code-hash/)

ink! provides an ability to replace the code under the given contract's address.
This is exactly what `set_code_hash()` function does.

However, developers needs to be mindful of storage compatibility.
You can read more about storage compatibility on [use.ink](https://use.ink/basics/upgradeable-contracts#replacing-contract-code-with-set_code_hash)

## [`set-code-hash`](set-code-hash-migration/)

When upgrading a contract, the new code may have a different storage layout. This example illustrates a method to 
migrate the storage from the old layout to the new layout. It does so by using an intermediate `migration` contract
which performs the storage upgrade. The workflow is as follows:


1. Upload a `migration` contract with a message `migrate` which performs the storage migration.
2. Set code hash to the `migration` contract.
3. Upload the upgraded version of the original contract.
4. Call `migrate` on the `migration` contract, passing the code hash of the new updated incrementer contract from `3.` 
This must happen as a single message, because following the storage migration, the contract will not be able to be 
called again, since it will fail to load the migrated storage.


## [Delegator](delegator/)

The Delegator pattern is based around the low level host function `delegate_call`.
It allows a contract to delegate its execution to some on-chain uploaded code.

It is different from a traditional cross-contract call
because the call is delegate to the **code**, not the contract.

Similarly, the storage compatibility issue is also applicable here.
However, there are certain nuances associated with using `delegate_call`.

First of all, as demonstrated in the example, if the delegated code intends to mutate the caller's storage,
a developer needs to be mindful. If the delegated code modifies layout-full storage
(i.e. it contains at least non-`Lazy`, non-`Mapping` field), the `CallFlags::TAIL_CALL` flag needs to be specified and the storage layouts must match.
This is due to the way ink! execution call stack is operated
(see [Stack Exchange Answer](https://substrate.stackexchange.com/a/3352/3098) for more explanation).

If the delegated code only modifies `Lazy` or `Mapping` field, the keys must be identical and `CallFlags::TAIL_CALL` is optional.
This is because `Lazy` and `Mapping` interact with the storage directly instead of loading and flushing storage states.

If your storage is completely layoutless (it only contains `Lazy` and `Mapping` fields), the order of fields and layout do not need to match for the same reason as mentioned above.

### Delegate dependency locks

The `delegator` contract depends upon the contract code to which it delegates. Since code
can be deleted by anybody if there are no instances of the contract on the chain, this 
would break the `delegator` contract. To prevent this, the `delegator` contract utilizes
the `lock_delegate_dependency` and `unlock_delegate_dependency` host functions. Calling
`lock_delegate_dependency` will prevent the code at the given hash from being deleted, 
until `unlock_delegate_dependency` is called from within the `delegator` contract instance.
Note that these two methods can be called by anybody executing the contract, so it is the
responsibility of the contract developer to ensure correct access control.

