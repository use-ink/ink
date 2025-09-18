# Contributing to ink!

TODO review

First of all, thank you for taking your time to contribute to ink!

> [I don't want to contribute, I just have some questions! :S](#I-dont-want-to-contribute-I-just-have-some-questions)

## Table of Contents
* [Code of Conduct](#code-of-conduct)
* [During Development](#during-development)
    * [Commits](#commits)
    * [Checklist](#checklist)
    * [Backwards Compatibility](#backwards-compatibility)
* [Continuous Integration](#continuous-integration)
* [Issues and Pull Requests](#issues-and-pull-requests)
    * [Issues](#issues)
    * [Pull Requests](#pull-requests)
* [Questions](#I-dont-want-to-contribute-I-just-have-some-questions)

## Code of Conduct

We are a friendly and welcoming open source community.
You can find our code of conduct [here](CODE_OF_CONDUCT.adoc).

## During Development

### Commits

Don't be afraid to have a bunch of commits while working on a pull-request. We end up
squashing all of them before merging to the `master` branch anyways.

But please keep your commits small and descriptive. A good rule of thumb is to
bundle semantic changes together in one commit. It makes the review
process easier - which means you get a 🟩 from Github sooner (that's why you're
contributing in the first place anyways, right?)

To help you out here's a [really good post](https://cbea.ms/git-commit/) on how to write good commit
messages.

### Stable vs Nightly Rust

For everything but `rustfmt` we use Rust `stable` together with `export RUSTC_BOOTSTRAP=1`.
You have to export this environment variable locally too! Setting it will enable nightly
features in stable Rust. So it's definitely a hack, but we decided on it since using nightly
in the CI and for tooling came with a lot of maintenance burden.

### Checklist

Below is a checklist for you before doing a pull request.

1. Name your branch with a prefix and a descriptive name in the style of `peter-refactor-storage-implementation`.
1. We won't accept a pull request with `FIXME` or `TODO` comments in it.
   Please try to fix them by yourself and resolve all remaining to-do items.
   If that is not possible then write an issue for each of them and link to the source line and commit with a proper description. For more information go [here](#Issues-&-pull-requests).
1. Document everything properly that you have written, refactored or touched. Especially keeping documentation up-to-date is very important. For larger portions please try to also update the ink! wiki or write a new entry there.
1. Write tests for your code. If your code is hard to test, try to find a design that allows for testing.
1. For performance critical parts we also want additional performance tests.
1. If you implement a fix for a reported bug also include a regression test alongside the fix if possible.
1. When commenting or documenting code make use of proper punctuation.
   This might seem pedantic, but we believe that in essence this is going to improve overall comment and documentation quality.
1. If possible try to sign your commits, e.g. using GPG keys. For more information about this go [here](https://help.github.com/en/articles/signing-commits).

Verify the following locally, otherwise the CI will fail:

1. Is `rustfmt` happy with it ?
    - `cargo +nightly fmt --all`
1. Is `clippy` happy with it?
    - `cargo clippy --all-targets --all-features` -- -D warnings;
1. Are Cargo.toml files formatted properly?
    * `cargo install zepter`
    * `zepter run`
1. Does the code still compile?
    - `cargo check --all-features`
1. Do all the integration tests/examples still compile?
    - `cargo contract check --manifest-path ./integration-tests/.../Cargo.toml`
1. Is the `wasm32` target still compiling?
    - `cargo check --no-default-features --target wasm32-unknown-unknown`
1. Are all the tests passing?
    - `cargo test --all-features --workspace`
1. Are all the tests for the integration tests/examples passing?
    - `cargo test --manifest-path ./integration-tests/.../Cargo.toml`

### Backwards Compatibility

ink! and `pallet-revive` are projects under active development. As the Contracts API
functions evolve to their newer versions we need to introduce them in ink! in a way that
keeps the current *MAJOR* ink! versions compatible with older Substrate nodes which
may not have these new API function.

In order to achieve this, please stick to the following workflow.

Imagine there is a `[seal0] function()` in the Revive pallet API and our
`ink_env::function()` uses its import under the hood. Then some time later we decide
to add a new `[seal1] function()` version to the pallet. In order to introduce it into
ink! and still be able to run contracts (which don't use this particular function) on
older nodes, please do the following:

1. Mark the old `ink_env::function()` (which depends on the imported `[seal0]` function)
   with the `#[deprecated]` attribute. Please, specify the `since` field with the ink!
   version which will introduce the new API. We will leave this function in for backwards
   compatibility purposes. Specifying some additional helpful info in the `note` field
   could also be a good idea.
2. Name the new function (which depends on the `[seal1] function()`) in a descriptive
   way, like `ink_env::function_whats_special()`.
3. Add the `# Compatibility` section to the docs comment of the new function.
4. Never use the new function with existing language features. Only use it with newly
   added functions.

You can have a look at the [PR#1284](https://github.com/use-ink/ink/pull/1284/files#diff-e7cc1cdb3856da1293c785de863703d5961c324aa2018decb0166ea1eb0631e8R191) for a reference of how the described way could be implemented.

## Issues and Pull Requests

Please always respect our [code of conduct](CODE_OF_CONDUCT.adoc) when writing issues and pull requests or taking part in any kind of discussion.

### Issues

You want to write an issue if you have a
- feature request
- bug report
- technical question

about the ink! project.

> Please use GitHub's search functionality to see if the question has already been asked,
the feature has already been proposed or the bug has already been reported.

Make use of the accompanied issue template.

### Pull Requests

Before working on a pull request please make sure that the work has not already been made, e.g. by another pull request solving the same thing.

For complex implementations you are advised to first discuss the feature implementation or bug fix using an issue.

A pull request should be doing or implementing exactly one semantic issue. So for example, when you refactor the code base in a pull request you should not also include code formatting into the same pull request.

It's totally fine to extract changes made in one pull request to multiple pull requests. It makes the review process easier (and hey, more 🟩 for you!).

If you've already opened a pull request, avoid force-pushing any new changes.

If your pull request is a work-in-progress, create it as a Draft pull request.

For a nice list of hints visit this [link][GitHub Perfect Pull Request].

### Reviews

We have GitHub set up in a way that core developers will automatically
be assigned to review your pull request. If you feel there is somebody
missing you're always free to add more people for a review.

For small changes one approval is typically enough to merge code, for
significant or critical changes there should be at least two ✅.

If you made major changes to your pull request since the last approval,
ping them for another approval please.

## I don't want to contribute, I just have some questions

For technical questions about the ink! and all other Polkadot projects, please post your questions to our [Stack Exchange community][Stack-Exchange-Link]. You can also stay tuned by joining our [Element channel][Riot-Smart-Contracts-ink] to be among first ones who gets our announcements.

[Stack-Exchange-Link]: https://substrate.stackexchange.com

[Riot-Smart-Contracts-ink]: https://riot.im/app/#/room/#ink:matrix.parity.io

[GitHub Perfect Pull Request]: https://github.blog/2015-01-21-how-to-write-the-perfect-pull-request/
