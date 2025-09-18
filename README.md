<div align="center">
    <img src="https://raw.githubusercontent.com/use-ink/ink/master/.images/ink-logo-glow.svg" alt="ink!" height="136" />
<h1 align="center">
    Write smart contracts in Rust for <a href="https://github.com/paritytech/polkadot-sdk">Polkadot SDK</a>.
</h1>

[![linux][a1]][a2] [![codecov][c1]][c2] [![loc][e1]][e2]<br/>
[![stack-exchange][s1]][s2] [![telegram][o1]][o2] [![X][p1]][p2]

[a1]: https://github.com/use-ink/ink/workflows/continuous-integration/badge.svg
[a2]: https://github.com/use-ink/ink/actions?query=workflow%3Acontinuous-integration+branch%3Amaster
[c1]: https://codecov.io/gh/use-ink/ink/branch/master/graph/badge.svg
[c2]: https://codecov.io/gh/use-ink/ink/branch/master
[e1]: https://tokei.rs/b1/github/use-ink/ink?category=code
[e2]: https://github.com/Aaronepower/tokei#badges
[f1]: https://img.shields.io/badge/click-blue.svg
[f2]: https://use-ink.github.io/ink/ink_storage
[g1]: https://img.shields.io/badge/click-blue.svg
[g2]: https://use-ink.github.io/ink/ink_env
[i1]: https://img.shields.io/badge/click-blue.svg
[i2]: https://use-ink.github.io/ink/ink_prelude
[j1]: https://img.shields.io/badge/click-blue.svg
[j2]: https://use-ink.github.io/ink/ink
[k1]: https://img.shields.io/badge/matrix-chat-brightgreen.svg?style=flat
[k2]: https://t.me/inkathon/1
[l1]: https://img.shields.io/discord/722223075629727774?style=flat-square&label=discord
[l2]: https://x.com/ink_lang
[s1]: https://img.shields.io/badge/click-white.svg?logo=StackExchange&label=ink!%20Support%20on%20StackExchange&labelColor=white&color=blue
[s2]: https://substrate.stackexchange.com/questions/tagged/ink?tab=Votes
[t1]: https://img.shields.io/badge/click-blue.svg
[t2]: https://use-ink.github.io/ink/ink_e2e
[o1]: https://img.shields.io/badge/click-white.svg?logo=Telegram&label=ink!%20Support%20on%20Telegram&labelColor=white&color=blue
[o2]: https://t.me/inkathon
[p1]: https://img.shields.io/twitter/follow/ink_lang
[p2]: https://x.com/ink_lang

[ink! Website](https://use.ink)&nbsp;&nbsp;•&nbsp;&nbsp;
[Developer Documentation](https://use.ink/docs)

</div>

## Getting started

The best way to start is to check out the [Getting Started](https://use.ink/getting-started/setup)
page in our documentation.

## Architecture & How It Works

The architecture of ink! is documented in the code and in this file: [ARCHITECTURE.md](./ARCHITECTURE.md).

## ink! Macros & Attributes Overview

### Entry Point

In a module annotated with `#[ink::contract]` these attributes are available:

| Attribute | Where Applicable | Description |
|:--|:--|:--|
| `#[ink(storage)]` | On `struct` definitions. | Defines the ink! storage struct. There can only be one ink! storage definition per contract. |
| `#[ink(message)]` | Applicable to methods. | Flags a method for the ink! storage struct as message making it available to the API for calling the contract. |
| `#[ink(constructor)]` | Applicable to method. | Flags a method for the ink! storage struct as constructor making it available to the API for instantiating the contract. |
| `#[ink(event)]` | On `struct` definitions. | Defines an ink! event. A contract can define multiple such ink! events. |
| `#[ink(anonymous)]` | Applicable to ink! events. | Tells the ink! codegen to treat the ink! event as anonymous which omits the event signature as topic upon emitting. Very similar to anonymous events in Solidity. |
| `#[ink(signature_topic = _)]` | Applicable to ink! events. | Specifies custom signature topic of the event that allows to use manually specify shared event definition. |
| `#[ink(topic)]` | Applicable on ink! event field. | Tells the ink! codegen to provide a topic hash for the given field. Every ink! event can only have a limited number of such topic fields. Similar semantics as to indexed event arguments in Solidity. |
| `#[ink(payable)]` | Applicable to ink! messages. | Allows receiving value as part of the call of the ink! message. ink! constructors are implicitly payable. |
| `#[ink(selector = S:u32)]` | Applicable to ink! messages and ink! constructors. | Specifies a concrete dispatch selector for the flagged entity. This allows a contract author to precisely control the selectors of their APIs making it possible to rename their API without breakage. |
| `#[ink(selector = _)]` | Applicable to ink! messages. | Specifies a fallback message that is invoked if no other ink! message matches a selector. |
| `#[ink(namespace = N:string)]` | Applicable to ink! trait implementation blocks. | Changes the resulting selectors of all the ink! messages and ink! constructors within the trait implementation. Allows to disambiguate between trait implementations with overlapping message or constructor names. Use only with great care and consideration! |
| `#[ink(impl)]` | Applicable to ink! implementation blocks. | Tells the ink! codegen that some implementation block shall be granted access to ink! internals even without it containing any ink! messages or ink! constructors. |

See [here](https://use-ink.github.io/ink/ink/attr.contract.html) for a more detailed description of those and also for details on the `#[ink::contract]` macro.

## Developer Documentation

We have [a comprehensive documentation portal](https://use.ink),
but if you are looking for the crate level documentation itself, then these are
the relevant links:

| Crate | Docs | Description                                                                                                                                                                              |
|:--|:--|:-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
`ink` | [![][j1]][j2] | Language features exposed by ink!. See [here](https://use-ink.github.io/ink/ink/attr.contract.html) for a detailed description of attributes which you can use in an `#[ink::contract]`. |
`ink_e2e` | [![][t1]][t2] | End-to-End testing framework for contracts.                                                                                                                                              |
`ink_env` | [![][g1]][g2] | Low-level interface for interacting with the smart contract executor. Contains [the off-chain testing API](https://use-ink.github.io/ink/ink_env/test/index.html) as well.               |
`ink_storage` | [![][f1]][f2] | Data structures available in ink!.                                                                                                                                                       |
`ink_prelude` | [![][i1]][i2] | Common API for no_std and std to access alloc crate types.                                                                                                                               |

## Community Badges

### Normal Design

[![Built with ink!](https://raw.githubusercontent.com/use-ink/ink/master/.images/badge.svg)](https://github.com/use-ink/ink)

```markdown
[![Built with ink!](https://raw.githubusercontent.com/use-ink/ink/master/.images/badge.svg)](https://github.com/use-ink/ink)
```

### Flat Design

[![Built with ink!](https://raw.githubusercontent.com/use-ink/ink/master/.images/badge_flat.svg)](https://github.com/use-ink/ink)

```markdown
[![Built with ink!](https://raw.githubusercontent.com/use-ink/ink/master/.images/badge_flat.svg)](https://github.com/use-ink/ink)
```

## Contributing

Visit our [contribution guidelines](CONTRIBUTING.md) for more information.

Use the scripts provided under `scripts/check-*` directory in order to run checks on either the workspace or all examples. Please do this before pushing work in a PR.

## License

The entire code within this repository is licensed under the [Apache License 2.0](LICENSE).
