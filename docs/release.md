# Versioning and releases

English | [中文](release.zh.md)

## Version policy

The latest release is 0.1.1. The Material crates and Gallery share the version in the root `[workspace.package]`. The [public facade and supported integration contract](integration.md#api-and-migration) define compatibility, including documented features, upstream types in public signatures, and supported platform behavior.

- Before `1.0.0`, use a patch bump for backward-compatible fixes and a minor bump for backward-compatible features or breaking changes. A minor bump resets the patch number to zero. If a release includes several kinds of changes, use the highest applicable bump.
- The repository owner decides when to declare the API stable with `1.0.0`. From `1.0.0` onward use [SemVer](https://semver.org/). Incompatible public API changes increment major, compatible features and public deprecations increment minor, and compatible fixes increment patch.
- Record every breaking change and its migration steps, including before `1.0.0`.

## Changelog

Record user-visible changes to the Material crates and Gallery under `Unreleased` in [CHANGELOG.md](../CHANGELOG.md) as they merge. Include features, observable fixes, accessibility and performance improvements, changes to supported platforms or dependencies that affect integration, and migrations. Exclude website and documentation changes, including when they accompany changes to the crates or Gallery. Group entries by affected area and summarize their impact. Credit contributors where applicable. Keep implementation and verification references in the corresponding issue or PR. Use the [GPUI Kit v0.9.1 release](https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1) as the format reference.

When the repository owner selects a version and release cutoff, review the full change range from the previous release tag to that cutoff. Use `v0.0.4`, the last Gallery pre-release, as the baseline for `0.1.0`. Reconcile the entries against the range, move the changes for this release to the new version heading with the publication date, and keep an `Unreleased` heading for subsequent work. The version section is the GitHub Release body. Include a contributor list where applicable and a `Full Changelog` link comparing the previous and new tags.

## Prepare and publish

GitHub Releases provide release notes and GitHub-generated source archives. Run the Gallery from a source checkout with `cargo gallery`.

1. The repository owner selects the version, scope, and timing. Prepare a release PR with the workspace version, exact internal dependency versions, `Cargo.lock`, and changelog for that scope. Update the documented latest version and dependency examples. Add migration instructions for breaking changes.
2. Verify the facade from a downstream crate on the minimum supported Rust toolchain, including affected feature combinations and public dependency types. Require green `CI required` on the release commit on `main`. Apply the [native platform acceptance criteria](verification.md) and record the environments, scenarios, and coverage limits in the release issue or PR.
3. For a release approved for crates.io, package all five library crates together and verify the packaged facade from a downstream crate with default and all features. After native acceptance, publish from the verified checkout in dependency order: `gpui-material-theme`, `gpui-material-foundation`, `gpui-material-components`, `gpui-material-extras`, then `gpui-material`. Confirm each version appears in the registry before publishing its dependents.
4. After the repository owner explicitly approves publication, tag the verified commit as `vX.Y.Z` and publish the matching GitHub Release using the changelog section. Verify the generated source archives and comparison link.
