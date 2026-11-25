# Application recipe

This example opens a Material window with a retained text field, a controlled switch, and a button that reports the current values. Use the [downstream integration guide](../../../docs/integration.md) for the supported Rust toolchain and other dependency configurations.

## `Cargo.toml`

```toml
[package]
name = "material-settings"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = "0.4.0"

# Recommended for smooth dev-build rendering (see practices.md for the full list).
[profile.dev.package]
gpui-base = { opt-level = 3 }
gpui-pre = { opt-level = 3 }
gpui-pre-platform = { opt-level = 3 }
gpui-pre-windows = { opt-level = 3 }
gpui-pre-macos = { opt-level = 3 }
gpui-pre-linux = { opt-level = 3 }
```

## `src/main.rs`

```rust
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, QuitMode, Render, Styled, Window,
    WindowOptions, div,
};
use gpui_material::{Button, MaterialRoot, MaterialTheme, Switch, TextField, TextFieldState};

struct SettingsView {
    display_name: Entity<TextFieldState>,
    reminders_enabled: bool,
}

impl SettingsView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            display_name: cx.new(|cx| TextFieldState::new(window, cx)),
            reminders_enabled: false,
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let switch_owner = cx.entity().downgrade();
        let save_owner = switch_owner.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(TextField::new(
                "display-name",
                "Display name",
                &self.display_name,
            ))
            .child(
                Switch::new("reminders", "Enable reminders", self.reminders_enabled).on_change(
                    move |checked, _, cx| {
                        let _ = switch_owner.update(cx, |view, cx| {
                            view.reminders_enabled = checked;
                            cx.notify();
                        });
                    },
                ),
            )
            .child(
                Button::new("save-settings", "Save settings").on_activate(move |_, cx| {
                    let _ = save_owner.update(cx, |view, cx| {
                        let display_name = view.display_name.read(cx).value();
                        println!(
                            "Display name: {display_name}; reminders: {}",
                            view.reminders_enabled
                        );
                    });
                }),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            gpui_material::init_components(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| SettingsView::new(window, cx))
            })
            .expect("open Material settings window");
        });
}
```

The view retains the `TextFieldState` entity across renders so editing, selection, and focus survive updates. The view owns `reminders_enabled` and applies the switch callback before notifying GPUI. The button reads both values at activation time and prints them to standard output. Replace that handler with the application's save command when adapting the example.
