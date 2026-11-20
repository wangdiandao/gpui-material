# Changelog

See [versioning and releases](docs/release.md) for the release process.

## Unreleased

### Modular compilation

- Partition `gpui-material-components` into coarse feature groups (`actions`, `selection`, `inputs`, `surfaces`, `feedback`, and `navigation`, with `default = ["full"]` for backwards compatibility), forward them through `gpui-material`, and wire each `gpui-material-extras` family to enable only the core groups it uses.

### Shapes

- Add the 35-shape Material 3 Expressive `MaterialShape` catalog in `gpui-material-theme` and reusable `ShapePolygon`, `ShapeFit`, `ShapeMorph`, and `ShapeMorphSequence` adapters in `gpui-material-foundation` (re-exported from `gpui-material`), consolidating `LoadingIndicator` onto Foundation and removing the direct `polymorpher` dependency from `gpui-material-components`.
- Add geometric shape clipping (`Shape::clip_path`, `clip_rect`, `clip_disc`, `clip_convex_polygon`, `exterior_mask`, and paint helpers), stroke/shadow painting (`fill_path`, `stroke_path`, `paint_stroke`, `paint_shadow`), and extended hit testing (`hit_test`, `intersects_bounds`).
- Extend `Surface` with `.polygon_shape(...)`, `.shape_fit(...)`, `.shape_clip_backdrop(...)`, and `.shape_hit_test(...)` for non-rectangular Material shapes, EvenOdd backdrop clipping, and shape-aware pointer hit testing.

### Theme and typography

- Add `project_base_theme` and `current_base_theme` (including Material scrollbar and resize-handle tokens), attach a local-theme-aware Material `DisplayScrollbar` renderer and complete `InputEditorStyle` diagnostics to multiline `TextField` editors, preserve window-local overrides in `ThemeScope::bind_action`, and avoid unnecessary redraws of locally overridden windows during application palette transitions.
- Unify the animation lifecycle in `gpui-material-foundation` (`animate_with_status`, `LoopClock`, `RetargetTransition`, `is_bounds_visible`) so keyed channels use a single GPUI entity, mid-flight interruptions preserve spring velocity or shorten tween reversals proportionally, reduced motion settles immediately, and off-screen, clipped, inactive, or hidden animated components pause without scheduling frames.
- Add `ColorScheme::from_seed_with` and `MaterialTheme::from_seed_with` supporting all seven `DynamicVariant` palettes (`TonalSpot`, `Vibrant`, `Expressive`, `Fidelity`, `Content`, `Monochrome`, `Neutral`) and `SpecVersion` (`Spec2021` and `Spec2025`), lightweight seed-based `ThemePreset` recipes, `ColorScheme::blend`, and extended `Info`, `Success`, and `Warning` semantic color roles (and their `On*` and `*Container` pairs; update exhaustive `ColorRole` matches).
- Add `Typography::with_text_scale` (default `1.0`, clamped `1.0..=2.0`) and `Typography::style` / `scale_style`, applied across all 15 Material type roles and Base typography projection.
- Animate application palette changes in `set_material_theme` with a retargetable Slow Effects spring that respects reduced motion, and add subtree-level `ThemeScope` and `current_material_theme` with overlay inheritance that keeps the application Base theme untouched.

### Icons

- Add the optional `gpui-material-symbols` companion crate with the full Outlined, Rounded, and Sharp catalog, fill and weight controls, desktop embedding, and asynchronous browser loading.

### Windows

- Add optional custom title bars with native Windows caption behavior and a Gallery demonstration.

## 0.1.2 - 2026-11-20

