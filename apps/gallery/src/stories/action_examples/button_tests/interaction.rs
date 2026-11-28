use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    App, AppContext, Context, ElementId, Entity, InputEvent, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Render, Role, StyleRefinement, Styled, TestAppContext, Window, WindowOptions,
    div, point, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, InteractionSource, InteractionUpdate, Switch, ToggleButton,
    foundation::{MaterialRoot, color, window_material_theme},
    theme::ColorRole,
};

use super::super::ActionExamples;
use crate::story::Story;

#[gpui::test]
fn action_and_selection_feedback_clear_after_pointer_cancellation(cx: &mut TestAppContext) {
    #[derive(Default)]
    struct Controls {
        disabled: bool,
        activations: usize,
    }
    impl Render for Controls {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let button = cx.entity();
            let switch = cx.entity();
            div()
                .flex()
                .gap_4()
                .child(
                    Button::new("button", "Action")
                        .disabled(self.disabled)
                        .on_activate(move |_, cx| {
                            button.update(cx, |model, _| model.activations += 1)
                        }),
                )
                .child(
                    Switch::new("switch", "Selection", false)
                        .disabled(self.disabled)
                        .on_change(move |_, _, cx| {
                            switch.update(cx, |model, _| model.activations += 1)
                        }),
                )
        }
    }
    let (handle, model) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut model = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let controls = cx.new(|_| Controls::default());
            model = Some(controls.clone());
            controls
        })
        .unwrap();
        (handle, model.unwrap())
    });
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let draw = |window: &mut Window, cx: &mut App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        let feedback = |id: &'static str, window: &Window, cx: &App| {
            if id == "switch" {
                f32::from(
                    find(window, &[], &(ElementId::from(id), "switch-thumb").into())
                        .unwrap()
                        .bounds()
                        .size
                        .width,
                )
            } else {
                let tint = color(
                    window_material_theme(window, cx)
                        .colors
                        .color(ColorRole::Primary),
                );
                crate::painted_quads(window)
                    .into_iter()
                    .find(|quad| quad.background == tint.into())
                    .unwrap()
                    .corner_radii
                    .top_left
                    .0
                    / window.scale_factor()
            }
        };
        draw(window, cx);
        for (id, resting, pressed) in [("button", 20., 8.), ("switch", 16., 28.)] {
            for disable in [false, true] {
                let position = find(window, &[], &id.into()).unwrap().bounds().center();
                window.dispatch_event(
                    MouseDownEvent {
                        button: MouseButton::Left,
                        position,
                        click_count: 1,
                        first_mouse: false,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                draw(window, cx);
                assert_eq!(feedback(id, window, cx), pressed);
                if disable {
                    model.update(cx, |model, cx| {
                        model.disabled = true;
                        cx.notify();
                    });
                    draw(window, cx);
                }
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position: point(px(700.), px(500.)),
                        click_count: 1,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                model.update(cx, |model, cx| {
                    model.disabled = false;
                    cx.notify();
                });
                draw(window, cx);
                assert_eq!(feedback(id, window, cx), resting);
                assert_eq!(model.read(cx).activations, 0);
            }
        }
    })
    .unwrap();
}

