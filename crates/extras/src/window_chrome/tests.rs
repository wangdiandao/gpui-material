use super::platform::configure_window_options;
use super::*;
use gpui::{TitlebarOptions, WindowDecorations, WindowOptions, point, px, size};

#[test]
fn chrome_options_preserve_the_callers_window_settings() {
    let min_size = size(px(360.), px(240.));
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("My window".into()),
            ..Default::default()
        }),
        window_min_size: Some(min_size),
        is_resizable: false,
        is_minimizable: false,
        ..Default::default()
    };

    let options = title_bar_window_options(title_bar_window_options(options));
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(titlebar.title.as_deref(), Some("My window"));
    assert_eq!(options.window_min_size, Some(min_size));
    assert!(!options.is_resizable);
    assert!(!options.is_minimizable);
}

#[test]
fn chrome_options_create_a_titlebar_configuration_when_absent() {
    let options = title_bar_window_options(WindowOptions {
        titlebar: None,
        ..Default::default()
    });
    assert!(options.titlebar.unwrap().appears_transparent);
}

#[test]
fn macos_options_center_traffic_lights_from_height_and_respect_caller_override() {
    let options =
        configure_window_options(WindowOptions::default(), px(40.), ChromePlatform::MacOs);
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(
        titlebar.traffic_light_position,
        Some(point(px(12.), px(13.)))
    );
    assert_eq!(options.window_decorations, None);

    let custom_pos = point(px(16.), px(18.));
    let preserved = configure_window_options(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                traffic_light_position: Some(custom_pos),
                ..Default::default()
            }),
            ..Default::default()
        },
        px(60.),
        ChromePlatform::MacOs,
    );
    assert_eq!(
        preserved.titlebar.unwrap().traffic_light_position,
        Some(custom_pos)
    );
}

#[test]
fn linux_options_request_client_decorations_unless_explicitly_set() {
    let options =
        configure_window_options(WindowOptions::default(), px(48.), ChromePlatform::Linux);
    assert_eq!(options.window_decorations, Some(WindowDecorations::Client));
    assert!(options.titlebar.unwrap().appears_transparent);

    let preserved = configure_window_options(
        WindowOptions {
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        },
        px(48.),
        ChromePlatform::Linux,
    );
    assert_eq!(
        preserved.window_decorations,
        Some(WindowDecorations::Server)
    );
}

#[test]
fn windows_options_do_not_force_traffic_lights_or_linux_decorations() {
    let options =
        configure_window_options(WindowOptions::default(), px(48.), ChromePlatform::Windows);
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(titlebar.traffic_light_position, None);
    assert_eq!(options.window_decorations, None);
}

#[test]
fn invalid_height_falls_back_to_default_for_macos_traffic_lights() {
    let options = configure_window_options(
        WindowOptions::default(),
        px(f32::NAN),
        ChromePlatform::MacOs,
    );
    assert_eq!(
        options.titlebar.unwrap().traffic_light_position,
        Some(point(px(12.), px(17.)))
    );
}