Changes since [v0.1.1](https://github.com/wangdiandao/gpui-material/releases/tag/v0.1.1).

### Dependencies

- Allow downstream applications to resolve compatible versions of `serde`, `web-time`, and `windows` at or above the existing minimums.

### Feedback

- Skip fill triangles outside the Ripple bounds before clipping.
- Refresh the owning view when control feedback changes and skip redraws on redundant release or cancel events.

### Text

- Reuse single-line ellipsis layouts for repeated text and widths.

### Gallery

- Show Material type roles and compare emphasis, font scaling, wrapping, and truncation in the Text story.
- Capture active frame timing in a single Gallery window with the optional frame profiler.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.1.1...v0.1.2`](https://github.com/wangdiandao/gpui-material/compare/v0.1.1...v0.1.2)

## 0.1.1 - 2026-11-16

Changes since [v0.1.0](https://github.com/wangdiandao/gpui-material/releases/tag/v0.1.0).

### API and dependency migration

- Match direct Base imports to 0.10.0 and retain GPUI 0.3.12.
- Add exhaustive-match arms for `AppBarVariant::TwoRows`, `DatePickerVariant::FullScreen`, `SearchViewVariant::FullScreenContained`, and `TimePickerVariant::Scroll`.
- Changed keyboard and accessibility slider adjustments now emit `Change` followed by `Release`. Update subscribers that interpreted `Release` as pointer-only.

### Components

- Use retained bottom-sheet stops, configurable drag and keyboard actions, and a measured Standard sheet slot in Scaffold.
- Compose standalone dial, text input, and wrapping time wheels or combine them with configurable standard and vibrant time picker dialogs.
- Customize adaptive navigation types, animated visibility, and the bar or rail layout while keeping destination selection and focus across width changes.
- Compose date pickers with customizable anchors, panels, dialog actions and state colors. Control the displayed month, partial range, display mode, and year availability.
- Compose Contained and Docked search with configurable editor slots, grouped virtual results, motion, appearance, and scroll behavior.
- Compose tabs with leading icons, custom content and badges, configurable indicators and dividers, and external scroll handles.
- Control Navigation Rail transitions and configure destination content, indicator padding, rail layout, and modal appearance and dismissal.
- Customize docked and detached side sheets with independent header and footer slots, typed dismissal, and per-instance appearance.
- Compose docked and floating toolbars with editable content, independently expandable slots, instance appearance, and retained scroll and settling behavior.
- Add Pull to Refresh for scrollable content with a controlled refresh request, gesture thresholds, and a replaceable Material loading indicator.
- Control FAB menu expansion and configure trigger animation, item content, palettes, and logical alignment.
- Compose adaptive layouts with up to three panes, resizable anchored separators, reflow and levitation, and retained transitions and navigation history.
- Compose dialog slots or a custom modal shell, configure instance appearance and dismissal, observe close reasons, and open optional native dialog windows.
- Center short navigation bar destinations, mix icon positions, add custom labels and badges, and customize colors and observe interactions per item.
- Customize mixed Button Group members, Connected segment shapes, weighted layout and press expansion, and controlled overflow.
- Use text-only and custom-content Extended FABs with controlled expansion, visibility motion, and per-instance appearance and interaction observation.
- Compose Standard and Segmented lists with custom row slots, checked and expandable rows, long-press actions, swipe dismissal, and per-state appearance.
- Attach Material Labels with custom content and caller-controlled persistent or shared interaction visibility.
- Compose custom clickable, selectable and checked Surfaces with inherited content colors, explicit shape and border, and configurable interaction feedback.
- Compose exposed dropdowns with custom editors, anchors and options, caller-controlled visibility, and configurable popup size, scroll state and appearance.
- Add display-only scrollbars with per-axis viewport tracking, custom geometry and colors, and configurable fading.
- Customize both Split Button segments and use a menu, independent command, or controlled toggle for the trailing segment.
- Configure carousel layout and fling behavior, read and control the retained item position, and apply standalone scroll parallax to custom content.
- Customize Snackbar content, controls, layout, and appearance, and observe action or dismissal results from the queue.
- Add customizable TwoRows app bars, logical multiline headings and logo slots, retained scroll strategies, and responsive app bar action rows and columns.
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
- Use GPUI Kit's Base 0.10.0 with its paired GPUI 0.3.12 snapshot. Downstream apps that import Base directly must match the new Base pin. The Base slider handles Windows UI Automation RangeValue SetValue.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.1.0...v0.1.1`](https://github.com/wangdiandao/gpui-material/compare/v0.1.0...v0.1.1)

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
