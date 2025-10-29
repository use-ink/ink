# How to release

Generally, we bump and publish the versions of all crates in this
mono-repository in lockstep.
It does not matter if it's a `PATCH`, `MINOR`, `MAJOR` release.

Reasons for doing it this way:
* Keeps our [`RELEASES.md`](https://github.com/use-ink/ink/blob/master/RELEASES.md)
  simple and easy to grasp.<br>
  We can still bundle all changes there together and don't have to go to a
  more fine-grained crate level.
* Easier to follow reported issues.<br>
  All crates are closely tied together anyway and if someone reports an issue with
  e.g. `3.1` we know what they mean.
* Easier to keep the workshop/documentation/playground/user interfaces in sync.
* Easier for users to follow changes.<br>
  Those will all be listed in one section of the changelog, bundled together,
  released at the same time.
* Third party tooling like dependabot can easily extract the changelog.

## Examples

Examples (so anything in the `integration-tests/` folder) are a bit of a special case
in our release pipeline since they are considered as ink! internals and not part of
the library per-say.

What this means is that any changes to the examples (breaking or not) should only be
considered a `PATCH` level change. Additionally, they should not be published to
crates.io.


## Checklist

We'll be using [`cargo-release`](https://github.com/crate-ci/cargo-release) to release
ink!. There are still a few manual steps though, and we hope to make this more streamlined
in the future.

1. Create a new feature branch off `master`.
1. Copy the release notes that appear in the [`CHANGELOG.md`](https://github.com/use-ink/ink/blob/master/CHANGELOG.md)
   into the PR description.
   - This will cause the individual PRs to be linked to the release in which they are
     included.
1. Bump the version in all TOML files to the new version.
    ```
    find . -type f -name *.toml -exec sed -i -e 's/$OLD_VERSION/$NEW_VERSION/g' {} \;
    ```
1. Make sure you've moved the changes in the `CHANGELOG.md` from `[Unreleased]` into a new
   section for the release.
1. Check that all notable PRs since the last release are now in the new release section,
   in case the `[Unreleased]` section was incomplete.<br>
   - Notable changes are changes that affect users in some way. This means that something
     like a change to our CI pipeline is likely not notable and should not be included.
1. Make sure you've merged the latest `master` into your branch.
1. Open a release PR
    - Wait for approvals from Core team members.
    - Ensure the entire CI pipeline is green.
1. Before releasing, take note of the release order:
    - `ink_e2e` is dependent on the `cargo-contract` crates, which are dependent on the `ink` "core" crates.
    - Therefore, first release all `ink` crates except `ink_e2e`.
    - Release `cargo-contract` crates.
    - Request update of `drink` client, which depends on the `cargo-contract` crates.
    - Release `ink_e2e`.
1. The `ink_sandbox` crate depends on a git commit of `polkadot-sdk`, hence it
   currently cannot be published to crates.io.
1. Do a dry run:
   ```bash
   fd Cargo.toml crates/ | \
   grep -v e2e | \
   grep -v sandbox | \
   xargs -n1 cargo no-dev-deps publish --allow-dirty --dry-run --manifest-path
   ```
   This command ignores the `e2e` and `sandbox` folder. It uses [`no-dev-deps`](https://crates.io/crates/cargo-no-dev-deps)
   for publishing, so that the `dev-dependencies` of ink! are ignored for publishing.
   They are not needed and due to a cycle it's also not possible to publish with them.
   Ignoring `dev-dependencies` is also why `--allow-dirty` is necessary.
4. Following a successful dry run, we can now publish to crates.io.
   - This will be done from the release branch itself.
   - This is because it is possible for the dry run to succeed but for the actual publish
     to fail and require some changes. So before running the next step:
     - Ensure there have been no new commits to `master` which are not included in this
       branch.
     - Notify core team members in the Element channel that no PRs should be merged to
       `master` during the release.
     - The above is to ensure that the bundled code pushed to crates.io is the same as
       the tagged release on GitHub.
5. Publish with
   ```bash
   fd Cargo.toml crates/ | \
   grep -v e2e | \
   grep -v sandbox | \
   xargs -n1 cargo no-dev-deps publish --allow-dirty --manifest-path
   ```
6. Following a successful release from the release PR branch, now the PR can be merged
   into `master`
    - Once merged, notify core team members in the Element channel that PRs can be merged
      again into `master`
7. Replace `vX.X.X` with the new version in the following command and then execute it:
    ```
    git tag -s vX.X.X && git push origin vX.X.X
    ```
    - Ensure your tag is signed with an offline GPG key!
    - Alternatively, the `Create release` GitHub UI below allows creating this tag when
      creating the release.
8. Update the [`ink-examples`](https://github.com/use-ink/ink-examples) repository with
   the content of `integration-tests` (minus `mother`, `lang-err-integration-tests` and
   `mapping-integration-tests`). Besides copying those folders over, the only change you
   need to do manually is to switch the dependencies in the `Cargo.toml`'s to use the
   published version of your release.
9.  Create a GitHub release for this tag. In the [tag overview](https://github.com/use-ink/ink/tags)
   you'll see your new tag appear. Click the `…` on the right of the tag and then
   `Create release`.
10. Paste the release notes that appear in the [`CHANGELOG.md`](https://github.com/use-ink/ink/blob/master/CHANGELOG.md)
   there. The title of the release should be `vX.X.X`.
11. Post an announcement to the Telegram channel: https://t.me/inkathon/1
12. Update the documentation with the new release:
    - In setup page: https://github.com/use-ink/ink-docs/blob/master/docs/getting-started/setup.md#rust--cargo
    - In contract template page: https://github.com/use-ink/ink-docs/blob/master/docs/basics/contract-template.md#cargotoml