#[gpui::test]
fn button_feedback_preserves_unrelated_cached_view(cx: &mut TestAppContext) {
    struct ButtonPane(gpui::FocusHandle, usize);
    impl Render for ButtonPane {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.1 += 1;
            Button::new("local-feedback", "Press")
                .track_focus(&self.0)
                .on_activate(|_, _| {})
        }
    }

    struct Sibling(usize);
    impl Render for Sibling {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.0 += 1;
            div().child("Cached sibling")
        }
    }

    struct Panes(Entity<ButtonPane>, Entity<Sibling>);
    impl Render for Panes {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(self.0.clone().cached(StyleRefinement::default().h(px(64.))))
                .child(self.1.clone().cached(StyleRefinement::default().h(px(64.))))
        }
    }

    let (handle, button, sibling, focus) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let focus = cx.focus_handle();
        let button = cx.new(|_| ButtonPane(focus.clone(), 0));
        let sibling = cx.new(|_| Sibling(0));
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Panes(button.clone(), sibling.clone()))
        })
        .unwrap();
        (handle, button, sibling, focus)
    });
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    let before = cx
        .update_window(handle.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            let position = find(window, &[], &"local-feedback".into())
                .unwrap()
                .bounds()
                .center();
            focus.focus(window, cx);
            window.dispatch_event(
                MouseMoveEvent {
                    position: point(px(500.), px(500.)),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.draw(cx).clear(cx);
            let counts = (button.read(cx).1, sibling.read(cx).0);
            window.dispatch_event(
                MouseMoveEvent {
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            counts
        })
        .unwrap();
    cx.update_window(handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        assert!(button.read(cx).1 > before.0);
        assert_eq!(sibling.read(cx).0, before.1);
    })
    .unwrap();
}

#[gpui::test]
fn button_keyboard_press_does_not_resume_after_focus_returns(cx: &mut TestAppContext) {
    struct Buttons;
    impl Render for Buttons {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .gap_4()
                .child(Button::new("first", "First").on_activate(|_, _| {}))
                .child(Button::new("second", "Second").on_activate(|_, _| {}))
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Buttons))
            .unwrap()
    });
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let draw = |window: &mut Window, cx: &mut App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        let radius = |window: &Window, cx: &App| {
            let tint = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::Primary),
            );
            let quad = crate::painted_quads(window)
                .into_iter()
                .find(|quad| quad.background == tint.into())
                .expect("filled button paints its primary container");
            quad.corner_radii.top_left.0 / window.scale_factor()
        };
        draw(window, cx);
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(radius(window, cx), 20.);
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key,
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(radius(window, cx), 8.);
        window.focus_next(cx);
        draw(window, cx);
        window.focus_prev(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"first".into()).unwrap().focused(),
            Some(true)
        );
        assert_eq!(
            radius(window, cx),
            20.,
            "a cancelled press must not resume when focus returns before key-up"
        );
    })
    .unwrap();
}

#[gpui::test]
fn button_interaction_sources_deliver_short_presses_and_detach(cx: &mut TestAppContext) {
    struct Controls {
        sources: [Entity<InteractionSource>; 2],
        second_source: usize,
        second_parent: Option<&'static str>,
        second_disabled: bool,
        clicks: usize,
        requested: Vec<bool>,
    }

    impl Render for Controls {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let action = cx.entity();
            let toggle = cx.entity();
            div()
                .flex()
                .gap_4()
                .child(
                    Button::new("observed-button", "Action")
                        .interaction_source(&self.sources[0])
                        .on_activate(move |_, cx| {
                            action.update(cx, |model, _| model.clicks += 1);
                        }),
                )
                .children(self.second_parent.map(|parent| {
                    div().id(parent).child(
                        ToggleButton::new("observed-toggle", "Selection", false)
                            .interaction_source(&self.sources[self.second_source])
                            .disabled(self.second_disabled)
                            .on_change(move |selected, _, cx| {
                                toggle.update(cx, |model, _| model.requested.push(selected));
                            }),
                    )
                }))
        }
    }

    let events = Rc::new(RefCell::new(Vec::<InteractionUpdate>::new()));
    let (handle, model, sources) = cx.update(|cx| {
        crate::init_gallery(cx);
        let sources = std::array::from_fn(|_| cx.new(|_| InteractionSource::new()));
        let model = cx.new(|cx: &mut Context<Controls>| {
            let events = events.clone();
            cx.subscribe(&sources[0], move |_, _, event, _| {
                events.borrow_mut().push(event.clone());
            })
            .detach();
            Controls {
                sources: sources.clone(),
                second_source: 0,
                second_parent: Some("original-parent"),
                second_disabled: false,
                clicks: 0,
                requested: Vec::new(),
            }
        });
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (window, model, sources)
    });
    let first = ElementId::from("observed-button");
    let second = ElementId::from("observed-toggle");
    let window_id = cx
        .update_window(handle.into(), |_, window, _| {
            window.activate_window();
            window.window_handle().window_id()
        })
        .unwrap();
    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let read = |source: usize, id: &ElementId, cx: &mut TestAppContext| {
        cx.update(|cx| sources[source].read(cx).state(window_id, id))
    };
    draw(cx);
    assert_eq!(read(0, &first, cx), Some(Default::default()));
    assert_eq!(read(0, &second, cx), Some(Default::default()));

    let press_edges = |id: &ElementId| {
        let mut pressed = false;
        events
            .borrow()
            .iter()
            .filter(|event| &event.id == id)
            .filter_map(|event| {
                let current = event.state?.pressed;
                (std::mem::replace(&mut pressed, current) != current).then_some(current)
            })
            .collect::<Vec<_>>()
    };
    events.borrow_mut().clear();
    cx.update_window(handle.into(), |_, window, cx| {
        let position = find(window, &[], &first).unwrap().bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    assert_eq!(press_edges(&first), [true, false]);
    assert_eq!(cx.update(|cx| model.read(cx).clicks), 1);

    cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
        .unwrap();
    draw(cx);
    events.borrow_mut().clear();
    cx.update_window(handle.into(), |_, window, cx| {
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
    })
    .unwrap();
    let edges = press_edges(&second);
    assert_eq!(edges, [true, false]);
    assert_eq!(cx.update(|cx| model.read(cx).requested.clone()), [true]);

    // Reuse the public ID in a new layout scope while the previous tracker
    // detaches. Its cleanup cannot clear the replacement's snapshot.
    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.second_parent = Some("replacement-parent");
            model.second_disabled = true;
            cx.notify();
        });
    });
    draw(cx);
    assert!(read(0, &second, cx).unwrap().disabled);

    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.second_source = 1;
            cx.notify();
        });
    });
    draw(cx);
    assert!(read(0, &second, cx).is_none());
    assert!(read(1, &second, cx).unwrap().disabled);

    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.second_parent = None;
            cx.notify();
        });
    });
    draw(cx);
    assert!(read(1, &second, cx).is_none());
    assert!(read(0, &first, cx).is_some());
}

