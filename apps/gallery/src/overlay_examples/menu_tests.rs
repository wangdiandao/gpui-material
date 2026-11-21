use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AccessibleAction, App, AppContext, Context, ElementId, InputEvent as _, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render,
    Role, ScrollHandle, StatefulInteractiveElement, TestAppContext, Window, WindowOptions, div,
    point, px,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{
    MaterialRoot, Menu, MenuColor, MenuItem, MenuItemStyle, MenuSurfaceStyle,
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerRole, Elevation},
};

use super::MenuTooltipExamples;
use crate::story::Story;

#[gpui::test]
fn menu_skips_disabled_items_and_restores_trigger(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Menu, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.simulate_keystrokes(window.into(), "enter");
    frame(cx);
    frame(cx);
    let bounds = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, _| {
            find(window, &[], &"menu".into()).unwrap().bounds()
        })
        .unwrap()
    };
    let collapsed = bounds(cx);
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    let expanded = bounds(cx);
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert_eq!(collapsed.top(), expanded.top());
    cx.update_window(window.into(), |_, window, _| {
        let row = find(window, &[], &"labels".into()).unwrap();
        assert_eq!(
            row.bounds().size.height,
            px(48.),
            "Compose MenuListItemContainerHeight"
        );
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "down");
    frame(cx);
    cx.simulate_keystrokes(window.into(), "right");
    frame(cx);
    frame(cx);
    cx.update_window(window.into(), |_, window, _| {
        assert!(
            find(window, &[], &"submenu-content".into()).is_some(),
            "submenu opened"
        );
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "down enter");
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let view = view.read(cx);
        assert_eq!(view.output.as_ref(), "Sort by date modified");
        assert!(view.trigger.is_focused(window));
    })
    .unwrap();
}

#[gpui::test]
fn submenu_accepts_pointer_selection_before_dismissing_its_parent(cx: &mut TestAppContext) {
    let submenu_exit = (ElementId::from("options"), "submenu-exit-1").into();

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Menu, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "enter");
    frame(cx);
    frame(cx);
    cx.simulate_keystrokes(window.into(), "down right");
    frame(cx);
    frame(cx);
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let submenu = find(window, &[], &"submenu-content".into())
            .unwrap()
            .bounds();
        let position = point(submenu.center().x, submenu.top() + px(68.));
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &"submenu-content".into()).is_some());
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(view.read(cx).output.as_ref(), "Sort by date modified");
        assert!(find(window, &[], &"menu".into()).is_none());
        assert!(find(window, &[], &submenu_exit).is_some());
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    cx.update_window(window.into(), |_, window, _| {
        assert!(find(window, &[], &submenu_exit).is_none());
    })
    .unwrap();
}

#[gpui::test]
fn composed_menu_toggles_live_visibility_before_redraw(cx: &mut TestAppContext) {
    type Activate = Rc<dyn Fn(&mut Window, &mut App)>;
    struct Example {
        activate: Rc<RefCell<Option<Activate>>>,
        requests: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let activate = self.activate.clone();
            let requests = self.requests.clone();
            Menu::composed(
                "menu",
                [MenuItem::new("entry", "Entry")],
                move |_, toggle| {
                    *activate.borrow_mut() = Some(toggle);
                    gpui_base::Button::new("trigger")
                        .child("Open menu")
                        .into_any_element()
                },
            )
            .on_open_change(move |open, _, _| requests.borrow_mut().push(open))
        }
    }

    let activate = Rc::new(RefCell::new(None::<Activate>));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example {
                activate: activate.clone(),
                requests: requests.clone(),
            })
        })
        .unwrap()
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let toggle = activate.borrow().as_ref().unwrap().clone();
        toggle(window, cx);
        toggle(window, cx);
    })
    .unwrap();
    assert_eq!(*requests.borrow(), [true, false]);
}

