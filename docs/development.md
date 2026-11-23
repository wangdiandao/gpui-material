# Development

English | [中文](development.zh.md)

<a id="贡献流程与分支命名"></a>

## Contribution workflow and branch naming

- Follow the [contribution guide](../CONTRIBUTING.md) for issue templates, design changes, and review.
- Branch from the latest `main` using `<type>/<short-topic>` in lowercase English with hyphens (for example `feat/split-button` or `fix/pointer-capture`), and link the issue in the PR description.
- Use `feat/`, `fix/`, `docs/`, `test/`, `refactor/`, `ci/`, `chore/`, or `release/` according to the change's purpose.

<a id="workspace-与组件开发"></a>

## Workspace and component development

- Production libraries live under `crates/`, grouped by component family. Gallery lives in `apps/gallery/` and exercises the public API through stories. [Architecture and dependencies](architecture.md) · [Components](component.md)
- Keep engineering checks and offline generation in `xtask/`.

<a id="编码规范"></a>

## Coding conventions

- Use the [documentation index](index.md) to locate the current specification, and follow the [design rules](../CONTRIBUTING.md#design-specification) when changing Material presentation.
- Update relevant documentation before or alongside code changes in the same PR.
- Keep paired English `foo.md`, Chinese `foo.zh.md`, and section fingerprints `foo.i18n.yaml` together. Link the counterpart immediately below the title and use the current page's language for links to other paired pages. Run `cargo xtask docs --write path/to/foo.md` after updating both pages. GitHub issue and PR templates use English. [DeepSeek Harness documentation conventions](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/i18n/README.md)
- Keep `theme` in pure Rust and `foundation` for GPUI/Base adapters. Applications or Base own business and input state; Material owns presentation and additional composition state. [Architecture and dependencies](architecture.md)
- Organize modules and functions by responsibility—public API, state transitions, geometry and layout math, style resolution, presentation, and tests—rather than arbitrary file-length limits. Preserve stable public re-exports, keep internal visibility narrow (`pub(super)` / `pub(crate)`), keep sibling dependencies acyclic, place substantial test suites in dedicated `#[cfg(test)]` submodules (including Gallery `*_tests.rs`), and leave generated data tables intact.
- Use idiomatic Rust naming and stable semantic identifiers. Document state ownership, defaults, focus, callbacks, and platform differences for public APIs.

<a id="测试规范"></a>

## Testing conventions

- Add or extend automated tests when introducing public behavior, fixing a reproducible regression, or covering an untested invariant. Keep tests repeatable and place them at the lowest effective layer (unit tests in `crates/`, headless `#[gpui::test]` suites in `apps/gallery/src/`).
- Cover applicable interaction and accessibility paths: keyboard and pointer input, focus, disabled/error states, cancellation, RTL, and reduced motion.
- Remove obsolete tests and helpers when behavior changes or coverage is superseded.

<a id="上游贡献"></a>

## Upstream contributions

- Confirm upstream defects with a minimal reproduction, check existing upstream issues and PRs, and follow upstream contribution and licensing rules.
- Keep patches focused on the shared defect, and track upstream review, release, and removal conditions for any local workaround in a local issue.
- Before bumping a pinned dependency, check the version, commit, and package identity against the upstream release record.

<a id="开发环境"></a>

## Development environment

Install [Rustup](https://rustup.rs/) and run Cargo from the repository root. [rust-toolchain.toml](../rust-toolchain.toml) pins the repository toolchain; published crates declare an MSRV of 1.95.0 in [Cargo.toml](../Cargo.toml).

| Platform | Build prerequisites |
| --- | --- |
| Windows 11 x64 | Visual Studio Build Tools with Desktop development with C++, MSVC x64, and the Windows SDK. |
| macOS on Apple Silicon | Xcode command-line tools. |
| Ubuntu 24.04 x64 | System packages listed in the `Install Linux platform dependencies` step of [Engineering CI](../.github/workflows/ci.yml). |

Run Gallery from the repository root, optionally passing a story name or ID:

```powershell
cargo gallery
cargo gallery text-field
```

The default workspace members are `gpui-material-theme` and `xtask`. Use `-p gpui-material-gallery` to target Gallery directly. From a worktree, set `CARGO_TARGET_DIR` to the main checkout's `target` directory to reuse incremental builds.

<a id="本地检查"></a>

## Local checks

Stage Markdown additions, renames, and removals before running `xtask` so they appear in the Git index. `cargo xtask check` checks Markdown links, heading anchors, bilingual pairs, generated tokens, and three-platform GPUI/Base dependency graphs.

```powershell
git diff --check origin/main
cargo xtask check
```

For Rust changes, run formatting, Clippy, and tests:

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Run Gallery UI tests alone with `cargo test --locked -p gpui-material-gallery --features test-support`. For token changes, follow the [generation steps](theme/theme.md#token-data).

<a id="性能与构建工具"></a>

## Performance and build tools

Use locked Release builds for rendering performance comparisons and `--reduce-motion` as a diagnostic comparison:

```powershell
cargo run --locked --release -p gpui-material-gallery -- button
cargo run --locked --release -p gpui-material-gallery --features frame-profile -- button --frame-profile
cargo test --locked --release -p gpui-material-foundation --lib ripple_clip_benchmark -- --ignored --nocapture
cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1
```

- `perf_bench` measures fixed-viewport frames across three attribution layers: **engine** (`engine/*`), isolated **components** (`component/*`), and **story vs. Gallery shell** (`story/*` vs. `gallery/*`).
- `ripple_clip_benchmark` times clipping prebuilt ripple meshes in isolation.
- `--frame-profile` samples a single native window over 10 seconds after a 2-second warm-up and reports CPU draw time and platform frame submission intervals (p50/p95/p99).

To measure cold and warm Gallery build times into a fresh report directory:

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-timings
```

<a id="文档网站"></a>

## Documentation website

The [website source](../website/) uses Astro, [MDUI](https://www.mdui.org/en/) web components, and Pagefind search. With Node.js 24 and npm installed:

```sh
npm --prefix website ci
npm --prefix website run dev
npm --prefix website run build
npm --prefix website run preview
```

The dev server serves at `/`; production builds default to `/gpui-material/` and output to `website/dist/` with the Pagefind index.

Edit shared guides under `docs/` and component examples under `website/component/` (each component page shares one Rust snippet across English and Chinese). The website catalog and sidebar use the `basic`, `form`, `layout`, and `advanced` groups, ordered alphabetically by Rust name within each group.

Browser previews compile the Rust Gallery to WebAssembly and mount each story in its own GPUI canvas (requiring WebGPU or WebGL2). To build local WASM previews before starting Astro:

```sh
rustup toolchain install nightly-2026-10-31 --profile minimal --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm --prefix website run build:wasm
```

In repository settings, set **Settings → Pages → Build and deployment → Source** to **GitHub Actions**. PRs touching website sources or WASM build inputs run the website build; pushes to `main` touching the website, its build inputs, or `docs/` deploy automatically.

<a id="持续集成与合并"></a>

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) selects jobs by comparing the checkout against the base commit (manual runs execute all jobs).

| Job | Checks |
| --- | --- |
| `Change scope` | Classifies changed files and checks diff whitespace. |
| `Documentation checks` | Runs `cargo xtask docs --check` for documentation and engineering changes. |
| `Website / Build documentation` | Builds and link-checks the website with the WASM Gallery when website sources or build inputs change. |
| `Repository checks` | Checks Rust formatting, generated tokens, three-platform dependency graphs, and the default workspace. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | Runs Clippy, workspace tests, headless Gallery UI tests, and native Gallery builds. |
| `CI required` | Confirms the selected jobs passed and unselected jobs were skipped. |

Rust build caches are isolated by platform, saved on `main`, and restored by PRs. Changes to `main` require a PR that is up to date with its base and passes `CI required` under the [review and merge rules](../CONTRIBUTING.md#pr-review-and-merge). For releases, see [versioning and releases](release.md#prepare-and-publish).
