# How to release

Generally, we bump and publish the versions of all crates in this
mono-repository in lockstep.
It does not matter if it's a `PATCH`, `MINOR`, `MAJOR` release.

Reasons for doing it this way:
* Keeps our [`RELEASES.md`](https://github.com/paritytech/ink/blob/master/RELEASES.md)
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

Examples (so anything in the `examples/` folder) are a bit of a special case in our
release pipeline since they are considered as ink! internals and not part of the library
per-say.

What this means is that any changes to the examples (breaking or not) should only be
considered a `PATCH` level change. Additionally, they should not be published to
crates.io.


## Checklist

1. Create a new feature branch off `master`.
1. Bump the version in all the TOML files to the new version.
    ```
    find . -type f -name *.toml -exec sed -i -e 's/$OLD_VERSION/$NEW_VERSION/g' {} \;
    ```
1. Make sure you've moved the changes in the `RELEASES.md` from `[Unreleased]` into a new
   section for the release.
1. Check that all notable PRs since the last release are now in the new release section,
   in case the `[Unreleased]` section was incomplete.<br>
   Notable changes are changes that affect users in some way. This means that something
   like a change to our CI pipeline is likely not notable and should not be included.
1. Make sure you've merged the latest `master` into your branch.
1. Open a release PR
    1. Wait for approvals from Core team members
    1. Ensure the entire CI pipeline is green
1. Execute `cargo unleash em-dragons --dry-run` in the ink! repository.
1. If successful, execute `cargo unleash em-dragons`.<br>
   In some versions of `cargo-unleash` there is a bug during publishing.
   If this is the case you need to do it manually, by executing `cargo publish --allow-dirty`
   for the individual crates. The `--allow-dirty` is necessary because `cargo-unleash`
   will have removed all `[dev-dependencies]`.
1. Merge the PR into `master` if everything went fine.
   We do this _after_ publishing since the publishing step can fail and we want to ensure
   that what gets merged into `master` is what actually got published.
1. Replace `vX.X.X` with the new version in the following command and then execute it:
    ```
    git checkout master && git pull && git tag vX.X.X && git push origin vX.X.X
    ```
1. Create a GitHub release for this tag. In the [tag overview](https://github.com/paritytech/ink/tags)
   you'll see your new tag appear. Click the `…` on the right of the tag and then `Create release`.
1. Paste the release notes that appear in the [`RELEASES.md`](https://github.com/paritytech/ink/blob/master/RELEASES.md)
   there. The title of the release should be `vX.X.X`.
1. Post an announcement to those Element channels:
    * [Smart Contracts & Parity ink!](https://matrix.to/#/#ink:matrix.parity.io)
    * [ink! Announcements](https://matrix.to/#/#ink-announcements:matrix.parity.io)
