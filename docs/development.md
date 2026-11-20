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
- Verify public behavior in the owning crate and exercise the affected variants and interactions in Gallery. Use a minimal entry point for independently reproducible scenarios. Keep engineering checks and offline generation in `xtask/`.

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

<a id="性能检查"></a>

## Rendering performance

Use the dev profile for day-to-day checks and locked Release builds for performance comparisons. Open the Button story to check idle, hover, and press/release separately. Use `--reduce-motion` as a diagnostic comparison.

```powershell
cargo run --locked --release -p gpui-material-gallery -- button
cargo run --locked --release -p gpui-material-gallery --features frame-profile -- button --frame-profile
cargo test --locked --release -p gpui-material-foundation --lib ripple_clip_benchmark -- --ignored --nocapture
cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1
```

Run the same Ripple microbenchmark on both revisions on the same host, with at least five separate runs per revision. When the baseline predates the benchmark, apply only the benchmark test to that revision. It times clipping prebuilt meshes, excluding shape construction and presentation. Record the commits, workload, timing spread, and relevant real-window checks in the Issue or PR. Report draw CPU time and presentation intervals separately when measuring native frames.

The `perf_bench` Release suite measures 20 warm-up frames followed by 100 timed frames at a fixed `1040 × 744` viewport (`16 ms` clock step) across three distinct attribution layers so regressions are not misattributed:

- **Engine layer (`engine/*`)**: bare GPUI window, 64 raw `div`/text quads, and a 200-row raw GPUI `list`, plus Foundation's `ripple_clip_benchmark`.
- **Component layer (`component/*`)**: isolated Material components mounted directly under `MaterialRoot` without Gallery chrome (`Button` 64-grid idle/active, `TextField` + multiline `TextArea` idle/typing, 200-row rich `ListItem` virtual list idle/scroll, `Scaffold` + `AppBar` + `NavigationRail` + `Tabs` + `NavigationBar` idle/transition, and `LoadingIndicator` + wavy `LinearProgress`/`CircularProgress` visible active vs. off-screen clipped vs. reduced motion).
- **Story vs. Gallery shell layer (`story/*` vs. `gallery/*`)**: standalone `Story::new_view` versus full `Gallery::new` (catalog sidebar, search field, toolbar, and `ListDetail` shell) on matching stories.

Reference Release baseline (`b3a538e`, Windows 11 x64, AMD Ryzen 9 8945HX, `rustc 1.99.1`, `--locked --release`, 3-run median p50/p95 for `perf_bench` and 5-run median for `ripple_clip_benchmark`):

| Benchmark | Layer | p50 (µs) | p95 (µs) | Quads | Scheduled frames |
| --- | --- | ---: | ---: | ---: | ---: |
| `engine/empty_window` | Engine | 7.9 | 8.6 | 0 | 0 |
| `engine/raw_quads_64` | Engine | 295.9 | 375.8 | 64 | 0 |
| `engine/raw_virtual_list_200` | Engine | 244.4 | 327.5 | 59 | 0 |
| `component/button_grid_64_idle` | Component | 2497.9 | 3353.5 | 112 | 0 |
| `component/button_grid_64_active` | Component | 2943.2 | 4711.8 | 112 | 1448 |
| `component/text_field_idle` | Component | 272.6 | 355.1 | 19 | 0 |
| `component/text_field_typing` | Component | 277.5 | 408.9 | 20 | 0 |
| `component/complex_list_200_idle` | Component | 1903.0 | 3591.2 | 40 | 0 |
| `component/complex_list_200_scroll` | Component | 1956.3 | 2570.2 | 41 | 244 |
| `component/navigation_scaffold_idle` | Component | 552.1 | 692.8 | 11 | 0 |
| `component/navigation_rail_transition` | Component | 574.9 | 713.7 | 11 | 263 |
| `component/animated_visible` | Component | 85.6 | 137.2 | 9 | 297 |
| `component/animated_clipped_offscreen` | Component | 27.6 | 51.3 | 1 | 0 |
| `component/animated_reduced_motion` | Component | 96.1 | 142.4 | 10 | 0 |
| `story/button` / `gallery/button` | Story / Gallery | 964.6 / 4903.2 | 1137.9 / 6249.2 | 35 / 47 | 0 / 0 |
| `story/text_field` / `gallery/text_field` | Story / Gallery | 752.7 / 4899.2 | 894.0 / 6317.1 | 54 / 64 | 0 / 0 |
| `story/list` / `gallery/list` | Story / Gallery | 4909.5 / 12027.4 | 6593.5 / 15308.4 | 28 / 45 | 0 / 0 |
| `story/navigation_rail` / `gallery/navigation_rail` | Story / Gallery | 657.0 / 5470.8 | 806.2 / 7908.8 | 22 / 47 | 0 / 0 |
| `story/loading_indicator` / `gallery/loading_indicator` | Story / Gallery | 676.5 / 4723.2 | 1011.4 / 7850.4 | 37 / 49 | 198 / 198 |

