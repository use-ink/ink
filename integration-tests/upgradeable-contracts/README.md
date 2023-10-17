# Upgradeable Contracts

There are different ways a contract can be upgraded in ink!

This folder illustrates some of the common and best practices to achieve upgradeability in your contracts.

## [`set-code-hash`](set-code-hash/)

ink! provides an ability to replace the code under the given contract's address.
This is exactly what `set_code_hash()` function does.

However, developers needs to be mindful of storage compatibility.
You can read more about storage compatibility on [use.ink](https://use.ink/basics/upgradeable-contracts#replacing-contract-code-with-set_code_hash)

## [Delegator](delegator/)

Delegator patter is based around a low level cross contract call function `delegate_call`.
It allows a contract to delegate its execution to some on-chain uploaded code.

It is different from a traditional cross-contract call
because the call is delegate to the **code**, not the contract.

Similarly, the storage compatibility issue is also applicable here.
However, there are certain nuances associated with using `delegate_call`.

First of all, as demonstrated in the example, if the delegated code intends to mutate the caller's storage,
a developer needs to be mindful. If the delegated code modifies layout-full storage
(i.e. it contains at least non-`Lazy`, non-`Mapping` field), the `.set_tail_call(true)` flag of `CallFlags` needs to be specified and the storage layouts must match.
This is due to the way ink! execution call stack is operated
(see [Stack Exchange Answer](https://substrate.stackexchange.com/a/3352/3098) for more explanation).

If the delegated code only modifies `Lazy` or `Mapping` field, the keys must be identical and `.set_tail_call(true)` is optional.
This is because `Lazy` and `Mapping` interact with the storage directly instead of loading and flushing storage states.

If your storage is completely layoutless (it only contains `Lazy` and `Mapping` fields), the order of fields and layout do not need to match for the same reason as mentioned above.

