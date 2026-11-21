# GPUI Material

English | [中文](README.zh.md)

<p align="center">
  <img src="./website/public/logo.svg" width="112" height="112" alt="GPUI Material logo" />
</p>

Build native Rust desktop applications with Material 3 Expressive components for GPUI. The library provides Material color, typography, shape, and motion, with GPUI Base handling input, focus, and window services.

## Try the Gallery

Run the Gallery from a source checkout. Search for a component in the sidebar, try its interactive examples, and switch between light and dark themes. Resize the window on the Adaptive layout page to see navigation and content panes respond to the available space.

```sh
git clone https://github.com/wangdiandao/gpui-material.git
cd gpui-material
cargo gallery
```

Windows 11 x64 with the MSVC toolchain is the primary supported environment. macOS ARM64 and Linux x64 receive CI builds and automated tests. See [Development and verification](docs/development.md) for prerequisites on each platform.

## Use in an application

Use Rust 1.95.0 or later with the 2024 edition. Add `gpui-material = "0.2.0"` to your dependencies. The [integration guide](docs/integration.md) lists the matching GPUI packages and shows a complete window example.

Initialize `MaterialRoot` and call `gpui_material::init_components` before opening a window through `MaterialRoot::open_window`. Build views from the `gpui_material` facade and keep controlled values and command outcomes in your application.

```rust
use gpui_material::Button;

let save = Button::new("save", "Save")
    .on_activate(|_, _| println!("Save requested"));
```

## Components and features

- **Actions and selection.** Buttons, floating actions, checkboxes, radio groups, switches, chips, and sliders.
- **Content and feedback.** Cards, lists, badges, progress indicators, loading indicators, and snackbars.
- **Input and overlays.** Filled, outlined, multiline, and secure text fields, exposed dropdowns, menus, tooltips, dialogs, and sheets.
- **Navigation and layout.** App bars, tabs, navigation bars and rails, scaffolds, list-detail, supporting panes, and feeds.

Enable optional features on `gpui-material` for additional components and theme capabilities.

| Feature | Adds |
| --- | --- |
| `expressive-groups` | Button groups and split buttons |
| `toolbars` | FAB menus, floating toolbars, and docked toolbars |
| `search` | Search bars and search views |
| `date-time` | Date, date range, and time pickers |
| `carousel` | Six carousel layouts |
| `window-chrome` | Cross-platform custom title bar and window controls |
| `dynamic-color` | Color schemes generated from a seed color |
| `serde` | Serialization for theme data |

See the [component guide](docs/component.md) for supported controls, variants, and optional component features.

## Project documentation

- [Documentation website](https://wangdiandao.github.io/gpui-material/)
- [Downstream integration](docs/integration.md)
- [Architecture and dependencies](docs/architecture.md)
- [Documentation index](docs/index.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

Original project code and documentation are licensed under [Apache-2.0](LICENSE). See [Third-party notices](THIRD_PARTY_NOTICES.md) for bundled assets and dependency notices.
