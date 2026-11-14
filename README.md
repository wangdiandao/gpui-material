# gpui-material

English | [中文](README.zh.md)

A native desktop Material 3 Expressive component library for Rust and GPUI. gpui-base provides behavior, input, focus, and infrastructure. gpui-material defines themes, drawing, component state presentation, and adaptive desktop composition.

The initial release targets light and dark themes, brand and dynamic color, keyboard and native assistive technology, Chinese input, RTL, window resizing, and high DPI. Windows is the primary acceptance platform. macOS and Linux have participated in capability probes since P1. [P2 engineering baseline](docs/phases/P02.md) · [Platforms and quality requirements](docs/verification.md)

Run `cargo gallery` on Windows 11 x64/MSVC to open the Gallery. The component examples use the `gpui-material` facade. [Toolchain, tests, and CI artifacts](docs/development.md)

Use `crates/material` as a path dependency from a checkout. Initialize `MaterialRoot` with a `MaterialTheme` before opening a window. `gpui_material::Button::new("save", "Save").on_activate(handler)` creates a labeled action. Selection controls request a change through the caller's callback, and sliders use a Base-backed `SliderState`. The facade forwards the optional `dynamic-color` and `serde` theme features.

Enable optional component families on the facade as needed.

| Feature | Components |
| --- | --- |
| `expressive-groups` | ButtonGroup and SplitButton |
| `toolbars` | FAB Menu and Floating and Docked Toolbar |
| `search` | SearchBar and SearchView |
| `date-time` | DatePicker, date range selection, and TimePicker |
| `carousel` | Six Carousel layouts |

- [P5 basic components](docs/phases/P05.md)
- [P8 composite and extended components](docs/phases/P08.md)
- [Documentation index](docs/index.md)
- [Contribution guide](CONTRIBUTION.md)
- [Component coverage matrix](docs/component-coverage.md)
- [Phase status](status.toml)

Project code and original documentation are licensed under [Apache-2.0](LICENSE). [Third-party notices](THIRD_PARTY_NOTICES.md)
