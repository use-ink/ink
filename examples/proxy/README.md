# Proxy Smart Contract

The proxy smart contract delegates any call that does not match a
selector of itself to another, specified contract.

The instantiator of the proxy contract on a blockchain can change
the code to which calls are forwarded.

This allows building upgradable contracts following the proxy pattern.

In order to test it out you need to do the following:

1. Build a contract containing some logic, e.g. our upgradeable flipper example:
   ```
   cargo +nightly contract build --manifest-path=examples/proxy/upgradeable_flipper/Cargo.toml
   ```
   You will receive the respective `upgradeable_flipper.contract` bundle in the `examples/proxy/upgradeable_flipper/target/ink/` folder.

   In order to perform migrations and have proxy working for contracts with different storage
   layouts, we use the [`Upgradeable`](upgradeable_flipper/upgradeable.rs) type wrapper, which ensures
   that we write different fields of desired struct to different storage locations, while also tracking
   the initialization status (e.g., we uploaded the code on chain, but haven't called the constructor).

2. Build the proxy contract:
   ```
   cargo +nightly contract build --manifest-path=examples/proxy/Cargo.toml
   ```
   You will receive the respective `proxy.contract` bundle in the `examples/proxy/target/ink/` folder.
3. Upload the `upgradeable_flipper.contract` to the chain.
4. Upload the `proxy.contract` to the chain. During instantiation specify the just instantiated
   `upgradeable_flipper` contract as the `delegate_to` parameter.
5. Switch the metadata of the just instantiated `proxy` contract to the metadata of the `upgradeable_flipper`
   contract. In the `polkadot-js` UI this can be done this way:
   1. Click the icon left of the instantiated `proxy` contract to copy the address
      of it into your clipboard.
   1. Click `Add an existing contract`, insert the just copied address, upload the `upgradeable_flipper.contract`
      for the `Contract ABI`.
6. Now you are able to run the operations provided by the `upgradeable_flipper` smart contract via
   the `proxy` contract.

To change the address of the smart contract where calls are forwarded to you would
switch the metadata (i.e. the `Contract ABI`) back to the `proxy` contract
and then invoke the `change_delegate_code` message.
