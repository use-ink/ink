# Contributing to ink!

First of all, thank you for taking your time to contribute to ink!

> [I don't want to contribute, I just have some questions! :S](#I-dont-want-to-contribute-I-just-have-some-questions)

## Code of Conduct

We are a friendly and welcoming open source community.
You can find our code of conduct [here](CODE_OF_CONDUCT.md).

## During Development

### Commits

Please follow our simple commit style guidelines.

1. Prepend all commits with a topic.
   Available topics are:
    - `[storage]` For changes to `ink_storage`.
    - `[env]` For changes to `ink_env`.
    - `[lang]` For changes to `ink_lang`.
    - `[allocator]` For changes to `ink_allocator`.
    - `[metadata]` For changes to `ink_metadata`.
    - `[prelude]` For changes to `ink_prelude`.
    - `[primitives]` For changes to `ink_primitives`.
    - `[ci]` For changes to the continuous integration.
    - `[chores]` For general non-technical changes.
    - `[examples]` For changes to the examples.

For other purposes and sections please try to find a good fit for the topic.

### Checklist

Below is a checklist for you before doing a pull request.
Following these will ensure that your pull request is going to be accepted.

1. Run `rustfmt` automatically.
1. Run `clippy` on your changes.
1. Run tests via `cargo test --release` for off-chain testing.
1. For critical parts perform some manual on-chain tests.
1. Build the code and run tests also for the `wasm32` target.
1. Try to run some examples and see if they are still working correctly.
1. Sometimes clippy lints can be unfortunate or even buggy and it can be very hard to fix those.
  In these situations you may skip the clippy lint with `#[clippy::skip]`, however,
  note that this always requires a good rational as a side-comment and probably link to the eventual clippy bug.
1. **Important** Strive for simple, clean and concise code.
  If your code is very complex - because it is trying to accomplish complex things - try to think about how another aproach or design could simplify it.
1. We won't accept a pull request with `FIXME` or `TODO` comments in it.
   Please try to fix them by yourself and resolve all remaining to-do items.
   If that is not possible then write an issue for each of them and link to the source line and commit with a proper description. For more information go [here](#Issues-&-pull-requests).
1. Document everything properly that you have written, refactored or touched. Especially keeping documentation up-to-date is very important. For larger portions please try to also update the ink! wiki or write a new entry there.
1. Write tests for your code. If your code is hard to test, try to find a design that allows for testability.
1. If needed also update the [`README`](README.md).
1. For performance critical parts we also want additional performance tests.
1. If you implement a fix for a reported bug also include a regression test alongside the fix if possible.
1. When commenting or documenting code make use of proper punctuation.
   This might seem pedantic but we believe that in essence this is going to improve overall comment and documentation quality.
1. If possible try to sign your commits, e.g. using GPG keys. For more information about this go [here](https://help.github.com/en/articles/signing-commits).

## Continuous Integration

Our continuous integration (CI) will check for the following properties of all changes.

1. Does the code still compile?
1. Is `rustfmt` happy with it?
1. Is `clippy` and all its enabled lints happy with it?
1. Are tests in debug mode passing?
1. ~~Are tests in release mode passing?~~
1. Is the test code coverage increasing or at least stable?
1. Is the `wasm-32` target still compiling?

Only if our very demanding CI is happy with your pull request we will eventually merge it.
Exceptions confirm the rule!

## Issues & Pull Requests

Please always respect our [code of conduct](CODE_OF_CONDUCT.md) when writing issues and pull requests or taking part in any kind of discussion.

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

A pull request should be doing or implementing exactly one semantic issue. So for example, when you refactor the code base in a pull request you should not also include code formattings into the same pull request.

For a nice list of hints visit this [link][GitHub Perfect Pull Reqest].

## I don't want to contribute, I just have some questions

For questions about the ink! project, about Parity Technologies or general technical related questions you are welcome to contact us via [Riot][Riot-Smart-Contracts-ink]. For technical questions specifically about the ink! and its sub-projects you may also file an issue. For more information about filing issues go [here](#Issues-&-pull-requests).

[Riot-Smart-Contracts-ink]: https://riot.im/app/#/room/#ink:matrix.parity.io

[GitHub Perfect Pull Reqest]: https://github.blog/2015-01-21-how-to-write-the-perfect-pull-request/
