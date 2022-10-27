# Upgradeable Contracts

There are three upgradeable contract examples in this folder, they differ
in key properties outlined below.

See [here](https://docs.openzeppelin.com/upgrades-plugins/1.x/proxies) for
more information on proxy patterns.


## [`forward-calls`](https://github.com/paritytech/ink/tree/master/examples/upgradeable-contracts/forward-calls)

* Forwards any call that does not match a selector of itself to another contract.
* The other contract needs to be deployed on-chain.
* State is stored in the storage of the contract to which calls are forwarded.


## [`set-code-hash`](https://github.com/paritytech/ink/tree/master/examples/upgradeable-contracts/set-code-hash)

* Updates the contract code using `set_code_hash`.
  This effectively replaces the code which is executed for the contract address.
* The other contract (`updated-incrementer`) needs to be deployed on-chain.
* State is stored in the storage of the originally instantiated contract (`incrementer`).

## Storage Compatibility

When working on the contract upgradeability, it is important to observe additional rules that are imposed on
the modifications of storage:

Please refer to the section of [Storage Compatibility](https://paritytech.github.io/ink/ink_env/fn.set_code_hash.html) in the ink! crate documentation.
