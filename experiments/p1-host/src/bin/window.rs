use gpui::{
    App, AppContext, Context, IntoElement, Role, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_base::{TestSupportExt, Theme};
use p1_host::open_probe;

struct WindowProbe;

impl Render for WindowProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scale = window.scale_factor();
        let label = format!("窗口探针 · {} · {:.0}%", std::env::consts::OS, scale * 100.);
        let colors = Theme::global(cx).tokens.colors;

        div()
            .id("window-probe")
            .role(Role::Status)
            .aria_label(label.clone())
            .test_support()
            .size_full()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(label)
    }
}
fn options(cx: &App) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("P1-01 窗口探针".into()),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(480.), px(280.)), cx)),
        ..Default::default()
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let window = open_probe(cx, options(cx), |_, cx| cx.new(|_| WindowProbe))
            .expect("open native Base Root");
        window
            .update(cx, |_, window, _| {
                println!(
                    "P1-01 native window: {} {}, scale={}",
                    std::env::consts::OS,
                    std::env::consts::ARCH,
                    window.scale_factor()
                );
            })
            .expect("inspect native window");
    });
}
