# Development and verification

[简体中文](development.md) · English

## Contribution workflow and branch naming

- Choose Bug report, Feature request, or Phase deliverable in `.github/ISSUE_TEMPLATE/`. Include a minimal reproduction for bugs, a use case and acceptance criteria for proposals, and dependencies and file ownership for phase tasks.
- Branch from the latest `main` and use `<type>/<issue-number>-<short-topic>`. Write the topic in lowercase English with hyphens. For changes without an associated issue, use `<type>/<short-topic>`, such as `ci/cross-platform-workflow`.
- Choose the prefix by purpose. Use `feat/` for features, `fix/` for fixes, `docs/` for documentation, `test/` for tests, `refactor/` for refactoring, `ci/` for CI, `chore/` for maintenance, and `release/` for releases. For example, use `fix/48-pointer-capture`. Record phase ownership in the issue, PR, and `status.toml`.
- Keep each PR focused on one problem. Use the template to link the issue, list public API changes and migration steps, and provide reproducible verification. For UI changes, identify the Gallery scenario and include before-and-after screenshots or a recording. [GPUI Kit contribution guide](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md)

## Workspace and component development

- Organize production libraries under `crates/` and group components by family. Gallery lives in `apps/gallery/` and provides runnable scenarios that call the production public API. This follows the separation between GPUI Component and Story. [Architecture and dependencies](architecture.en.md) · [Component library](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/component) · [Story](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/story)
- Verify public behavior in the owning crate and exercise the affected variants and interactions in Gallery. Use a minimal entry point for independently reproducible scenarios. Keep isolated probes in `experiments/` and engineering checks and offline generation in `xtask/`.

## Coding conventions