#[gpui::test]
fn extended_fab_interpolates_to_its_square_extent(cx: &mut TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let actions = cx.new(|_| ActionExamples::for_story(Story::ExtendedFab));
            view = Some(actions.clone());
            actions
        })
        .unwrap();
        (handle, view.unwrap())
    });
    let widths = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            [0_usize, 1, 2].map(|idx| {
                find(window, &[], &("extended-fab-size", idx).into())
                    .unwrap()
                    .bounds()
                    .size
                    .width
            })
        })
        .unwrap()
    };
    let start = widths(cx);
    cx.update(|cx| {
        view.update(cx, |actions, cx| {
            actions.collapsed = true;
            cx.notify();
        })
    });
    assert_eq!(widths(cx), start);
    cx.executor().advance_clock(Duration::from_millis(40));
    let middle = widths(cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    let end = widths(cx);
    assert_eq!(end, [px(56.), px(80.), px(96.)]);
    for idx in 0..3 {
        assert!(end[idx] < middle[idx] && middle[idx] < start[idx]);
    }
}

#[gpui::test]
fn material_buttons_activate_once_and_toggles_update_the_owner(cx: &mut TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let actions = cx.new(|_| ActionExamples::default());
            view = Some(actions.clone());
            actions
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1000.), px(2000.)));
    let draw = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        for (id, expected, destination) in [
            (ElementId::from(("action-variant", 0_usize)), 1, None),
            (
                ElementId::from(("action-variant", 0_usize)),
                1,
                Some(point(px(999.), px(1999.))),
            ),
            (ElementId::from("action-disabled"), 1, None),
            (ElementId::from(("toggle-variant", 0_usize)), 1, None),
        ] {
            let target = find(window, &[], &id).unwrap();
            assert_eq!(target.role(), Some(Role::Button));
            assert!(target.visible(), "{id}: {:?}", target.bounds());
            let position = target.bounds().center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: destination.unwrap_or(position),
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert_eq!(view.read(cx).count, expected);
        }
        assert!(view.read(cx).toggles[0]);
        let toggle = find(window, &[], &("toggle-variant", 0_usize).into()).unwrap();
        assert_eq!(toggle.focused(), Some(true));
    })
    .unwrap();
    cx.update_window(handle.into(), |_, window, cx| {
        let keystroke = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        draw(window, cx);
        assert!(!view.read(cx).toggles[0]);
        assert_eq!(view.read(cx).count, 1);
    })
    .unwrap();
}
