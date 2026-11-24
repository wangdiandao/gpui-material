# Design sources

English | [中文](design-sources.zh.md)

Last synced on 2026-11-13. Pinned sources include Material site snapshots, published source packages, algorithm references, and the GPUI version pairing. Propose any Material specification version update through the [contribution process](../CONTRIBUTING.md#design-specification). After maintainers approve adoption, update this page and the affected specifications.

<a id="采用顺序"></a>

## Source precedence

1. [Material 3](https://m3.material.io/) Expressive and applicable M3 guidance define component names, structure, variants, and visual and interaction semantics.
2. Pinned Compose Material 3 serves as the implementation reference for tokens, states, defaults, and layout.
3. Official Material Color Utilities and AndroidX graphics-shapes serve as the algorithm and geometry references.
4. Pinned GPUI and `gpui-base` define desktop event, drawing, and platform boundaries. Document desktop adaptations and their rationale in the owning specification.
5. Pinned Material Components Android supplements implementation references such as Side Sheet and Docked Toolbar, including deprecation and replacement guidance. Material Web provides historical context.

Resolve differences by using pinned public guidance for scope and pinned sources for implementation details. Document accepted desktop adaptations and replacement interactions in the relevant component or architecture documentation.

<a id="material-规范"></a>

## Material specification

The [site directory](https://m3.material.io/components) and component Overview, Specs, Guidelines, and Accessibility tabs are pinned to data version `2026-10-13_06-10-19`. The [snapshot index](material-snapshot.md) records the pinned JSON URLs and SHA-256 checksums by page. Embedded token tables come from the same release; regenerate checked-in token outputs whenever the source data changes.

| Area | Scope | Official entry point |
| --- | --- | --- |
| Tokens | Reference, System, and Component layers, with traceable original names and Rust fields. | [Design tokens](https://m3.material.io/foundations/design-tokens/overview) |
| Color | Static light and dark colors, role pairs, brand and dynamic seeds, contrast, and fixed roles. | [Color roles](https://m3.material.io/styles/color/roles), [Dynamic color](https://m3.material.io/styles/color/dynamic) |
| Typography | Baseline and emphasized roles, weight, line height, tracking, and overridable font fallback. | [Typography](https://m3.material.io/styles/typography/overview) |
| Shape | Corner scale, state shapes, the 35 Expressive rounded polygons, custom rounded polygons, general morphing, geometric clipping, and shape-aware hit testing. | [Shape](https://m3.material.io/styles/shape/overview-principles), [Shape morph](https://m3.material.io/styles/shape/shape-morph) |
| Motion | Standard and Expressive schemes, distinguishing spatial motion from effects such as color and opacity, with retargeting and reduced motion. | [Motion](https://m3.material.io/styles/motion/overview/specs) |
| Elevation | Surface roles, tonal elevation, shadow, and scrim across light and dark modes. | [Elevation](https://m3.material.io/styles/elevation/overview) |
| Interaction | Hover, focus-visible, pressed, dragged, disabled, and component value states. | [States](https://m3.material.io/foundations/interaction/states/overview), [Inputs](https://m3.material.io/foundations/interaction/inputs) |
| Layout | Available window size, density, RTL, font scaling, and visual and hit areas. | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview), [RTL](https://m3.material.io/foundations/layout/bidirectionality-rtl) |
| Accessibility | Names, roles, values, states, actions, relationships, announcements, and platform bridges. | [Assistive technology](https://m3.material.io/foundations/overview/assistive-technology) |

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
| Material Components Android | [`66b0eeb53afeb6025ffbbd37734b08e5bce197ed`][mdc] | `docs/components/` and corresponding Java sources |
| Material Symbols | [`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`][symbols] | `symbols/` (bundled icons, styles, axes, and licenses recorded in the asset manifests) |

The [Compose release notes][compose-release] list stable 1.4.0 and release candidate 1.5.0-rc01; the latter's source package supplies the Expressive reference, with a change range ending at [`1f549b6f5ba84cb3553be9c79110c4c5f6413039`][compose-tree]. Locate sources by versioned packages and checksums. [Adaptive release notes][adaptive-release] · [graphics-shapes release notes][shapes-release]

Geometry and color algorithms are cross-checked against the pinned upstream suites in [foundation shape](../crates/foundation/src/shape.rs) (adapted from AndroidX `MorphTest` and `RoundedPolygonTest`) and [theme color reference](../crates/theme/tests/color_reference.rs) (adapted from Material Color Utilities).

<a id="gpui-与-base"></a>

## GPUI and Base

[GPUI Kit v0.11.0][kit-release] and the crates.io `gpui-base` metadata both point to [`e97fe438129c5e4ccab504f2e6fafd9a4e396bba`][kit]. The [pinned workspace manifest][kit-manifest] and release package metadata establish the versions below.

| Cargo package | Exact version | crates.io SHA-256 |
| --- | --- | --- |
| `gpui-base` | `=0.11.0` | `604d277dd339c63d13a3016d22a84622cc7511fdfd8e88573c0709088ede3500` |
| `gpui-pre` (Rust import `gpui`) | `=0.3.14` | `ab22c23c1304a0b3f34f64568f59a4ce2abbfab3727273a90c66b37924d31089` |
| `gpui-pre-platform` (Rust import `gpui_platform`) | `=0.3.14` | `86a2a4a341e905e5dfd6f128ff829509a4053f75e29aa34fa58537a0a3a03ed5` |

Both `gpui-pre` packages set `package.metadata.gpui-pre.zed-rev` to [`f126950f9d3b62666f97437ae5d4ccac2272c4db`][zed]. Use exact versions from the same upstream source so Cargo resolves a single GPUI core and platform package identity. In this Base version, `Theme` implements app `Global`. [GPUI Kit Actions][gpui-publish] · [Theme access in theme.rs][base-theme]

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
[kit-release]: https://github.com/longbridge/gpui-kit/releases/tag/v0.11.0
[kit]: https://github.com/longbridge/gpui-kit/tree/e97fe438129c5e4ccab504f2e6fafd9a4e396bba
[kit-manifest]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/Cargo.toml
[zed]: https://github.com/zed-industries/zed/commit/f126950f9d3b62666f97437ae5d4ccac2272c4db
[gpui-publish]: https://github.com/longbridge/gpui-kit/actions/runs/45088262624
[base-theme]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/crates/base/src/theme.rs
