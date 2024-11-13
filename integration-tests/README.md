# ink! Integration Tests

This folder contains a set of example contracts for ink!.
The main purpose of the examples in this folder is for testing ink!
end to end as part of the development/CI process.
Examples are split into two directories: `public`,
which showcases best practices and real-world scenarios,
and `internal`, which contains examples testing specific features
that are distinct from real-world examples.

We have an external repository for ink! contract examples:
[`ink-examples`](https://github.com/use-ink/ink-examples).

All public examples in this folder can also be found
in this repository. The difference is that the contracts in this
folder may already contain unreleased breaking changes.

For end users it's better to take a look at the
[`ink-examples`](https://github.com/use-ink/ink-examples)
repository. The contracts in there are ensured to be running with
the latest published ink! version.

## Documentation

For more information on contract testing and end-to-end tests,
please refer to the [documentation](https://use.ink/basics/contract-testing/#end-to-end-e2e-tests).

## License

The examples in this folder are released into the public domain.
We hope they help you build something great with ink!.

See the [LICENSE file](LICENSE) in this folder for more details.
