# Changelog

See [versioning and releases](docs/release.md) for the release process.

## Unreleased

### Theme, foundation, and facade

- Add `AppearanceMode` (`Light`, `Dark`, `System`) with `AppearanceMode::resolve` (and `serde` support when `feature = "serde"` is enabled) in `gpui-material-theme`.
- Add system appearance and reduced-motion sync helpers in `gpui-material-foundation` (re-exported from `gpui-material`): `appearance_from_window`, `system_appearance`, `observe_system_appearance`, `observe_window_system_appearance`, `MaterialRoot::sync_system_appearance`, `MaterialRoot::sync_window_system_appearance`, `sync_system_reduce_motion`, and re-exported `ReduceMotionPreference` / `set_reduce_motion_preference`.

## 0.4.1 - 2026-11-26

Changes since [v0.4.0](https://github.com/wangdiandao/gpui-material/releases/tag/v0.4.0).

### Theme, foundation, and facade

- Add opt-in `test-support` feature forwarding across `gpui-material-foundation`, `gpui-material-components`, `gpui-material-extras`, and `gpui-material`, and expose `gpui_material::test` (`TestWindowExt`, `ScopedWindow`, `ClickOptions`, `TestQueryExt`, `TestEventExt`, `TestInput`, `TestAppContextExt`, `ElementSnapshot`, `TestSupportExt`, and `painted_quads`) for headless UI integration testing built solely on public GPUI and `gpui-base` APIs.

### Components

- Paint pill-clipped animated Material 3 state layers and bounded press ripples via `InteractionFeedback::paint_with_opacity` across `NavigationDrawer`, `NavigationRail`, `NavigationBar`, and `Tabs` destinations without requiring an explicit `interaction_source`.
- Animate active pill indicator horizontal expansion, indicator opacity, and icon/label color interpolation across `NavigationDrawer`, `NavigationRail`, and `NavigationBar` (and smooth icon/label color transitions on `Tabs`), honoring reduced motion.
- Add `NavigationDrawer::open(bool)` for smooth Standard-mode docked width collapse/expand transitions and `NavigationRail::footer(...)` for bottom-anchored rail content across collapsed, expanded, and modal presentations.
- Fix `Snackbar` overlay hit-testing and click-through by occluding active snackbar surfaces (`.occlude()`) and stopping click event propagation on snackbar action and dismiss buttons (`cx.stop_propagation()`).

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.4.0...v0.4.1`](https://github.com/wangdiandao/gpui-material/compare/v0.4.0...v0.4.1)

## 0.4.0 - 2026-11-25

Changes since [v0.3.3](https://github.com/wangdiandao/gpui-material/releases/tag/v0.3.3).

### Breaking API and migration

- **Breaking (`gpui-material-theme`)**: Rename `MaterialTheme` fluent builders from `with_colors`, `with_typography`, `with_motion`, `with_layout`, `with_density`, and `with_direction` to bare names `colors`, `typography`, `motion`, `layout`, `density`, and `direction`.
  - Migration: `MaterialTheme::dark().with_density(Density::Compact).with_direction(Direction::Rtl)` → `MaterialTheme::dark().density(Density::Compact).direction(Direction::Rtl)`.
- **Breaking (`gpui-material-theme`)**: Change `FontFamilies` fields (`brand`, `plain`, `brand_fallbacks`, `plain_fallbacks`) from `String` / `Vec<String>` to `Arc<str>` / `Arc<[Arc<str>]>` so `MaterialTheme::clone()` avoids per-clone heap allocations while keeping `serde` compatibility.
  - Migration: construct with `Arc::from("Roboto")` / `FontFamilies::new("Roboto", "Roboto")` or `FontFamilies::with_fallbacks(...)` accepting `impl Into<Arc<str>>`.
- **Breaking (`gpui-material-foundation`)**: Rename `Surface::with_theme` to `Surface::theme`.
  - Migration: `Surface::new(...).with_theme(theme)` → `Surface::new(...).theme(theme)`.
- **Breaking (`gpui-material-components`, `gpui-material-extras`, `gpui-material`)**: Unify per-control style primitives (`ButtonShapes` → `ControlShapes`, `ButtonBorder` → `ControlBorder`, `ButtonElevations` and `ChipElevations` → `ControlElevations` with `dragged: Option<Elevation>`, `ButtonPadding` and physical `TextField::content_padding(Edges<Pixels>)` → `gpui_material_theme::LogicalEdges`).
  - Migration: replace `ButtonShapes` with `ControlShapes`, `ButtonBorder` with `ControlBorder`, `ButtonElevations`/`ChipElevations` with `ControlElevations`, and `ButtonPadding`/`Edges<Pixels>` on `content_padding` with `LogicalEdges`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Remove the legacy `ButtonTone` type alias; use `SemanticTone` directly.
  - Migration: `ButtonTone::Danger` → `SemanticTone::Danger`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Rename `EmptyState` to `EmptyView` so `*State` strictly denotes retained GPUI state entities.
  - Migration: `EmptyState::new(...)` → `EmptyView::new(...)`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Change `Avatar::fallback` to accept a repeatable element factory (`impl Fn() -> E + 'static`) instead of a one-shot `IntoElement`, preventing blank fallbacks on re-render.
  - Migration: `.fallback(icon)` → `.fallback(move || icon())`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Replace per-component edge inset structs (`ScaffoldInsets`, `ListItemPadding`, `AppBarPadding`, `BottomAppBarPadding`, `NavigationRailPadding`, `PaneMargins`, `SheetMargins`) and physical `Edges<Pixels>` on `Card::content_padding`, `MenuSurfaceStyle.padding`, and `MenuItemStyle.padding` with `gpui_material_theme::LogicalEdges`; replace `CardBorder` with `ControlBorder` and `CardElevations`/`ListItemElevations` with `ControlElevations`.
  - Migration: replace the removed structs and `Edges<Pixels>` padding values with `LogicalEdges`, `ControlBorder`, and `ControlElevations`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Add explicit `settings-tiles = ["surfaces", "selection", "feedback"]` feature (included in `full`) for `SwitchTile`, `CheckboxTile`, `RadioTile`, `SliderTile`, `ChoiceTile`, and `ChoiceOption`, replacing implicit cross-feature intersection exports.
  - Migration: enable `settings-tiles` (or `full`) when importing settings tile components without default features.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Unify overlay close callbacks to `on_closed` (`Dialog::on_close`, `Sheet::on_close`, `NavigationRail::on_modal_close` → `on_closed`), rename `Menu::on_dismiss` to `Menu::on_dismiss_request`, and align `NavigationRail` modal/headline builders with `NavigationDrawer` (`NavigationRail::modal` → `modal_state`, plus `.modal_properties(...)` and `.headline(...)`).
  - Migration: rename `.on_close(...)` / `.on_modal_close(...)` → `.on_closed(...)`, `Menu::on_dismiss(...)` → `Menu::on_dismiss_request(...)`, and `NavigationRail::modal(...)` → `NavigationRail::modal_state(...)`.
- **Breaking (`gpui-material-components`, `gpui-material`)**: Unify `Card::on_activate(cb)` to a single-argument callback (set accessible name via `.accessible_label(...)`), rename `ListItem::supporting_text` to `ListItem::supporting`, split `ListItem::track_focus(&focus, tab_stop)` into `.track_focus(&focus)` + `.tab_stop(bool)`, and change `BreadcrumbItem::icon` to accept a repeatable element factory (`impl Fn() -> E + 'static`).
  - Migration: `card.on_activate("Open", cb)` → `card.accessible_label("Open").on_activate(cb)`; `item.supporting_text("...")` → `item.supporting("...")`; `item.track_focus(&f, tab)` → `item.track_focus(&f).tab_stop(tab)`; `BreadcrumbItem::new(...).icon(icon)` → `.icon(move || icon())`.
- **Breaking (`gpui-material`)**: Make `gpui-material-components` an optional dependency behind the new `display` feature (implied by `actions`, `selection`, `inputs`, `surfaces`, `settings-tiles`, `feedback`, `navigation`, `full`, and extras families that depend on components), and use explicit `dep:` syntax for optional extras dependencies so `gpui-material-extras` and `gpui-material-components` are never leaked as public features. Applications enabling only `window-chrome` or `carousel` with `default-features = false` no longer compile `gpui-material-components` or its embedded SVGs.
  - Migration: add `features = ["display"]` if you previously depended on `gpui-material` with `default-features = false` and no component-group features while importing `Text`, `Icon`, `Avatar`, etc.
- **Breaking (`gpui-material-extras`, `gpui-material`)**: Migrate `date_time` public APIs (`DateSelection`, `DatePickerState`, `TimePickerState`, date bounds, and formatting/parsing hooks) from `chrono::{NaiveDate, NaiveTime, Weekday}` to `jiff::civil::{Date, Time, Weekday}` (re-exported from `gpui_material::date_time`), removing `chrono` and unused `web-time` from `crates/extras`.
  - Migration: replace `chrono::NaiveDate` / `NaiveTime` / `Weekday` with `gpui_material::date_time::{Date, Time, Weekday}` (i.e. `jiff::civil::{Date, Time, Weekday}`).
- **Breaking (`gpui-material-extras`, `gpui-material`)**: Unify remaining extras style and callback APIs: replace `CarouselPadding` and `ToolbarPadding` with `LogicalEdges`, `CarouselBorder` with `SurfaceBorder`, `ToolbarElevations` with `ControlElevations`, `ButtonGroupItemStyle` and `SplitButtonSegmentStyle` with `ButtonSegmentStyle`, and `ButtonGroupAlignment` and `ToolbarArrangement` with `ContentArrangement`; rename `ButtonGroupItem::appearance` to `.variant(...)`, and rename `DatePicker::on_close`, `TimePicker::on_close`, `TimePickerDialog::on_close`, and `VibrantTimePickerDialog::on_close` to `.on_closed(...)`.
  - Migration: update the removed type names to `LogicalEdges`, `SurfaceBorder`, `ControlElevations`, `ButtonSegmentStyle`, and `ContentArrangement`; rename `.appearance(...)` → `.variant(...)` on `ButtonGroupItem` and `.on_close(...)` → `.on_closed(...)` on date/time pickers.

### Theme, foundation, and design-symbols

- Add shared style value primitives `LogicalEdges` (with `non_negative_f32` sanitization and directional resolution) and `ColorSpec` (`Role(ColorRole) | Custom(Argb)`) in `gpui-material-theme`, and decouple `dynamic_color` custom/status color resolution from internal `material-color-utils` cache mutation.
- Cache all 35 normalized `MaterialShape` polygons, memoize polygon `Surface` geometry across frames, upgrade `Shape` stroke caching to multi-width `RefCell` storage (hitting 1px borders and 3px focus rings simultaneously), cache polygon shadow and exterior mask paths, use O(n) centroid triangulation for rounded rectangles, and add a full-coverage inscribed-rect fast path in `clipped_ripple`.
- Implement Material 3 Expressive 2dp outer-offset focus rings (`FOCUS_RING_OUTER_OFFSET = px(2.)`, `FOCUS_RING_WIDTH = px(3.)`) with configurable `FocusRingPlacement::Outer | Inset`.
- Cache projected Base themes in `MaterialRoot` and `ThemeScope` for zero-clone `current_base_theme` lookups, fix `set_material_theme` / `set_window_material_theme` skipping `refresh_installed_fonts` on equal themes, avoid repeated system font enumeration before `MaterialRoot::init`, and invalidate `TRACKED_LAYOUT_CACHE` on font cache revision changes.
- Guard non-finite animation targets in `motion`, expose reusable `OverlayMotion` and `ScaledOverlay`, prevent `FeedbackState` `RefMut` reentrancy panics in `control_feedback`, and reverse horizontal arrow keys under `Direction::Rtl` in `range_semantics`.
- Add `#![forbid(unsafe_code)]`, fine-grained `embed-outlined` / `embed-rounded` / `embed-sharp` features, single-lock poison-tolerant SVG caching, and compact 2-decimal coordinate formatting in `gpui-material-design-symbols`.

### Components

- Expose `Button::on_accessible_activate` and `IconButton::on_accessible_activate` to register only `AccessibleAction::Click` without attaching `on_click`, preventing mouse-down/click conflicts when used as triggers for `Menu::composed` or `gpui_base::Popover`.
- Add component-level `.disabled(bool)` to `TextField` and `ExposedDropdown`, single-thumb convenience builders (`single_thumb`, `single_thumb_style`, `single_interaction_source`, `single_track_focus`) to `Slider`, and density-aware visual sizing for `Checkbox`, `Radio`, `Switch`, and `Slider` while preserving 48 dp interactive targets.
- Fix Windows UIA and interaction defects across display, actions, selection, slider, and input controls:
  - Expose readable text names in Windows UIA for `Text`, stop assigning a hardcoded `"material-text"` ID when omitted, validate finite positive `font_scale`, eliminate `.expect("single-line text")` panics, and key ellipsis caching by style and tint.
  - Fix `Slider` `RefCell` reentrancy in `on_drag_move`, expose full track bounds on the `Role::Slider` UIA node, roll back drag values on `Escape`, guard arrow keys against modifiers, defer `ValueLabel` painting, guard zero-size bounds against `NaN`, and allow overriding vertical slider height.
  - Fix `TextField` dual scrollbar ID collisions under `ScrollbarAxis::Both`, IME `marked_text_range` placeholder overlap, unconditional outside-click `window.blur`, dynamic resting label clearance for multiple trailing controls, and suppressed `aria-invalid` when disabled (also on `Checkbox`).
  - Fix `ExposedDropdown` per-frame list remeasurement during open/close animations, unscoped internal element IDs, readonly cursor masking `secondary_trigger`, and add `Home`/`End`/`Tab` keyboard handling.
  - Fix disabled `Switch` colors on elevated surfaces (using translucent `OnSurface` instead of pre-blending with `ColorRole::Surface`), add keyboard activation feedback and click `stop_propagation` on `Checkbox`/`Radio`/`Switch`, preserve child `Radio::on_select` inside `RadioGroup`, keep focus rings visible on loading `Action` controls, switch `ChipGroup` to `Role::Group` with modifier guards, and support rotation/RTL mirroring plus finite size checks on `Icon::custom`.
  - Remove `button/spinner.rs` paint-phase `state.update` antipattern, eliminate double `layout_as_root` on `ExtendedFab`, remove duplicate `components/src/motion.rs` in favor of `foundation::OverlayMotion` / `ScaledOverlay`, and split oversized component modules into focused submodules.
- Wire `theme.layout.density` into `ListItem`, `MenuItem`, `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `AppBar`, and `BottomAppBar` while preserving minimum interactive hit targets, unify modal overlay layering (`ZLayer`) and shared `ModalHost` / `navigation/overflow_strip`, remove remaining `mod.rs` files, and split oversized surface, overlay, feedback, and navigation modules.
- Fix Windows UIA, focus, and concurrency defects across surfaces, overlays, feedback, and navigation:
  - Merge `ListItem` outer unnamed `Role::Option` and inner named `Role::Button` into a unified named accessible node exposing `selected`, expand long-press/context-menu hit area across leading/content/trailing slots without clobbering caller `aria_description`, and add `Home`/`End` navigation to `List`.
  - Remove duplicate outer `group` `aria_label` on `Tooltip`, fix `suppress_focus` lock after hover+Escape, and preserve custom rich content during exit animation.
  - Scope `SnackbarHost` measured width state by `cx.entity_id()` to prevent multi-instance infinite frame loops, notify on enqueue, and only capture `return_focus` when focus actually enters the snackbar.
  - Scope internal element IDs and `use_keyed_state` keys by parent `self.id` across navigation components (`items`, `rail`, `adaptive_navigation`, `adaptive`, `scaffold`, `multi_pane`, `app_bar/actions`, `drawer`, `stepper`) and share focus state keys across adaptive drawer transitions.
  - Fix `Dialog` dismissal when unfocused or closing inactive native windows, read `Dialog`/`Sheet` exit durations from `theme.motion`, preserve trigger on `Menu::composed().disabled(true)` with sibling submenu mutual exclusion and reduced `PresentedMenu` cloning, defer `cx.notify()` out of `render()` in `BottomSheetState::configure`/`BottomAppBar`/`NavigationRail`, guard non-finite scroll offsets and `Sheet::extent`, clamp `Progress` minimum wavelength and gate offscreen transitions, fix `PullToRefresh` `>= 1.0` threshold and offscreen RAF gating, cache `Skeleton` polygon paths, add `Live` semantics to `Banner` and indeterminate `Progress`, lazily allocate focus/ripple on static `Card`/`DropTarget` and block `Card` click while dragged, add RTL `logical_anchor` on `ChoiceTile`, `intercept_keystrokes` Escape on `VerticalDragHandle`, `!modifiers.modified()` keyboard guards, `Drawer` `scroll_to_item`, `MultiPane` `saturating_duration_since` and 3-pane boundary clamp, `Breadcrumbs` `ContentMask`, and valid `Stepper` ARIA roles.

### Extras and facade

- Add single-step `gpui_material::init(theme, cx)` (installing `MaterialRoot` and conditionally registering component keybindings), re-export `window_material_theme` and `set_window_material_theme` at the facade root, and add `#![forbid(unsafe_code)]` to `gpui-material`.
- Tighten `xtask` dependency-graph verification to forbid implicit `gpui-material-extras` / `gpui-material-components` feature leakage, assert `gpui-material` with only `carousel` or `window-chrome` does not pull in `gpui-material-components`, and assert `date-time` depends on `jiff` without `chrono` or `web-time`.
- Remove remaining `mod.rs` files (`carousel.rs`, `window_chrome.rs`), replace wildcard re-exports in `gpui-material-extras` with explicit exports, split oversized extras modules into focused submodules, and wire `theme.layout.density` into `SearchBar`, `Toolbar`, `ButtonGroup`, and `SplitButton`.
- Fix P0/P1 defects across extras:
  - `date_time`: keep numeric `HH:MM` time inputs left-to-right under RTL per M3 & Unicode BiDi, add continuous clock-dial pointer drag with minute transition on release, accept both ASCII `" - "` and en-dash `" – "` in docked range input, sync calendar cursor month in `set_draft`, guard docked Escape against active IME composition, add `!modifiers.modified()` and zero-count `wrap()` guards, and avoid per-frame `state.update` from inline `DatePickerLocale`.
  - `carousel`: prevent `NaN` division-by-zero corruption in `geometry.rs` and guard non-finite scroll inputs in `CarouselState`, fix right-edge parallax clipping on wide items, and reuse a single bump timer instead of spawning per wheel event.
  - `window_chrome`: fix Win32 `GetWindowTextLengthW`/`GetWindowTextW` signed-to-`usize` safety, propagate `PostMessageW` failure so `remove_window`/`zoom_window` fallbacks run, cache window titles to avoid per-frame Win32 FFI allocations, keep macOS traffic-light padding on the physical left under RTL, auto-hide client title bars under Linux `Decorations::Server`, and attach `.track_focus(...)` to caption buttons.
  - `toolbar`, `groups`, `search`: eliminate double `layout_as_root` on expanding toolbar slots, add M3 Level 3 default elevation to floating `Toolbar`, use `logical_anchor` for RTL toolbar overflow menus, scope internal element IDs by `self.id`, add accessible label support to `SplitButton` outer `Role::Group`, and fix `search/scroll.rs` initial non-zero offset hide and duplicate same-frame sampling.

### Gallery

- Raise the default Gallery window size to `1280×820` so component playgrounds open in side-by-side preview/configuration layout, compute the stacked breakpoint from actual available content width (accounting for sidebar state), and stretch the stacked preview column to full width.
- Move the light/dark theme toggle permanently to the top bar and remove the floating bottom-right theme toggle `Fab` that obscured component controls; mount `DisplayScrollbar::vertical` on the Gallery scroll container.
- Eliminate Windows UIA duplicate-name collisions across playground scenario chips, choice controls, switches, and `Overview` buttons; expose tab selection semantics (`Role::TabList` / `Role::Tab` / `selected`) for top-bar sections and `Live preview | Specs` tabs; make the top-bar brand button, `Overview` shape buttons, 44–48 px theme swatches, and all component cards keyboard-focusable and activatable with uniform 2-column card widths.
- Render playground `Rust code` with a monospace font stack and horizontal scrolling, fix `#playground-event-status` contrast inside local light/dark `ThemeScope` previews, unify `--window-chrome` (`ChromePreview`) surfaces with `ColorRole::SurfaceContainer` and compact `Block close` sizing, and register embedded `IBM Plex Sans` with `DisplayScrollbar` in `gpui-material-gallery-web`.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.3.3...v0.4.0`](https://github.com/wangdiandao/gpui-material/compare/v0.3.3...v0.4.0)

## 0.3.3 - 2026-11-24

Changes since [v0.3.2](https://github.com/wangdiandao/gpui-material/releases/tag/v0.3.2).

### Dependencies and API migration

- Upgrade `gpui-base` to `=0.11.0` and paired `gpui-pre` / `gpui-pre-platform` / `gpui-pre-web` snapshots to `=0.3.14`, enabling upstream IME surrounding-text redaction for masked text fields (`longbridge/gpui-kit#4059`). Downstream apps that import GPUI or Base directly must match the new pins.
- Adapt `gpui_material::foundation::Shape` to GPUI `0.3.14`'s opaque curve-based `gpui::Path`: `clip_rect`, `clip_disc`, and `clip_convex_polygon` now return `Option<Path>`, `clip_path`/`paint_clipped` are replaced by `clip_shape`/`paint_clipped_shape` accepting `&Shape`, and `Shape::fill_bounds` is added.

### Components

- Avoid animating weighted `ButtonGroup` member widths from the viewport width on initial mount before the container width is measured.

### Gallery

- Display the project SVG logo in the Gallery top bar instead of the placeholder letter badge, and optimize rendering/geometry crates in `[profile.dev.package]` for smoother development builds.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.3.2...v0.3.3`](https://github.com/wangdiandao/gpui-material/compare/v0.3.2...v0.3.3)

## 0.3.2 - 2026-11-23

Changes since [v0.3.1](https://github.com/wangdiandao/gpui-material/releases/tag/v0.3.1).

### Components

- Prevent `ButtonGroupVariant::Connected` from borrowing neighbor width on press, and shrink horizontal button padding (down to 4 dp per side) before ellipsizing labels when a fixed-width button is compressed below its preferred width.
- Blur focused `TextField` editors on outside clicks (including supporting text and non-focusable areas) and when disabled, keep focus without transient blur when pressing the built-in clear button, and blur focused `ExposedDropdown` editors on outside dismissal.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.3.1...v0.3.2`](https://github.com/wangdiandao/gpui-material/compare/v0.3.1...v0.3.2)

## 0.3.1 - 2026-11-23

Changes since [v0.3.0](https://github.com/wangdiandao/gpui-material/releases/tag/v0.3.0).

### Theme and foundation

- Expose Material 3 custom colors and harmonization behind `dynamic-color` (`CustomColorGroup`, `harmonize_color`, `ColorScheme::custom_color`, and `ColorScheme::custom_color_with_contrast`).
- Add ergonomic M3 helpers: `Argb::{from_rgba, with_alpha, alpha, red, green, blue}`, `ColorRole::on_role`, `ColorScheme::with_color`, `MaterialTheme::{with_colors, with_typography, with_motion, with_layout, with_density, with_direction}`, `CornerShape::{uniform, top, bottom, start, end}`, and `ThemeScope::density`.
- Support `Home`/`End` in `move_roving_focus` (unifying keyboard navigation across `NavigationDrawer`, `Stepper`, `List`, `Menu`, `RadioGroup`, `ButtonGroup`, `Toolbar`, and `FabMenu`) and `Home`/`End`/`PageUp`/`PageDown` in `range_semantics` for `Slider`.

### Components

- Add `TextField::show_label(bool)`, density-aware default minimum height with `TextField::min_height(Pixels)`, and `TextField::indicator_width(unfocused, focused)`.
- Add variant shorthand constructors (`filled`, `tonal`, `elevated`, `outlined` on `ToggleButton`; `standard`, `filled`, `tonal`, `outlined` on `IconToggleButton`) and controlled `.loading(bool)` busy state to `ToggleButton` and `IconToggleButton`.
- Add `.show_label(bool)` and `.label_content(...)` to `Checkbox`, `Radio`, and `Switch`, and preserve caller-supplied `Radio::track_focus(&FocusHandle)` handles inside `RadioGroup`.
- Add `Slider::track_focus(RangeThumb, &FocusHandle)` for caller-owned thumb focus handles.
- Support `Backspace`/`Delete` keyboard removal on focused removable `Chip` actions, and add group-level `.tonal(bool)`, `.elevated(bool)`, and `.disabled(bool)` defaults to `ChipGroup`.
- Add `Feed::accessible_label` with `Role::List` semantics and promote `Feed` to `Verified` (62/62 documented components verified in CI), alongside standalone `SupportingPane::new` narrow toggle coverage.
- Fix `NavigationDrawer` font scaling (`NavigationItems::font_scale`), RTL section header/headline alignment, `.divider_before(...)` across `.entries`/`.section`/`.push_section`, and icon tint inheritance (`.inherit_color()`).
- Add `List::{accessible_label, item}`, `ListItem::tone(SemanticTone)`, desktop secondary click (`MouseButton::Right`) and `contextmenu` key triggering for `ListItem::on_long_press`, caller `.track_focus` preservation in `List` and `ListSection`, `.track_focus` / `.interaction_source` across `SwitchTile`, `CheckboxTile`, `RadioTile`, `SliderTile`, and `ChoiceTile`, `ChoiceTile` accessible value description, and `.trailing(...)` on `CheckboxTile` and `RadioTile`.
- Add `Badge::{visible, offset}` and scaled-font `min_h` fix, `Card::{content_padding, gap, accessible_label}` with rounded child clipping, `Text::{align, max_lines, color_override}`, public `Icon::{inherit_color, color_override}`, and `DropTarget::{track_focus, interaction_source}` with disabled `Role::Button` preservation, `ThemeScope` action binding, and dynamic corner radii.

### Extras

- Add `SemanticTone` support to `ButtonGroup`, `ButtonGroupItem`, and `SplitButton`; fix `Toolbar` and `CaptionButtons` focus ring tokens to `ColorRole::Secondary`; add `ToolbarAction::with_icon` and overflow menu icons; add `SearchView` `Home`/`End` and input-`Down` navigation plus `SearchState::trigger_focus` and `SearchBar::track_focus`; add `TimePicker` dialog options, `DatePicker`/`TimePicker::disabled`, and `DatePickerState`/`TimePickerState::trigger_focus`; add `CaptionButtonLabels` localization for `CaptionButtons` and `TitleBar`.

### Overlays and menus

- Enhance `Menu::context()` and `MaterialElementExt::context_menu(...)` to open at the secondary-click cursor position (viewport-clamped, repositioning on subsequent right-clicks), avoid hijacking trigger `Enter`/`Space` actions, and support `Shift+F10` and `ContextMenu` keyboard triggers; add lazy builders `Menu::new_with` and `MaterialElementExt::context_menu_with`, plus `Menu::disabled(bool)`.
- Add `Sheet::show_header(bool)` and apply `Sheet::max_width(Pixels)` to side sheets as well as bottom sheets (keeping the 400 dp side-sheet default).
- Add `Dialog::confirm_disabled(bool)`, `Dialog::confirm_loading(bool)`, `DialogState::focus_handle()`, and `DialogState::scroll_handle()`.
- Add `Snackbar::dismiss_label(...)`, `SnackbarHost::pending_len()`, `SnackbarHost::clear_queue(cx)`, and pause timed snackbar auto-dismiss while hovered or focused.
- Add `Tooltip::show_delay(Duration)` and `Tooltip::hide_delay(Duration)` overrides.

### Gallery

- Reorganize the Gallery around top-level `Components`, `Styles`, and `Examples` sections with a grouped Material 3 sidebar catalog, an interactive Overview landing page (hero shapes, live theme preset swatches, interactive controls, and grouped component cards), dedicated `Color` (with click-to-copy hex swatches), `Typography`, `Shape` (with interactive 35-shape inspector), and `Motion` style reference pages, and a status bar displaying the active theme and package version while keeping existing `Story` IDs compatible.
- Redesign all 50 component pages around a `Live preview | Specs` layout with a local `ThemeScope` preview canvas, declarative `Configuration` card (scenarios, controls, and reset), live Rust `Code` snippet card with copy action, and lazy-loaded full `Specs` views, plus a live breakpoint badge on the `Adaptive layout` example page.

### Performance

- Guard no-op state updates against redundant `cx.notify()` and `window.refresh()` calls across Foundation theme updates (`set_material_theme`, `set_window_material_theme`), Components (`PullToRefreshState`, `SwipeToDismissState`, `BottomSheetState`, `Tooltip`, `ExposedDropdownState`, `AppBarScrollState`), and Extras (`SearchScrollState`, `ToolbarScrollState`, `ScrollFieldState`, `DatePickerState`, `TimePickerState`).
- Virtualize the Gallery component catalog with `gpui::list`, cache filtered catalog indices on search input changes, and skip no-op navigation and window-move notifications, cutting Gallery shell frame overhead by 58%–75% in the Release `perf_bench` suite.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.3.0...v0.3.1`](https://github.com/wangdiandao/gpui-material/compare/v0.3.0...v0.3.1)

## 0.3.0 - 2026-11-22

Changes since [v0.2.0](https://github.com/wangdiandao/gpui-material/releases/tag/v0.2.0).

### API and migration

- **Breaking**: Rename the optional `windows-chrome` Cargo feature and `gpui_material::windows_chrome` module to `window-chrome` and `gpui_material::window_chrome` across `gpui-material-extras`, `gpui-material`, and `gpui-material-gallery`, and remove the Windows-only compilation gate.
- **Breaking**: `IconName` adds `IconName::Visibility` and `IconName::VisibilityOff`. Update exhaustive `IconName` matches to cover the new variants, or add a fallback arm.

### Window chrome and cross-platform defaults

- Add cross-platform `window_chrome` support covering Windows native hit testing and Snap Layouts, macOS native traffic-light centering (`title_bar_window_options_with_height`) and leading inset spacing, and Linux `Decorations::Client` / `Decorations::Server` + `WindowControls` integration.
- Expand `TitleBar` and `CaptionButtons` with customizable `.height(...)`, `.title(...)`, `.title_role(...)`, `.title_font(...)`, `.title_font_family(...)`, `.title_font_size(...)`, `.leading(...)`, `.trailing(...)`, `.traffic_light_padding(...)`, `.colors(TitleBarColors)`, `.caption_colors(CaptionButtonColors)`, `.caption_button_width(...)`, `.show_caption_buttons(...)`, `.on_close(...)`, and `.platform(ChromePlatform)`, replacing hardcoded `"Segoe UI"` and `"Segoe Fluent Icons"` with Material theme typography and vector caption icons.
- Update `FontFamilies::default()` in `gpui-material-theme` with cross-platform Latin and CJK fallbacks (`SF Pro Text`, `Segoe UI`, `Noto Sans`, `PingFang SC`, `Microsoft YaHei UI`, `Noto Sans CJK SC`) and enable `cargo gallery --window-chrome` on all desktop targets.

### Components

- Add `ButtonTone` (`Default`, `Danger`, `Success`, `Warning`, `Info`) semantic shortcuts to `Button`, `ToggleButton`, `IconButton`, and `IconToggleButton` while preserving per-instance `ButtonColors` and `ButtonBorder` precedence.
- Promote `SemanticTone` to core (retaining `pub type ButtonTone = SemanticTone;`) and propagate `.tone(SemanticTone)` across `Badge`, `Chip`, `Dialog` (`.confirm_tone`), `MenuItem` (plus `.destructive()`), `Snackbar`, `LinearProgress`, and `CircularProgress`; add `Badge::{standalone_label, standalone_count, color_role}`, `Dialog::{alert, on_confirm}`, `MenuItem::{shortcut, divider}`, `SnackbarHost::{push_message, replace}`, `Tooltip::shortcut`, variant/typography shorthands on `Button`, `IconButton`, `Card`, `Chip`, `TextField`, and `Text` (including `.muted()`), and the fluent postfix `MaterialElementExt` trait (`.badge_dot()`, `.badge_label(...)`, `.badge_count(...)`, and with `feedback`: `.tooltip(...)`, `.context_menu(...)`).
- Add `.track_focus(&FocusHandle)` to `Chip`, `Checkbox`, `Radio`, `Switch`, and `Card`, preserve `ThemeScope` in toggle and selection/list/card action callbacks, match disabled label opacity and single-line truncation across `Checkbox`, `Radio`, and `Switch`, clip split `Chip` primary and removal action feedback along the interior seam, and keep overlapping range `Slider` thumb selection responsive after zero-delta drag moves.
- Add core `Avatar` display component (`initials`, `icon`, `image` with `.fallback`, `custom`, `AvatarSize`, `AvatarShape`, `AvatarColors`) and overlapping `AvatarGroup` stack with uniform sizing, surface-matched cutout rings, `+N` overflow truncation, RTL mirroring, and summary accessibility labels.
- Add core presentational `EmptyState` layout shell with an optional decorative icon container (`MaterialShape` polygon or `CornerShape` backdrop) or custom `.visual`, title, supporting copy/content, centered RTL-aware primary/secondary action slots, and configurable `.max_width` / `.title_role`.
- Add four M3E convenience components without new feature-group dependency edges: in-flow `Banner` (`feedback`, with `SemanticTone`, actions, dismiss, and `Role::Status`/`Role::Alert`), controlled `ChipGroup` (`selection`, with `Single`/`SingleRequired`/`Multiple` selection, `.single_select`, wrapping/scrollable layout, and roving focus), core `Skeleton` (`rect`/`text_line`/`circle`/`shape` placeholders on `LoopClock` + `is_bounds_visible` with zero offscreen/reduced-motion frames), and width-responsive `Breadcrumbs` (`navigation`, collapsing middle segments into an anchored Material `Menu`).
- Add `ButtonGroup::connected`, `.single_select`, and `.single_optional_select` ergonomics shortcuts to `gpui-material-extras` (`expressive-groups`).
- Add `DropTarget` (`surfaces` feature) for external OS file drops (`ExternalPaths`) and typed in-app GPUI drag payloads with validation feedback and optional click/keyboard activation.
- Add `Stepper` (`navigation` feature) with horizontal and vertical orientations, `StepStatus`, `.active_step`, `.non_linear`, RTL mirroring, and roving keyboard navigation.
- Add standalone `NavigationDrawer` (`navigation` feature) with Standard and Modal (side-sheet) modes, sections, headers, dividers, 56 dp pill indicators, and roving vertical navigation.
- Add `BottomAppBar` (`navigation` feature) with 80 dp Material styling, action and trailing FAB slots, `Scaffold::bottom_bar` compatibility, and `AppBarScrollBehavior` / `AppBarScrollState` hide-on-scroll support.
- Add settings convenience tiles `SwitchTile`, `CheckboxTile`, `RadioTile`, `SliderTile` (`surfaces` + `selection`), `ChoiceTile` / `ChoiceOption` (`surfaces` + `feedback`), and `ListSection` / `ListSectionItem` (`surfaces`) with optional `LabelMedium` section headings, true M3E Segmented grouped corner geometry (`16/4`, 2 dp gap), and unified vertical roving focus.
- Add `Button::loading(bool)` and `IconButton::loading(bool)` (`actions` feature, no dependency on `feedback`) with busy accessibility semantics, activation suppression, resting-width preservation, and an inline Material circular spinner respecting `is_bounds_visible` and `reduced_motion`.
- Bundle official Material Symbols Outlined `IconName::Visibility` and `IconName::VisibilityOff` (both fill variants) from the pinned upstream revision.
- Fix `Sheet::expandable` drag handle creating an unlabeled keyboard tab stop, delay `Tooltip` motion sampling until its trigger anchor is measured, prevent `SnackbarHost` first-frame zero-width layout and bound exiting queue growth under reduced motion, and defer render-time transition notifications across `NavigationRail`, `AdaptiveNavigation`, and `AdaptivePaneState`.

### Inputs

- Add `TextFieldObfuscationMode` (`Hidden`, `Visible`) and `TextField::obfuscation_mode(...)` for `TextField::secure` visibility toggles, re-export `TextFieldMaskPattern` through the `inputs` feature facade, and add Gallery examples and tests covering password visibility toggling, structured mask patterns, and pre-commit validation.
- Add `TextField::clearable(bool)` (`.clear_label(...)`) and `TextField::visibility_toggle(visible, on_toggle)` (`.visibility_labels(show, hide)`) inside the `inputs` feature without adding a dependency on `actions`, automatically syncing secure obfuscation mode and preserving focus.
- Add `ExposedDropdown::with_shared_options` for `O(1)` `Rc<[DropdownOption]>` option sharing, accelerate dropdown filtering with zero-allocation ASCII case-insensitive matching and cached active indices, reserve trailing inset for `TextField` resting labels when `.trailing(...)` is present, and cache per-frame character counts.

### Performance, theme, foundation, and symbols

- Store all 61 Material color roles inline in `ColorScheme` for allocation-free `O(1)` lookups and blending, expose `ColorScheme::baseline`, `appearance`, and `color` as `const fn`, and preserve the compact `{ appearance, overrides }` `serde` format.
- Cache resolved system and custom `Font` family descriptors and letter-spaced `LineLayout` runs in `gpui-material-foundation`, cache tessellated `Shape` stroke paths and transparent `Surface` rounded feedback geometry, prune finished ripples before `Escape` cancel checks, and expose `elevation_shadow_pair` for allocation-free canvas shadow painting.
- Accelerate `clipped_ripple` with precomputed 64-gon unit-circle normals and inscribed-disk / triangle-AABB fast paths.
- Add `const fn` constructors and accessors `SymbolAxes::new`, `SymbolAxes::is_filled`, and `SymbolAxes::get_weight` in `gpui-material-design-symbols`, and cache outlined SVG bytes by `(codepoint, axes)` in `SymbolFont::svg_data` so repeated lookups and alias symbols share a single `Arc<[u8]>`.
- Snapshot `Menu` exit rows lazily, short-circuit `AdaptivePaneState` history resolution once all three pane roles are assigned, replace per-frame multi-pane geometry `Vec` allocations with stack arrays, and avoid redundant root layout passes and focus map clones in `AdaptiveNavigation`, `AppBar`, `Scaffold`, and `NavigationItems`.
- Binary-search single-line ellipsis truncation in `Text`, deduplicate `ExtendedFab` label shaping, reuse cached rounded feedback geometry and stack-allocated `elevation_shadow_pair` shadows across `Button`, `Chip`, `Card`, `ListItem`, and selection ripples, add static corner and line-count fast paths in `ListItem`, reuse active `DisplayScrollbar` fade wake timers across intermediate scroll frames, predefine `const` `Progress` easing curves, and skip redundant `SwipeToDismiss` width notifications when settled at rest.
- Optimize `gpui-material-extras` runtime hot paths: release `SearchState` result focus handles when returning to idle, replace `O(N)` search result focus scans with `O(1)` scope checks, skip redundant `SearchView` and `DatePicker` state updates when idle, cache monthly calendar day IDs and accessible labels alongside static day, two-digit, and dial-mark tables, reuse active `ScrollField` settle timers across rapid scroll events, precompute contained `Carousel` keylines once per layout and share measured layouts without extra next-frame re-renders, skip zero-radius `CarouselParallax` corner paths, preserve `ButtonGroup` focus handles and cached label widths across unchanged renders, and gate `Toolbar` footprint animation frame requests on `exit_direction.is_some()`.

### Gallery

- Lazily instantiate Gallery story views on first visit while retaining visited story state, scope `ModalExamples` entity creation to the active story (`Dialog` vs `Sheets`), exclude the desktop `Gallery` shell from `wasm32` builds, and consolidate Gallery tests into a single test binary.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.2.0...v0.3.0`](https://github.com/wangdiandao/gpui-material/compare/v0.2.0...v0.3.0)

## 0.2.0 - 2026-11-21

Changes since [v0.1.2](https://github.com/wangdiandao/gpui-material/releases/tag/v0.1.2).

### API and migration

- Update exhaustive `ColorRole` matches to cover the nine new semantic status roles (`Info`, `OnInfo`, `InfoContainer`, `Success`, `OnSuccess`, `SuccessContainer`, `Warning`, `OnWarning`, and `WarningContainer`).
- Lower the workspace minimum supported Rust version (`rust-version`) from `1.99.1` to `1.95.0` and set `default-features = false` on the workspace `gpui-material-components` dependency so Cargo versions below `1.99` can evaluate workspace inheritance.

### Dependencies

- Allow downstream applications to resolve compatible versions of internal dependencies (`polymorpher`, `material-color-utils`, and `serde_json` dev-dependency) at or above the existing minimums while keeping public GPUI/Base and workspace lockstep pins exact.

### Modular compilation

- Partition `gpui-material-components` into coarse feature groups (`actions`, `selection`, `inputs`, `surfaces`, `feedback`, and `navigation`, with `default = ["full"]` for backwards compatibility), forward them through `gpui-material`, and wire each `gpui-material-extras` family to enable only the core groups it uses.

### Performance

- Add a repeatable locked Release benchmark suite (`perf_bench` in `gpui-material-gallery`) covering `Button`, `TextField`, complex virtualized lists, navigation chrome, and animated indicators while separating GPUI engine baseline, isolated component overhead, standalone story cost, and Gallery shell overhead.

### Shapes

- Add the 35-shape Material 3 Expressive `MaterialShape` catalog in `gpui-material-theme` and reusable `ShapePolygon`, `ShapeFit`, `ShapeMorph`, and `ShapeMorphSequence` adapters in `gpui-material-foundation` (re-exported from `gpui-material`), consolidating `LoadingIndicator` onto Foundation and removing the direct `polymorpher` dependency from `gpui-material-components`.
- Add geometric shape clipping (`Shape::clip_path`, `clip_rect`, `clip_disc`, `clip_convex_polygon`, `exterior_mask`, and paint helpers), stroke/shadow painting (`fill_path`, `stroke_path`, `paint_stroke`, `paint_shadow`), and extended hit testing (`hit_test`, `intersects_bounds`).
- Extend `Surface` with `.polygon_shape(...)`, `.shape_fit(...)`, `.shape_clip_backdrop(...)`, and `.shape_hit_test(...)` for non-rectangular Material shapes, EvenOdd backdrop clipping, and shape-aware pointer hit testing.

### Theme and typography

- Add `project_base_theme` and `current_base_theme` (including Material scrollbar and resize-handle tokens), attach a local-theme-aware Material `DisplayScrollbar` renderer and complete `InputEditorStyle` diagnostics to multiline `TextField` editors, preserve window-local overrides in `ThemeScope::bind_action`, and avoid unnecessary redraws of locally overridden windows during application palette transitions.
- Unify the animation lifecycle in `gpui-material-foundation` (`animate_with_status`, `LoopClock`, `RetargetTransition`, `is_bounds_visible`) so keyed channels use a single GPUI entity, mid-flight interruptions preserve spring velocity or shorten tween reversals proportionally, reduced motion settles immediately, and off-screen, clipped, inactive, or hidden animated components pause without scheduling frames.
- Add `ColorScheme::from_seed_with` and `MaterialTheme::from_seed_with` supporting all seven `DynamicVariant` palettes (`TonalSpot`, `Vibrant`, `Expressive`, `Fidelity`, `Content`, `Monochrome`, `Neutral`) and `SpecVersion` (`Spec2021` and `Spec2025`), lightweight seed-based `ThemePreset` recipes, `ColorScheme::blend`, and extended `Info`, `Success`, and `Warning` semantic color roles (and their `On*` and `*Container` pairs).
- Add `Typography::with_text_scale` (default `1.0`, clamped `1.0..=2.0`) and `Typography::style` / `scale_style`, applied across all 15 Material type roles and Base typography projection.
- Animate application palette changes in `set_material_theme` with a retargetable Slow Effects spring that respects reduced motion, and add subtree-level `ThemeScope` and `current_material_theme` with overlay inheritance that keeps the application Base theme untouched.

### Icons

- Add the optional `gpui-material-design-symbols` companion crate with the full Outlined, Rounded, and Sharp catalog, fill and weight controls, desktop embedding, and asynchronous browser loading.

### Windows

- Add optional custom title bars with native Windows caption behavior and a Gallery demonstration.

### Contributors

- [@wangdiandao](https://github.com/wangdiandao)

**Full Changelog** [`v0.1.2...v0.2.0`](https://github.com/wangdiandao/gpui-material/compare/v0.1.2...v0.2.0)

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
