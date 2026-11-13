use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, QuitMode, Render,
    Role, StatefulInteractiveElement, Styled, TextAlign, TitlebarOptions, Window, WindowBounds,
    WindowHandle, WindowOptions, canvas, div, px, size,
};
use gpui_base::{Button, Root, TestSupportExt};
use gpui_material_foundation::{
    MaterialRoot, Surface, color, material_theme, set_material_theme, shape_text,
};
use gpui_material_theme::{
    Appearance, ColorRole, Elevation, MaterialTheme, TypeRole, TypographyEmphasis,
};

const TITLE: &str = "Material Gallery";

fn main() {
    let two_windows = std::env::args().any(|arg| arg == "--two-windows");
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(move |cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            let result = open_gallery(cx, false).and_then(|_| {
                if two_windows {
                    open_gallery(cx, true)?;
                }
                Ok(())
            });
            if let Err(error) = result {
                eprintln!("Could not open Material Gallery: {error:#}");
                std::process::exit(1);
            }
        });
}

fn open_gallery(cx: &mut App, local_dark: bool) -> gpui::Result<WindowHandle<Root>> {
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(
                if local_dark {
                    "Material Gallery · Dark preview"
                } else {
                    TITLE
                }
                .into(),
            ),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(580.), px(360.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    MaterialRoot::open_window(options, cx, |_, cx| cx.new(|_| Gallery { local_dark }))
}

struct Gallery {
    local_dark: bool,
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_theme = material_theme(cx);
        let theme = if self.local_dark {
            MaterialTheme::baseline(Appearance::Dark)
        } else {
            app_theme.clone()
        };
        let appearance = theme.colors.appearance();
        let sample = "Material 3 · 中文排版";
        let title = shape_text(
            sample,
            TypeRole::HeadlineSmall.style(TypographyEmphasis::Baseline),
            &theme.typography,
            1.,
            color(theme.colors.color(ColorRole::OnSurface)),
            window,
            cx,
        )
        .expect("single-line gallery sample");
        let height = title.line_height();
        let colors = &theme.colors;
        let accent = color(colors.color(ColorRole::Primary));
        let on_accent = color(colors.color(ColorRole::OnPrimary));
        let focus = color(colors.color(ColorRole::Secondary));
        let status = format!("Display scale {:.0}%", window.scale_factor() * 100.);
        let theme_status = format!("Window theme {appearance:?}");

        Surface::new(ColorRole::Surface)
            .with_theme(theme.clone())
            .size_full()
            .flex()
            .flex_col()
            .p_6()
            .gap_4()
            .child(
                div()
                    .id("typography")
                    .role(Role::Label)
                    .aria_label(sample)
                    .test_support()
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, cx| {
                                let _ =
                                    title.paint(bounds.origin, TextAlign::Left, None, window, cx);
                            },
                        )
                        .w_full()
                        .h(height),
                    ),
            )
            .child(
                Surface::new(ColorRole::Surface)
                    .with_theme(theme)
                    .tonal_elevation(Elevation::Level3)
                    .shadow_elevation(Elevation::Level2)
                    .rounded_lg()
                    .p_4()
                    .child(
                        div()
                            .id("window-theme")
                            .role(Role::Status)
                            .aria_label(theme_status.clone())
                            .test_support()
                            .child(theme_status),
                    )
                    .child(
                        div()
                            .id("display-scale")
                            .role(Role::Status)
                            .aria_label(status.clone())
                            .test_support()
                            .child(status),
                    ),
            )
            .child(
                Button::new("toggle-theme")
                    .px_4()
                    .py_2()
                    .rounded_full()
                    .border_2()
                    .border_color(accent)
                    .bg(accent)
                    .text_color(on_accent)
                    .focus_visible(|style| style.border_color(focus))
                    .child("Switch app theme")
                    .on_click(|_, _, cx| {
                        let appearance = match material_theme(cx).colors.appearance() {
                            Appearance::Light => Appearance::Dark,
                            Appearance::Dark => Appearance::Light,
                        };
                        set_material_theme(MaterialTheme::baseline(appearance), cx);
                    }),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use gpui_base::{Theme, ThemeAppearance, test_support::find};

    #[gpui::test]
    fn projects_theme_and_paints_status_in_two_windows(cx: &mut TestAppContext) {
        let (first, second) = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            (
                open_gallery(cx, false).unwrap(),
                open_gallery(cx, true).unwrap(),
            )
        });
        let label = |cx: &mut TestAppContext, handle: WindowHandle<Root>, id: &str| {
            let id = gpui::ElementId::from(gpui::SharedString::from(id.to_owned()));
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let status = find(window, &[], &id).expect("rendered status");
                assert_eq!(status.role(), Some(Role::Status));
                assert!(status.visible());
                let bounds = status.bounds();
                assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
                assert!(bounds.right() <= window.viewport_size().width);
                assert!(bounds.bottom() <= window.viewport_size().height);
                status.label().unwrap().to_owned()
            })
            .unwrap()
        };
        assert!(label(cx, first, "display-scale").starts_with("Display scale "));
        assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
        cx.update(|cx| set_material_theme(MaterialTheme::baseline(Appearance::Dark), cx));
        assert_eq!(label(cx, first, "window-theme"), "Window theme Dark");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
        cx.update(|cx| {
            let base = Theme::global(cx);
            assert_eq!(base.appearance, ThemeAppearance::Dark);
            assert_eq!(
                base.tokens.colors.primary,
                color(material_theme(cx).colors.color(ColorRole::Primary))
            );
        });
    }

    #[gpui::test]
    fn mixed_script_uses_one_layout_for_metrics_and_paint(cx: &mut TestAppContext) {
        let handle = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            open_gallery(cx, false).unwrap()
        });
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let theme = material_theme(cx);
            let style = TypeRole::BodyLarge.style(TypographyEmphasis::Baseline);
            let line = shape_text(
                "A中文B",
                style,
                &theme.typography,
                1.25,
                color(theme.colors.color(ColorRole::OnSurface)),
                window,
                cx,
            )
            .unwrap();
            assert_eq!(line.line_height(), px(style.line_height * 1.25));
            assert_eq!(line.line().len(), "A中文B".len());
            assert!(line.line().x_for_index(4) > line.line().x_for_index(1));
            assert_eq!(
                line.line().x_for_index(line.line().len()),
                line.line().width()
            );
        })
        .unwrap();
    }
}
