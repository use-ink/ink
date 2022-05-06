# Payment Channel Example

## What is this example about?

It demonstrates a payment channel in ink!.

The implementation is based on [this post](https://programtheblockchain.com/posts/2018/03/02/building-long-lived-payment-channels/)
which has an implementation in Solidity.

## On-Chain Deployment

`ink_env::ecdsa_recover()` uses an [unstable interface](https://github.com/paritytech/substrate/tree/master/frame/contracts#unstable-interfaces) 
of the contracts pallet. The unstable interfaces needs to be enabled for it to work on-chain.
