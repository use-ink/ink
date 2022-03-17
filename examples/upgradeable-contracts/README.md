# Upgradeable Contracts

There are two upgradeable contract examples in this folder, they differ
in key properties outlined below.

See [here](https://docs.openzeppelin.com/upgrades-plugins/1.x/proxies) for
more information on proxy patterns.


## `forward-calls`

* Forwards any call that does not match a selector of itself to another contract.
* The other contract needs to be deployed on-chain.
* State is stored in the storage of the contract to which calls are forwarded.


## `delegate-call`

* Executes any call that does not match a selector of itself with the code of another contract.
* The other contract does not need to be deployed on-chain. 
* State is stored in the storage of the originally called contract.
