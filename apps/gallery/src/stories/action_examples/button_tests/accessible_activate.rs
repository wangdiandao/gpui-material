use gpui::{
    AppContext, Context, Entity, InputEvent, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    TestAppContext, Window, WindowOptions, div,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Button, Icon, IconButton, IconName, InteractionSource, foundation::MaterialRoot,
};

use super::super::row;

#[gpui::test]
fn button_and_icon_button_on_accessible_activate_keeps_feedback_without_pointer_click(
    cx: &mut TestAppContext,
) {
    struct View {
        source: Entity<InteractionSource>,
        acc_focus: gpui::FocusHandle,
        accessible_hits: usize,
        full_hits: usize,
    }

    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let acc_btn = cx.entity();
            let acc_icon = cx.entity();
            let override_full = cx.entity();
            let override_acc = cx.entity();
            row()
                .child(
                    Button::new("acc-btn", "Open menu")
                        .track_focus(&self.acc_focus)
                        .interaction_source(&self.source)
                        .on_accessible_activate(move |_, cx| {
                            acc_btn.update(cx, |this, _| this.accessible_hits += 1);
                        }),
                )
                .child(
                    IconButton::new(
                        "acc-icon",
                        "More options",
                        Icon::new(IconName::ArrowDropDown),
                    )
                    .interaction_source(&self.source)
                    .on_accessible_activate(move |_, cx| {
                        acc_icon.update(cx, |this, _| this.accessible_hits += 1);
                    }),
                )
                .child(
                    Button::new("override-to-full", "Run")
                        .on_accessible_activate(|_, _| {})
                        .on_activate(move |_, cx| {
                            override_full.update(cx, |this, _| this.full_hits += 1);
                        }),
                )
                .child(
                    IconButton::new(
                        "override-to-acc",
                        "Trigger",
                        Icon::new(IconName::ArrowDropDown),
                    )
                    .on_activate(move |_, cx| {
                        override_acc.update(cx, |this, _| this.full_hits += 1);
                    })
                    .on_accessible_activate(|_, _| {}),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|cx| View {
            source: cx.new(|_| InteractionSource::new()),
            acc_focus: cx.focus_handle(),
            accessible_hits: 0,
            full_hits: 0,
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view)
    });
    let window = handle.into();
    cx.update_window(window, |_, window, _| window.activate_window())
        .unwrap();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);

    let window_id = window.window_id();
    let source = cx.read_entity(&view, |this, _| this.source.clone());

    cx.update_window(window, |_, window, _| {
        let btn_node = find(window, &[], &"acc-btn".into()).unwrap();
        assert_eq!(btn_node.role(), Some(Role::Button));
        assert_eq!(btn_node.label(), Some("Open menu"));
        let icon_node = find(window, &[], &"acc-icon".into()).unwrap();
        assert_eq!(icon_node.role(), Some(Role::Button));
        assert_eq!(icon_node.label(), Some("More options"));
    })
    .unwrap();

    // Pointer clicks on on_accessible_activate buttons keep press feedback without firing callback.
    for id in ["acc-btn", "acc-icon"] {
        let center = cx
            .update_window(window, |_, window, _| {
                find(window, &[], &id.into()).unwrap().bounds().center()
            })
            .unwrap();
        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: center,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        frame(cx);
        assert!(cx.read_entity(&source, |s, _| {
            s.state(window_id, &id.into()).unwrap().pressed
        }));
        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: center,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        frame(cx);
        assert!(!cx.read_entity(&source, |s, _| {
            s.state(window_id, &id.into()).unwrap().pressed
        }));
    }
    assert_eq!(cx.read_entity(&view, |this, _| this.accessible_hits), 0);

    // Keyboard Space on on_accessible_activate button does not fire on_click either.
    let acc_focus = cx.read_entity(&view, |this, _| this.acc_focus.clone());
    cx.update_window(window, |_, window, cx| {
        acc_focus.focus(window, cx);
        let space = Keystroke::parse("space").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: space.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: space }.to_platform_input(), cx);
    })
    .unwrap();
    frame(cx);
    assert_eq!(cx.read_entity(&view, |this, _| this.accessible_hits), 0);
    assert_eq!(cx.read_entity(&view, |this, _| this.full_hits), 0);

    // Chaining .on_activate after .on_accessible_activate restores pointer activation,
    // and chaining .on_accessible_activate after .on_activate suppresses it.
    for id in ["override-to-full", "override-to-acc"] {
        let center = cx
            .update_window(window, |_, window, _| {
                find(window, &[], &id.into()).unwrap().bounds().center()
            })
            .unwrap();
        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: center,
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
                    position: center,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
    }
    frame(cx);
    assert_eq!(cx.read_entity(&view, |this, _| this.full_hits), 1);
}

#[gpui::test]
fn accessible_activate_triggers_do_not_conflict_with_popover_or_menu_mouse_up(
    cx: &mut TestAppContext,
) {
    use gpui_base::Popover;
    use gpui_material::{Menu, MenuItem};

    #[derive(Default)]
    struct View {
        popover_open: bool,
    }

    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            let toggle_popover = owner.clone();
            row()
                .child(Menu::composed(
                    "composed-menu",
                    [MenuItem::new("entry", "Entry")],
                    |_open, toggle| {
                        Button::new("menu-btn", "Open menu")
                            .on_accessible_activate(move |window, cx| toggle(window, cx))
                            .into_any_element()
                    },
                ))
                .child(
                    Popover::new("custom-popover")
                        .open(self.popover_open)
                        .on_open_change(move |open, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                this.popover_open = *open;
                                cx.notify();
                            });
                        })
                        .trigger_with(move |_, _, _| {
                            IconButton::new(
                                "popover-icon",
                                "Open popover",
                                Icon::new(IconName::ArrowDropDown),
                            )
                            .on_accessible_activate(move |_, cx| {
                                let _ = toggle_popover.update(cx, |this, cx| {
                                    this.popover_open = !this.popover_open;
                                    cx.notify();
                                });
                            })
                            .into_any_element()
                        })
                        .content(|_, _, _| div().id("popover-body").test_support().child("Body")),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| View::default());
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view)
    });
    let window = handle.into();
    cx.update_window(window, |_, window, _| window.activate_window())
        .unwrap();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);

    // Popover trigger: mouse down opens, releasing mouse up must keep it open.
    let popover_center = cx
        .update_window(window, |_, window, _| {
            find(window, &[], &"popover-icon".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: popover_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    assert!(cx.read_entity(&view, |this, _| this.popover_open));

    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: popover_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    assert!(cx.read_entity(&view, |this, _| this.popover_open));

    // Close popover before testing Menu::composed.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.popover_open = false;
            cx.notify();
        });
    });
    frame(cx);

    // Menu::composed trigger: mouse down opens, releasing mouse up must keep it open.
    let menu_center = cx
        .update_window(window, |_, window, _| {
            find(window, &[], &"menu-btn".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: menu_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    let menu_open = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| {
            find(window, &[], &"menu".into()).is_some()
        })
        .unwrap()
    };
    assert!(menu_open(cx));

    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: menu_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    assert!(menu_open(cx));
}
