# P3 acceptance record

[简体中文](P03-evidence.md) · English

The local environment was Windows 11 x64 build 26200 with Rust/Cargo 1.99.1 and MSVC. Pure theme and xtask shared the main checkout's development build directory. The PR was linked to [P3-02](https://github.com/wangdiandao/gpui-material/issues/75) and hosted results were checked in its Windows MSVC job.

| Check | Pinned source, expectation, and observation | Status |
| --- | --- | --- |
| Sources | Compose common 1.5.0-rc01 source SHA-256 was `38e0515354240bf1e15b6e8891bc02608973f08dc0b8da8cd87a51dfe9ab1031`. AndroidX graphics-shapes 1.1.0 was `50586aa6c7b3a542a51b2abaf659f7a6c33d8f1a57aaf518dcb74f3d`. Breakpoints matched the [frozen index](../material-snapshot.md). | pass |
| Tokens | 555 entries across Reference, System, and Component. Offline Rust and Markdown generation matched the repository. Type mismatches, illegal layers, missing targets, duplicates, and cycles were rejected. Samples retain Filled Button `Primary`, hover level 1, and Small Button pressed shape. | pass |
| Static colors and elevation | Baseline primary was `ff6750a4` light and `ffd0bcff` dark. Level 1 overlays were `fff6eefa` and `ff1e1b24` using Compose's logarithmic formula rounded to 8-bit sRGB. | pass |
| Seeds and semantic roles | TypeScript Tonal Spot Spec2021 at MCU revision `194144d75ed7d34f06264a625e086db81a7ab9aa`. Seven seeds × two appearances × four contrast levels (−1, 0, 0.5, 1) × 49 roles yielded 2,744 exact ARGB matches. Dark red at default contrast had `primaryFixed = ffffdad4`. | pass |
| HCT and palettes | Absolute HCT error for seven seeds was below `1e-6` and reconstructed ARGB matched exactly. Blue at tone 40 was `ff343dff`. P1's direct Rust 0.1.3 call returned 34.204355 for out-of-range contrast. The clamped adapter returned 21, matching the pinned official implementation. | pass |
| Typography | Compared 15 baseline and 15 emphasized roles to pinned `TypeScaleTokens.kt`. Display Large was 57/64 sp, tracking −0.2, weight 400. Emphasized Body Large was 16/24 sp, tracking 0.15, weight 500. Verified font overrides, doubled scale, and invalid-scale fallback. | pass |
| Shape and motion | Square width and height 1 and corner radius 0.3. Triangle had three vertices, radius 1, rounding 0.2, and rotation −90°. Circle had 10 vertices and circle radius 1. Corners covered logical start/end and narrow bounds. Twelve springs across the two schemes matched pinned Compose. Reduced motion completed immediately. | pass |
| Layout | Verified thresholds 600, 840, 1200, and 1600 and values immediately below them, RTL mapping, density, and the 48 logical unit minimum interactive size. | pass |
| Dependencies | Checked normal/build/dev graphs for the default workspace and theme feature combinations on three platforms. Theme's default production graph contains only itself. MCU is enabled only by `dynamic-color`, has no features enabled, and `serde` is independent. | pass |

## Reproduction

```powershell
$env:CARGO_NET_OFFLINE = 'true'
cargo xtask tokens
cargo xtask tokens --check
Remove-Item Env:CARGO_NET_OFFLINE
cargo test --locked -p gpui-material-theme --no-default-features
cargo test --locked -p gpui-material-theme --features dynamic-color
cargo test --locked -p gpui-material-theme --features serde
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo xtask check origin/main
```

The [official vectors](../../crates/theme/tests/mcu-spec-2021.json) use pinned MCU `Hct.fromInt`, `Hct.from`, `TonalPalette.fromInt`, `Contrast.ratioOfTones`, and `SchemeTonalSpot`. TypeScript 5.9.3 compiled upstream `typescript/index.ts` to ES2022 modules, run on Node.js 24.14.1. Resolution adds `.js` to extensionless relative imports. Scheme rows contain seed, isDark, contrast, and ARGB values sorted by role name. Constructor arguments were `(Hct.fromInt(seed), isDark, contrast, '2021', 'phone')`. HCT rows contain seed, hue, chroma, tone, and reconstructed ARGB. Repository tests need only the checked-in vectors.

The [theme specification](../theme/theme.en.md) records desktop policy. Drawing, glyph measurement, motion scheduling, and system integration are accepted in their owning phases.

## Dependency admission

`material-color-utils =0.1.3` corresponds to `d39ff59a1fca34bdc60e124de2f9f7541fff3c39`. The package checksum is `675994c58762c15c884efab42998c09209f2c451be302e839222fa9e805363fa`. The bundled license is Apache-2.0. At verification the GitHub project was active and its last push was 2026-03-04. Rerun pinned vectors and graph checks when changing implementations.

The Windows production graph directly depends on bon 3.10.3, indexmap 2.14.2, papaya 0.2.5, and thiserror 2.0.21. Transitive dependencies include bon's build macros, indexmap's hashbrown, papaya's seize, and Windows platform crates. Inspect exact versions and enabled features with the following commands.

```powershell
cargo tree --locked -p gpui-material-theme --no-default-features --features dynamic-color --edges normal,build
cargo tree --locked -p gpui-material-theme --features dynamic-color --edges features --invert material-color-utils
```

## Windows CI and acceptance

The [start PR](https://github.com/wangdiandao/gpui-material/pull/76) passed Windows MSVC and merged. The path gate activated P3 scope against the P2 freeze baseline.

The [theme PR](https://github.com/wangdiandao/gpui-material/pull/78) passed [Windows MSVC](https://github.com/wangdiandao/gpui-material/actions/runs/43410534729) on current main. It covered formatting, phase paths, documentation, offline token consistency, three-platform dependency resolution, Clippy, workspace tests, and a native Windows Gallery build. macOS and Linux results covered dependency resolution.

Windows 11 passed 18 workspace tests, including 10 for pure theme, seven for xtask, and one for Gallery. Theme also ran with default, `dynamic-color`, and `serde` configurations. Both files reproduced the original SHA-256 offline. Appending stale content made `cargo xtask tokens --check` exit 1, and regeneration restored the bytes. A separate comparison with the Compose source package matched all 150 fields across 30 typography styles.

The 555 tokens comprise 96 Reference, 410 System, and 49 Component entries. [Freeze record](../../status.toml)
