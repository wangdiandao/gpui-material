use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, InputEvent, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render, Role, Styled,
    TestAppContext, VisualTestContext, Window, WindowOptions, div, px, size,
};
use gpui_base::test_support::{find, snapshots};
use gpui_material::{
    Button, Dialog, DialogAction, DialogCloseReason as Reason, DialogProperties, DialogState,
    MaterialRoot,
};

#[derive(Default, Clone)]
struct Observations {
    closes: Rc<RefCell<Vec<Reason>>>,
    requests: Rc<RefCell<Vec<Reason>>>,
    allow_dismiss: Rc<Cell<bool>>,
    allow_confirm: Rc<Cell<bool>>,
    underlay_clicks: Rc<Cell<usize>>,
    action: Rc<RefCell<Option<DialogAction>>>,
}

struct Probe {
    state: Entity<DialogState>,
    trigger: FocusHandle,
    observations: Observations,
    properties: DialogProperties,
    custom: bool,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let underlay_clicks = self.observations.underlay_clicks.clone();
        let observes_request = self.observations.clone();
        let observes_close = self.observations.closes.clone();
        let allow_confirm = self.observations.allow_confirm.clone();
        let mut dialog = Dialog::new("policy-dialog", &self.state, "Review project changes")
            .properties(self.properties)
            .scrim_opacity(0.)
            .on_dismiss_request(move |reason, _, _| {
                observes_request.requests.borrow_mut().push(reason);
                observes_request.allow_dismiss.get()
            })
            .on_close(move |reason, _, _| observes_close.borrow_mut().push(reason))
            .confirm("Confirm changes", move |_, _| allow_confirm.get());
        if self.custom {
            let action = self.observations.action.clone();
            dialog = dialog
                .title_content(div().child("Project changes"))
                .show_cancel(false)
                .width(px(290.))
                .confirm_content(move |confirm| {
                    *action.borrow_mut() = Some(confirm.clone());
                    Button::new("apply", "Apply changes")
                        .on_activate(move |window, cx| confirm.activate(window, cx))
                });
        }
        div()
            .size_full()
            .child(
                Button::new("underlay", "Review")
                    .track_focus(&self.trigger)
                    .on_activate(move |_, _| underlay_clicks.set(underlay_clicks.get() + 1)),
            )
            .child(dialog)
    }
}

fn open_probe(
    custom: bool,
    properties: DialogProperties,
    cx: &mut TestAppContext,
) -> (
    AnyWindowHandle,
    Entity<DialogState>,
    FocusHandle,
    Observations,
) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(DialogState::new);
        let trigger = cx.focus_handle();
        let observations = Observations::default();
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| {
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                Probe {
                    state: state.clone(),
                    trigger: trigger.clone(),
                    observations: observations.clone(),
                    properties,
                    custom,
                }
            })
        })
        .unwrap()
        .into();
        (window, state, trigger, observations)
    })
}

fn frame(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        window.simulate_next_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn click(window: AnyWindowHandle, position: Point<Pixels>, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
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
    frame(window, cx);
}

fn center(window: AnyWindowHandle, id: &'static str, cx: &mut TestAppContext) -> Point<Pixels> {
    cx.update_window(window, |_, window, _| {
        find(window, &[], &id.into()).unwrap().bounds().center()
    })
    .unwrap()
}

fn reopen(window: AnyWindowHandle, state: &Entity<DialogState>, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        state.update(cx, |state, cx| state.open(window, cx))
    })
    .unwrap();
    frame(window, cx);
}

