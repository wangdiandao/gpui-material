# Design sources and version baseline

English | [中文](design-sources.zh.md)

Verified on 2026-11-13. The frozen baseline includes Material site snapshots, published source packages, algorithm references, and the GPUI version pairing. Before an upgrade, review coverage, tokens, behavior, platforms, licensing, and dependency graphs, then update this page and affected specifications.

<a id="采用顺序"></a>

## Source precedence

1. [Material 3](https://m3.material.io/) Expressive and applicable M3 guidance define component names, structure, variants, and visual and interaction semantics.
2. Pinned Compose Material 3 verifies tokens, states, defaults, and layout. The Kotlin files in the component matrix come from that source package.
3. Official Material Color Utilities and AndroidX graphics-shapes are the algorithm and test vector references.
4. Pinned GPUI and gpui-base define desktop event, drawing, and platform boundaries. Document input paths and the reasons for desktop adaptations.
5. Pinned Material Components Android supplements implementation references such as Side Sheet and Docked Toolbar, including deprecation and replacement guidance. Material Web provides historical context.

Resolve differences by using pinned public guidance for scope and pinned sources for implementation details. The component's owning phase documents deviations and replacement interactions. Record test results in the associated issue or PR. Review scope before adopting new upstream functionality. Record device applicability as specified by the site.

<a id="material-规范"></a>

## Material specification

The [site directory](https://m3.material.io/components) and component Overview, Specs, Guidelines, and Accessibility tabs were checked against data version `2026-10-13_06-10-19`. The [snapshot index](material-snapshot.md) records pinned JSON and SHA-256 by page. Embedded token tables are from the same release. P3 and the relevant component phases verify values, references, and generation rules. This repository keeps project specifications, page locations, and checksums.

| Area | Scope and verification | Official entry point |
| --- | --- | --- |
| Tokens | Reference, System, and Component, with traceable original names and Rust fields. P3 verifies reference closure and reproducible generation. | [Design tokens](https://m3.material.io/foundations/design-tokens/overview) |
| Color | Static light and dark colors, role pairs, brand and dynamic seeds, contrast, and fixed roles. P3 uses MCU to verify semantic colors and boundaries. | [Color roles](https://m3.material.io/styles/color/roles), [Dynamic color](https://m3.material.io/styles/color/dynamic) |
| Typography | Baseline and emphasized roles, weight, line height, tracking, and overridable font fallback. P3 compares roles and P4 verifies glyphs and metrics. | [Typography](https://m3.material.io/styles/typography/overview) |
| Shape | Corner scale, state shapes, limited rounded polygon topologies, and morphing. P3 compares geometry parameters and P4 checks drawing, clipping, and hit testing. | [Shape](https://m3.material.io/styles/shape/overview-principles), [Shape morph](https://m3.material.io/styles/shape/shape-morph) |
| Motion | Standard and Expressive schemes, distinguishing spatial motion from effects such as color and opacity. P4 verifies retargeting, reduced motion, and stopping redraws. | [Motion](https://m3.material.io/styles/motion/overview/specs) |
| Elevation | Surface roles, tonal elevation, shadow, and scrim. Verify light and dark separately. | [Elevation](https://m3.material.io/styles/elevation/overview) |
| Interaction | Hover, focus-visible, pressed, dragged, disabled, and component value states. P4 verifies input cancellation and state sources. | [States](https://m3.material.io/foundations/interaction/states/overview), [Inputs](https://m3.material.io/foundations/interaction/inputs) |
| Layout | Available window size, density, RTL, font scaling, and visual and hit areas. P7 verifies state continuity across layout changes. | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview), [RTL](https://m3.material.io/foundations/layout/bidirectionality-rtl) |
| Accessibility | Names, roles, values, states, actions, relationships, and announcements. P1 verifies platform bridges and component phases verify semantics and actual interaction. | [Assistive technology](https://m3.material.io/foundations/overview/assistive-technology) |

<a id="源码与算法参考"></a>

## Source code and algorithm references

| Source | Pinned version or revision | Checksum or reference |
| --- | --- | --- |
| Compose Material 3 common | `androidx.compose.material3:material3:1.5.0-rc01` | [Google Maven source package][compose] · `38e0515354240bf1e15b6e8891bc02608973f08dc0b8da8cd87a51dfe9ab1031` |
| Compose Material 3 Android | `androidx.compose.material3:material3-android:1.5.0-rc01` | [Google Maven source package][android] · `0e9c5b37e75203b44b4987b3ccc0db41637303103ca21739691d7533cbe7cbb1` |
| Adaptive Navigation Suite | `androidx.compose.material3:material3-adaptive-navigation-suite:1.5.0-rc01` | [Google Maven source package][nav-suite] · `fdbc86bff0c7f9f4ca8fdf9097515297af39200f65a82129d269f4a4fe11de64` |
| Adaptive Layout | `androidx.compose.material3.adaptive:adaptive-layout:1.3.0` | [Google Maven source package][adaptive] · `bd87db4bdbf51d7df02872b0c7e95ae82a3b7c61b7f2e021c407d30a899c22ca` |
| Adaptive Navigation | `androidx.compose.material3.adaptive:adaptive-navigation:1.3.0` | [Google Maven source package][adaptive-nav] · `c64c0ddec14fed5daae373c535c8e81e1f0ef1922aa258b624828448de18346d` |
| AndroidX graphics-shapes | `androidx.graphics:graphics-shapes:1.1.0`, release change range ends at `cb163bee53214245ad7a23367396a21b7f85ae82` | [Google Maven source package][shapes] · `50586aa6c7b3a542a51b2abaf659f7a6c33d8c5d637d8f1a57aaf518dcb74f3d` · [Tests at the same revision][shapes-tests] |
| Google Material Color Utilities | [`194144d75ed7d34f06264a625e086db81a7ab9aa`][mcu] | `typescript/hct/hct_test.ts`, `typescript/scheme/dynamic_scheme_test.ts`, `typescript/dynamiccolor/{dynamic_color,fixed_dynamic_color}_test.ts`, `dart/test/{contrast,palettes}_test.dart` |
| Material Components Android | [`66b0eeb53afeb6025ffbbd37734b08e5bce197ed`][mdc] | `docs/components/` and the corresponding Java sources. The matrix records their purpose. |
| Material Symbols | [`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`][symbols] | `symbols/`. Record each icon, style, axis value, and license when used. |

The [Compose release notes][compose-release] list stable 1.4.0 and release candidate 1.5.0-rc01. The latter's source package provides the Expressive baseline, with a change range ending at [`1f549b6f5ba84cb3553be9c79110c4c5f6413039`][compose-tree]. Locate sources by versioned packages and checksums. Use the Android implementation to clarify interactions. [Adaptive release notes][adaptive-release] · [graphics-shapes release notes][shapes-release]

P1 shape vectors come from pinned `RoundedPolygon.kt`, `CornerRounding.kt`, and `Morph.kt`. Starting cases are endpoint recovery in `MorphTest.cubicsTest`, radius allocation in `RoundedPolygonTest.roundingSpaceUsageTest`, and parameters in `CornerRoundingTest.cornerRoundingTest`. Cover rounded triangles, squares, circles, and polygons with matching and different vertex counts at progress 0, 0.5, and 1. Record radius, smoothing, cubic endpoints and control points, closure, bounds, and sampling error.

Pinned MCU vectors include tone 40 of blue `0xff0000ff` yielding `0xff343dff`, red `0xffff0000` under dark Tonal Spot at contrast 0 yielding `primaryFixed = 0xffffdad4`, and clamped contrast between tones −10 and 110 yielding 21 within 0.001. P3 adds seed round trips, light and dark modes, low, default, and high contrast, and fixed roles. Record versions, algorithm specifications, expected values, and tolerances.

<a id="gpui-与-base"></a>

## GPUI and Base

[GPUI Kit v0.9.1][kit-release] and the crates.io `gpui-base` metadata both point to [`39ab849a93b5f15507b969164d49891db8d5b470`][kit]. The [pinned workspace manifest][kit-manifest] and release package metadata establish the table. P0 pins the version contract. P1 probes and P2 workspace record the versions in their own manifests and lockfiles.

| Cargo package | Exact version | crates.io SHA-256 |
| --- | --- | --- |
| `gpui-base` | `=0.9.1` | `f3e9ae5237737cfd110d0ff513fcd9c20f97d98ebabd0a73164068177f0aca4f` |
| `gpui-pre` (Rust import `gpui`) | `=0.3.12` | `29db89613527079737f53752eab27fb65c9a7dd94979256c65a0e590c9fd34d8` |
| `gpui-pre-platform` (Rust import `gpui_platform`) | `=0.3.12` | `d2b785bdcbd229f762096aebf4f9ae913bdaa9db49794907cabaf516eb55227b` |

Both `gpui-pre` packages set `package.metadata.gpui-pre.zed-rev` to [`d2a4fb6ded7c470be25d1541281da3882495b65b`][zed]. Use exact versions from the same source as upstream and verify Cargo resolves one GPUI core identity and one platform identity. In this Base version, `Theme` implements app `Global`. [GPUI Kit Actions][gpui-publish] · [Theme access in theme.rs][base-theme]

[compose]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3/1.5.0-rc01/material3-1.5.0-rc01-sources.jar
[android]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3-android/1.5.0-rc01/material3-android-1.5.0-rc01-sources.jar
[nav-suite]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3-adaptive-navigation-suite/1.5.0-rc01/material3-adaptive-navigation-suite-1.5.0-rc01-sources.jar
[adaptive]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/adaptive/adaptive-layout/1.3.0/adaptive-layout-1.3.0-sources.jar
[adaptive-nav]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/adaptive/adaptive-navigation/1.3.0/adaptive-navigation-1.3.0-sources.jar
[shapes]: https://dl.google.com/dl/android/maven2/androidx/graphics/graphics-shapes/1.1.0/graphics-shapes-1.1.0-sources.jar
[shapes-tests]: https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/
[mcu]: https://github.com/material-foundation/material-color-utilities/tree/194144d75ed7d34f06264a625e086db81a7ab9aa
[mdc]: https://github.com/material-components/material-components-android/tree/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components
[symbols]: https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/symbols
[compose-release]: https://developer.android.com/jetpack/androidx/releases/compose-material3#1.5.0-rc01
[compose-tree]: https://github.com/androidx/androidx/tree/1f549b6f5ba84cb3553be9c79110c4c5f6413039/compose/material3
[adaptive-release]: https://developer.android.com/jetpack/androidx/releases/compose-material3-adaptive#1.3.0
[shapes-release]: https://developer.android.com/jetpack/androidx/releases/graphics#graphics-shapes-1.1.0
[kit-release]: https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1
[kit]: https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470
[kit-manifest]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/Cargo.toml
[zed]: https://github.com/zed-industries/zed/commit/d2a4fb6ded7c470be25d1541281da3882495b65b
[gpui-publish]: https://github.com/longbridge/gpui-kit/actions/runs/42539746081
[base-theme]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/crates/base/src/theme.rs