- Find the current specification in the [documentation index](index.en.md) and check the allowed paths in [phase status](../status.toml). Follow the [repair process](phases.en.md#repairs) when revising a frozen contract.
- Update the Chinese page and its adjacent English `.en.md` page in the same PR, and maintain the language links at the top. README, third-party notices, and data indexes provide both languages in one file. GitHub issue and PR templates use English.
- Keep theme in pure Rust and use foundation for GPUI/Base adapters. The application or Base owns business and input state. Material owns presentation and additional composition state. Reuse existing public APIs where possible. [Architecture and dependencies](architecture.en.md)
- Use idiomatic Rust naming and stable semantic identifiers. Explain invariants, lifetimes, and platform differences in comments. Document state ownership, defaults, focus, callbacks, and platform support for public APIs.

## Testing conventions

- Verify the affected behavior for every change. Start with existing tests, build checks, or a minimal reproduction. Add a permanent test when a phase requirement, public behavior, or critical invariant lacks coverage, or when a reproducible defect has a meaningful risk of recurring. Run the relevant existing checks for documentation, formatting, or internal changes with sufficient coverage.
- Verify observable behavior at the lowest effective layer. Extend existing cases where possible and add UI, integration, or native verification when that layer has its own risks. Tests should be repeatable and show the contract they cover and how a regression fails. Verify actual rendering, keyboard and pointer input, semantics, focus, cancellation, and native paths where applicable. [Platforms and quality requirements](verification.en.md)
- Use temporary commands or scripts for one-off fixes, environment diagnosis, migrations, and phase probes. Record significant results and reproduction steps in the relevant document. Keep the smallest permanent assertion when it fills a gap in supported behavior.
- When changing behavior or replacing a probe, remove obsolete cases and helpers after checking for unique coverage and references.

## Upstream contributions

- Confirm general upstream defects with a minimal reproduction and search existing issues and PRs. Follow upstream contribution, testing, licensing, and review rules.
- Keep patches focused on the shared defect and include a minimal verification. Track review, merge, release, and the conditions for removing any local adapter in a project issue.
- Before changing a dependency pin, verify package identity, platform regressions, and the affected scope. Check the version, commit, and source against the upstream release record.

## Development environment

On Windows 11 x64, install [Rustup](https://rustup.rs/) and the Desktop development with C++ workload in Visual Studio Build Tools, including MSVC x64 and the Windows SDK. The root toolchain file pins Rust, rustfmt, and Clippy. The first build downloads the packages in the lockfile.

```powershell
rustc -vV
cargo check --locked
cargo gallery
cargo test --locked -p gpui-material-gallery --features test-support
```

The default workspace build covers the pure theme and engineering tools. Select Gallery with `-p gpui-material-gallery` or `cargo gallery`. The normal Gallery build uses platform defaults. UI integration tests enable `test-support`. Run isolated P1 probes with their own `--manifest-path`.

On macOS, use Apple Silicon and the Xcode command-line tools. Linux CI uses Ubuntu 24.04 x64. See `Install Linux platform dependencies` in the [workflow](../.github/workflows/ci.yml) for the system packages. Both platforms use the same Rust toolchain and Cargo commands.

Check that the Gallery title bar reads “Material Gallery” and the body shows “Window check” and a “Display scale” value matching the current display. Narrow the window and confirm the text remains visible. On Windows, inspect the StatusBar named “Display scale …%” in the UI Automation tree. Close the window from the title bar or with Alt+F4 and confirm the process exits normally. Record the Windows version, target, scaling, screenshot, semantics, and exit result. [Platforms and quality requirements](verification.en.md)

## Local checks

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo xtask check
cargo xtask check origin/main
```

The `check` command validates phase status, freeze evidence and acceptance commit ancestry, local links and heading anchors in tracked Markdown, and GPUI/Base package identities and features. With a base commit, it compares Git paths and checks them against the base phase's scope. A trailing slash denotes a directory. File entries match the full path. Merge freeze evidence into the acceptance commit before recording the freeze in a separate PR. The documentation check runs offline. Use `cargo xtask tokens --check` to check generated tokens or `cargo xtask tokens` to update the Rust tables and index after editing the data. The dependency check resolves normal and test graphs for Windows, macOS, and Linux. [Token data](theme/theme.en.md#token-data)

## Cold and warm build baseline

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/p2-baseline
```

Use a new report directory for each measurement. Prefetch the Cargo registry and source caches. The cold check and cold build each start in an empty target directory. The command then reuses each target directory for the corresponding warm run. The tool records four wall-clock times, HTML timelines, the Git commit, target, and toolchain. The last Gallery build unit includes code generation and linking. Use the normal Gallery features and the development profile without overriding Cargo's job count. Record the OS, CPU, environment, and cache policy when comparing runs. Include the key timings and reproduction command in the acceptance record.

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) runs for PRs targeting `main`, pushes to `main`, and manual dispatches. The job layout follows [GPUI Kit CI](https://github.com/longbridge/gpui-kit/blob/main/.github/workflows/ci.yml).

| Job | Checks |
| --- | --- |
| `Repository checks` | Formatting, diff whitespace, phase scope, documentation links, generated tokens, three-platform dependency graphs, and the default workspace. PRs and main pushes use the corresponding base commit. Manual runs check without a base. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | Clippy, workspace tests, headless Gallery UI tests, and native Gallery builds on Windows Server 2025 x64, macOS 15 ARM64, and Ubuntu 24.04 x64. Other platforms continue if one fails. |
| `CI required` | Passes when the repository checks and all three native jobs succeed. |

Rust build caches are isolated by platform. Successful pushes to `main` save the caches and PRs can restore them. The P1 workflow was disabled on GitHub and removed from the verification branch. [P1 verification record](phases/P01-evidence.en.md) · [Removal commit](https://github.com/wangdiandao/gpui-material/commit/c0596fede13cd4663f28b1eff8bc72ff2f1d5014)

Hosted CI covers builds and automated tests on three platforms. The initial release follows the native acceptance criteria in [platforms and quality requirements](verification.en.md).

The Windows job uploads `gallery-windows-x64-msvc` with the executable, Cargo timeline, and licensing information for native retesting. It runs `cargo build --locked -p gpui-material-gallery --profile gallery-artifact --timings`. This profile disables debug assertions so GPUI can embed DirectX shaders at build time using the Windows SDK's x64 `fxc.exe`. CI sets `GPUI_FXC_PATH` to the SDK compiler. Normal `cargo gallery` builds read the shader sources from the local Cargo cache. Releases follow the applicable delivery phase.

The GitHub ruleset requires PRs to update `main`, be current with the base branch, and resolve review conversations. During migration, `Windows MSVC` remains the required check. Switch to the GitHub Actions check `CI required` after the new workflow merges and that check first succeeds on `main`.

### First cross-platform CI run

On 2026-11-13, a manual run on `ci/cross-platform-workflow` verified `1a28571be2058ca237aaf04a0552f2e1d69aa098`. Repository checks, Linux x64, macOS ARM64, Windows MSVC, and `CI required` passed. The native jobs took 3m 45s, 4m 37s, and 7m 17s respectively. Reproduce with `gh workflow run ci.yml --ref ci/cross-platform-workflow`. The PR path gate also passed locally with `cargo xtask check origin/main`. [Actions run](https://github.com/wangdiandao/gpui-material/actions/runs/43416482503) · [Windows Gallery artifact](https://github.com/wangdiandao/gpui-material/actions/runs/43416482503/artifacts/14032242512)
