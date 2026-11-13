# Architecture and dependencies

English | [中文](architecture.zh.md)

gpui-material provides Material tokens, themes, visual states, layout, and composition. gpui-base provides reusable behavior, focus, input, scrolling, state, and infrastructure. P1 uses [probes](phases/P01.md) to confirm capabilities, adapters, and dependency graphs. [Design sources](design-sources.md#gpui-and-base)

<a id="crate-依赖方向"></a>

## Crate dependency direction

Arrows point from caller to dependency. The table shows when each layer first ships and which modules can follow.

| Crate | First delivery | Dependencies and responsibilities |
| --- | --- | --- |
| `gpui-material-theme` | P3 | Pure Rust. Owns tokens, semantic colors, typography, shape descriptions, motion parameters, and layout policy. Exposes pure value types. |
| `gpui-material-foundation` | P4 | → theme, gpui-base, and GPUI from the same source. Provides MaterialRoot, theme projection, drawing, state layers, ripple, geometry, and motion adapters. |
| `gpui-material-components` | P5 | → foundation and theme as needed. P5 adds controls/display, P6 adds inputs/overlays, and P7 adds navigation. |
| `gpui-material-extras` | P8 | → components, foundation, and relevant theme APIs. Enable by family and declare additional dependencies separately. |
| `gpui-material` | P5 | → shipped public implementation crates. Forwards features as needed. Extras are optional. |
| `gpui-material-gallery` | P2 engineering host, individual demos from P5 | → the shipped facade and host platform entry point. Builds by component family and stays outside the default build set. |
| `xtask` | P2 engineering tool, P3 token generation | Phase and build record checks, source index, and offline generation. Independent of GPUI. |

Inputs, overlays, and navigation may use controls/display. The TextField editing model exposes editing commands to the composition layer, which integrates ExposedDropdown and the editing menu. Navigation provides slots for search and toolbars. Implementation crates share lower-level interfaces. The facade provides the user entry point. Production depends on gpui-base and uses `gpui-component` as an upstream integration reference.

<a id="状态与主题"></a>

## State and theme

- Callers own business values and stable identifiers. Base owns interaction state, input, selection, and focus. Material owns the theme, state presentation policy, and additional composition state. Controlled components request changes through a common path. Document callback order and synchronous reentrancy boundaries.
- Expose semantic contracts such as variant, size, disabled, selected, and on_change, along with slots for component parts. Prefer concrete types for shared interfaces. Keep third-party color and shape types inside adapters.
- `MaterialTheme` is the project theme source. The pinned Base `Theme` is an app `Global` with appearance, semantic tokens, and defaults for scrollbars, split resizing, plots, and caret blinking. The app commits a theme change explicitly and synchronizes projection and refresh. Callers pass local styles to a window or subtree explicitly, as demonstrated by the two-window probe.
- P4 centralizes mapping real Base interaction states to Material presentation. Disabled suppresses hover and active styles. P1 verifies public access to disabled, cancelled, keyboard, and focus-visible state.
- A managed data file is the sole numeric source for tokens. xtask generates the Rust tables and index, which are checked in for ordinary builds. Shape scope follows the rounded polygon topologies covered by the probes.

<a id="候选依赖"></a>

## Candidate dependencies

P0 pins Base and GPUI. The owning phases evaluate other candidates and record their purpose, enabled features, resolved graph, maintenance, licensing, replaceable interfaces, and verification results.

| Candidate | Admission phase | Purpose and initial constraints | Verification and replacement boundary |
| --- | --- | --- | --- |
| `gpui-base =0.9.1` / `gpui-pre =0.3.12` / `gpui-pre-platform =0.3.12` | P1 probes, P2 workspace | Pin crates.io sources and retain the upstream pairing of font-kit, x11, wayland, and runtime_shaders. Use test-support for acceptance. Inspector is optional. Apache-2.0. | Use `cargo metadata` and feature graphs to check a single core and platform identity. Windows runs the required probes. macOS and Linux probes are exploratory. Foundation wraps public Base APIs. |
| [`material-color-utils 0.1.3`][mcu-rust] | Admitted in P3 | HCT, tonal palettes, and dynamic semantic colors. Disable default features to avoid image, serde, and rayon. Direct dependencies are bon, indexmap, papaya, and thiserror. Published metadata uses license-file. The bundled license is Apache-2.0. | Adapt library types inside theme. [Theme PR](https://github.com/wangdiandao/gpui-material/pull/78) |
| [`polymorpher 0.1.4`][polymorpher] | P1 feasibility, P4 admission | Rounded polygons and morphing. Retain the empty default feature set and use basic geometry. Directly depends on euclid. Apache-2.0. | Compare curves, bounds, topology, clipping, hit testing, and performance against AndroidX. Foundation owns GPUI path conversion. |
| [Material Symbols][symbols] | P4/P5 | Pin the necessary assets and let applications replace icons. Apache-2.0. | Verify asset mapping, icon fallback, and RTL. Record distributed assets and licenses individually. |
| `chrono` and [Base Calendar][calendar] | P1 behavior, P8 date-family admission | Base declares the `0.4.38` compatible range. The pinned upstream lockfile resolves 0.4.45. MIT or Apache-2.0. | Check version unification, calendar semantics, and boundaries. Reuse Base and chrono for date arithmetic and use a separate locale formatter. |
| [`rust-i18n 4.2.4`][i18n] | P4 defines localization entry points. Reassess when catalogs are needed. | Optional application adapter, MIT. Start with overridable text and formatters. The candidate introduces a macro and support crate. | Measure build cost and dependency graph. Verify string coverage and actual layout. Keep translation library types inside the application adapter. |

Extras define families such as expressive-groups, search, date-time, and carousel as needed. Theme supports static data with dynamic-color and serde on demand. Keep image-based color extraction in tooling where possible. Required keyboard, semantics, and motion are part of complete component behavior.

Use package-level commands in CI and development to isolate theme, components, and Gallery. Use build timings and runtime measurements to guide architecture changes. Prefer concrete internal types with type erasure where needed.

<a id="p1-已验证的适配边界"></a>

## Boundaries verified in P1

P1 acceptance uses Windows 11 x64/MSVC. P2 can reuse the pinned Base 0.9.1 and GPUI 0.3.12 pairing. Pure theme algorithms and shape candidates are admitted in P3 and P4 respectively. Hosted macOS and Linux runs and pixel samples are exploratory. [P1 acceptance PR](https://github.com/wangdiandao/gpui-material/pull/51)

| Mechanism | Adapter and boundary |
| --- | --- |
| Activation and drag | Base manages button activation and focus. Composition owns roving focus. Windows capture loss, disabling, and Esc feed the cancellation path. The pinned Windows GPUI lacks capture-loss events. The probe verified a temporary Win32 capture check on the UI thread. Retest native capture loss and the next drag when replacing it. [Issue #48](https://github.com/wangdiandao/gpui-material/issues/48) |
| Text and semantics | Base owns editing state, clipboard, masking, and input handlers. GPUI owns the native bridge, glyph fallback, and layout. Use the same shaped line for measurement and drawing. Windows UIA RangeValue SetValue reaches Base SliderState through a GPUI semantic action. Remove the adapter and retest when upstream Slider handles it directly. [Issue #49](https://github.com/wangdiandao/gpui-material/issues/49) |
| Drawing and motion | Foundation can reuse GPUI fill meshes for hit testing and state-layer clipping. The Windows probe approximates a shape-matched shadow with three translucent fills. Geometry retargeting, color, and opacity reuse Base spring/transition scheduling. The probes verify system reduced motion and stopping redraws at rest. |
| Overlays and scrolling | Base Popover and Dialog own placement, scrim, and focus scope. Composition maps logical alignment and manages open state, focus restoration, and drag cancellation. GPUI hit occlusion can isolate inner scrolling. Value labels subscribe to Base SliderState. |
| Theme and dates | Base Theme is an app-level Global. Material uses explicit projection and local styles for window-level policy. Pure color and date probes run independently. Reuse Base Calendar for date boundaries and disabled dates. |

[mcu-rust]: https://github.com/RuurdBijlsma/material-color-utilities-rs/tree/d39ff59a1fca34bdc60e124de2f9f7541fff3c39
[polymorpher]: https://github.com/Aiving/polymorpher/tree/aaeb6f69b0ced6cc7079713994832b5d7ee3ff50
[symbols]: design-sources.md#source-code-and-algorithm-references
[calendar]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/crates/base/src/calendar.rs
[i18n]: https://github.com/longbridge/rust-i18n/tree/5f3ea5ba03395f2f2908e3587e01788cfdfd3555
