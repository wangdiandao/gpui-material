use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AccessibleAction, App, AppContext, Context, ElementId, InputEvent as _, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    StatefulInteractiveElement, TestAppContext, Window, WindowOptions, div, point, px,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{MaterialRoot, Menu, MenuItem};

use super::super::MenuTooltipExamples;
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
            .on_dismiss_request(move |_, cx| {
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