#[gpui::test]
fn controlled_menu_applies_only_accepted_requests_and_keeps_nested_escape_local(
    cx: &mut TestAppContext,
) {
    #[derive(Default)]
    struct Example {
        open: bool,
        accept: bool,
        checked: bool,
        requests: Vec<bool>,
        checked_requests: Vec<bool>,
        dismissals: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            let dismiss = owner.clone();
            let checked = owner.clone();
            let toggles = MenuItem::new("controlled-check", "Show labels")
                .checked(self.checked)
                .content(|| "Visible check label")
                .supporting(|| div().id("item-support").test_support().child("More detail"))
                .on_checked_change(move |value, _, cx| {
                    let _ = checked.update(cx, |state, cx| {
                        state.checked_requests.push(value);
                        state.checked = value;
                        cx.notify();
                    });
                });
            Menu::composed(
                "controlled-menu",
                [
                    toggles,
                    MenuItem::new("controlled-submenu", "Sort by")
                        .submenu([MenuItem::new("controlled-descending", "Descending")]),
                ],
                |_, toggle| {
                    gpui_base::Button::new("controlled-trigger")
                        .child("Open menu")
                        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            toggle(window, cx);
                        })
                        .into_any_element()
                },
            )
            .open(self.open)
            .on_open_change(move |value, _, cx| {
                let _ = owner.update(cx, |state, cx| {
                    state.requests.push(value);
                    if state.accept {
                        state.open = value;
                    }
                    cx.notify();
                });
            })
            .on_dismiss(move |_, cx| {
                let _ = dismiss.update(cx, |state, _| state.dismissals += 1);
            })
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Example::default());
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (window, view)
    });
    let window = window.into();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let read_open = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| {
            find(window, &[], &"menu".into()).is_some()
        })
        .unwrap()
    };
    frame(cx);
    cx.update_window(window, |_, window, cx| window.focus_next(cx))
        .unwrap();
    let focus = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| window.focused(cx))
            .unwrap()
    };
    let trigger = focus(cx).expect("the trigger is focusable");
    cx.update_window(window, |_, window, cx| {
        let target = find(window, &[], &"controlled-trigger".into()).unwrap();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: target.bounds().center(),
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    assert!(!read_open(cx));
    assert_eq!(focus(cx).as_ref(), Some(&trigger));

    cx.update(|cx| view.update(cx, |state, _| state.accept = true));
    cx.simulate_keystrokes(window, "enter");
    frame(cx);
    frame(cx);
    assert!(read_open(cx));
    cx.update_window(window, |_, window, _| {
        let row = find(window, &[], &"controlled-check".into()).unwrap();
        assert_eq!(row.role(), Some(Role::MenuItemCheckBox));
        assert_eq!(row.label(), Some("Show labels"));
        assert!(find(window, &[], &"item-support".into()).is_some());
    })
    .unwrap();

    cx.simulate_keystrokes(window, "down right");
    frame(cx);
    frame(cx);
    let nested_open = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| {
            find(window, &[], &"submenu-content".into()).is_some()
        })
        .unwrap()
    };
    assert!(nested_open(cx));
    cx.simulate_keystrokes(window, "escape");
    frame(cx);
    assert!(!nested_open(cx));
    assert!(read_open(cx));
    cx.update(|cx| {
        assert_eq!(view.read(cx).requests, [true, true]);
        assert_eq!(view.read(cx).dismissals, 0);
        view.update(cx, |state, _| state.accept = false);
    });
    let active_item = focus(cx).expect("closing the submenu returns focus to its row");
    cx.simulate_keystrokes(window, "escape");
    frame(cx);
    assert!(read_open(cx));
    assert_eq!(focus(cx), Some(active_item));

    cx.simulate_keystrokes(window, "up enter");
    frame(cx);
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.requests, [true, true, false, false]);
        assert_eq!(state.dismissals, 2);
        assert_eq!(state.checked_requests, [true]);
        view.update(cx, |state, cx| {
            state.open = false;
            cx.notify();
        });
    });
    frame(cx);
    assert!(!read_open(cx));
    assert_eq!(focus(cx).as_ref(), Some(&trigger));
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.requests, [true, true, false, false]);
        assert_eq!(state.dismissals, 2);
        view.update(cx, |state, cx| {
            state.accept = true;
            state.open = true;
            cx.notify();
        });
    });
    frame(cx);
    assert!(read_open(cx));
    cx.update_window(window, |_, window, cx| {
        let viewport = window.viewport_size();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: point(viewport.width - px(10.), viewport.height - px(10.)),
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    assert!(!read_open(cx));
    assert_eq!(focus(cx).as_ref(), Some(&trigger));
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.requests, [true, true, false, false, false]);
        assert_eq!(state.dismissals, 3);
    });
}