#[gpui::test]
fn dialog_dismissal_reasons_and_transparent_backdrop_are_independent(cx: &mut TestAppContext) {
    let (window, state, trigger, observations) = open_probe(false, DialogProperties::default(), cx);
    cx.update_window(window, |_, window, cx| trigger.focus(window, cx))
        .unwrap();
    reopen(window, &state, cx);
    let outside = center(window, "underlay", cx);
    click(window, outside, cx);
    assert_eq!(*observations.requests.borrow(), [Reason::OutsideClick]);
    assert_eq!(observations.underlay_clicks.get(), 0);
    assert!(cx.update(|cx| state.read(cx).is_open()));

    observations.allow_dismiss.set(true);
    click(window, outside, cx);
    assert_eq!(*observations.closes.borrow(), [Reason::OutsideClick]);
    assert_eq!(observations.underlay_clicks.get(), 0);
    assert!(
        cx.update_window(window, |_, window, _| trigger.is_focused(window))
            .unwrap()
    );

    reopen(window, &state, cx);
    cx.simulate_keystrokes(window, "escape");
    assert_eq!(
        *observations.closes.borrow(),
        [Reason::OutsideClick, Reason::Escape]
    );
    reopen(window, &state, cx);
    cx.simulate_keystrokes(window, "enter");
    assert!(cx.update(|cx| state.read(cx).is_open()));
    let cancel = center(window, "cancel", cx);
    click(window, cancel, cx);
    assert_eq!(
        *observations.closes.borrow(),
        [Reason::OutsideClick, Reason::Escape, Reason::Cancel]
    );
    assert_eq!(
        *observations.requests.borrow(),
        [Reason::OutsideClick, Reason::OutsideClick, Reason::Escape]
    );

    reopen(window, &state, cx);
    observations.allow_confirm.set(true);
    cx.simulate_keystrokes(window, "enter");
    assert_eq!(
        cx.update(|cx| state.read(cx).last_close_reason()),
        Some(Reason::Confirm)
    );
    assert_eq!(observations.closes.borrow().last(), Some(&Reason::Confirm));
}

#[gpui::test]
fn dialog_custom_slots_keep_semantics_focus_and_generation(cx: &mut TestAppContext) {
    let (window, state, trigger, observations) = open_probe(
        true,
        DialogProperties {
            dismiss_on_escape: false,
            dismiss_on_outside_click: false,
        },
        cx,
    );
    observations.allow_dismiss.set(true);
    observations.allow_confirm.set(true);
    cx.update_window(window, |_, window, cx| trigger.focus(window, cx))
        .unwrap();
    reopen(window, &state, cx);
    cx.update_window(window, |_, window, _| {
        let nodes = snapshots(window);
        assert!(nodes.iter().any(|node| node.role() == Some(Role::Dialog)
            && node.label() == Some("Review project changes")));
        let action = find(window, &[], &"apply".into()).unwrap();
        assert_eq!(action.role(), Some(Role::Button));
        assert_eq!(action.label(), Some("Apply changes"));
        assert!(find(window, &[], &"cancel".into()).is_none());
        assert_eq!(
            find(window, &[], &"policy-dialog".into())
                .unwrap()
                .bounds()
                .size
                .width,
            px(290.)
        );
    })
    .unwrap();
    cx.simulate_keystrokes(window, "escape enter");
    let outside = center(window, "underlay", cx);
    click(window, outside, cx);
    assert!(observations.requests.borrow().is_empty());
    assert_eq!(observations.underlay_clicks.get(), 0);
    assert!(cx.update(|cx| state.read(cx).is_open()));

    let stale_action = observations.action.borrow().clone().unwrap();
    cx.update_window(window, |_, window, cx| {
        state.update(cx, |state, cx| state.close(window, cx));
        assert!(trigger.is_focused(window));
        assert_eq!(
            state.read(cx).last_close_reason(),
            Some(Reason::Programmatic)
        );
        assert!(observations.closes.borrow().is_empty());
        state.update(cx, |state, cx| state.open(window, cx));
        stale_action.activate(window, cx);
        assert!(state.read(cx).is_open());
    })
    .unwrap();
    frame(window, cx);
    for key in ["tab", "tab", "shift-tab"] {
        cx.simulate_keystrokes(window, key);
        frame(window, cx);
        cx.update_window(window, |_, window, _| assert!(!trigger.is_focused(window)))
            .unwrap();
    }
    cx.update_window(window, |_, window, cx| {
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
        window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
    })
    .unwrap();
    assert_eq!(*observations.closes.borrow(), [Reason::Confirm]);
    assert!(
        cx.update_window(window, |_, window, _| trigger.is_focused(window))
            .unwrap()
    );
}

