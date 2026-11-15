# Development and verification

English | [中文](development.zh.md)

<a id="贡献流程与分支命名"></a>

## Contribution workflow and branch naming

- Follow the [contribution guide](../CONTRIBUTING.md) for Issue templates, design changes, and review.
- Branch from the latest `main` and use `<type>/<short-topic>`, such as `ci/cross-platform-workflow`. Write the topic in lowercase English with hyphens. Link the issue in the PR description.
- Choose the prefix by purpose. Use `feat/` for features, `fix/` for fixes, `docs/` for documentation, `test/` for tests, `refactor/` for refactoring, `ci/` for CI, `chore/` for maintenance, and `release/` for releases. For example, use `fix/pointer-capture`.

<a id="workspace-与组件开发"></a>

## Workspace and component development

- Organize production libraries under `crates/` and group components by family. Gallery lives in `apps/gallery/` and provides runnable scenarios that call the production public API. This follows the separation between GPUI Component and Story. [Architecture and dependencies](architecture.md) · [Component library](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/component) · [Story](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/story)
- Verify public behavior in the owning crate and exercise the affected variants and interactions in Gallery. Use a minimal entry point for independently reproducible scenarios. Keep isolated probes in `experiments/` and engineering checks and offline generation in `xtask/`.

<a id="编码规范"></a>

## Coding conventions

