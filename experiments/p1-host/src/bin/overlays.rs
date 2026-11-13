use gpui::{
    Anchor, AppContext, Context, Entity, FocusHandle, IntoElement, Render, Role, ScrollHandle,
    Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, black, div, prelude::*, px,
    relative, size,
};
use gpui_base::{
    Button, Dialog, DialogPopup, Popover, TestSupportExt, Theme,
    slider::{Slider, SliderIndicator, SliderState, SliderThumb, SliderTrack},
};
use p1_host::open_probe;

struct OverlayProbe {
    trigger: FocusHandle,
    child_trigger: FocusHandle,
    modal_trigger: FocusHandle,
    modal_focus: FocusHandle,
    first: FocusHandle,
    last: FocusHandle,
    modal_open: bool,
    rtl: bool,
    inner_scroll: ScrollHandle,
    outer_scroll: ScrollHandle,
    slider: Entity<SliderState>,
}

impl OverlayProbe {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            trigger: cx.focus_handle(),
            child_trigger: cx.focus_handle(),
            modal_trigger: cx.focus_handle(),
            modal_focus: cx.focus_handle(),
            first: cx.focus_handle(),
            last: cx.focus_handle(),
            modal_open: false,
            rtl: false,
            inner_scroll: ScrollHandle::new(),
            outer_scroll: ScrollHandle::new(),
            slider: cx.new(|_| SliderState::new().max(100.).default_value(42.)),
        }
    }
}

struct OverlayDrag;

impl Render for OverlayDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("浮层内拖动")
    }
}

