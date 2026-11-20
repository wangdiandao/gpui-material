# Versioning and releases

English | [中文](release.zh.md)

## Version policy

The latest release is 0.1.2. The Material crates and Gallery share the version in the root `[workspace.package]`. The [public facade and supported integration contract](integration.md#api-and-migration) define compatibility, including documented features, upstream types in public signatures, and supported platform behavior.

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
3. Run the manual [Release workflow](../.github/workflows/release.yml) on `main` with `version` set to `X.Y.Z` and `publish: false` (dry-run) to confirm workspace manifests, changelog extraction, and packaging before publication. The verification job packages all six library crates (`gpui-material-theme`, `gpui-material-foundation`, `gpui-material-components`, `gpui-material-extras`, `gpui-material`, and `gpui-material-symbols`) and checks the unpacked archives from independent downstream crates on the workspace MSRV: `gpui-material` with default features and `--all-features`, and `gpui-material-symbols` with and without `embed-assets`.
4. After native acceptance and explicit owner approval, run the [Release workflow](../.github/workflows/release.yml) on `main` with `version` set to `X.Y.Z` and `publish: true`. After the `release` environment approval, the workflow authenticates to crates.io via Trusted Publishing (OIDC), publishes the facade dependencies in order (`gpui-material-theme`, `gpui-material-foundation`, `gpui-material-components`, `gpui-material-extras`), then `gpui-material`, and finally `gpui-material-symbols`, waiting for each version to appear in the crates.io sparse index before publishing dependents and verifying registry checksums. In the same run, it creates the annotated `vX.Y.Z` tag and GitHub Release from the matching changelog section, and verifies the generated source archives and comparison link.

### Release workflow setup

The [Release workflow](../.github/workflows/release.yml) uses GitHub Actions OIDC (`rust-lang/crates-io-auth-action`) with the `release` GitHub Environment instead of a long-lived `CARGO_REGISTRY_TOKEN` secret.

- In GitHub repository settings, configure the `release` environment with required reviewers and restrict deployments to `main`.
- On crates.io, add a Trusted Publishing binding on each of the six crates pointing to `wangdiandao/gpui-material`, workflow filename `release.yml`, and environment `release`.
- crates.io requires a crate to exist before Trusted Publishing can be configured. For a brand-new crate's initial release (such as the first release of `gpui-material-symbols`), publish that crate's first version manually from the same verified commit and configure Trusted Publishing afterward; the workflow detects already-published, non-yanked versions, verifies their checksums, and skips republishing them so retries and partial resumptions stay idempotent.