#[gpui::test]
fn expressive_menu_paints_group_row_and_position_overrides(cx: &mut TestAppContext) {
    struct Example {
        scroll: ScrollHandle,
        position: Point<Pixels>,
        activations: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            div().child(
                Menu::new(
                    "styled-menu",
                    gpui_base::Button::new("styled-trigger").child("Menu"),
                    [
                        MenuItem::section(
                            "project-group",
                            "Project",
                            [
                                MenuItem::new("group-first", "First").on_activate(move |_, cx| {
                                    let _ = owner.update(cx, |state, cx| {
                                        state.activations += 1;
                                        cx.notify();
                                    });
                                }),
                                MenuItem::new("group-selected", "Selected")
                                    .selected(true)
                                    .appearance(MenuItemStyle {
                                        selected_shape: Some(CornerRole::Medium.shape()),
                                        selected_container: Some(ColorRole::PrimaryContainer),
                                        selected_content: Some(ColorRole::OnPrimaryContainer),
                                        ..Default::default()
                                    }),
                            ],
                        )
                        .group_appearance(MenuSurfaceStyle {
                            shape: Some(CornerRole::ExtraLarge.shape()),
                            container: Some(ColorRole::SecondaryContainer),
                            border: Some((px(2.), ColorRole::Outline)),
                            ..Default::default()
                        }),
                        MenuItem::section(
                            "following-group",
                            "More actions",
                            (0..20usize).map(|index| {
                                MenuItem::new(("following-item", index), format!("Action {index}"))
                            }),
                        ),
                    ],
                )
                .color(MenuColor::Vibrant)
                .appearance(MenuSurfaceStyle {
                    shadow_elevation: Some(Elevation::Level4),
                    spacing: Some(px(4.)),
                    ..Default::default()
                })
                .content(|| {
                    div()
                        .id("menu-header")
                        .test_support()
                        .child("Style controls")
                })
                .position(self.position)
                .track_scroll(&self.scroll)
                .open(true),
            )
        }
    }

    let scroll = ScrollHandle::new();
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Example {
            scroll: scroll.clone(),
            position: point(px(180.), px(100.)),
            activations: 0,
        });
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (window, view)
    });
    let window = window.into();
    for _ in 0..2 {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
    let menu_position = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| {
            find(window, &[], &"menu".into()).unwrap().bounds().origin
        })
        .unwrap()
    };
    assert_eq!(menu_position(cx), point(px(180.), px(100.)));
    cx.update_window(window, |_, window, cx| {
        let group = find(window, &[], &"project-group".into()).unwrap();
        assert_eq!(group.role(), Some(Role::Group));
        assert_eq!(group.label(), Some("Project"));
        assert!(find(window, &[], &"menu-header".into()).is_some());
        let theme = window_material_theme(window, cx);
        let tint = |role| color(theme.colors.color(role));
        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == tint(ColorRole::TertiaryContainer).into())
        );
        let group_fill = quads
            .iter()
            .find(|quad| {
                quad.background == tint(ColorRole::SecondaryContainer).into()
                    && quad.corner_radii.top_left.0 == 28. * window.scale_factor()
            })
            .expect("styled group fill");
        assert!(quads.iter().any(|quad| {
            quad.bounds == group_fill.bounds
                && quad.border_color == tint(ColorRole::Outline)
                && quad.border_widths.top.0 == 2. * window.scale_factor()
        }));
        assert!(quads.iter().any(|quad| {
            quad.background == tint(ColorRole::PrimaryContainer).into()
                && quad.corner_radii.top_left.0 == 12. * window.scale_factor()
        }));
    })
    .unwrap();

    let focus = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| window.focused(cx))
            .unwrap()
    };
    let first = focus(cx).unwrap();
    cx.simulate_keystrokes(window, "down");
    let selected = focus(cx).unwrap();
    assert_ne!(selected, first);
    cx.simulate_keystrokes(window, "down up");
    assert_eq!(focus(cx), Some(selected));

    cx.update_window(window, |_, window, cx| {
        let row = find(window, &[], &"group-first".into()).unwrap();
        let position = row.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();

    assert!(scroll.max_offset().y > px(60.));
    scroll.set_offset(point(px(0.), -px(60.)));
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    assert_eq!(scroll.offset().y, -px(60.));
    assert_eq!(menu_position(cx), point(px(180.), px(100.)));

    cx.update_window(window, |_, window, cx| {
        let viewport = window.viewport_size();
        view.update(cx, |state, cx| {
            state.position = point(viewport.width + px(40.), viewport.height + px(40.));
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let menu = find(window, &[], &"menu".into()).unwrap().bounds();
        assert!(menu.right() <= viewport.width && menu.bottom() <= viewport.height);
    })
    .unwrap();
}

#[gpui::test]
fn menu_item_divider_shortcut_destructive_and_context_menu_ext(cx: &mut TestAppContext) {
    use std::cell::Cell;

    use gpui::{Hsla, RenderOnce, Styled};
    use gpui_material::{Button, MaterialElementExt, SemanticTone};

    #[derive(IntoElement)]
    struct InkProbe {
        slot: Rc<Cell<Option<Hsla>>>,
        label: &'static str,
    }

    impl RenderOnce for InkProbe {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            self.slot.set(Some(window.text_style().color));
            div().child(self.label)
        }
    }

    struct ContextMenuView {
        danger_ink: Rc<Cell<Option<Hsla>>>,
        disabled_danger_ink: Rc<Cell<Option<Hsla>>>,
    }

    impl Render for ContextMenuView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let danger_ink = self.danger_ink.clone();
            let disabled_danger_ink = self.disabled_danger_ink.clone();
            div().p_4().child(
                Button::outlined("ctx-trigger", "Right click me").context_menu(
                    "ext-context-menu",
                    [
                        MenuItem::new("save-cmd", "Save").shortcut("Ctrl+S"),
                        MenuItem::new("custom-trailing-cmd", "Export")
                            .shortcut("Ctrl+E")
                            .trailing(|| div().id("custom-trailing").test_support().child("TXT")),
                        MenuItem::divider("menu-sep"),
                        MenuItem::new("danger-cmd", "Delete")
                            .destructive()
                            .content(move || InkProbe {
                                slot: danger_ink.clone(),
                                label: "Delete",
                            }),
                        MenuItem::new("disabled-danger-cmd", "Purge")
                            .tone(SemanticTone::Danger)
                            .disabled(true)
                            .content(move || InkProbe {
                                slot: disabled_danger_ink.clone(),
                                label: "Purge",
                            }),
                        MenuItem::new("selected-success-cmd", "Pinned")
                            .selected(true)
                            .tone(SemanticTone::Success),
                    ],
                ),
            )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| ContextMenuView {
                danger_ink: Rc::new(Cell::new(None)),
                disabled_danger_ink: Rc::new(Cell::new(None)),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let window = window.into();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);

    // Secondary click opens the context menu created via MaterialElementExt::context_menu.
    cx.update_window(window, |_, window, cx| {
        let trigger = find(window, &[], &"ctx-trigger".into()).unwrap();
        let position = trigger.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Right,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Right,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    frame(cx);

    cx.update_window(window, |_, window, cx| {
        let theme = window_material_theme(window, cx);
        let error = color(theme.colors.color(ColorRole::Error));
        let disabled_ink = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);
        let success_container = color(theme.colors.color(ColorRole::SuccessContainer));

        assert!(find(window, &[], &"menu".into()).is_some());
        assert!(find(window, &[], &"menu-sep".into()).is_some());

        let save = find(window, &[], &"save-cmd".into()).unwrap();
        assert!(
            find(window, save.path(), &"shortcut".into()).is_some(),
            "shortcut hint renders in trailing slot"
        );

        let custom = find(window, &[], &"custom-trailing-cmd".into()).unwrap();
        assert!(find(window, custom.path(), &"custom-trailing".into()).is_some());
        assert!(
            find(window, custom.path(), &"shortcut".into()).is_none(),
            "explicit trailing slot overrides shortcut hint"
        );

        assert_eq!(view.read(cx).danger_ink.get(), Some(error));
        assert_eq!(view.read(cx).disabled_danger_ink.get(), Some(disabled_ink));
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == success_container.into()),
            "selected Success menu item paints SuccessContainer"
        );
        assert_eq!(save.focused(), Some(true));
    })
    .unwrap();

    // Down arrow moves: save-cmd -> custom-trailing-cmd -> skips divider -> danger-cmd -> skips disabled -> selected-success-cmd.
    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"custom-trailing-cmd".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();

    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"danger-cmd".into()).unwrap().focused(),
            Some(true),
            "keyboard navigation must skip MenuItem::divider"
        );
    })
    .unwrap();

    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"selected-success-cmd".into())
                .unwrap()
                .focused(),
            Some(true),
            "keyboard navigation skips disabled destructive item"
        );
    })
    .unwrap();

    // Up arrow skips back over disabled item and divider.
    cx.simulate_keystrokes(window, "up up");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"custom-trailing-cmd".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();
}