impl Render for OverlayProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let child_trigger = self.child_trigger.clone();
        let modal_owner = cx.entity().downgrade();
        let slider_value = self.slider.read(cx).value().end();
        let value_label = format!("当前值 {slider_value:.0}");
        let slider_position = self.slider.read(cx).percentage().end;
        let anchor = if self.rtl {
            Anchor::TopRight
        } else {
            Anchor::TopLeft
        };
        let trigger = move |id, label, focus: &FocusHandle| {
            Button::new(id)
                .track_focus(focus)
                .border_1()
                .border_color(colors.border)
                .px_3()
                .py_2()
                .focus_visible(move |style| style.border_color(colors.ring))
                .accessibility_label(label)
                .child(label)
        };

        div()
            .size_full()
            .tab_group()
            .flex()
            .justify_end()
            .when(self.rtl, |element| {
                element.justify_start().flex_row_reverse()
            })
            .items_start()
            .gap_2()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                Button::new("direction")
                    .child(if self.rtl { "RTL" } else { "LTR" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.rtl = !this.rtl;
                        cx.notify();
                    })),
            )
            .child(
                trigger("modal-trigger", "打开模态…", &self.modal_trigger).on_click(cx.listener(
                    |this, _, window, cx| {
                        this.modal_open = true;
                        this.modal_focus.focus(window, cx);
                        cx.notify();
                    },
                )),
            )
            .child(
                Popover::new("outer")
                    .anchor(anchor)
                    .offset(px(4.))
                    .trigger(trigger("outer-trigger", "打开浮层…", &self.trigger))
                    .content(move |_, _, _| {
                        div()
                            .id("outer-panel")
                            .role(Role::Group)
                            .test_support()
                            .aria_label("外层浮层")
                            .min_w(px(176.))
                            .bg(colors.surface)
                            .border_1()
                            .border_color(colors.border)
                            .p_2()
                            .child(
                                Popover::new("inner")
                                    .anchor(anchor)
                                    .offset(px(4.))
                                    .trigger(trigger(
                                        "inner-trigger",
                                        "打开子浮层…",
                                        &child_trigger,
                                    ))
                                    .content(move |_, _, _| {
                                        div()
                                            .id("inner-panel")
                                            .role(Role::Status)
                                            .test_support()
                                            .aria_label("子浮层已打开")
                                            .min_w(px(176.))
                                            .bg(colors.surface)
                                            .border_1()
                                            .border_color(colors.border)
                                            .p_2()
                                            .child("子浮层已打开")
                                    }),
                            )
                    }),
            )
            .child(
                Dialog::new(cx)
                    .open(self.modal_open)
                    .focus_handle(self.modal_focus.clone())
                    .accessibility_label("滚动与拖动")
                    .on_ok(|_, _, _| false)
                    .on_open_change(move |open, _, window, cx| {
                        let _ = modal_owner.update(cx, |this, cx| {
                            this.modal_open = open;
                            window.release_pointer();
                            cx.stop_active_drag(window);
                            this.modal_trigger.focus(window, cx);
                            cx.notify();
                        });
                    })
                    .backdrop(div().absolute().size_full().bg(black().opacity(0.36)))
                    .popup(
                        DialogPopup::new()
                            .w(px(240.))
                            .p_3()
                            .bg(colors.surface)
                            .child(trigger("modal-first", "首项", &self.first))
                            .child(
                                div()
                                    .id("outer-scroll")
                                    .test_support()
                                    .h(px(100.))
                                    .overflow_y_scroll()
                                    .track_scroll(&self.outer_scroll)
                                    .child(
                                        div()
                                            .id("inner-scroll")
                                            .test_support()
                                            .h(px(64.))
                                            .overflow_y_scroll()
                                            .occlude()
                                            .track_scroll(&self.inner_scroll)
                                            .child(
                                                div()
                                                    .id("overlay-drag")
                                                    .test_support()
                                                    .role(Role::Group)
                                                    .aria_label("浮层内拖动")
                                                    .child("浮层内拖动")
                                                    .on_drag(OverlayDrag, |_, _, _, cx| {
                                                        cx.new(|_| OverlayDrag)
                                                    }),
                                            )
                                            .child(div().h(px(160.))),
                                    )
                                    .child(div().h(px(160.))),
                            )
                            .child(
                                Slider::new(&self.slider)
                                    .accessibility_label("标签示例值")
                                    .child(
                                        SliderTrack::new(&self.slider)
                                            .relative()
                                            .w_full()
                                            .h_4()
                                            .bg(colors.muted)
                                            .child(SliderIndicator::new(&self.slider).size_full())
                                            .child(
                                                SliderThumb::new(&self.slider)
                                                    .absolute()
                                                    .top_0()
                                                    .left(relative(slider_position))
                                                    .ml(px(-8.))
                                                    .size_4()
                                                    .rounded_full()
                                                    .bg(colors.primary),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .id("value-label")
                                    .test_support()
                                    .role(Role::Status)
                                    .aria_label(value_label.clone())
                                    .child(value_label),
                            )
                            .child(trigger("modal-last", "末项", &self.last)),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(480.), px(320.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-07 浮层探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| cx.new(OverlayProbe::new)).unwrap();
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn nested_popovers_keep_focus_and_fit_in_window(cx: &mut gpui::TestAppContext) {
    use gpui::App;
    use gpui_base::test_support;

    let (window, view) = cx.update(|cx| {
        gpui_base::init(cx);
        let view = cx.new(OverlayProbe::new);
        let window = open_probe(cx, WindowOptions::default(), |_, _| view.clone()).unwrap();
        (window, view)
    });

    let frame = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        frame(window, cx);
        view.read(cx).trigger.clone().focus(window, cx);
        frame(window, cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "enter");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        let outer = test_support::find(window, &[], &"outer-panel".into()).unwrap();
        assert!(outer.bounds().right() <= window.viewport_size().width);
        view.read(cx).child_trigger.clone().focus(window, cx);
        frame(window, cx);
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "enter");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        let inner = test_support::find(window, &[], &"inner-panel".into()).unwrap();
        assert!(inner.bounds().right() <= window.viewport_size().width);
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(test_support::find(window, &[], &"inner-panel".into()).is_none());
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_some());
        assert!(view.read(cx).child_trigger.is_focused(window));
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_none());
        assert!(view.read(cx).trigger.is_focused(window));
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn popover_repositions_after_resize_and_dismisses_on_outside_press(cx: &mut gpui::TestAppContext) {
    use gpui::{InputEvent, MouseButton, MouseDownEvent, point};
    use gpui_base::test_support;

    let window = cx.update(|cx| {
        gpui_base::init(cx);
        open_probe(cx, WindowOptions::default(), |_, cx| {
            cx.new(OverlayProbe::new)
        })
        .unwrap()
    });
    cx.simulate_window_resize(window.into(), size(px(480.), px(320.)));
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        let trigger = test_support::find(window, &[], &"outer-trigger".into()).unwrap();
        let modifiers = window.modifiers();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: trigger.bounds().center(),
                modifiers,
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.run_until_parked();

    cx.simulate_window_resize(window.into(), size(px(260.), px(200.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let panel = test_support::find(window, &[], &"outer-panel".into()).unwrap();
        assert_eq!(window.scale_factor(), 1.5);
        assert!(panel.bounds().left() >= px(0.));
        assert!(panel.bounds().right() <= window.viewport_size().width);
        assert!(panel.bounds().bottom() <= window.viewport_size().height);

        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: point(px(8.), px(190.)),
                modifiers: window.modifiers(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_none());
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn modal_owns_focus_scrim_scroll_and_drag(cx: &mut gpui::TestAppContext) {
    use gpui::{
        App, InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent,
        MouseMoveEvent, MouseUpEvent, ScrollDelta, ScrollEvent, point,
    };
    use gpui_base::test_support;

    let (window, view) = cx.update(|cx| {
        gpui_base::init(cx);
        let view = cx.new(OverlayProbe::new);
        let window = open_probe(cx, WindowOptions::default(), |_, _| view.clone()).unwrap();
        (window, view)
    });
    cx.simulate_window_resize(window.into(), size(px(480.), px(320.)));
    let frame = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };
    let open = |window: &mut Window, cx: &mut App| {
        let keystroke = Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        frame(window, cx);
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        frame(window, cx);
    };
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        frame(window, cx);
        view.read(cx).modal_trigger.clone().focus(window, cx);
        frame(window, cx);
        open(window, cx);
        assert!(view.read(cx).modal_open);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "tab");
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(view.read(cx).modal_open);
        assert!(view.read(cx).first.is_focused(window));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "shift-tab");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(view.read(cx).last.is_focused(window));
        let value = test_support::find(window, &[], &"value-label".into()).unwrap();
        let slider = test_support::find(
            window,
            &[],
            &("slider", view.read(cx).slider.entity_id()).into(),
        )
        .unwrap();
        assert!(value.bounds().top() >= slider.bounds().bottom());
        let outer = test_support::find(window, &[], &"outer-scroll".into()).unwrap();
        let inner = test_support::find(window, &[], &"inner-scroll".into()).unwrap();
        assert!(slider.bounds().top() >= outer.bounds().bottom());
        let modifiers = window.modifiers();
        let scroll = |window: &mut Window, cx: &mut App, position, y| {
            window.dispatch_event(
                ScrollEvent {
                    position,
                    delta: ScrollDelta::Pixels(point(px(0.), px(y))),
                    modifiers,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            frame(window, cx);
        };
        scroll(window, cx, inner.bounds().center(), -24.);
        assert_eq!(view.read(cx).inner_scroll.offset().y, px(-24.));
        assert_eq!(view.read(cx).outer_scroll.offset().y, px(0.));
        scroll(
            window,
            cx,
            point(outer.bounds().center().x, inner.bounds().bottom() + px(8.)),
            -24.,
        );
        assert_eq!(view.read(cx).inner_scroll.offset().y, px(-24.));
        assert_eq!(view.read(cx).outer_scroll.offset().y, px(-24.));
        view.read(cx).inner_scroll.set_offset(point(px(0.), px(0.)));
        view.read(cx).outer_scroll.set_offset(point(px(0.), px(0.)));
        frame(window, cx);

        let source = test_support::find(window, &[], &"overlay-drag".into()).unwrap();
        let position = source.bounds().center();
        let outside = point(px(8.), px(280.));
        window.dispatch_event(
            MouseDownEvent {
                position,
                modifiers,
                button: MouseButton::Left,
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        frame(window, cx);
        for position in [position + point(px(12.), px(0.)), outside] {
            window.dispatch_event(
                MouseMoveEvent {
                    position,
                    modifiers,
                    pressed_button: Some(MouseButton::Left),
                }
                .to_platform_input(),
                cx,
            );
            frame(window, cx);
            assert!(cx.has_active_drag());
            assert!(view.read(cx).modal_open);
        }
        assert_eq!(view.read(cx).outer_scroll.offset().y, px(0.));
        assert_eq!(view.read(cx).inner_scroll.offset().y, px(0.));
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(!view.read(cx).modal_open);
        assert!(!cx.has_active_drag());
        assert!(view.read(cx).modal_trigger.is_focused(window));
        let position = point(px(8.), px(280.));
        let modifiers = window.modifiers();
        window.dispatch_event(
            MouseUpEvent {
                position,
                modifiers,
                button: MouseButton::Left,
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        frame(window, cx);
    })
    .unwrap();

    cx.update_window(window.into(), |_, window, cx| {
        open(window, cx);
        assert!(view.read(cx).modal_open);
        let background = test_support::find(window, &[], &"outer-trigger".into()).unwrap();
        let modifiers = window.modifiers();
        window.dispatch_event(
            MouseDownEvent {
                position: background.bounds().center(),
                modifiers,
                button: MouseButton::Left,
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        frame(window, cx);
        assert!(!view.read(cx).modal_open);
        assert!(view.read(cx).modal_trigger.is_focused(window));
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_none());
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn popover_maps_logical_start_in_both_directions(cx: &mut gpui::TestAppContext) {
    use gpui_base::test_support;

    for rtl in [false, true] {
        let window = cx.update(|cx| {
            gpui_base::init(cx);
            open_probe(cx, WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let mut probe = OverlayProbe::new(cx);
                    probe.rtl = rtl;
                    probe.trigger.focus(window, cx);
                    probe
                })
            })
            .unwrap()
        });
        cx.simulate_window_resize(window.into(), size(px(480.), px(320.)));
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.simulate_keystrokes(window.into(), "enter");
        cx.run_until_parked();
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let panel = test_support::find(window, &[], &"outer-panel".into())
                .unwrap()
                .bounds();
            let trigger = test_support::find(window, &[], &"outer-trigger".into())
                .unwrap()
                .bounds();
            assert!(panel.left() >= px(0.) && panel.right() <= window.viewport_size().width);
            assert!(panel.top() >= trigger.bottom());
            let trigger_is_right = trigger.center().x > window.viewport_size().width / 2.;
            assert_ne!(trigger_is_right, rtl);
            if rtl {
                assert!(panel.left() > px(0.) && panel.right() > trigger.right());
            } else {
                assert!(
                    panel.right() < window.viewport_size().width && panel.left() < trigger.left()
                );
            }
        })
        .unwrap();
    }
}
