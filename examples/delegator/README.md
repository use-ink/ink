# Delegator Smart Contract

The delegator smart contract is our showcase for executing other smart contracts on-chain.

It consists in total of 4 different smart contract:

- Delegator (root): Delegates calls either to the Adder or Subber smart contract
- Adder: Increases a value in the Accumulator smart contract
- Subber: Decreases a value in the Accumulator smart contract
- Accumulator: Owns a simple `i32` value that can be incremented or decremented

In order to test this bundle of smart contracts you need to do the following:

1. Compile all contracts using the `./build-all.sh` script.
   You will receive the respective `.contract` bundles for all the smart contracts in the `target/ink/` folder.
1. Upload the `.contract` bundle of Accumulator, Adder, Subber and the Delegator to the chain.
   While doing so note down their respective code hashes. You will see the contract hash on the success page
   (or success event) after having uploaded the contract to the chain.
1. Instantiate the Delegator smart contract given all of the code hashes and a starting value.
   The Delegator smart contract will take over the work of instantiating the other smart contracts for you.
1. Now you are able to run the operations provided by the Delegator smart contract.
   Namely `delegate` to delegate the call to either the Adder or the Subber to either increase or decrease
   the value stored in the Accumulator smart contract respectively and `switch` to switch the currently
   delegated-to smart contract.
   The initial delegated-to smart contract is the Adder.