- Use the [documentation index](index.md) to find the current specification. Follow the [pinned design and review rules](../CONTRIBUTING.md#design-specification) when changing Material presentation.
- Keep the English `foo.md`, Chinese `foo.zh.md`, and section fingerprints in `foo.i18n.yaml` together. Link the counterpart immediately below the title and use the current page's language for links to other paired pages. Update both pages in the same PR and run `cargo xtask docs --write path/to/foo.md` to record the reviewed pair. `cargo xtask check` verifies pairing, language links, heading depth, code samples, and fingerprints. GitHub issue and PR templates use English. [DeepSeek Harness documentation conventions](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/i18n/README.md)
- Keep theme in pure Rust and use foundation for GPUI/Base adapters. The application or Base owns business and input state. Material owns presentation and additional composition state. Reuse existing public APIs where possible. [Architecture and dependencies](architecture.md)
- Use idiomatic Rust naming and stable semantic identifiers. Explain invariants, lifetimes, and platform differences in comments. Document state ownership, defaults, focus, callbacks, and platform support for public APIs.

<a id="测试规范"></a>

## Testing conventions

- Verify the affected behavior for every change. Start with existing tests, build checks, or a minimal reproduction. Add a permanent test when a specification requirement, public behavior, or critical invariant lacks coverage, or when a reproducible defect has a meaningful risk of recurring. Run the relevant existing checks for documentation, formatting, or internal changes with sufficient coverage.
- Verify observable behavior at the lowest effective layer. Extend existing cases where possible and add UI, integration, or native verification when that layer has its own risks. Tests should be repeatable and show the contract they cover and how a regression fails. Verify actual rendering, keyboard and pointer input, semantics, focus, cancellation, and native paths where applicable. [Platforms and quality requirements](verification.md)
- Use temporary commands or scripts for one-off fixes, environment diagnosis, migrations, and capability probes. Record significant results and reproduction steps in the issue or PR. Keep the smallest permanent assertion when it fills a gap in supported behavior.
- When changing behavior or replacing a probe, remove obsolete cases and helpers after checking for unique coverage and references.

<a id="上游贡献"></a>

## Upstream contributions

- Confirm general upstream defects with a minimal reproduction and search existing issues and PRs. Follow upstream contribution, testing, licensing, and review rules.
- Keep patches focused on the shared defect and include a minimal verification. Track review, merge, release, and the conditions for removing any local adapter in a project issue.
- Before changing a dependency pin, verify package identity, platform regressions, and the affected scope. Check the version, commit, and source against the upstream release record.

<a id="开发环境"></a>

## Development environment

On Windows 11 x64, install [Rustup](https://rustup.rs/) and the Desktop development with C++ workload in Visual Studio Build Tools, including MSVC x64 and the Windows SDK. The root toolchain file pins Rust, rustfmt, and Clippy. The first build downloads the packages in the lockfile.

```powershell
rustc -vV
cargo check --locked
cargo gallery
cargo test --locked -p gpui-material-gallery --features test-support
```

The default workspace build covers the pure theme and engineering tools. Select Gallery with `-p gpui-material-gallery` or `cargo gallery`. The normal Gallery build uses platform defaults. UI integration tests enable `test-support`.

On macOS, use Apple Silicon and the Xcode command-line tools. Linux CI uses Ubuntu 24.04 x64. See `Install Linux platform dependencies` in the [workflow](../.github/workflows/ci.yml) for the system packages. Both platforms use the same Rust toolchain and Cargo commands.

Open a component story in “Material Gallery”, resize into the compact layout, and verify you can reopen the catalog and select a story. Check that “Display scale” matches the current display. On Windows, inspect the StatusBar named “Display scale …%” in the UI Automation tree. Close the window from the title bar or with Alt+F4 and confirm the process exits normally. Record the environment and results in the PR. [Platforms and quality requirements](verification.md)

<a id="独立探针"></a>

## Standalone probes

The host probe checks pinned GPUI and Base behavior directly, including real-window startup, native capture loss, text input, semantics, motion, drawing, and overlays. Select an entry point with `--bin`. Run the probe with its own manifest and lockfile. Point `CARGO_TARGET_DIR` to the main checkout's `target` directory for incremental builds.

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo run --locked --manifest-path experiments/gpui-host/Cargo.toml --bin window
cargo test --locked --manifest-path experiments/gpui-host/Cargo.toml --features test-support
```

<a id="本地检查"></a>

## Local checks

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo xtask check
git diff --check origin/main
```

The `check` command validates tracked Markdown links, headings, and bilingual pairs, checks generated tokens, and verifies GPUI/Base package identities and features on the Windows, macOS, and Linux dependency graphs. Stage Markdown additions, renames, and removals so the checker sees the intended Git index. The documentation check runs offline. Use `cargo xtask tokens --check` to check generated tokens or `cargo xtask tokens` to update the Rust tables and index after editing the data. [Token data](theme/theme.md#token-data)

<a id="冷热构建基线"></a>

## Cold and warm build baseline

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-baseline
```

Use a new report directory for each measurement. Prefetch the Cargo registry and source caches. The cold check and cold build each start in an empty target directory. The command then reuses each target directory for the corresponding warm run. The tool records four wall-clock times, HTML timelines, the Git commit, target, and toolchain. The last Gallery build unit includes code generation and linking. Use the normal Gallery features and the development profile without overriding Cargo's job count. Record the key timings, reproduction command, and relevant environment in the issue or PR.

<a id="持续集成与合并"></a>

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) runs for PRs targeting `main`, pushes to `main`, and manual dispatches. The job layout follows [GPUI Kit CI](https://github.com/longbridge/gpui-kit/blob/main/.github/workflows/ci.yml).

| Job | Checks |
| --- | --- |
| `Repository checks` | Formatting, documentation links and pairs, generated tokens, three-platform dependency graphs, and the default workspace. Diff whitespace is checked against the base commit for PRs and pushes with a base. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | Clippy, workspace tests, headless Gallery UI tests, and native Gallery builds on Windows Server 2025 x64, macOS 15 ARM64, and Ubuntu 24.04 x64. Other platforms continue if one fails. |
| `CI required` | Passes when the repository checks and all three native jobs succeed. |

Rust build caches are isolated by platform. Successful pushes to `main` save the caches and PRs can restore them.

Hosted CI covers builds and automated tests on three platforms. The initial release follows the native acceptance criteria in [platforms and quality requirements](verification.md).

The Windows job uploads `gallery-windows-x64-msvc` with the executable, Cargo timeline, and licensing information for native retesting. It runs `cargo build --locked -p gpui-material-gallery --profile gallery-artifact --timings`. This profile disables debug assertions so GPUI can embed DirectX shaders at build time using the Windows SDK's x64 `fxc.exe`. CI sets `GPUI_FXC_PATH` to the SDK compiler. Normal `cargo gallery` builds read the shader sources from the local Cargo cache. [Release procedure](release.md#prepare-and-publish)

The GitHub ruleset requires a PR for changes to `main`. Bring the PR up to date with its base and follow the [review and merge rules](../CONTRIBUTING.md#pr-review-and-merge).

<a id="windows-分发"></a>

## Windows distribution

Run the following commands in PowerShell 7 from the accepted checkout. Set `GPUI_FXC_PATH` to the installed x64 Windows SDK shader compiler. A matching Windows engineering build from CI can also be used as the executable input. Fetch the locked Windows dependencies before running the offline packager. Rust's `rust-docs` component supplies the standard library licensing materials.

```powershell
rustup component add rust-docs
cargo fetch --locked --target x86_64-pc-windows-msvc
$env:GPUI_FXC_PATH = 'C:\path\to\the\Windows SDK\x64\fxc.exe'
cargo build --locked -p gpui-material-gallery --profile gallery-artifact
./scripts/package-windows-gallery.ps1 -Executable target/gallery-artifact/gpui-material-gallery.exe -OutputDirectory target/gallery-distribution
Get-FileHash target/gallery-distribution/gpui-material-gallery-windows-x64.zip -Algorithm SHA256
Get-Content target/gallery-distribution/SHA256SUMS.txt
```

Compare the hashes, extract the ZIP into a new directory, run `gpui-material-gallery.exe`, and verify the real-window scenarios in the acceptance issue. If `CARGO_TARGET_DIR` points outside the checkout, pass the corresponding artifact path to the packaging script.

The script records the checkout commit, Rust toolchain, runtime dependency inventory and license material. Package a clean build from the same commit. The checksum identifies each ZIP as produced. Rebuilding on another machine can produce different binary bytes and ZIP metadata. CI rehearses the ZIP locally and retains its standard engineering artifact.
