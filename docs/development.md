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

Choose the lowest effective test layer for the behavior being verified:

| Layer | Mechanism & location | Scope |
| --- | --- | --- |
| Pure logic unit tests | Regular `#[test]` in `crates/` | Token tables, theme math, layout geometry, parsers, and state transitions that do not need a GPUI `App` or `Window`. |
| Facade UI integration tests | Single-binary `#[gpui::test]` in `crates/material/tests/integration/` | Public facade contracts, `gpui_material::test` harness semantics, controlled callbacks, `ThemeScope` isolation, and downstream end-to-end flows using only `gpui_material` exports. |
| Gallery headless UI tests | `#[gpui::test]` in `apps/gallery/src/` | Component presentation, Material quad/clip geometry, animations/reduced motion, Story & Gallery shell workflows. |
| Offscreen pixel rendering tests | Opt-in `apps/gallery/tests/rendering.rs` (`--test rendering`, `test = false`, `harness = false`) | Real offscreen DirectX (Windows hidden-window readback, with WARP fallback), Metal (macOS), and WGPU/lavapipe (Linux) rasterization checks for embedded Material Symbols SVGs, GPUI `Path` shapes/checkmarks/progress arcs, Expressive polygon `Surface` clipping, light/dark/high-contrast tonal surfaces, text visibility, focus-ring contrast, and RTL layout/icon mirroring that `painted_quads()` cannot prove. |
| Native desktop & accessibility verification | Manual run of `cargo gallery` | Platform boundaries outside headless GPUI: OS IME candidate windows (such as Microsoft Pinyin), system clipboard services, screen-reader announcements (Windows UIA/Narrator, macOS VoiceOver), and native window chrome. |

Follow these test authoring rules:

- **Prefer concise, high-signal tests**: combine builder and variant checks into focused scenario tests instead of fragmenting trivial per-setter tests; use plain `#[test]` whenever GPUI context is unnecessary.
- **Aggregate integration targets**: keep integration tests as submodules of a single `tests/integration/main.rs` binary per crate to avoid linking redundant copies of GPUI and all components. Keep standalone targets only when required (such as `catalog_snippets` or `harness = false` offscreen `rendering`).
- **Use `gpui_material::test` instead of hand-rolling events**: enable `test-support` and drive interactions with `TestWindowExt` (`find`, `try_find`, `find_all`, `within`, `render_frame`, `click`, `click_at`, `click_with_options`, `right_click`, `double_click`, `hover`, `scroll`, `drag`, `drag_to`, `press`, `input`), `TestQueryExt` (`elements`, `find_by_label`, `find_all_by_role`), `TestEventExt` (stepwise pointer/key/modifier gestures), `TestInput` (explicit UTF-16 composition bridge), and `TestAppContextExt::wait_for` (bounded async test-clock polling outside `update_window`).
- **Observe native semantics honestly**: place `.test_support()` before `.track_focus(&handle)` so `focused()` observes the real handle and catches missed bindings. Never add test-only state props or treat `disabled() == None` as proof that a control is enabled—attempt the interaction and assert the application outcome. Re-query immutable `ElementSnapshot` values after interactions.
- **Cover core interaction paths**: keyboard and pointer input, focus traversal and restoration, disabled/error states, pointer/Esc cancellation, RTL, and reduced motion (`cx.set_reduce_motion(true)`).
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

Stage Markdown additions, renames, and removals before running `xtask` so they appear in the Git index. `cargo xtask check` validates Markdown links against tracked and staged repository files, heading anchors, bilingual pairs (reporting all broken links and stale records together), generated tokens, three-platform GPUI/Base dependency graphs, and synchronization of the compile-checked `website/component/catalog.mjs` Rust snippets (`cargo xtask snippets --check`). Running `cargo xtask tokens` also refreshes `docs/theme/token-index.i18n.yaml` automatically.

```powershell
git diff --check origin/main
cargo xtask check
```

For Rust changes, run formatting, Clippy, Rustdoc, and tests:

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --locked --workspace --no-deps --all-features
cargo test --locked --workspace --all-features
```

Useful targeted test commands:

```powershell
cargo test --locked -p gpui-material --features test-support --test integration
cargo test --locked -p gpui-material-gallery --features test-support
cargo test --locked -p gpui-material-gallery --features desktop,test-support --test rendering
cargo check --locked -p gpui-material --no-default-features
cargo check --locked -p gpui-material --no-default-features --features display
```

`--test rendering` runs real offscreen DirectX (Windows hidden-window readback, with WARP fallback), Metal (macOS), and WGPU/lavapipe (Linux) pixel assertions, printing an explicit diagnostic skip only if a non-interactive Windows session cannot attach DirectX. For token changes, follow the [generation steps](theme/theme.md#token-data).

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

To measure cold and warm Gallery build times into a fresh report directory (the runner touches a leaf source file before warm measurements and removes its temporary target directories automatically):

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

In repository settings, set **Settings → Pages → Build and deployment → Source** to **GitHub Actions**. `npm --prefix website run build:wasm` honors `RUSTUP_TOOLCHAIN`, verifies `wasm-bindgen --version`, and builds `--release` with Cargo's `wasm-release` profile; `npm --prefix website run build` also validates every `source` path in `website/component/catalog.mjs`, while `cargo xtask snippets --check` and `cargo test --locked -p gpui-material --test catalog_snippets --all-features` compile-check every Rust snippet against the workspace crates. PRs touching website sources, `docs/`, or WASM build inputs run the website build; pushes to `main` touching the website, its build inputs, or `docs/` deploy automatically.

<a id="持续集成与合并"></a>

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) selects jobs by comparing the checkout against the base commit (manual runs execute all jobs).

| Job | Checks |
| --- | --- |
| `Change scope` | Classifies changed files (`docs`, `website`, `checks`, `release_scripts`, `native`) and checks diff whitespace. |
| `Repository checks` | Runs conditional repository-level checks in one job: `cargo xtask docs --check`, Rust formatting, generated tokens, three-platform dependency graphs, default workspace, default-feature Gallery, and slim-feature compile checks (`gpui-material` / `gpui-material-components` with `--no-default-features` and minimal slices), plus `release.py` syntax/unit tests. |
| `Website / Build documentation` | Builds and link-checks the website with the WASM Gallery when website sources, `docs/`, or WASM build inputs change. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | Runs Clippy, workspace library/integration tests (`--lib --tests`), headless Gallery UI tests, native Gallery builds (`--all-features`), offscreen pixel rendering tests (`--test rendering` across Windows DirectX, macOS Metal, and Linux WGPU/lavapipe), and Rustdoc + doctest checks (on Linux x64, with `mold` linking). |
| `CI required` | Confirms the selected jobs passed and unselected jobs were skipped. |

Rust build caches are isolated by platform, saved on `main`, and restored by PRs. Changes to `main` require a PR that is up to date with its base and passes `CI required` under the [review and merge rules](../CONTRIBUTING.md#pr-review-and-merge). For releases, see [versioning and releases](release.md#prepare-and-publish).
