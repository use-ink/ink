# Monthly Update: Parity Smart Contracts

## What is this?
Hey there reader 👋

This is meant to be a high-level overview of the changes that have taken place as part of
Parity's Web Assembly (Wasm) smart contract efforts. These efforts include the [`ink!`](https://github.com/paritytech/ink)
programming language, the [`pallet-contracts`](https://github.com/paritytech/substrate/tree/master/frame/contracts) execution environment, the [`cargo-contract`](https://github.com/paritytech/cargo-contract)
development tool, and the [Canvas parachain](https://github.com/paritytech/cumulus/tree/master/polkadot-parachains/canvas-kusama). For anything UI related go check out the
[`contract-ui` repo](https://github.com/paritytech/contracts-ui).

If these update are too high-level for you you can always check out the release notes for
the various projects:
- [`ink!`](https://github.com/paritytech/ink/blob/master/RELEASES.md)
- [`pallet-contracts`](https://github.com/paritytech/substrate/blob/master/frame/contracts/CHANGELOG.md)
- [`cargo-contract`](https://github.com/paritytech/cargo-contract/blob/master/CHANGELOG.md)

If you're new here and aren't sure where to get started with ink! check out our
[guided tutorial for beginners](https://docs.substrate.io/tutorials/v3/ink-workshop/pt1/).

This was inspired by the [Polkadot Staking Progress Report](https://gist.github.com/kianenigma/aa835946455b9a3f167821b9d05ba376) ❤️

## March 2022

### ink! 3.0 🦑
The star of the show this month is the release of ink! 3.0.

The team has been working on this release for a while. If you're a developer you're
probably aware of this thanks to the long list of release candidates (`v3.0.0-rcX`) we've
pushed out in preparation for the stable 3.0 release.

The main idea behind ink! 3.0 is that it's just Rust. The syntax is pretty much
identical, and all the tooling you know and love, such as `rust-analyzer` and Clippy,
just works.

Some other included changes worth highlighting:
- Reduced contract sizes for better performance in a parachain context
- Support for cross-contract calls, making it easier to write complex applications
- Trait support, allowing for a more Rusty development experience

### cargo-contract 1.0
Alongside the release of ink! 3.0 we're also releasing `cargo-contract` 1.0.
`cargo-contract` is a handy developer tool for building, testing, and deploying ink!
contracts.

Some notable updates in this release include:
- Support for uploading and calling contracts on a chain which includes `pallet-contract`
- Linting rules for ink! contracts, giving developers warnings about common errors

### Delegate Call
Another update worth pointing out is the addition of `delegate_call`. What this allows is
a way for contracts to forward a call they received to another contract.

One use case for this is implementing a [Proxy Upgrade Pattern](https://docs.openzeppelin.com/upgrades-plugins/1.x/proxies) for smart contracts.

```
User ---- tx ---> Proxy ----------> Implementation_v0
                     |
                      ------------> Implementation_v1
                     |
                      ------------> Implementation_v2
```

This required some work in both `pallet-contracts` and ink!. Shoutout to the team over
at [Supercolony](https://supercolony.net/) for driving the development here!
