# Architecture and dependencies

English | [中文](architecture.zh.md)

GPUI Material provides Material tokens, themes, visual states, layout, and composition. GPUI Base provides reusable behavior, focus, input, scrolling, state, and infrastructure. [Design sources](design-sources.md#gpui-and-base)

<a id="crate-依赖方向"></a>

## Crate dependency direction

Arrows point from caller to dependency. The table describes each layer's dependency and responsibility.

| Crate | Dependencies and responsibilities |
| --- | --- |
| `gpui-material-theme` | Pure Rust. Owns tokens, semantic colors, typography, shape descriptions, motion parameters, and layout policy. Exposes pure value types. |
| `gpui-material-foundation` | → theme, gpui-base, and GPUI from the same source. Provides MaterialRoot, theme projection, drawing, state layers, ripple, geometry, and motion adapters. |
| `gpui-material-components` | → foundation and theme as needed. Owns controls, display, inputs, overlays, and navigation. |
| `gpui-material-extras` | → components, foundation, and relevant theme APIs. Enable by family and declare additional dependencies separately. |
| `gpui-material` | → shipped public implementation crates. Forwards features as needed. Extras are optional. |
| `gpui-material-gallery` | → the shipped facade. The package provides the `gpui_material_story` library and a desktop executable outside the default workspace build. |
| `xtask` | Documentation and dependency checks, build timings, and offline token generation. Independent of GPUI. |

Inputs, overlays, and navigation may use controls/display. The TextField editing model exposes editing commands to the composition layer, which integrates ExposedDropdown and the editing menu. Navigation provides slots for search and toolbars. Implementation crates share lower-level interfaces. The facade provides the user entry point. Production depends on gpui-base and uses `gpui-component` as an upstream integration reference.

A host can depend on `apps/gallery` with `default-features = false` to use `gpui_material_story` without the desktop launcher. Call `init_gallery` before opening the first window. Mount `Gallery::new` for the full catalog or `Story::new_view` for a single component example under a `gpui_base::Root`. The Story identifier can be used in a URL and resolved with `Story::find`.

<a id="状态与主题"></a>

## State and theme

Search and exposed dropdowns retain complete result identities and scroll state while GPUI's variable-height list constructs visible rows and bounded overscan. Search preserves focus handles by result ID; dropdowns keep focus in the Base editor and retain the active option ID. Measurement is invalidated for changed content, width, typography, the installed-font snapshot, rem size, and dropdown animation scale. A logical row anchor reveals an unmeasured keyboard target without laying out its entire prefix. The lightweight `List` remains suitable for small collections; passing it a visible slice alone does not implement full-model keyboard navigation.

The dropdown keeps a narrow IME-aware keyboard adapter over Base Input and Popup. The pinned Base Select/Combobox root adds its own semantic/focus owner and cannot veto action handling during composition, so wrapping the existing editable TextField would change its contract. Button and selection feedback share one presentation event adapter; Base still owns activation and controlled values, while Material retains ripple timing and shape transitions.

- Callers own business values and stable identifiers. Base owns interaction state, input, selection, and focus. Material owns the theme, state presentation policy, and additional composition state. Controlled components request changes through a common path. Document callback order and synchronous reentrancy boundaries.
- Expose semantic contracts such as variant, size, disabled, selected, and on_change, along with slots for component parts. Prefer concrete types for shared interfaces. Keep third-party color and shape types inside adapters.
- `MaterialTheme` is the project theme source. The pinned Base `Theme` is an app `Global` with appearance, semantic tokens, and defaults for scrollbars, split resizing, plots, and caret blinking. The app commits a theme change explicitly and synchronizes projection and refresh. Callers pass local styles to a window or subtree explicitly, as demonstrated by the two-window probe.
- Foundation maps Base interaction state to Material presentation. Disabled suppresses hover and active styles. Verify changes to disabled, cancellation, keyboard, and focus-visible behavior.
- A managed data file is the sole numeric source for tokens. xtask generates the Rust tables and index, which are checked in for ordinary builds. Shape scope follows the rounded polygon topologies covered by the probes.

Foundation shares typography, ripple feedback, and the desktop shadow projection across component families. Press ripples are separate from hover, focus, and drag state layers; reduced motion uses a static press indication. Single-line text shares tracked glyph measurement and drawing. Wrapping text delegates line breaking to GPUI with Material font families, weight, size, and line height; the pinned native wrapping API does not expose custom letter spacing. GPUI's two-layer shadow projection is a desktop adaptation, not a pixel-identical reproduction of Android's platform shadow renderer.

<a id="依赖规则"></a>

## Dependency policy

Pin Base and GPUI as described in the [version baseline](design-sources.md#gpui-and-base). For dependency proposals, record the purpose, enabled features, resolved graph, maintenance, licensing, replacement boundaries, and verification results in the issue or PR.

| Dependency or candidate | Purpose and constraints | Verification and replacement boundary |
| --- | --- | --- |
| `gpui-base =0.9.1` / `gpui-pre =0.3.12` / `gpui-pre-platform =0.3.12` | Pin crates.io sources and retain the upstream pairing of font-kit, x11, wayland, and runtime_shaders. Use test-support for acceptance. Inspector is optional. Apache-2.0. | Use `cargo metadata` and feature graphs to check a single core and platform identity. Apply the relevant [platform checks](verification.md) when updating the pins. Foundation wraps public Base APIs. |
| [`material-color-utils 0.1.3`][mcu-rust] | HCT, tonal palettes, and dynamic semantic colors. Disable default features to avoid image, serde, and rayon. Direct dependencies are bon, indexmap, papaya, and thiserror. Published metadata uses license-file. The bundled license is Apache-2.0. | Adapt library types inside theme. [Theme PR](https://github.com/wangdiandao/gpui-material/pull/78) |
| [`polymorpher 0.1.4`][polymorpher] | Rounded polygons and morphing. Retain the empty default feature set and use basic geometry. Directly depends on euclid. Apache-2.0. | Compare curves, bounds, topology, clipping, hit testing, and performance against AndroidX. Foundation owns GPUI path conversion. |
| [Material Symbols][symbols] | Pin the necessary assets and let applications replace icons. Apache-2.0. | Verify asset mapping, icon fallback, and RTL. Record distributed assets and licenses individually. |
| `chrono 0.4.45` and [Base Calendar][calendar] | The `date-time` feature enables a direct chrono dependency in extras with its default features disabled. It resolves to the same package as the existing Base and GPUI graph. MIT or Apache-2.0. | Base owns calendar and range draft transitions. Chrono supplies checked calendar arithmetic and typed dates and times. Application formatters and parsers control locale presentation and entry. |
| [`rust-i18n 4.2.4`][i18n] | Optional application adapter, MIT. Start with overridable text and formatters. The candidate introduces a macro and support crate. | Measure build cost and dependency graph. Verify string coverage and actual layout. Keep translation library types inside the application adapter. |

Extras define families such as expressive-groups, search, date-time, and carousel as needed. Theme supports static data with dynamic-color and serde on demand. Keep image-based color extraction in tooling where possible. Required keyboard, semantics, and motion are part of complete component behavior.

Use package-level commands in CI and development to isolate theme, components, and Gallery. Use build timings and runtime measurements to guide architecture changes. Prefer concrete internal types with type erasure where needed.

<a id="桌面适配边界"></a>

## Desktop adapter boundaries

Use the pinned Base and GPUI versions when verifying these boundaries. The [foundation shape tests](../crates/foundation/src/shape.rs) exercise Material geometry in the workspace test suite.

| Mechanism | Adapter and boundary |
| --- | --- |
| Activation and drag | Base manages button activation and focus. Composition owns roving focus. Windows capture loss, disabling, and Esc feed the cancellation path. The pinned Windows GPUI lacks capture-loss events. The probe verified a temporary Win32 capture check on the UI thread. Retest native capture loss and the next drag when replacing it. [Issue #48](https://github.com/wangdiandao/gpui-material/issues/48) |
| Text and semantics | Base owns editing state, clipboard, masking, and input handlers. GPUI owns the native bridge, glyph fallback, and layout. Use the same shaped line for measurement and drawing. Windows UIA RangeValue SetValue reaches Base SliderState through a GPUI semantic action. Remove the adapter and retest when upstream Slider handles it directly. [Issue #49](https://github.com/wangdiandao/gpui-material/issues/49) |
| Drawing and motion | Foundation can reuse GPUI fill meshes for hit testing and state-layer clipping. The Windows probe approximates a shape-matched shadow with three translucent fills. Geometry retargeting, color, and opacity reuse Base spring/transition scheduling. The probes verify system reduced motion and stopping redraws at rest. |
| Overlays and scrolling | Base Popover and Dialog own placement, scrim, and focus scope. Composition maps logical alignment and manages open state, focus restoration, and drag cancellation. GPUI hit occlusion can isolate inner scrolling. Value labels subscribe to Base SliderState. |
| Theme and dates | Base Theme is an app-level Global. Material uses explicit projection and local styles for window-level policy. The theme crate tests color algorithms directly. Reuse Base Calendar for date boundaries and disabled dates. |

`Icon::new(IconName::Favorite)` renders embedded official Material Symbols SVGs through GPUI; `.filled(true)` selects the official filled variant. Named icons inherit their control slot's text size and foreground so size variants, disabled ink and animated colors remain consistent. Outside a slot, set `.size(px(24.))`; `.color(ColorRole::Primary)` is an explicit override. Icons are decorative unless `.label(id, name)` is set; icon-only buttons retain their own accessible labels. `.mirror_in_rtl(true)` is opt-in for directional symbols, and `.rotate(...)` composes with it. Existing `Icon::svg` and `Icon::custom` remain available with their original 24dp/OnSurfaceVariant defaults. Embedded icons need no application asset registration or font. The [asset manifest](../crates/components/assets/material-symbols/manifest.toml) pins unchanged upstream files and hashes; the [license](../crates/components/assets/material-symbols/LICENSE) ships with them. Glyph placeholders and bespoke static icon paths are not used for built-in component icons; animated selection geometry stays in its owning component.

[mcu-rust]: https://github.com/RuurdBijlsma/material-color-utilities-rs/tree/d39ff59a1fca34bdc60e124de2f9f7541fff3c39
[polymorpher]: https://github.com/Aiving/polymorpher/tree/aaeb6f69b0ced6cc7079713994832b5d7ee3ff50
[symbols]: design-sources.md#source-code-and-algorithm-references
[calendar]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/crates/base/src/calendar.rs
[i18n]: https://github.com/longbridge/rust-i18n/tree/5f3ea5ba03395f2f2908e3587e01788cfdfd3555
