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
