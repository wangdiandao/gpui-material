# Downstream integration

English | [中文](integration.zh.md)

Use Rust 1.99.1 or later with the 2024 edition. The verified minimum toolchain is Rust 1.99.1. Consume the facade from a checkout and pin the checkout commit in your application. Keep the application's Cargo lockfile under version control.

## Add the facade

The following example assumes the Material checkout is next to the application. Match GPUI package identities exactly so the facade and application share the same `App`, `Window` and `Entity` types.

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = { path = "../gpui-material/crates/material" }
```

An application that uses Base directly should also pin `gpui-base = "=0.9.1"`. When moving the checkout to a new commit, run `cargo tree -d` to check for multiple versions and verify the platform and input behavior that the application relies on.

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

## API and migration

Search and dropdown virtualization require no caller migration. Result and option IDs must remain unique and stable across reordering. `ListItem::track_focus(&FocusHandle, bool)` lets a caller-owned virtual collection supply retained row focus and its roving tab stop; that collection must also own full-model navigation and reveal offscreen targets. The existing `List` API is unchanged.

`foundation::set_window_material_theme` overrides Material presentation, including explicit Base editor colors and Material overlay surfaces. It does not replace the application-global Base theme used by standalone Base controls, scrollbars, resize handles, or caret-blink policy. Use explicit local styles for those integrations. Rendering a window never swaps the Base global theme.

The supported application entry point is the [public facade](../crates/material/src/lib.rs), including its documented `foundation` and `theme` paths. Enable optional families at the facade with the [feature names in the README](../README.md). `date-time` exposes its API under `gpui_material::date_time`. `dynamic-color` and `serde` forward to the pure theme crate. GPUI and Base types in public signatures follow the pinned upstream versions. Gallery modules and the internal organization of the workspace crates may change.

Applications using the 0.0.4 facade can move their checkout to the accepted P9 commit with the same imports, feature names, and state initialization. The next version line remains pre-1.0. Review changes to this contract and the migration steps when updating to a new minor release.

## Platform support

| Target | Scope |
| --- | --- |
| Windows 11 x64, MSVC, DirectX | Initial support. Native keyboard and pointer input, mouse wheel, Microsoft Pinyin, UI Automation and Narrator use the Windows acceptance scenarios. The package is a standalone ZIP. |
| macOS 15 ARM64 | CI builds and automated tests. Exploratory native support. |
| Ubuntu 24.04 x64 | CI builds and automated tests. Exploratory native support. |

Touch-first gestures and unverified touchpad or mixed-DPI hardware configurations remain outside the initial native coverage. The pinned Windows Base slider supports keyboard and semantic increment/decrement. Programmatic UI Automation RangeValue writes are tracked in [#49](https://github.com/wangdiandao/gpui-material/issues/49). Native pointer capture loss in GPUI is tracked in [#48](https://github.com/wangdiandao/gpui-material/issues/48).

[Platform acceptance scenarios](verification.md) · [Pinned sources](design-sources.md) · [P9 acceptance](phases/P09.md)