#[gpui::test]
fn native_dialog_uses_an_independent_window_and_reports_window_close(cx: &mut TestAppContext) {
    let (parent, state, trigger, observations) = open_probe(false, DialogProperties::default(), cx);
    let child = cx
        .update_window(parent, |_, parent, cx| {
            trigger.focus(parent, cx);
            parent.activate_window();
            let observes_request = observations.clone();
            let closes = observations.closes.clone();
            Dialog::open_window(
                &state,
                parent,
                WindowOptions::default(),
                cx,
                move |state, _, _| {
                    let observes_request = observes_request.clone();
                    let closes = closes.clone();
                    Dialog::unstructured("native-shell", state, "Native project")
                        .width(px(300.))
                        .body_content(|_, dismiss| {
                            Button::new("done", "Done")
                                .on_activate(move |window, cx| dismiss.activate(window, cx))
                        })
                        .on_dismiss_request(move |reason, _, _| {
                            observes_request.requests.borrow_mut().push(reason);
                            observes_request.allow_dismiss.get()
                        })
                        .on_close(move |reason, _, _| closes.borrow_mut().push(reason))
                },
            )
            .unwrap()
            .unwrap()
        })
        .unwrap();
    assert_ne!(child.window_id(), parent.window_id());
    assert_eq!(cx.windows().len(), 2);
    frame(parent, cx);
    assert!(
        cx.update_window(parent, |_, window, _| {
            find(window, &[], &"policy-dialog".into()).is_none()
        })
        .unwrap()
    );
    let mut native = VisualTestContext::from_window(child, cx);
    cx.simulate_window_resize(child, size(px(232.), px(160.)));
    frame(child, cx);
    native.update(|window, _| {
        assert!(snapshots(window).iter().any(
            |node| node.role() == Some(Role::Dialog) && node.label() == Some("Native project")
        ));
        assert!(find(window, &[], &"dialog-content".into()).is_none());
        assert!(find(window, &[], &"cancel".into()).is_none());
        assert!(
            find(window, &[], &"native-shell".into())
                .unwrap()
                .bounds()
                .size
                .width
                <= px(232.)
        );
    });
    assert!(!native.simulate_close());
    assert!(cx.update(|cx| state.read(cx).is_open()));
    observations.allow_dismiss.set(true);
    assert!(native.simulate_close());
    native.update(|window, _| window.remove_window());
    assert_eq!(cx.windows().len(), 1);
    assert_eq!(
        *observations.requests.borrow(),
        [Reason::WindowClose, Reason::WindowClose]
    );
    assert_eq!(*observations.closes.borrow(), [Reason::WindowClose]);
    assert_eq!(
        cx.update(|cx| state.read(cx).last_close_reason()),
        Some(Reason::WindowClose)
    );
    assert!(
        cx.update_window(parent, |_, window, _| trigger.is_focused(window))
            .unwrap()
    );
}

#[gpui::test]
fn escape_closes_nested_dialogs_and_restores_each_trigger(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Dialog, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let (first, second) = cx.update(|cx| {
        let view = view.read(cx);
        (view.primary.clone(), view.secondary.clone())
    });
    let trigger = cx
        .update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.focus_next(cx);
            let trigger = window.focused(cx).unwrap();
            first.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            second.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            trigger
        })
        .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| {
        assert!(first.read(cx).is_open());
        assert!(!second.read(cx).is_open());
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update_window(window.into(), |_, window, cx| {
        assert!(!first.read(cx).is_open());
        assert!(trigger.is_focused(window));
    })
    .unwrap();
}

#[gpui::test]
fn dialog_body_stays_with_the_panel_during_dismissal(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;
    use std::time::Duration;

    let (window, dialog) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut dialog = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = ModalExamples::new(Story::Dialog, window, cx);
                dialog = Some(view.primary.clone());
                view
            })
        })
        .unwrap();
        (window, dialog.unwrap())
    });
    cx.update_window(window.into(), |_, window, cx| {
        dialog.update(cx, |dialog, cx| dialog.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(220));
    let body = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"dialog-content".into()).map(|body| body.bounds())
        })
        .unwrap()
    };
    let opened = body(cx).unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    let first_exit = body(cx).unwrap();
    assert_eq!(first_exit.size, opened.size);
    assert!((first_exit.top() - opened.top()).abs() <= px(1.));
    cx.executor().advance_clock(Duration::from_millis(60));
    let exiting = body(cx).unwrap();
    assert_eq!(opened.size, exiting.size);
    assert!(exiting.top() > opened.top() && exiting.top() < opened.top() + px(8.));
    cx.executor().advance_clock(Duration::from_millis(250));
    assert!(body(cx).is_none());
}

