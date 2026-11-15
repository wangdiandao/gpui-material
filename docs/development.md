# Development and verification

English | [中文](development.zh.md)

<a id="贡献流程与分支命名"></a>

## Contribution workflow and branch naming

- Follow the [contribution guide](../CONTRIBUTING.md) for Issue templates, design changes, and review.
- Branch from the latest `main` and use `<type>/<short-topic>`, such as `ci/cross-platform-workflow`. Write the topic in lowercase English with hyphens. Link the issue in the PR description.
- Choose the prefix by purpose. Use `feat/` for features, `fix/` for fixes, `docs/` for documentation, `test/` for tests, `refactor/` for refactoring, `ci/` for CI, `chore/` for maintenance, and `release/` for releases. For example, use `fix/pointer-capture`.

<a id="workspace-与组件开发"></a>

## Workspace and component development

- Organize production libraries under `crates/` and group components by family. Gallery lives in `apps/gallery/` and its stories exercise the production public API. [Architecture and dependencies](architecture.md) · [Components](component.md)
- Verify public behavior in the owning crate and exercise the affected variants and interactions in Gallery. Use a minimal entry point for independently reproducible scenarios. Keep isolated probes in `experiments/` and engineering checks and offline generation in `xtask/`.

<a id="编码规范"></a>

## Coding conventions

- Use the [documentation index](index.md) to find the current specification. Follow the [pinned design and review rules](../CONTRIBUTING.md#design-specification) when changing Material presentation.
- When a change needs documentation updates, complete them before starting implementation. Submit the documentation and code in the same commit or PR when they belong together.
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

Install [Rustup](https://rustup.rs/) and run Cargo from the repository root. [rust-toolchain.toml](../rust-toolchain.toml) pins the compiler, rustfmt, and Clippy.

| Platform | Build prerequisites |
| --- | --- |
| Windows 11 x64 | Visual Studio Build Tools with Desktop development with C++, MSVC x64, and the Windows SDK. |
| macOS on Apple Silicon | Xcode command-line tools. |
| Ubuntu 24.04 x64 | System packages listed in the `Install Linux platform dependencies` step of [Engineering CI](../.github/workflows/ci.yml). |

Check the compiler host with `rustc -vV`. Run Gallery and optionally pass a story name or ID to open it directly.

```powershell
cargo gallery
cargo gallery text-field
```

The default workspace members are the theme crate and `xtask`. Use `-p gpui-material-gallery` to select Gallery explicitly. From a worktree, point `CARGO_TARGET_DIR` to the main checkout's `target` to reuse developer builds.

<a id="独立探针"></a>

## Standalone probes

Use the [GPUI host probe](../experiments/gpui-host/) to isolate a suspected GPUI or Base regression, such as native capture loss, text input, or accessibility semantics. Its binaries share a manifest and lockfile. `cargo run` opens the window probe. `cargo test` runs the automated probe tests.

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo run --locked --manifest-path experiments/gpui-host/Cargo.toml --bin window
cargo test --locked --manifest-path experiments/gpui-host/Cargo.toml --features test-support
```

Run these commands from the main checkout. In a worktree, set `CARGO_TARGET_DIR` to that main checkout's absolute `target` path. Use the owning crate and Gallery for Material behavior.

<a id="本地检查"></a>

## Local checks

For documentation and repository checks, stage Markdown additions, renames, and removals so `xtask` sees them in the Git index. The checker validates local links, heading anchors, bilingual pairs, generated tokens, and GPUI/Base dependency graphs for Windows, macOS, and Linux.

```powershell
git diff --check origin/main
cargo xtask check
```

For Rust changes, run formatting and the affected tests. The commands below match the workspace-wide lint and test scope in CI.

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

For Gallery UI tests alone, run `cargo test --locked -p gpui-material-gallery --features test-support`. For token data changes, follow the [generation steps](theme/theme.md#token-data).

<a id="冷热构建基线"></a>

## Cold and warm build baseline

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-baseline
```

Use a new report directory for each measurement. Prefetch the Cargo registry and source caches. The cold check and cold build each start in an empty target directory. The command then reuses each target directory for the corresponding warm run. The tool records four wall-clock times, HTML timelines, the Git commit, target, and toolchain. The last Gallery build unit includes code generation and linking. Use the normal Gallery features and the development profile without overriding Cargo's job count. Record the key timings, reproduction command, and relevant environment in the issue or PR.

<a id="持续集成与合并"></a>

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) runs for PRs targeting `main`, pushes to `main`, and manual dispatches.

| Job | Checks |
| --- | --- |
| `Repository checks` | Formatting, documentation links and pairs, generated tokens, three-platform dependency graphs, and the default workspace. Diff whitespace is checked against the base commit for PRs and pushes with a base. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | Clippy, workspace tests, headless Gallery UI tests, and native Gallery builds on Windows Server 2025 x64, macOS 15 ARM64, and Ubuntu 24.04 x64. Other platforms continue if one fails. |
| `CI required` | Passes when the repository checks and all three native jobs succeed. |

Rust build caches are isolated by platform. Successful pushes to `main` save the caches and PRs can restore them.

The Windows job rehearses ZIP packaging and uploads the `gallery-windows-x64-msvc` engineering artifact with the executable, build timings, and licensing information. For release acceptance, use the [Windows real-window checks](verification.md#windows-gallery-acceptance).

The GitHub ruleset requires a PR for changes to `main`. Bring the PR up to date with its base and follow the [review and merge rules](../CONTRIBUTING.md#pr-review-and-merge).

<a id="windows-分发"></a>

## Windows distribution

Package from a clean checkout of the release commit in PowerShell 7. Prefer the executable in the `gallery-windows-x64-msvc` artifact from the CI run for that commit. For a local build, set `GPUI_FXC_PATH` to the Windows SDK's x64 `fxc.exe` and use the profile that embeds DirectX shaders.

```powershell
cargo build --locked -p gpui-material-gallery --profile gallery-artifact
```

Replace the example `$exe` path below when using the CI executable or a custom `CARGO_TARGET_DIR`. The offline packager uses the locked Windows packages and the Rust standard library licenses supplied by `rust-docs`.

```powershell
rustup component add rust-docs
cargo fetch --locked --target x86_64-pc-windows-msvc
$exe = 'target/gallery-artifact/gpui-material-gallery.exe'
$output = Join-Path ([IO.Path]::GetTempPath()) ('gpui-material-gallery-' + [guid]::NewGuid())
./scripts/package-windows-gallery.ps1 -Executable $exe -OutputDirectory $output

$name = 'gpui-material-gallery-windows-x64.zip'
$zip = Join-Path $output $name
$sha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
$expected = (Get-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt') -Raw).Trim()
if ($expected -cne "$sha  $name") { throw 'Gallery ZIP checksum mismatch' }
Expand-Archive -LiteralPath $zip -DestinationPath (Join-Path $output 'extracted')
```

Check the commit in `BUILD.txt`, the dependency inventory in `DEPENDENCIES.txt`, and the bundled licenses. Run the extracted executable and follow the [Windows Gallery acceptance checks](verification.md#windows-gallery-acceptance). Upload the tested ZIP and `SHA256SUMS.txt` together. Remove temporary build and extraction outputs after use. [Release procedure](release.md#prepare-and-publish)
