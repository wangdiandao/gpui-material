//! Contract tests for `gpui_material::test` harness primitives.

use std::time::Duration;

use gpui::{
    AppContext, Context, FocusHandle, InteractiveElement, IntoElement, Modifiers, MouseButton,
    ParentElement, Render, Role, StatefulInteractiveElement, Styled, TestAppContext, Window,
    WindowBounds, WindowOptions, div, point, px, size,
};
use gpui_material::{
    MaterialRoot, MaterialTheme,
    test::{
        ClickOptions, TestAppContextExt as _, TestEventExt as _, TestQueryExt as _,
        TestSupportExt as _, TestWindowExt as _,
    },
};

#[gpui::test]
fn scoped_queries_accessibility_queries_and_step_events_work(cx: &mut TestAppContext) {
    struct TwoPanels {
        toolbar_clicks: usize,
        dialog_clicks: usize,
        shift_clicks: usize,
        pointer_down_count: usize,
        pointer_up_inside: usize,
        focus: FocusHandle,
    }

    impl Render for TwoPanels {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(
                    div().id("toolbar").flex().gap_2().child(
                        div()
                            .id("save")
                            .role(Role::Button)
                            .aria_label("Save toolbar")
                            .test_support()
                            .track_focus(&self.focus)
                            .w(px(120.))
                            .h(px(40.))
                            .on_click(cx.listener(|this, event: &gpui::ClickEvent, _, cx| {
                                this.toolbar_clicks += 1;
                                if event.modifiers().shift {
                                    this.shift_clicks += 1;
                                }
                                cx.notify();
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    this.pointer_down_count += 1;
                                }),
                            )
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    this.pointer_up_inside += 1;
                                }),
                            ),
                    ),
                )
                .child(
                    div().id("dialog").flex().gap_2().child(
                        div()
                            .id("save")
                            .role(Role::Button)
                            .aria_label("Save dialog")
                            .test_support()
                            .w(px(120.))
                            .h(px(40.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dialog_clicks += 1;
                                cx.notify();
                            })),
                    ),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        let mut entity = None;
        let bounds = gpui::Bounds {
            origin: gpui::Point::default(),
            size: size(px(480.), px(320.)),
        };
        let handle = MaterialRoot::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |_, cx| {
                let v = cx.new(|cx| TwoPanels {
                    toolbar_clicks: 0,
                    dialog_clicks: 0,
                    shift_clicks: 0,
                    pointer_down_count: 0,
                    pointer_up_inside: 0,
                    focus: cx.focus_handle(),
                });
                entity = Some(v.clone());
                v
            },
        )
        .unwrap();
        (handle, entity.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        assert_eq!(window.find_all("save").len(), 2);
        assert_eq!(window.find_all_by_role(Role::Button).len(), 2);
        assert_eq!(
            window.find_by_label("Save dialog").label(),
            Some("Save dialog")
        );
        assert!(window.try_find_by_label("Missing").is_none());

        // Scoped queries disambiguate identical child IDs without adding containers.
        window.within("toolbar").click("save", cx);
        window.within("dialog").click("save", cx);
        window.within("toolbar").click_with_options(
            "save",
            ClickOptions::new().with_modifiers(Modifiers::shift()),
            cx,
        );
        assert_eq!(window.modifiers(), Modifiers::default());

        // Stepwise pointer drag-out cancels inside mouse_up.
        let center = window.within("toolbar").find("save").bounds().center();
        window.pointer_down(center, MouseButton::Left, cx);
        window.pointer_move(point(px(400.), px(280.)), Some(MouseButton::Left), cx);
        window.pointer_up(point(px(400.), px(280.)), MouseButton::Left, cx);
    })
    .unwrap();

    cx.update(|cx| {
        let model = view.read(cx);
        assert_eq!(model.toolbar_clicks, 2);
        assert_eq!(model.dialog_clicks, 1);
        assert_eq!(model.shift_clicks, 1);
        assert_eq!(model.pointer_down_count, 3);
        assert_eq!(model.pointer_up_inside, 2);
    });
}

#[gpui::test]
async fn wait_for_polls_test_clock_until_deferred_ui_appears(cx: &mut TestAppContext) {
    struct DeferredView {
        ready: bool,
    }

    impl Render for DeferredView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().children(self.ready.then(|| {
                div()
                    .id("ready-badge")
                    .role(Role::Status)
                    .aria_label("Loaded")
                    .test_support()
                    .w(px(80.))
                    .h(px(24.))
            }))
        }
    }

    let (handle, view) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let v = cx.new(|_| DeferredView { ready: false });
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (handle, entity.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("ready-badge").is_none());
    })
    .unwrap();

    cx.update(|cx| {
        let view = view.clone();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            view.update(cx, |this, cx| {
                this.ready = true;
                cx.notify();
            });
        })
        .detach();
    });

    cx.wait_for(handle.into(), Duration::from_millis(200), |window, _| {
        window
            .try_find("ready-badge")
            .is_some_and(|snapshot| snapshot.visible() && snapshot.label() == Some("Loaded"))
    })
    .await;
}

#[gpui::test]
#[should_panic(expected = "focus binding was not observed")]
fn missed_focus_binding_order_is_diagnosed(cx: &mut TestAppContext) {
    struct MisorderedFocus(FocusHandle);

    impl Render for MisorderedFocus {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("broken-focus")
                .role(Role::Button)
                // Intentionally wrong order: track_focus before test_support.
                .track_focus(&self.0)
                .test_support()
                .w(px(60.))
                .h(px(32.))
        }
    }

    let handle = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| MisorderedFocus(cx.focus_handle()))
        })
        .unwrap()
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let _ = window.find("broken-focus").focused();
    })
    .unwrap();
}