#[gpui::test]
fn task_dialog_adapts_at_compact_breakpoint_without_losing_state(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;
    use std::time::Duration;

    let (window, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut state = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = ModalExamples::new(Story::Dialog, window, cx);
                state = Some(view.secondary.clone());
                view
            })
        })
        .unwrap();
        (window, state.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(400.), px(600.)));
    cx.update_window(window.into(), |_, window, cx| {
        state.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let panel = find(window, &[], &"guidelines-dialog".into())
            .unwrap()
            .bounds();
        let bar = find(window, &[], &"dialog-app-bar".into())
            .unwrap()
            .bounds();
        let body = find(window, &[], &"dialog-content".into())
            .unwrap()
            .bounds();
        assert_eq!(panel.size, window.viewport_size());
        assert_eq!(bar.size.height, px(56.));
        assert!(body.top() >= bar.bottom());
        assert!(body.bottom() <= panel.bottom());
    })
    .unwrap();
    let focus = cx
        .update_window(window.into(), |_, window, cx| {
            window.focus_next(cx);
            window.focused(cx).unwrap()
        })
        .unwrap();
    cx.simulate_window_resize(window.into(), size(px(800.), px(600.)));
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(state.read(cx).is_open());
        assert!(
            focus.is_focused(window),
            "resizing preserves the keyboard focus handle"
        );
        assert!(find(window, &[], &"dialog-app-bar".into()).is_none());
        let panel = find(window, &[], &"guidelines-dialog".into())
            .unwrap()
            .bounds();
        assert_eq!(panel.size.width, px(560.));
        assert!(panel.size.height < window.viewport_size().height);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| assert!(!state.read(cx).is_open()));
}

#[gpui::test]
fn modal_examples_scope_demo_state_to_active_story(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;

    cx.update(|cx| {
        crate::init_gallery(cx);
        let mut dialog_view = None;
        let mut sheets_view = None;
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let dialog = cx.new(|cx| ModalExamples::new(Story::Dialog, window, cx));
            let sheets = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            dialog_view = Some(dialog.clone());
            sheets_view = Some(sheets);
            dialog
        })
        .unwrap();
        let dialog_view = dialog_view.unwrap();
        let sheets_view = sheets_view.unwrap();
        assert!(dialog_view.read(cx).dialog.is_some());
        assert!(dialog_view.read(cx).sheet.is_none());
        assert!(sheets_view.read(cx).dialog.is_none());
        assert!(sheets_view.read(cx).sheet.is_some());
    });
}

#[gpui::test]
fn dialog_alert_on_confirm_and_confirm_tone_work_in_basic_and_full_screen_modes(
    cx: &mut TestAppContext,
) {
    use gpui_material::{DialogVariant, SemanticTone};

    struct AlertProbe {
        state: Entity<DialogState>,
        variant: DialogVariant,
        confirmed: Rc<Cell<usize>>,
    }

    impl Render for AlertProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let confirmed = self.confirmed.clone();
            div().size_full().child(
                Dialog::alert(
                    "delete-alert",
                    &self.state,
                    "Delete workspace?",
                    "This action cannot be undone.",
                )
                .variant(self.variant)
                .confirm_tone(SemanticTone::Danger)
                .on_confirm("Delete", move |_, _| {
                    confirmed.set(confirmed.get() + 1);
                }),
            )
        }
    }

    let confirmed = Rc::new(Cell::new(0usize));
    let (window, state, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(DialogState::new);
        let view = cx.new(|_| AlertProbe {
            state: state.clone(),
            variant: DialogVariant::Basic,
            confirmed: confirmed.clone(),
        });
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), state, view)
    });

    cx.update_window(window, |_, window, cx| {
        window.activate_window();
        state.update(cx, |state, cx| state.open(window, cx));
    })
    .unwrap();
    frame(window, cx);

    cx.update_window(window, |_, window, _| {
        assert!(
            find(window, &[], &"cancel".into()).is_none(),
            "Dialog::alert hides the default cancel button"
        );
        let confirm = find(window, &[], &"confirm".into()).expect("confirm button rendered");
        assert_eq!(confirm.label(), Some("Delete"));
    })
    .unwrap();

    // Activate confirm button via Enter while dialog container has focus; on_confirm closes immediately.
    cx.simulate_keystrokes(window, "enter");
    frame(window, cx);
    assert_eq!(confirmed.get(), 1);
    assert!(!cx.update(|cx| state.read(cx).is_open()));

    // Verify full-screen compact layout also renders confirm_tone + on_confirm without cancel.
    cx.simulate_window_resize(window, size(px(480.), px(640.)));
    cx.update_window(window, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.variant = DialogVariant::FullScreen;
            cx.notify();
        });
        state.update(cx, |state, cx| state.open(window, cx));
    })
    .unwrap();
    frame(window, cx);
    cx.update_window(window, |_, window, _| {
        assert!(find(window, &[], &"dialog-app-bar".into()).is_some());
        assert!(find(window, &[], &"cancel".into()).is_none());
        assert_eq!(
            find(window, &[], &"confirm".into()).unwrap().label(),
            Some("Delete")
        );
    })
    .unwrap();
}
