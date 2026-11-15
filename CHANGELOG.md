# Changelog

See [versioning and releases](docs/release.md) for the release process.

## Unreleased

### Components

- Set divider thickness and color for each horizontal or vertical rule. A zero thickness selects a one-device-pixel hairline.
- Compose text fields with configurable label placement, leading and trailing elements, value affixes, custom hints and supporting content, and required, invalid, and character count presentation.

### Gallery

- The GPUI Material Gallery uses the project logo as its Windows application icon.

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
