# Downstream integration

English | [中文](integration.zh.md)

Use Rust 1.95.0 or later with the 2024 edition. Keep the application's Cargo lockfile under version control.

## Add the facade

Add the facade from crates.io. Match GPUI package identities exactly so the facade and application share the same `App`, `Window` and `Entity` types.

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = "0.4.1"
```

An application that uses Base directly should also pin `gpui-base = "=0.11.0"`. For a source integration, use `{ path = "../gpui-material/crates/material" }` and pin the checkout commit. After an upgrade, run `cargo tree -d` to check for duplicate package versions.

```rust
use gpui::{AppContext, Context, IntoElement, QuitMode, Render, Window, WindowOptions};
use gpui_material::{Button, MaterialRoot, MaterialTheme};

struct Demo;

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("save", "Save").on_activate(|_, _| println!("Save requested"))
    }
}

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            gpui_material::init(MaterialTheme::default(), cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Demo))
                .expect("open Material window");
        });
}
```

`gpui_material::init` installs the application theme, initializes Base, and registers component key bindings when component features are enabled (internally calling `MaterialRoot::init` and, when `display` is enabled, `init_components`). The window opened through `MaterialRoot::open_window` has the Base root that manages overlays and focus traversal. The application owns controlled values and command outcomes. Keep entity-backed state, such as `SliderState` and `TextFieldState`, alive across renders.

## Material Symbols

Add `gpui-material-design-symbols = "0.4.1"` alongside `gpui-material` for the full Material Symbols catalog. In a source integration, use `gpui-material-design-symbols = { path = "../gpui-material/crates/design-symbols" }`. The catalog contains all 4,304 names at the pinned revision, including aliases. Use `Symbol::ALL` to browse it or `Symbol::from_name("favorite")` to resolve an upstream name. [Upstream codepoints and font assets](https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/variablefont)

On desktop, enable `embed-assets` on `gpui-material-design-symbols` and open an Outlined, Rounded, or Sharp font with `SymbolFont::embedded`. The fonts preserve `FILL` and `wght` with grade 0 and optical size 24. `SymbolAxes::default()` (or `SymbolAxes::new()` in `const` contexts) uses an unfilled glyph and weight 400. Use `.filled(true)` for FILL=1, `Weight::new(550)` for an intermediate weight in the inclusive 100–700 range, and `is_filled()` or `get_weight()` to inspect configured axes.

```rust
use gpui_material::Icon;
use gpui_material_design_symbols::{Symbol, SymbolAxes, SymbolFont, SymbolStyle, Weight};

let font = SymbolFont::embedded(SymbolStyle::Rounded);
let axes = SymbolAxes::default().filled(true).weight(Weight::BOLD);
let favorite = font.svg_data(Symbol::Favorite, axes).expect("favorite glyph");
let icon = Icon::svg_data(favorite);
```

Keep the returned SVG bytes in view or application state if you reuse the same glyph and axes across renders. `Icon::svg_data` inherits the icon slot's size and color and supports `.label(...)`, `.size(...)`, `.color(...)`, rotation, and RTL mirroring. Fill and weight are applied when generating the SVG. Existing `Icon::new(IconName::Favorite)` uses the built-in system icons with zero asset setup.

For browser WASM, serve the desired `.ttf` files and their license as separate static assets. The application controls when a style loads and can retain the resulting `SymbolFont` or the prepared SVG bytes in view state. While awaiting the font, the view can render a built-in icon and notify GPUI after replacing it. `fetch` reports network, HTTP, and invalid-font errors. Use a same-origin URL or configure CORS on the asset host.

```rust
use gpui_material_design_symbols::{Symbol, SymbolAxes, SymbolFont};

