# Changelog

See [versioning and releases](docs/release.md) for the release process.

## Unreleased

### Components

- Compose exposed dropdowns with custom editors, anchors and options, caller-controlled visibility, and configurable popup size, scroll state and appearance.
- Add display-only scrollbars with per-axis viewport tracking, custom geometry and colors, and configurable fading.
- Configure carousel layout and fling behavior, read and control the retained item position, and apply standalone scroll parallax to custom content.
- Customize Snackbar content, controls, layout, and appearance, and observe action or dismissal results from the queue.
- Compose Chips with custom content and Input avatars, apply Expressive Tonal presets and per-state appearance, and observe their interactions.
- Reverse vertical sliders and customize tracks, individual handles, colors, markers, value indicators, and interaction observation.
- Customize linear and circular progress strokes, gaps, and wave motion, and resize or replace the linear stop marker.
- Customize Card shape, border, colors, and per-state elevation, and observe shared interactions.
- Customize Checkbox state colors and the mark and outline strokes, and observe interaction through a shared source.
- Customize selected, unselected, and disabled radio indicator colors and observe shared radio interactions.
- Supply measured progress, custom morph shapes, and instance colors and container corners for Loading Indicators.
- Position Scaffold FABs at the start, center, end, or over the bottom bar. Resolve content insets against measured bars and rail space.
- Set divider thickness and color for each horizontal or vertical rule. A zero thickness selects a one-device-pixel hairline.
- Compose text fields with configurable label placement, leading and trailing elements, value affixes, custom hints and supporting content, and required, invalid, and character count presentation.
- Observe Button and ToggleButton interaction changes through an optional shared interaction source.
- Customize IconButton and IconToggleButton shapes, colors, and outlines, and observe their interactions through a shared source.
- Set text-field colors, shape, content padding and input typography per instance, observe the Base editor's laid-out geometry, and share field interaction state.
- Customize Button and ToggleButton content, shapes, colors, borders, padding, and elevations per instance.
- Render standalone and anchored badges with custom content and per-instance colors.
- Select values with the standalone `date-time` scroll field, with caller-owned selection, custom rows, and accessible value descriptions.
- Compose a standalone `Scrim` with window-theme or custom color, adjustable opacity, and optional accessible dismissal.
- Customize FAB appearance and observe its interactions. Control FAB visibility with a logical scale pivot and independent scale and opacity motion.
- Add SwipeToDismiss for caller-owned content with horizontal gestures, logical direction, and accessible dismissal commands.
- Add the vertical drag handle with per-state appearance, shared interactions, and caller-controlled pane resizing.
- Show persistent rich tooltips with caller-owned visibility and customizable focus, content, placement, caret and appearance.
- Customize switch thumb content and individual state colors, and observe switch interactions through a shared source.

### Gallery

- The GPUI Material Gallery uses the project logo as its Windows application icon.

### Menus

- Add the Vibrant menu palette, styled groups and rows, custom popup content, positioning and an external scroll handle.
- Control menu visibility from application state, handle dismissal requests, and customize menu-item content while retaining keyboard navigation and accessible item names.

### Distribution

- The Gallery now ships as source. Run `cargo gallery` from a checkout.

## 0.1.0 - 2026-11-15

Changes since [`v0.0.4`](https://github.com/wangdiandao/gpui-material/releases/tag/v0.0.4).

### Breaking changes

- `IconName` adds `DarkMode`, `LightMode`, `LeftPanelClose`, and `LeftPanelOpen`. Handle the new variants in exhaustive matches, or add a fallback if they can share the same behavior. [#221](https://github.com/wangdiandao/gpui-material/pull/221)

### Gallery and components

- Reuse the Material Gallery catalog or a single interactive story from the source checkout in another GPUI host, including WebAssembly builds. [#238](https://github.com/wangdiandao/gpui-material/pull/238)
- Browse component previews with a searchable catalog and a dedicated compact-window layout. The Gallery app bar provides theme controls and opens the project repository. [#227](https://github.com/wangdiandao/gpui-material/pull/227)
- Keep the docked search button fully visible in a vertical toolbar and align carousel scrolling with the painted positions at the trailing edge. [#221](https://github.com/wangdiandao/gpui-material/pull/221), [#227](https://github.com/wangdiandao/gpui-material/pull/227)
- Keep exposed dropdown interactions inside their visible hit areas. Opening from the trailing control, selecting an option, and dismissing outside the menu follow their respective pointer paths. [#233](https://github.com/wangdiandao/gpui-material/pull/233)

### Search and text fields

- Virtualize large search result and dropdown option lists while preserving item identity, keyboard navigation, and focus. Refresh measured rows when fonts change. [#229](https://github.com/wangdiandao/gpui-material/pull/229)
- Apply the Material window palette to Base-backed editors in that window while preserving the app-level Base theme. [#229](https://github.com/wangdiandao/gpui-material/pull/229)
- Hide the caret in selection-only dropdowns while retaining keyboard focus. [#227](https://github.com/wangdiandao/gpui-material/pull/227)

### Motion

- Apply the target value immediately when custom spring stiffness cannot produce a valid duration. [#227](https://github.com/wangdiandao/gpui-material/pull/227)

### Integration and distribution

- Install the Material facade from crates.io with `gpui-material = "0.1.0"`. The facade continues to expose the same GPUI and Base package types.
- Document the supported facade, optional features, and migration from the `0.0.4` source checkout. Virtualized lists can track caller-owned row focus through `ListItem::track_focus`. [#215](https://github.com/wangdiandao/gpui-material/pull/215), [#229](https://github.com/wangdiandao/gpui-material/pull/229)
- Package the Windows x64 Gallery as a standalone ZIP with a SHA-256 checksum, locked runtime dependency inventory, and bundled licenses. [#216](https://github.com/wangdiandao/gpui-material/pull/216), [#217](https://github.com/wangdiandao/gpui-material/pull/217)

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.0.4...v0.1.0`](https://github.com/wangdiandao/gpui-material/compare/v0.0.4...v0.1.0)
