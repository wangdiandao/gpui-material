mod action_examples;
mod display_examples;
mod drawing;
mod input_examples;
mod selection_examples;
mod slider_examples;

use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, QuitMode, Render,
    Role, StatefulInteractiveElement, Styled, TextAlign, TitlebarOptions, Window, WindowBounds,
    WindowHandle, WindowOptions, canvas, div, px, size,
};
use gpui_base::{Button, Root, TestSupportExt};
use gpui_material::foundation::{
    MaterialRoot, Surface, color, material_theme, set_material_theme, shape_text,
};
use gpui_material::theme::{
    Appearance, ColorRole, Elevation, MaterialTheme, TypeRole, TypographyEmphasis,
};
use gpui_material::{Fab, FloatingAction};

const TITLE: &str = "Material Gallery";

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let two_windows = arguments.iter().any(|arg| arg == "--two-windows");
    let rtl = arguments.iter().any(|arg| arg == "--rtl");
    let reduced_motion = arguments.iter().any(|arg| arg == "--reduce-motion");
    let dark = arguments.iter().any(|arg| arg == "--dark");
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(move |cx| {
            init_gallery(cx);
            if rtl || dark {
                let mut theme = MaterialTheme::baseline(if dark {
                    Appearance::Dark
                } else {
                    Appearance::Light
                });
                if rtl {
                    theme.layout.direction = gpui_material::theme::Direction::Rtl;
                }
                set_material_theme(theme, cx);
            }
            if reduced_motion {
                cx.set_reduce_motion(true);
            }
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

fn init_gallery(cx: &mut App) {
    MaterialRoot::init(MaterialTheme::default(), cx);
    cx.bind_keys([gpui::KeyBinding::new(
        "escape",
        input_examples::CancelDrag,
        Some("FoundationInput"),
    )]);
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
        window_bounds: Some(WindowBounds::centered(size(px(580.), px(620.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    MaterialRoot::open_window(options, cx, |window, cx| {
        cx.new(|cx| Gallery {
            local_dark,
            display: cx.new(|_| display_examples::DisplayExamples::default()),
            sliders: cx.new(slider_examples::SliderExamples::new),
            selection: cx.new(selection_examples::SelectionExamples::new),
            actions: cx.new(|_| action_examples::ActionExamples::default()),
            drawing: cx.new(drawing::Drawing::new),
            input: cx.new(|cx| input_examples::InputExamples::new(window, cx)),
        })
    })
}

struct Gallery {
    local_dark: bool,
    display: gpui::Entity<display_examples::DisplayExamples>,
    sliders: gpui::Entity<slider_examples::SliderExamples>,
    selection: gpui::Entity<selection_examples::SelectionExamples>,
    actions: gpui::Entity<action_examples::ActionExamples>,
    drawing: gpui::Entity<drawing::Drawing>,
    input: gpui::Entity<input_examples::InputExamples>,
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
            .child(
                div()
                    .id("gallery-scroll")
                    .tab_group()
                    .overflow_y_scroll()
                    .size_full()
                    .flex()
                    .flex_col()
                    .p_6()
                    .gap_4()
                    .pb(px(88.))
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
                                        let _ = title.paint(
                                            bounds.origin,
                                            TextAlign::Left,
                                            None,
                                            window,
                                            cx,
                                        );
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
                    .child(self.drawing.clone())
                    .child(self.input.clone())
                    .child(self.actions.clone())
                    .child(self.display.clone())
                    .child(self.sliders.clone())
                    .child(self.selection.clone())
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
                            .accessibility_label("Switch app theme")
                            .child("Switch app theme")
                            .on_click(|_, _, cx| {
                                let appearance = match material_theme(cx).colors.appearance() {
                                    Appearance::Light => Appearance::Dark,
                                    Appearance::Dark => Appearance::Light,
                                };
                                let mut theme = material_theme(cx).clone();
                                theme.colors =
                                    gpui_material::theme::ColorScheme::baseline(appearance);
                                set_material_theme(theme, cx);
                            }),
                    ),
            )
            .child(FloatingAction::new(
                Fab::new("floating-theme", "Toggle reduced motion", "◐")
                    .on_activate(|_, cx| cx.set_reduce_motion(!cx.reduce_motion())),
            ))
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
            init_gallery(cx);
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
            let mut typography = theme.typography.clone();
            typography.plain.primary = "An unavailable Material test font".into();
            let line = shape_text(
                "A中文B",
                style,
                &typography,
                1.25,
                color(theme.colors.color(ColorRole::OnSurface)),
                window,
                cx,
            )
            .unwrap();
            assert_eq!(line.line_height(), px(style.line_height * 1.25));
            assert_eq!(line.line().len(), "A中文B".len());
            assert!(line.line().font_id_for_index(1).is_some());
            assert!(line.line().x_for_index(4) > line.line().x_for_index(1));
            assert_eq!(
                line.line().x_for_index(line.line().len()),
                line.line().width()
            );
        })
        .unwrap();
    }
}
