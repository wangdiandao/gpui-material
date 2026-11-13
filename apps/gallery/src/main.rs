use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, QuitMode, Render,
    Role, StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowHandle,
    WindowOptions, div, px, size,
};
use gpui_base::{Root, TestSupportExt, Theme};

const TITLE: &str = "Material Gallery";

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            if let Err(error) = open_gallery(cx) {
                eprintln!("Could not open Material Gallery: {error:#}");
                std::process::exit(1);
            }
        });
}

fn open_gallery(cx: &mut App) -> gpui::Result<WindowHandle<Root>> {
    gpui_base::init(cx);
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(TITLE.into()),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(520.), px(300.)), cx)),
        window_min_size: Some(size(px(320.), px(200.))),
        ..Default::default()
    };
    cx.open_window(options, |window, cx| {
        let view = cx.new(|_| Gallery);
        cx.new(|cx| Root::new(view, window, cx))
    })
}

struct Gallery;

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let status = format!("Display scale {:.0}%", window.scale_factor() * 100.);

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_6()
            .gap_2()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(div().text_xl().child("Window check"))
            .child(
                div()
                    .id("display-scale")
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .test_support()
                    .child(status),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn paints_gallery_status_inside_base_root(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| open_gallery(cx).expect("open Gallery in test window"));
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);

        let status = gpui_base::test_support::find(window, &[], &"display-scale".into())
            .expect("rendered scale status");
        assert_eq!(status.role(), Some(Role::Status));
        let expected = format!("Display scale {:.0}%", window.scale_factor() * 100.);
        assert_eq!(status.label(), Some(expected.as_str()));
        assert!(status.visible());
        let bounds = status.bounds();
        assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
        assert!(bounds.right() <= window.viewport_size().width);
        assert!(bounds.bottom() <= window.viewport_size().height);
    })
    .unwrap();
}
