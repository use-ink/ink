# Delegator Smart Contract

The delegator smart contract is our showcase for executing other smart contracts on-chain.

It consists in total of 4 different smart contract:

- Delegator (root): Delegates calls either to the Adder or Subber smart contract
- Adder: Increases a value in the Accumulator smart contract
- Subber: Decreases a value in the Accumulator smart contract
- Accumulator: Owns a simple `i32` value that can be incremented or decremented

In order to test this bundle of smart contracts you need to execute the
following steps.

You can upload the contracts using our [Canvas UI](https://paritytech.github.io/canvas-ui/#/).
If you want to test it locally, our [Canvas Node](https://paritytech.github.io/ink-docs/getting-started/setup#installing-the-canvas-node)
is an easy way to get a local smart contract chain running.

1. Compile all contracts using the `./build-all.sh` script.
   You will receive the respective `.contract` bundles for all the smart contracts in the `target/ink/` folder:
   * `target/ink/delegator.contract`
   * `target/ink/adder/adder.contract`
   * `target/ink/subber/subber.contract`
   * `target/ink/accumulator/accumulator.contract`
1. Upload the `.contract` bundle of Accumulator, Adder and Subber to the chain.
1. Note down the respective code hashes of the uploaded contracts. You can
   copy the contract hashes [from the page of uploaded contracts](https://paritytech.github.io/canvas-ui/#/instantiate):<br/>
   [<img src="./.images/code-hashes.png" width="600" alt="Code Hashes Overview" />](https://paritytech.github.io/canvas-ui/#/instantiate)
1. Instantiate the Delegator smart contract given all of the code hashes and a starting value.
   Make sure the endowment is big enough (if you're using our `canvas-node` it's `1000000`).
   The Delegator smart contract will take over the work of instantiating the other smart contracts for you.
1. Now you are able to run the operations provided by the Delegator smart contract.
   Namely `delegate` to delegate the call to either the Adder or the Subber to either increase or decrease
   the value stored in the Accumulator smart contract respectively and `switch` to switch the currently
   delegated-to smart contract.
   The initial delegated-to smart contract is the Adder.

   <br/>

   > __Note:__<br/>
   > Depending on your Substrate version you might encounter [a bug with the pre-filled gas estimation of the UI](https://github.com/paritytech/substrate/issues/8693)
   > and get the error `ExtrinsicFailed: OutOfGas`.
   > As a workaround set the maximum allowed gas manually (e.g. to 5000).