#[cfg(feature = "test-support")]
mod gpui_tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AppContext as _, Bounds, Context, IntoElement, ParentElement, Pixels, Render, Styled,
        TestAppContext, Window, WindowOptions, canvas, div, px, size,
    };
    use gpui_material_foundation::{MaterialRoot, set_window_material_theme};
    use gpui_material_theme::{Appearance, Direction, MaterialTheme};

    use super::*;

    fn bounds_probe(
        slot: Rc<Cell<Option<Bounds<Pixels>>>>,
        width: Pixels,
        height: Pixels,
    ) -> impl IntoElement {
        canvas(
            move |bounds, _, _| {
                slot.set(Some(bounds));
            },
            |_, _, _, _| {},
        )
        .w(width)
        .h(height)
    }

    #[gpui::test]
    fn macos_rtl_keeps_traffic_light_inset_on_physical_left(cx: &mut TestAppContext) {
        struct RtlMacChrome {
            leading: Rc<Cell<Option<Bounds<Pixels>>>>,
            trailing: Rc<Cell<Option<Bounds<Pixels>>>>,
        }

        impl Render for RtlMacChrome {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().flex().flex_col().size_full().child(
                    TitleBar::new("rtl-mac-bar")
                        .platform(ChromePlatform::MacOs)
                        .title("RTL App")
                        .traffic_light_padding(px(84.))
                        .leading(bounds_probe(self.leading.clone(), px(60.), px(28.)))
                        .trailing(bounds_probe(self.trailing.clone(), px(60.), px(28.))),
                )
            }
        }

        let leading = Rc::new(Cell::new(None));
        let trailing = Rc::new(Cell::new(None));
        let handle = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::baseline(Appearance::Light), cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let mut theme = MaterialTheme::baseline(Appearance::Light);
                theme.layout.direction = Direction::Rtl;
                set_window_material_theme(Some(theme), window, cx);
                cx.new(|_| RtlMacChrome {
                    leading: leading.clone(),
                    trailing: trailing.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(800.), px(400.)));

        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();

        let leading_bounds = leading.get().expect("leading slot should be laid out");
        let trailing_bounds = trailing.get().expect("trailing slot should be laid out");

        assert!(
            trailing_bounds.left() >= px(84.),
            "RTL trailing slot must not overlap physical-left traffic lights, got {trailing_bounds:?}"
        );
        assert!(
            leading_bounds.left() > trailing_bounds.right(),
            "In RTL, leading ({leading_bounds:?}) should sit on the physical right of trailing ({trailing_bounds:?})"
        );
    }

    #[gpui::test]
    fn linux_server_decorations_auto_hide_title_bar(cx: &mut TestAppContext) {
        struct LinuxServerChrome {
            leading: Rc<Cell<Option<Bounds<Pixels>>>>,
            below: Rc<Cell<Option<Bounds<Pixels>>>>,
        }

        impl Render for LinuxServerChrome {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(
                        TitleBar::new("linux-server-bar")
                            .platform(ChromePlatform::Linux)
                            .title("Linux Server Window")
                            .leading(bounds_probe(self.leading.clone(), px(40.), px(28.))),
                    )
                    .child(bounds_probe(self.below.clone(), px(100.), px(40.)))
            }
        }

        let leading = Rc::new(Cell::new(None));
        let below = Rc::new(Cell::new(None));
        // Headless TestWindow defaults to Decorations::Server.
        let handle = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::baseline(Appearance::Light), cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| LinuxServerChrome {
                    leading: leading.clone(),
                    below: below.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(800.), px(400.)));

        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();

        assert!(
            leading.get().is_none(),
            "Linux Decorations::Server should auto-hide TitleBar slots"
        );
        let below_bounds = below.get().expect("content below title bar should lay out");
        assert_eq!(
            below_bounds.top(),
            px(0.),
            "auto-hidden TitleBar should occupy zero height"
        );
    }

    #[gpui::test]
    fn caption_buttons_track_keyboard_focus(cx: &mut TestAppContext) {
        struct CaptionFocusChrome;

        impl Render for CaptionFocusChrome {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(CaptionButtons::new("focus-captions").platform(ChromePlatform::Windows))
            }
        }

        let handle = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::baseline(Appearance::Light), cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| CaptionFocusChrome)
            })
            .unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(600.), px(300.)));

        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            assert!(window.focused(cx).is_none());
        })
        .unwrap();

        let mut handles = Vec::new();
        for step in 0..3 {
            let focused = cx
                .update_window(handle.into(), |_, window, cx| {
                    window.focus_next(cx);
                    window.refresh();
                    window.draw(cx).clear(cx);
                    window.focused(cx)
                })
                .unwrap();
            let focused = focused.unwrap_or_else(|| {
                panic!("expected caption button #{step} to receive keyboard focus")
            });
            assert!(
                !handles.contains(&focused),
                "expected distinct FocusHandle for caption button #{step}"
            );
            handles.push(focused);
        }
        assert_eq!(handles.len(), 3);
    }
}
