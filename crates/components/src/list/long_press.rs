use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, ClickEvent, GestureTuning, Hitbox, HitboxBehavior, IntoElement, LongPressEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Styled, Subscription,
    Task, TouchPhase, Window, canvas,
};

use crate::action::Activate;

#[derive(Default)]
struct Hold {
    action: Option<Activate>,
    pending: Option<(Point<Pixels>, Task<()>)>,
    consumed_mouse_click: bool,
}

impl Hold {
    fn cancel(&mut self) {
        self.consumed_mouse_click |= self.pending.take().is_some();
    }
}

pub(super) struct LongPress {
    hold: Rc<RefCell<Hold>>,
    _activation: Subscription,
}

impl LongPress {
    pub fn mount(action: Option<Activate>, window: &mut Window, cx: &mut App) -> Rc<Self> {
        let mount = window
            .use_keyed_state("list-long-press", cx, |window, cx| {
                let hold = Rc::new(RefCell::new(Hold::default()));
                let activation = cx.observe_window_activation(window, {
                    let hold = hold.clone();
                    move |_, window, _| {
                        if !window.is_window_active() {
                            hold.borrow_mut().cancel();
                        }
                    }
                });
                Rc::new(Self {
                    hold,
                    _activation: activation,
                })
            })
            .read(cx)
            .clone();
        let mut state = mount.hold.borrow_mut();
        if action.is_none() {
            state.cancel();
        }
        state.action = action;
        drop(state);
        mount
    }

    pub fn blocks_activation(&self, event: &ClickEvent) -> bool {
        matches!(event, ClickEvent::Mouse(_)) && self.hold.borrow().consumed_mouse_click
    }

    pub fn cancel(&self) {
        self.hold.borrow_mut().cancel();
    }

    pub fn target(&self) -> impl IntoElement + use<> {
        let hold = self.hold.clone();
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| listen(hold.clone(), hitbox.clone(), window),
        )
        .absolute()
        .inset_0()
    }
}

fn listen(hold: Rc<RefCell<Hold>>, hitbox: Hitbox, window: &mut Window) {
    let tuning = GestureTuning::default();
    window.on_mouse_event({
        let hold = hold.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if event.button != MouseButton::Left {
                return;
            }
            if phase.capture() {
                let mut state = hold.borrow_mut();
                state.pending = None;
                state.consumed_mouse_click = false;
            } else if hitbox.is_hovered(window) && !window.default_prevented() {
                let deadline = cx.background_executor().timer(tuning.long_press_duration);
                let weak = Rc::downgrade(&hold);
                let task = window.spawn(cx, async move |cx| {
                    deadline.await;
                    cx.update(|window, cx| {
                        let Some(hold) = weak.upgrade() else {
                            return;
                        };
                        let action = {
                            let mut state = hold.borrow_mut();
                            if state.pending.take().is_none() || !window.is_window_active() {
                                return;
                            }
                            state.consumed_mouse_click = true;
                            state.action.clone()
                        };
                        if let Some(action) = action {
                            action(window, cx);
                        }
                    })
                    .ok();
                });
                hold.borrow_mut().pending = Some((event.position, task));
            }
        }
    });
    window.on_mouse_event({
        let hold = hold.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            if !phase.capture() {
                return;
            }
            let mut state = hold.borrow_mut();
            if state.pending.as_ref().is_some_and(|(origin, _)| {
                let delta = event.position - *origin;
                event.pressed_button != Some(MouseButton::Left)
                    || delta.x.abs().max(delta.y.abs()) >= tuning.touch_slop
                    || cx.has_active_drag()
                    || window.captured_hitbox().is_some()
            }) {
                state.cancel();
            }
        }
    });
    window.on_mouse_event({
        let hold = hold.clone();
        move |event: &MouseUpEvent, phase, _, _| {
            if event.button == MouseButton::Left && phase.capture() {
                hold.borrow_mut().pending = None;
            }
        }
    });
    window.on_mouse_event(move |event: &LongPressEvent, phase, window, cx| {
        if phase.bubble()
            && event.phase == TouchPhase::Started
            && !window.default_prevented()
            && hitbox.is_hovered_at(event.start_position, window)
        {
            let action = hold.borrow().action.clone();
            if let Some(action) = action {
                window.prevent_default();
                cx.stop_propagation();
                action(window, cx);
            }
        }
    });
}
