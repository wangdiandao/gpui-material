# Downstream integration

English | [中文](integration.zh.md)

Use Rust 1.95.0 or later with the 2024 edition. Keep the application's Cargo lockfile under version control.

## Add the facade

Add the facade from crates.io. Match GPUI package identities exactly so the facade and application share the same `App`, `Window` and `Entity` types.

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = "0.3.0"
```

An application that uses Base directly should also pin `gpui-base = "=0.10.0"`. For a source integration, use `{ path = "../gpui-material/crates/material" }` and pin the checkout commit. After an upgrade, run `cargo tree -d` to check for multiple versions and verify the platform and input behavior that the application relies on.

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
            MaterialRoot::init(MaterialTheme::default(), cx);
            gpui_material::init_components(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Demo))
                .expect("open Material window");
        });
}
```

`MaterialRoot::init` installs the application theme and initializes Base. `init_components` installs component key bindings. The window opened through `MaterialRoot::open_window` has the Base root that manages overlays and focus traversal. The application owns controlled values and command outcomes. Keep entity-backed state, such as `SliderState` and `TextFieldState`, alive across renders.

## Material Symbols

Add `gpui-material-design-symbols = "0.3.0"` alongside `gpui-material` for the full Material Symbols catalog. In a source integration, use `gpui-material-design-symbols = { path = "../gpui-material/crates/design-symbols" }`. The catalog contains all 4,304 names at the pinned revision, including aliases. Use `Symbol::ALL` to browse it or `Symbol::from_name("favorite")` to resolve an upstream name. [Upstream codepoints and font assets](https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/variablefont)

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

Search and dropdown virtualization require no caller migration. Result and option IDs must remain unique and stable across reordering. `ListItem::track_focus(&FocusHandle, bool)` lets a caller-owned virtual collection supply retained row focus and its roving tab stop; that collection must also own full-model navigation and reveal offscreen targets. The existing `List` API is unchanged.

`set_material_theme` transitions application colors with the destination motion scheme's Slow Effects spring. A new request starts from the displayed palette. Initialization and reduced motion apply colors immediately. The destination typography and layout take effect when the change is requested. Base receives the application palette and scaled text metrics through the application theme projection.

`foundation::set_window_material_theme` sets the Material theme for one window. Use `ThemeScope::new(child).theme(theme).direction(Direction::Rtl)` to override a subtree. Both builders are optional. Direction alone inherits the surrounding theme. Material descendants, multiline `TextField` scrollbars, deferred exit overlays, and `ThemeScope::bind_action` callbacks resolve the closest scope before the window and application defaults without mutating `gpui_base::Theme::global`. Standalone Base controls can call `foundation::current_base_theme(window, cx)` (or `foundation::project_base_theme(theme, cx)`) to read the matching Base tokens and scrollbar styles for the active local scope.

The supported application entry point is the [public facade](../crates/material/src/lib.rs), including its documented `foundation` and `theme` paths. By default `gpui-material` enables `full`, which compiles all core component groups (`actions`, `selection`, `inputs`, `surfaces`, `feedback`, and `navigation`). To compile only a subset of core components, set `default-features = false` and list the required groups (for example, `features = ["actions", "inputs"]`). Enable optional families at the facade with the [feature names in the README](../README.md); each optional family activates only the core groups it uses. `date-time` exposes its API under `gpui_material::date_time`. `dynamic-color` and `serde` forward to the pure theme crate. GPUI and Base types in public signatures follow the pinned upstream versions. Gallery modules and the internal organization of the workspace crates may change.

When upgrading to 0.3.0, replace the `windows-chrome` Cargo feature and `gpui_material::windows_chrome` module path with the cross-platform `window-chrome` feature and `gpui_material::window_chrome` module, and update exhaustive `IconName` matches to cover `IconName::Visibility` and `IconName::VisibilityOff`.

When upgrading to 0.2.0, update exhaustive `ColorRole` matches to cover the nine new semantic status variants (`Info`, `OnInfo`, `InfoContainer`, `Success`, `OnSuccess`, `SuccessContainer`, `Warning`, `OnWarning`, and `WarningContainer`). The minimum supported Rust version is now `1.95.0`, and `gpui-material` keeps `full` enabled by default so existing component imports continue to build unchanged.

When upgrading to 0.1.1, align direct Base imports with 0.10.0 and keep the paired GPUI packages at 0.3.12. Update exhaustive matches for `AppBarVariant::TwoRows`, `DatePickerVariant::FullScreen`, `SearchViewVariant::FullScreenContained`, and `TimePickerVariant::Scroll`. Changed keyboard and accessibility slider adjustments now emit `Release` after `Change`. Update subscribers that interpreted `Release` as pointer-only.

Applications updating the 0.0.4 facade to 0.1.0 can keep their imports, feature names, and state initialization. Add arms for the four new `IconName` variants when matching the enum exhaustively, or use a fallback for variants that share behavior. See the [changelog](../CHANGELOG.md) for the variant names and other changes.

## Platform support

Enable `window-chrome` to use a [cross-platform custom title bar](component.md#window-chrome) in selected windows across Windows, macOS, and Linux. Configure window options with `title_bar_window_options` at window creation and mount `TitleBar` under the existing application root.

| Target | Scope |
| --- | --- |
| Windows 11 x64, MSVC, DirectX | Initial support. Native keyboard and pointer input, mouse wheel, Microsoft Pinyin, UI Automation and Narrator use the Windows acceptance scenarios. The package is a standalone ZIP. |
| macOS 15 ARM64 | CI builds and automated tests. Exploratory native support. |
| Ubuntu 24.04 x64 | CI builds and automated tests. Exploratory native support. |

[Platform acceptance scenarios](verification.md) · [Pinned sources](design-sources.md) · [Versioning and releases](release.md)
