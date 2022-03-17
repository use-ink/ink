# Upgradeable Smart Contract

The proxy smart contract forwards any call that does not match a
selector of itself to another, specified contract.

The instantiator of the proxy contract on a blockchain can change
the address to which calls are forwarded.

This allows building upgradeable contracts following the proxy pattern.
Note though that the state is still stored in the contract to which
calls are forwarded.

In order to test it out you need to do the following:

1. Build a contract containing some logic, e.g. our flipper example:
   ```
   cargo +nightly contract build --manifest-path=examples/flipper/Cargo.toml
   ```
   You will receive the respective `flipper.contract` bundle in the `examples/flipper/target/ink/` folder.
1. Build the proxy contract:
   ```
   cd upgradeable-contracts/forward-calls/
   cargo +nightly contract build
   ```
   You will receive the respective `forwards_calls.contract` bundle in the `target/ink/` folder.
1. Upload the `flipper.contract` to the chain.
1. Upload the `forwards_calls.contract` to the chain. During instantiation specify the just instantiated
   `flipper` contract as the `forward_to` parameter.
1. Switch the metadata of the just instantiated `forwards_calls` contract to the metadata of the `flipper`
   contract. In the `polkadot-js` UI this can be done this way:
   1. Click the icon left of the instantiated `forwards_calls` contract to copy the address
      of it into your clipboard.
   1. Click `Add an existing contract`, insert the just copied address, upload the `flipper.contract`
      for the `Contract ABI`.
1. Now you are able to run the operations provided by the `flipper` smart contract via
   the `forwards_calls` contract.

To change the address of the smart contract where calls are forwarded to you would
switch the metadata (i.e. the `Contract ABI`) back to the `forwards_calls` contract
and then invoke the `change_forward_address` message.