let font = SymbolFont::fetch("/fonts/material-symbols-rounded.ttf").await?;
let favorite = font.svg_data(Symbol::Favorite, SymbolAxes::default())?;
```

The [companion assets](../crates/design-symbols/assets/) include the three ready-to-serve fonts. `embed-assets` embeds fonts only on native targets, including when Cargo enables the feature for a WASM build. For a custom subset or application-managed download, pass the TrueType bytes to `SymbolFont::from_bytes`. Requests for glyphs omitted by a subset return `SymbolError::MissingGlyph`.

To regenerate the assets, place the three upstream `.ttf` files, their `.codepoints` files, and the upstream `LICENSE` in one directory. Install fontTools 4.63.0 and run `python xtask/material-symbols.py <source-directory>` from the checkout. [Pinned inputs and checksums](../crates/design-symbols/assets/manifest.toml)

## API and migration

Search and dropdown virtualization require no caller migration. Result and option IDs must remain unique and stable across reordering. `ListItem::track_focus(&FocusHandle)` paired with `.tab_stop(bool)` lets a caller-owned virtual collection supply retained row focus and its roving tab stop; that collection must also own full-model navigation and reveal offscreen targets. The existing `List` API is unchanged.

`set_material_theme` transitions application colors with the destination motion scheme's Slow Effects spring. A new request starts from the displayed palette. Initialization and reduced motion apply colors immediately. The destination typography and layout take effect when the change is requested. Base receives the application palette and scaled text metrics through the application theme projection.

`set_window_material_theme` (also available as `foundation::set_window_material_theme`, alongside `window_material_theme`) sets the Material theme for one window. Use `ThemeScope::new(child).theme(theme).direction(Direction::Rtl).strings(strings)` to override a subtree. All builders are optional: direction or strings alone inherit the surrounding theme. Material descendants, multiline `TextField` scrollbars, deferred exit overlays, and `ThemeScope::bind_action` callbacks resolve the closest scope before the window and application defaults without mutating `gpui_base::Theme::global`. Standalone Base controls can call `foundation::current_base_theme(window, cx)` (or `foundation::project_base_theme(theme, cx)`) to read the matching Base tokens and scrollbar styles for the active local scope.

Built-in default control labels and accessibility copy (such as `Banner` dismiss, `TextField` clear and password visibility toggles, `ExposedDropdown` options toggles, `Dialog` cancel, `Sheet` close and resize handle, `Scrim` backdrop dismissal, overflow triggers, and `SearchView` / `CaptionButtons` defaults) read from `MaterialStrings` whenever an explicit per-component setter is omitted. Call `set_material_strings(strings, cx)` (read via `material_strings(cx)` or `current_material_strings(window, cx)`) to localize defaults application-wide with zero third-party i18n dependencies, or use `ThemeScope::new(child).strings(strings)` to override strings for a subtree (including deferred overlays via `ThemeScope::inherit`). Explicit per-component setters (`.dismiss_label(...)`, `.clear_label(...)`, `.close_label(...)`, `.accessibility_label(...)`, etc.) always take precedence over `MaterialStrings`.

To follow operating-system appearance and reduced-motion preferences without manual `WindowAppearance` mapping, read the initial appearance with `system_appearance(cx)` (or `appearance_from_window(window.appearance())`, which maps `Light`/`VibrantLight` to `Appearance::Light` and `Dark`/`VibrantDark` to `Appearance::Dark`) and keep a window synchronized via `observe_system_appearance(window, cx, AppearanceMode::System, build_theme)` (or `MaterialRoot::sync_system_appearance`). The helper applies the initial resolved theme immediately, subscribes to `window.observe_window_appearance`, skips redundant updates when the resolved `Appearance` is unchanged, and updates the application theme through `set_material_theme` while preserving window-level overrides on other windows. Use `observe_window_system_appearance` (or `MaterialRoot::sync_window_system_appearance`) to track OS appearance inside a single window via `set_window_material_theme`. `gpui_material::init` reads the OS reduced-motion setting once through `gpui-base`; call `sync_system_reduce_motion(cx)` to refresh the OS reading on demand, or `set_reduce_motion_preference(ReduceMotionPreference::..., cx)` to apply an explicit override or resume system tracking.

The supported application entry point is the [public facade](../crates/material/src/lib.rs), including its documented `foundation` and `theme` paths. By default `gpui-material` enables `full`, which compiles all core component groups (`display`, `actions`, `selection`, `inputs`, `surfaces`, `settings-tiles`, `feedback`, and `navigation`). To compile a minimal slice, set `default-features = false` and list only the required features (for example, `features = ["display"]` for lightweight display primitives, `features = ["actions", "inputs"]` for selected core groups, or `features = ["window-chrome"]` / `features = ["carousel"]` without compiling `gpui-material-components` or its embedded SVGs at all). Enable optional families at the facade with the [feature names in the README](../README.md); each optional family activates only the core groups it uses. `date-time` exposes its API and `jiff::civil::{Date, Time, Weekday}` re-exports under `gpui_material::date_time`. `dynamic-color` and `serde` forward to the pure theme crate. GPUI and Base types in public signatures follow the pinned upstream versions. Gallery modules and the internal organization of the workspace crates may change.

When upgrading to 0.4.0, prefer single-step `gpui_material::init(theme, cx)` at startup, rename `MaterialTheme` `with_*` builders to bare names (`colors`, `typography`, `motion`, `layout`, `density`, `direction`), and replace per-component padding/shape/border/elevation structs with `LogicalEdges`, `ControlShapes`, `ControlBorder`, and `ControlElevations`. Rename `EmptyState` to `EmptyView`, unify overlay close callbacks to `.on_closed(...)`, enable `display` or `settings-tiles` explicitly when using `default-features = false`, and migrate `date_time` callers from `chrono` to `jiff::civil::{Date, Time, Weekday}` re-exported under `gpui_material::date_time`. See the [changelog](../CHANGELOG.md) for the complete breaking-change list.

When upgrading to 0.3.3, align direct GPUI imports with `0.3.14` and Base imports with `0.11.0`. Callers using `foundation::Shape` clipping helpers should update `clip_rect`, `clip_disc`, and `clip_convex_polygon` for `Option<gpui::Path>` and switch `clip_path`/`paint_clipped` to `clip_shape`/`paint_clipped_shape`.

When upgrading to 0.3.0, replace the `windows-chrome` Cargo feature and `gpui_material::windows_chrome` module path with the cross-platform `window-chrome` feature and `gpui_material::window_chrome` module, and update exhaustive `IconName` matches to cover `IconName::Visibility` and `IconName::VisibilityOff`.

When upgrading to 0.2.0, update exhaustive `ColorRole` matches to cover the nine new semantic status variants (`Info`, `OnInfo`, `InfoContainer`, `Success`, `OnSuccess`, `SuccessContainer`, `Warning`, `OnWarning`, and `WarningContainer`). The minimum supported Rust version is now `1.95.0`, and `gpui-material` keeps `full` enabled by default so existing component imports continue to build unchanged.

When upgrading to 0.1.1, align direct Base imports with 0.10.0 and keep the paired GPUI packages at 0.3.12. Update exhaustive matches for `AppBarVariant::TwoRows`, `DatePickerVariant::FullScreen`, `SearchViewVariant::FullScreenContained`, and `TimePickerVariant::Scroll`. Changed keyboard and accessibility slider adjustments now emit `Release` after `Change`. Update subscribers that interpreted `Release` as pointer-only.

Applications updating the 0.0.4 facade to 0.1.0 can keep their imports, feature names, and state initialization. Add arms for the four new `IconName` variants when matching the enum exhaustively, or use a fallback for variants that share behavior. See the [changelog](../CHANGELOG.md) for the variant names and other changes.

## Platform support

Enable `window-chrome` to use a [cross-platform custom title bar](component.md#window-chrome) in selected windows across Windows, macOS, and Linux. Configure window options with `title_bar_window_options` at window creation and mount `TitleBar` under the existing application root.

| Target | Status |
| --- | --- |
| Windows 11 x64 (MSVC, DirectX) | Primary supported desktop platform, covering native keyboard/pointer/wheel input, Microsoft Pinyin IME, UI Automation, and Narrator. |
| macOS 15 ARM64 | Built and tested in CI; native desktop integration in progress. |
| Ubuntu 24.04 x64 | Built and tested in CI; native desktop integration in progress. |

<a id="ui-测试"></a>

## UI testing

Enable `test-support` only under `[dev-dependencies]` so production builds carry zero observation overhead:

```toml
[dev-dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14", features = ["test-support"] }
gpui-material = { version = "0.4.1", features = ["test-support"] }
```

With `test-support` enabled, `gpui_material::test` exposes headless UI testing helpers built solely on public GPUI and `gpui-base` APIs (`TestWindowExt`, `ScopedWindow`, `ClickOptions`, `TestQueryExt`, `TestEventExt`, `TestInput`, `TestAppContextExt::wait_for`, `ElementSnapshot`, and `TestSupportExt`). Register custom elements with `div().id("status").test_support()` (before `.track_focus(&handle)` when focusable), open test windows with `MaterialRoot::open_window`, and assert both native accessibility snapshots and application state:

```rust
use gpui::{AppContext, TestAppContext, WindowOptions};
use gpui_material::{
    MaterialRoot, MaterialTheme,
    test::{TestSupportExt as _, TestWindowExt as _},
};

#[gpui::test]
fn saves_form_through_the_ui(cx: &mut TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let v = cx.new(|cx| MyForm::new(window, cx));
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (handle, entity.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("name", cx);
        window.input("Ada", cx);
        window.press("backspace", cx);
        window.click("save", cx);
        assert_eq!(window.find("status").label(), Some("Saved: Ad"));
    })
    .unwrap();
}
```

Use `cx.wait_for(handle.into(), timeout, |window, cx| ...).await` outside `update_window` for deferred or asynchronous UI state. When tests also need real GPU rasterization checks, `Window::render_to_image` captures offscreen RGBA frames without presenting a visible window across Windows DirectX (with WARP fallback), macOS Metal, and Linux WGPU/lavapipe; see [apps/gallery/tests/rendering.rs](../apps/gallery/tests/rendering.rs). See the executable end-to-end example in [crates/material/tests/integration/ui.rs](../crates/material/tests/integration/ui.rs) and the [testing conventions](development.md#testing-conventions).

[Cookbook](cookbook.md) · [Design sources](design-sources.md) · [Versioning and releases](release.md)
