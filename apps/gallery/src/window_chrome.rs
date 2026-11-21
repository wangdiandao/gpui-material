use gpui::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_material::{
    Switch, Text,
    foundation::{color, window_material_theme},
    theme::{ColorRole, TypeRole},
    window_chrome::TitleBar,
};
use gpui_material_story::Gallery;

pub(super) struct ChromePreview {
    gallery: Entity<Gallery>,
    block_close: bool,
    blocked_requests: usize,
}

impl ChromePreview {
    pub(super) fn new(gallery: Entity<Gallery>, window: &Window, cx: &mut Context<Self>) -> Self {
        let preview = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            preview
                .update(cx, |this, cx| {
                    if !this.block_close {
                        return true;
                    }
                    this.blocked_requests += 1;
                    cx.notify();
                    false
                })
                .unwrap_or(true)
        });
        Self {
            gallery,
            block_close: false,
            blocked_requests: 0,
        }
    }
}

impl Render for ChromePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let surface = color(theme.colors.color(ColorRole::Surface));
        let preview = cx.weak_entity();
        let block_close = Switch::new("block-window-close", "Block close", self.block_close)
            .on_change(move |value, _, cx| {
                preview
                    .update(cx, |this, cx| {
                        this.block_close = value;
                        cx.notify();
                    })
                    .ok();
            });
        let status = format!("Blocked close requests: {}", self.blocked_requests);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(TitleBar::new("window-title-bar").content(block_close))
            .child(
                div()
                    .id("blocked-close-status")
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .bg(surface)
                    .px_4()
                    .pb_1()
                    .child(Text::new(status).role(TypeRole::LabelMedium)),
            )
            .child(div().flex_1().min_h_0().child(self.gallery.clone()))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        AppContext, Context, ElementId, InputEvent as _, InteractiveElement, IntoElement,
        ParentElement, Render, Role, Styled, TestAppContext, Window, WindowOptions, div, px, size,
    };
    use gpui_base::{Root, test_support::find};
    use gpui_material::{
        Button, MaterialRoot, MaterialTheme,
        foundation::set_window_material_theme,
        theme::{Appearance, Direction, TypeRole},
        window_chrome::{ChromePlatform, TitleBar, title_bar_window_options_with_height},
    };
    use gpui_material_story::init_gallery;

    struct TitleBarHarness {
        platform: ChromePlatform,
        height: gpui::Pixels,
        traffic_light_padding: gpui::Pixels,
        show_caption_buttons: Option<bool>,
        closed_count: Rc<Cell<usize>>,
        leading_count: Rc<Cell<usize>>,
        trailing_count: Rc<Cell<usize>>,
    }

    impl Render for TitleBarHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let closed = self.closed_count.clone();
            let leading = self.leading_count.clone();
            let trailing = self.trailing_count.clone();
            let mut bar = TitleBar::new("custom-title-bar")
                .title("Cross-Platform App")
                .height(self.height)
                .title_role(TypeRole::TitleMedium)
                .title_font_family("Roboto")
                .title_font_size(px(15.))
                .traffic_light_padding(self.traffic_light_padding)
                .platform(self.platform)
                .leading(
                    Button::new("chrome-leading", "Menu").on_activate(move |_, _| {
                        leading.set(leading.get() + 1);
                    }),
                )
                .content(div().id("chrome-content").child("Center content"))
                .trailing(
                    Button::new("chrome-trailing", "Action").on_activate(move |_, _| {
                        trailing.set(trailing.get() + 1);
                    }),
                )
                .on_close(move |_, _| {
                    closed.set(closed.get() + 1);
                });
            if let Some(show) = self.show_caption_buttons {
                bar = bar.show_caption_buttons(show);
            }
            div().flex().flex_col().size_full().child(bar)
        }
    }

    fn click_id(cx: &mut TestAppContext, handle: gpui::WindowHandle<Root>, id: ElementId) {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let target = find(window, &[], &id).unwrap();
            assert!(target.visible());
            let position = target.bounds().center();
            for input in [
                gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
            ] {
                window.dispatch_event(input, cx);
            }
        })
        .unwrap();
        cx.run_until_parked();
    }

    #[gpui::test]
    fn windows_chrome_renders_custom_height_slots_and_accessible_caption_buttons(
        cx: &mut TestAppContext,
    ) {
        let closed = Rc::new(Cell::new(0));
        let leading = Rc::new(Cell::new(0));
        let trailing = Rc::new(Cell::new(0));
        let handle = cx.update(|cx| {
            init_gallery(cx);
            cx.set_reduce_motion(true);
            let options = title_bar_window_options_with_height(WindowOptions::default(), px(56.));
            MaterialRoot::open_window(options, cx, |_, cx| {
                cx.new(|_| TitleBarHarness {
                    platform: ChromePlatform::Windows,
                    height: px(56.),
                    traffic_light_padding: px(84.),
                    show_caption_buttons: None,
                    closed_count: closed.clone(),
                    leading_count: leading.clone(),
                    trailing_count: trailing.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(900.), px(600.)));

        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);

            let min = find(window, &[], &"minimize".into()).unwrap();
            let max = find(window, &[], &"maximize-restore".into()).unwrap();
            let close = find(window, &[], &"close".into()).unwrap();
            let leading_btn = find(window, &[], &"chrome-leading".into()).unwrap();
            let trailing_btn = find(window, &[], &"chrome-trailing".into()).unwrap();

            assert_eq!(min.role(), Some(Role::Button));
            assert_eq!(min.label(), Some("Minimize"));
            assert_eq!(max.role(), Some(Role::Button));
            assert_eq!(max.label(), Some("Maximize"));
            assert_eq!(close.role(), Some(Role::Button));
            assert_eq!(close.label(), Some("Close"));

            assert_eq!(close.bounds().size.height, px(56.));
            assert!(leading_btn.bounds().left() < trailing_btn.bounds().left());
            assert!(trailing_btn.bounds().right() <= min.bounds().left());
            assert!(min.bounds().right() <= max.bounds().left());
            assert!(max.bounds().right() <= close.bounds().left());
        })
        .unwrap();

        click_id(cx, handle, "chrome-leading".into());
        click_id(cx, handle, "chrome-trailing".into());
        click_id(cx, handle, "close".into());
        assert_eq!(leading.get(), 1);
        assert_eq!(trailing.get(), 1);
        assert_eq!(closed.get(), 1);
    }

    #[gpui::test]
    fn macos_chrome_reserves_traffic_light_padding_and_hides_caption_buttons(
        cx: &mut TestAppContext,
    ) {
        let closed = Rc::new(Cell::new(0));
        let leading = Rc::new(Cell::new(0));
        let trailing = Rc::new(Cell::new(0));
        let handle = cx.update(|cx| {
            init_gallery(cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| TitleBarHarness {
                    platform: ChromePlatform::MacOs,
                    height: px(40.),
                    traffic_light_padding: px(84.),
                    show_caption_buttons: None,
                    closed_count: closed.clone(),
                    leading_count: leading.clone(),
                    trailing_count: trailing.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(900.), px(600.)));

        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);

            let leading_btn = find(window, &[], &"chrome-leading".into()).unwrap();
            assert!(
                leading_btn.bounds().left() >= px(84.),
                "leading slot should sit after macOS traffic-light padding, got {:?}",
                leading_btn.bounds().left()
            );
            assert!(find(window, &[], &"minimize".into()).is_none());
            assert!(find(window, &[], &"maximize-restore".into()).is_none());
            assert!(find(window, &[], &"close".into()).is_none());
        })
        .unwrap();
    }

    #[gpui::test]
    fn linux_chrome_respects_server_vs_client_decorations_and_rtl_layout(cx: &mut TestAppContext) {
        let closed = Rc::new(Cell::new(0));
        let leading = Rc::new(Cell::new(0));
        let trailing = Rc::new(Cell::new(0));

        // Under headless TestWindow, default window_decorations is Server -> caption buttons hidden.
        let server_handle = cx.update(|cx| {
            init_gallery(cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| TitleBarHarness {
                    platform: ChromePlatform::Linux,
                    height: px(44.),
                    traffic_light_padding: px(78.),
                    show_caption_buttons: None,
                    closed_count: closed.clone(),
                    leading_count: leading.clone(),
                    trailing_count: trailing.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(server_handle.into(), size(px(800.), px(500.)));
        cx.update_window(server_handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            assert!(find(window, &[], &"close".into()).is_none());
        })
        .unwrap();

        // Explicit CSD caption buttons in RTL reverse caption order and handle pointer clicks.
        let csd_handle = cx.update(|cx| {
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let mut theme = MaterialTheme::baseline(Appearance::Light);
                theme.layout.direction = Direction::Rtl;
                set_window_material_theme(Some(theme), window, cx);
                cx.new(|_| TitleBarHarness {
                    platform: ChromePlatform::Linux,
                    height: px(44.),
                    traffic_light_padding: px(78.),
                    show_caption_buttons: Some(true),
                    closed_count: closed.clone(),
                    leading_count: leading.clone(),
                    trailing_count: trailing.clone(),
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(csd_handle.into(), size(px(800.), px(500.)));
        cx.update_window(csd_handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let min = find(window, &[], &"minimize".into()).unwrap();
            let max = find(window, &[], &"maximize-restore".into()).unwrap();
            let close = find(window, &[], &"close".into()).unwrap();
            assert_eq!(close.bounds().size.height, px(44.));
            // In RTL, Close sits on the physical left edge.
            assert!(close.bounds().right() <= max.bounds().left());
            assert!(max.bounds().right() <= min.bounds().left());
        })
        .unwrap();

        click_id(cx, csd_handle, "close".into());
        assert_eq!(closed.get(), 1);
    }
}
