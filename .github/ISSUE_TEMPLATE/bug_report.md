---
name: Bug report
about: Create a report to help us improve
title: ''
labels: ''
assignees: ''

---

**Questions**

- Is this a bug concerning building of ink! smart contracts?
    - If **yes** then you might instead want to file a bug report to ink!'s build tool: https://github.com/paritytech/cargo-contract
- Is this a bug that appears when interacting with smart contracts via the Polkadot JS browser UI?
    - If **yes** then you might instead want to file a bug report to the Polkadot JS Apps project instead: https://github.com/polkadot-js/apps

**FAQ**

- Are you using the nightly Rust channel to compile your smart contract?
- Are you using an up-to-date `cargo-contract`? Check via `cargo contract --version`.
    - If you are unsure please tell us your version noted there.
- Are you using an up-to-date ink! version in your smart contract?
    - You can check this by looking into the `[dependencies]` section of your `Cargo.toml`.
    - If you are unsure please tell us your version noted there.

**Describe the bug**
A clear and concise description of what the bug or odd behavior is.

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Additional context**
Add any other context about the problem here.
