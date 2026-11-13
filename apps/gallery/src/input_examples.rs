use gpui::{
    Anchor, AppContext, Axis, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Role, ScrollHandle, StatefulInteractiveElement, Styled,
    Subscription, Window, div, px, relative,
};
use gpui_base::{
    Button, Dialog, DialogPopup, Popover, Radio, RadioGroup, TestSupportExt, Theme,
    input::{Input, InputAccessibility, InputState},
    slider::{SliderIndicator, SliderState, SliderThumb, SliderTrack},
};
use gpui_material_foundation::{
    DragCapture, RangeThumb, Surface, isolated_scroll, logical_anchor, material_theme,
    move_roving_focus, range_semantics, scrim_color, value_label,
};
use gpui_material_theme::{ColorRole, Direction, Elevation};

gpui::actions!(foundation_gallery, [CancelDrag]);

pub(crate) struct InputExamples {
    direction: Direction,
    choice: usize,
    radios: [FocusHandle; 3],
    trigger: FocusHandle,
    nested_trigger: FocusHandle,
    dialog_trigger: FocusHandle,
    dialog_focus: FocusHandle,
    dialog_open: bool,
    outer_open: bool,
    nested_open: bool,
    range: Entity<SliderState>,
    range_focus: FocusHandle,
    text: Entity<InputState>,
    outer_scroll: ScrollHandle,
    inner_scroll: ScrollHandle,
    drag: Option<DragCapture>,
    drag_focus: FocusHandle,
    drag_status: &'static str,
    _range: Subscription,
    _activation: Subscription,
}

impl InputExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let range = cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.));
        let range_subscription = cx.observe(&range, |_, _, cx| cx.notify());
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.cancel_drag(window, cx);
            }
        });
        Self {
            direction: Direction::Ltr,
            choice: 0,
            radios: std::array::from_fn(|_| cx.focus_handle()),
            trigger: cx.focus_handle(),
            nested_trigger: cx.focus_handle(),
            dialog_trigger: cx.focus_handle(),
            dialog_focus: cx.focus_handle(),
            dialog_open: false,
            outer_open: false,
            nested_open: false,
            range,
            range_focus: cx.focus_handle().tab_stop(true),
            text: cx.new(|cx| {
                let mut state = InputState::new(window, cx);
                state.set_accessibility(
                    InputAccessibility::new(Some(Role::TextInput))
                        .with_label(Some("Text input".into()))
                        .with_accessibility_id(Some("foundation-text".into())),
                );
                state
            }),
            outer_scroll: ScrollHandle::new(),
            inner_scroll: ScrollHandle::new(),
            drag: None,
            drag_focus: cx.focus_handle(),
            drag_status: "Ready to drag",
            _range: range_subscription,
            _activation: activation,
        }
    }

    fn cancel_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(drag) = self.drag.take() {
            drag.cancel(window, cx);
            self.drag_status = "Drag cancelled";
            cx.notify();
        }
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.drag.take().is_some() {
            self.drag_status = "Drag ended";
            cx.notify();
        }
    }
}

struct DragPreview;

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Surface::new(ColorRole::PrimaryContainer)
            .rounded_md()
            .p_2()
            .child("Dragging")
    }
}