`ripple_clip_benchmark` (5-run median): `capsule expanding` 8,445.6 ns/clip (37 tris), `capsule covered` 20,071.0 ns/clip, `morph expanding` 11,330.8 ns/clip (40 tris), `morph covered` 21,587.1 ns/clip. On this baseline, the Gallery shell (42-item catalog sidebar, search field, toolbar, and `ListDetail` frame) accounts for ~3.9–7.1 ms of p50 frame time independently of the isolated component under test.

The optional single-window frame profile prints a start marker after a two-second warm-up, then reports draw count and p50/p95/p99 for CPU draw time and active frame submission intervals over ten seconds. Run separate captures for idle, pointer sweeps, and repeated presses with the same input sequence on both revisions. Submission intervals measure GPUI's platform submissions.

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

<a id="文档网站"></a>

## Documentation website

The [website source](../website/) uses Astro and [MDUI](https://www.mdui.org/en/) web components. Pagefind indexes the static pages. With Node.js 24 and npm installed, run these commands from the repository root.

```sh
npm --prefix website ci
npm --prefix website run dev
npm --prefix website run build
npm --prefix website run preview
```

The development server uses `/`. The production build defaults to `/gpui-material/` and writes `website/dist/`. The production preview includes the search index.

Edit shared guides under `docs/` and component examples under `website/component/`. Add English and Chinese text together. Each component page shares one Rust example across both languages.

The website component catalog and sidebar use the same basic, form, layout, and advanced groups. The sidebar lists Rust names alphabetically within each group. Write imports separately from usage snippets, and put interactive browser previews under Examples.

Use MDUI for website navigation and controls. The browser previews compile the Rust Gallery to WebAssembly and open each story in its own GPUI canvas. Size the canvas to the frame so the GPUI viewport handles wheel input and scrollbar dragging. The browser needs WebGPU or WebGL2. For desktop-specific behavior, run the native Gallery.

To enable the previews locally, install the web toolchain and matching bindings generator, then build the WASM assets before starting Astro.

```sh
rustup toolchain install nightly-2026-10-31 --profile minimal --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm --prefix website run build:wasm
```

The local build uses Cargo's dev profile. The Pages workflow builds and publishes optimized WASM with the site.

Set **Settings → Pages → Build and deployment → Source** to **GitHub Actions**. PRs that change website sources or WASM build inputs run the website build. Changes to the website, its build inputs, or documentation trigger the Pages build and deployment on `main`. To redeploy, run **Deploy website** on `main` from the Actions tab.

<a id="冷热构建基线"></a>

## Cold and warm build baseline

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-baseline
```

Use a new report directory for each measurement. Prefetch the Cargo registry and source caches. The cold check and cold build each start in an empty target directory. The command then reuses each target directory for the corresponding warm run. The tool records four wall-clock times, HTML timelines, the Git commit, target, and toolchain. The last Gallery build unit includes code generation and linking. Use the normal Gallery features and the development profile without overriding Cargo's job count. Record the key timings, reproduction command, and relevant environment in the issue or PR.

<a id="持续集成与合并"></a>

## CI and merging

[Engineering CI](../.github/workflows/ci.yml) selects checks for PRs targeting `main` and pushes to `main` by comparing the checkout with the event's base commit. A manual run executes the full checks.

| Job | Checks |
| --- | --- |
| `Change scope` | Classifies changed files and checks diff whitespace when a base commit is available. |
| `Documentation checks` | Runs `cargo xtask docs --check` for documentation and engineering changes. Checks Markdown links and bilingual pairs. |
| `Website / Build documentation` | Builds and link-checks the site with the WASM Gallery for PRs that change website sources or website build inputs. The Pages workflow reuses this build on `main`. |
| `Repository checks` | For engineering changes, checks Rust formatting, generated tokens, three-platform dependency graphs, and the default workspace. |
| `Windows MSVC`, `macOS ARM64`, `Linux x64` | For engineering changes, runs Clippy, workspace tests, headless Gallery UI tests, and native Gallery builds. Other platforms continue if one fails. |
| `CI required` | Requires the jobs selected for the change to pass and verifies the other jobs were skipped. |

A docs-only PR runs the documentation checks. A website-only PR runs the website build. Changes in both areas run both checks. Changes to shared Rust build inputs, such as the Gallery, libraries, or workspace manifests, also run the native checks. The browser-only Gallery entry point belongs to the website scope.

Rust build caches are isolated by platform. Successful pushes to `main` save the caches and PRs can restore them.

For release acceptance, follow the [Windows real-window checks](verification.md#windows-gallery-acceptance).

The GitHub ruleset requires a PR for changes to `main`. Bring the PR up to date with its base and follow the [review and merge rules](../CONTRIBUTING.md#pr-review-and-merge).
