# Pure theme

English | [中文](theme.zh.md)

`gpui-material-theme` provides `MaterialTheme` to applications. The application chooses light or dark appearance, color overrides, font families, motion scheme, and density. The rendering layer computes layout from the available logical width and passes the current text scaling and reduced-motion preferences.

| Area | Defaults and usage |
| --- | --- |
| Color | `ColorScheme::baseline` uses the Compose light or dark roles. Use `color` for a role, `pair` for foreground and background, and `set_color` for an override. Callers check foreground contrast after overriding the background. Container roles provide baseline elevation. `surface_at_elevation` applies the Compose tonal overlay formula. |
| Seed color | `dynamic-color` enables `MaterialTheme::from_seed`. Brand seeds and system seeds supplied by the caller use Tonal Spot Spec2021. The input alpha is ignored. Contrast clamps to −1 through 1, with NaN falling back to 0. |
| Typography | `TypeRole::style` resolves baseline and emphasized styles for 15 roles. Font size, line height, and tracking use sp. Brand and Plain both prefer Roboto, followed by Segoe UI, Microsoft YaHei UI, and Noto Sans CJK SC. Applications can replace the list by role family. The host resolves fonts and glyph fallback. |
| Shape | `CornerShape` uses logical corner order, maps it by direction, and limits adjacent corner sums. Fully rounded corners use half the shorter side. Circle, Square, and Triangle provide fixed morph endpoint parameters. The renderer owns normalization, contour interpolation, drawing, and hit testing. |
| Motion | Standard is the default and Expressive is optional. Spatial and effects motion each have Fast, Default, and Slow springs. Effects springs are critically damped. Reduced motion resolves to immediate completion. Query fixed durations and Bézier curves through tokens. |
| Layout | Available-width thresholds are 600, 840, 1200, and 1600 logical units. Exact thresholds use the larger class. Logical start and end map by LTR or RTL. Use the host's text scale for font size, line height, and tracking. |
| Density | The desktop policy adjusts visual component sizes by Compact −4, Standard 0, or Comfortable +4 logical units. Interactive size keeps a minimum of 48 logical units. Each component phase applies the policy and verifies layout and hit testing. |
| Serialization | `serde` independently enables serialization of theme values. Color overrides use semantic role keys. Missing roles resolve from the current appearance's baseline. |

<a id="token-数据"></a>

## Token data

The [single numeric source](../../tokens/material.json) groups data by pinned source. The [generated index](token-index.md) retains Material names, Rust constants, and exact upstream symbols. Tables that expose only Kotlin symbols in Compose map to Material's three-layer naming convention. Static color tables keep light/dark keys under each role. Filled Button and Small Button keys verify Component coverage for color, shape, size, and state. Each component family adds data in its owning phase.

Reference holds foundational values such as palettes and font families. System defines semantics and shared scales. Component can refer to System or Reference. Color references retain `ColorRole` and resolve against the caller's current theme. Same-layer aliases support static colors and composite typography. The generator checks duplicate names, Rust identifier collisions, units, ranges, layer rules, missing targets, and cycles.

```powershell
cargo xtask tokens
cargo xtask tokens --check
cargo xtask docs --write docs/theme/token-index.md
```

The generator reads checked-in data and writes the Rust tables and both language versions of the Markdown index. Review both indexes before refreshing the pairing record. Normal builds compile the checked-in tables. CI verifies the generated files and dependency graphs with `cargo xtask check`.

<a id="来源与桌面裁决"></a>

## Sources and desktop decisions

- Static colors, palettes, typography values, corners, state layers, elevation, and motion come from pinned Compose Material 3 1.5.0-rc01. The Material site specifies Roboto as the default family. Compose uses the platform sans-serif abstraction. The desktop project expresses this choice with a configurable family list.
- Dynamic color uses Tonal Spot Spec2021, verified in P0 and P1. The 49 semantic roles include legacy background and surface-variant, inverse, fixed, outline, surface containers, scrim, and MCU shadow.
- Rounded polygons use the three topologies verified in P1. Circle uses the final AndroidX circle radius, Square uses rectangle width and height, and Triangle uses the circumradius. Parameters are compared against pinned Compose `MaterialShapes.kt` and AndroidX graphics-shapes 1.1.0. Uniformly scaling adjacent corners of rounded rectangles is a desktop geometry policy.
- Available width comes from the pinned Material Breakpoints table. The 48 logical unit minimum comes from Compose `LocalMinimumInteractiveComponentSize`. Density offsets, font fallback order, and immediate completion for reduced motion are desktop project conventions.

See [design sources](../design-sources.md) for pinned versions and checksums.