impl Render for InputExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = material_theme(cx);
        let colors = Theme::global(cx).tokens.colors;
        let scrim = scrim_color(theme, 0.32);
        let anchor = logical_anchor(Anchor::TopLeft, self.direction);
        let trigger = move |id, label: &'static str, focus: &FocusHandle| {
            Button::new(id)
                .track_focus(focus)
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .focus_visible(move |style| style.border_color(colors.ring))
                .accessibility_label(label)
                .child(label)
        };
        let popup = move || {
            Surface::new(ColorRole::SurfaceContainerHigh)
                .shadow_elevation(Elevation::Level2)
                .rounded_md()
                .p_3()
        };
        let nested_trigger = self.nested_trigger.clone();
        let nested_open = self.nested_open;
        let owner = cx.entity().downgrade();
        let drag_owner = owner.clone();
        let outer_owner = owner.clone();
        let inner_owner = owner.clone();
        let nested_activation_owner = owner.clone();
        let position = self.range.read(cx).percentage().end;
        let label = value_label(&self.range, RangeThumb::End, cx, |value| {
            format!("{value:.0}").into()
        });
        let range = range_semantics(
            div()
                .id("foundation-range")
                .track_focus(&self.range_focus)
                .aria_label("Level"),
            &self.range,
            RangeThumb::End,
            Axis::Horizontal,
            cx,
        )
        .w_48()
        .px_2()
        .py_2()
        .border_1()
        .border_color(colors.border)
        .rounded_md()
        .focus_visible(move |style| style.border_color(colors.ring))
        .child(
            SliderTrack::new(&self.range)
                .relative()
                .h_4()
                .bg(colors.muted)
                .rounded_full()
                .child(SliderIndicator::new(&self.range).size_full())
                .child(
                    SliderThumb::new(&self.range)
                        .absolute()
                        .left(relative(position))
                        .ml(px(-8.))
                        .size_4()
                        .rounded_full()
                        .bg(colors.primary),
                ),
        );

        div()
            .id("foundation-input")
            .key_context("FoundationInput")
            .on_action(cx.listener(|this, _: &CancelDrag, window, cx| {
                this.cancel_drag(window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .flex()
            .flex_col()
            .gap_2()
            .child(
                RadioGroup::new("roving-focus")
                    .axis(Axis::Horizontal)
                    .flex()
                    .gap_2()
                    .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                        let items = [
                            (&this.radios[0], true),
                            (&this.radios[1], false),
                            (&this.radios[2], true),
                        ];
                        if let Some(ix) = move_roving_focus(
                            &items,
                            &event.keystroke.key,
                            Axis::Horizontal,
                            this.direction,
                            window,
                            cx,
                        ) {
                            this.choice = ix;
                            cx.stop_propagation();
                            cx.notify();
                        }
                    }))
                    .children(
                        ["First", "Unavailable", "Last"]
                            .into_iter()
                            .enumerate()
                            .map(|(ix, label)| {
                                let owner = owner.clone();
                                Radio::new(("choice", ix))
                                    .track_focus(&self.radios[ix])
                                    .checked(self.choice == ix)
                                    .disabled(ix == 1)
                                    .tab_stop(self.choice == ix)
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(colors.border)
                                    .focus_visible(move |style| style.border_color(colors.ring))
                                    .accessibility_label(label)
                                    .child(label)
                                    .on_change(move |_, _, _, cx| {
                                        let _ = owner.update(cx, |this, cx| {
                                            this.choice = ix;
                                            cx.notify();
                                        });
                                    })
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("direction")
                            .accessibility_label("Switch overlay direction")
                            .px_3()
                            .py_2()
                            .child(if self.direction == Direction::Ltr {
                                "Use RTL placement"
                            } else {
                                "Use LTR placement"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.direction = if this.direction == Direction::Ltr {
                                    Direction::Rtl
                                } else {
                                    Direction::Ltr
                                };
                                cx.notify();
                            })),
                    )
                    .child(
                        Popover::new("outer-popup")
                            .anchor(anchor)
                            .open(self.outer_open)
                            .on_open_change(move |open, window, cx| {
                                let _ = outer_owner.update(cx, |this, cx| {
                                    this.outer_open = *open;
                                    if !open {
                                        this.nested_open = false;
                                        this.trigger.focus(window, cx);
                                    }
                                    cx.notify();
                                });
                            })
                            .trigger(
                                trigger("open-popover", "Open popover", &self.trigger).on_click(
                                    cx.listener(|this, event, window, cx| {
                                        if matches!(event, gpui::ClickEvent::Keyboard(_)) {
                                            this.trigger.focus(window, cx);
                                            this.outer_open = !this.outer_open;
                                            cx.notify();
                                        }
                                    }),
                                ),
                            )
                            .content(move |_, _, _| {
                                popup().child(
                                    Popover::new("inner-popup")
                                        .anchor(anchor)
                                        .open(nested_open)
                                        .on_open_change(move |open, window, cx| {
                                            let _ = inner_owner.update(cx, |this, cx| {
                                                this.nested_open = *open;
                                                if !open {
                                                    this.nested_trigger.focus(window, cx);
                                                }
                                                cx.notify();
                                            });
                                        })
                                        .trigger(
                                            trigger(
                                                "open-nested",
                                                "Open nested popover",
                                                &nested_trigger,
                                            )
                                            .on_click(
                                                move |event, window, cx| {
                                                    if matches!(
                                                        event,
                                                        gpui::ClickEvent::Keyboard(_)
                                                    ) {
                                                        let _ = nested_activation_owner.update(
                                                            cx,
                                                            |this, cx| {
                                                                this.nested_trigger
                                                                    .focus(window, cx);
                                                                this.nested_open =
                                                                    !this.nested_open;
                                                                cx.notify();
                                                            },
                                                        );
                                                    }
                                                },
                                            ),
                                        )
                                        .content(move |_, _, _| {
                                            popup().child(
                                                div()
                                                    .id("nested-panel")
                                                    .role(Role::Status)
                                                    .aria_label("Nested popover open")
                                                    .test_support()
                                                    .child("Nested popover open"),
                                            )
                                        }),
                                )
                            }),
                    )
                    .child(
                        trigger("open-dialog", "Open dialog", &self.dialog_trigger).on_click(
                            cx.listener(|this, _, window, cx| {
                                this.dialog_open = true;
                                this.dialog_focus.focus(window, cx);
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .child(
                div().flex().items_center().gap_3().child(range).child(
                    div()
                        .id("level-label")
                        .role(Role::Status)
                        .aria_label(format!("Level {label}"))
                        .test_support()
                        .child(label),
                ),
            )
            .child(
                div()
                    .id("drag-source")
                    .role(Role::Group)
                    .aria_label("Drag sample")
                    .test_support()
                    .track_focus(&self.drag_focus)
                    .on_mouse_down(MouseButton::Left, {
                        let focus = self.drag_focus.clone();
                        move |_, window, cx| focus.focus(window, cx)
                    })
                    .px_3()
                    .py_2()
                    .border_1()
                    .border_color(colors.border)
                    .on_drag((), move |_, _, window, cx| {
                        let _ = drag_owner.update(cx, |this, cx| {
                            let weak = cx.entity().downgrade();
                            this.drag = Some(DragCapture::watch(window, cx, move |_, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.drag.take();
                                    this.drag_status = "Capture lost";
                                    cx.notify();
                                });
                            }));
                            this.drag_status = "Dragging";
                            cx.notify();
                        });
                        cx.new(|_| DragPreview)
                    })
                    .child("Drag sample"),
            )
            .child(
                div()
                    .id("drag-status")
                    .role(Role::Status)
                    .aria_label(self.drag_status)
                    .test_support()
                    .child(self.drag_status),
            )
            .child(
                Dialog::new(cx)
                    .open(self.dialog_open)
                    .focus_handle(self.dialog_focus.clone())
                    .accessibility_label("Text and scrolling")
                    .on_open_change(move |open, _, window, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.dialog_open = open;
                            if !open {
                                this.cancel_drag(window, cx);
                                this.dialog_trigger.focus(window, cx);
                            }
                            cx.notify();
                        });
                    })
                    .backdrop(div().absolute().size_full().bg(scrim))
                    .popup(
                        DialogPopup::new()
                            .w_64()
                            .p_4()
                            .rounded_lg()
                            .bg(colors.surface)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child("Text and scrolling")
                                    .child(
                                        Button::new("close-dialog")
                                            .accessibility_label("Close dialog")
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(colors.border)
                                            .focus_visible(move |style| {
                                                style.border_color(colors.ring)
                                            })
                                            .child("Close")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.dialog_open = false;
                                                this.cancel_drag(window, cx);
                                                this.dialog_trigger.focus(window, cx);
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .border_1()
                                            .border_color(colors.border)
                                            .p_2()
                                            .child(Input::new(&self.text)),
                                    )
                                    .child(
                                        div()
                                            .id("outer-scroll")
                                            .test_support()
                                            .h_32()
                                            .overflow_y_scroll()
                                            .track_scroll(&self.outer_scroll)
                                            .child(
                                                isolated_scroll("inner-scroll", &self.inner_scroll)
                                                    .test_support()
                                                    .h_20()
                                                    .children((1..=8).map(|number| {
                                                        div()
                                                            .p_1()
                                                            .child(format!("Inner row {number}"))
                                                    })),
                                            )
                                            .children((1..=8).map(|number| {
                                                div().p_1().child(format!("Outer row {number}"))
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        InputEvent, MouseDownEvent, MouseMoveEvent, ScrollDelta, ScrollEvent, TestAppContext,
        point, size,
    };
    use gpui_base::test_support::find;
    use gpui_material_foundation::MaterialRoot;

    #[gpui::test]
    fn focus_range_drag_and_dialog_use_the_production_adapters(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle =
                MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |window, cx| {
                    let input = cx.new(|cx| InputExamples::new(window, cx));
                    view = Some(input.clone());
                    input
                })
                .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(600.), px(440.)));
        let draw = |window: &mut Window, cx: &mut gpui::App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            draw(window, cx);
            view.read(cx).radios[0].clone().focus(window, cx);
            draw(window, cx);
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "right");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(view.read(cx).radios[2].is_focused(window));
            assert_eq!(view.read(cx).choice, 2);
            view.read(cx).range_focus.clone().focus(window, cx);
            draw(window, cx);
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "right");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            draw(window, cx);
            assert_eq!(
                find(window, &[], &"level-label".into()).unwrap().label(),
                Some("Level 4")
            );
            let source = find(window, &[], &"drag-source".into())
                .unwrap()
                .bounds()
                .center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: source,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            window.dispatch_event(
                MouseMoveEvent {
                    position: source + point(px(24.), px(0.)),
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert!(cx.has_active_drag());
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "escape");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(!cx.has_active_drag());
            assert_eq!(view.read(cx).drag_status, "Drag cancelled");
            let position = window.mouse_position();
            window.dispatch_event(
                gpui::MouseUpEvent {
                    position,
                    modifiers: Default::default(),
                    button: MouseButton::Left,
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
            view.read(cx).dialog_trigger.clone().focus(window, cx);
            draw(window, cx);
            assert!(view.read(cx).dialog_trigger.is_focused(window));
        })
        .unwrap();
        cx.update_window(handle.into(), |_, window, cx| {
            let keystroke = gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
            draw(window, cx);
            assert!(view.read(cx).dialog_open);
            let inner = find(window, &[], &"inner-scroll".into()).unwrap().bounds();
            window.dispatch_event(
                ScrollEvent {
                    position: inner.center(),
                    delta: ScrollDelta::Pixels(point(px(0.), px(-24.))),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert_eq!(view.read(cx).inner_scroll.offset().y, px(-24.));
            assert_eq!(view.read(cx).outer_scroll.offset().y, px(0.));
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "escape");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(!view.read(cx).dialog_open);
            assert!(view.read(cx).dialog_trigger.is_focused(window));
        })
        .unwrap();
    }
}
