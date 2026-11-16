use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, ClickEvent, ElementId, FocusHandle, MouseButton, Pixels, Point,
    StatefulInteractiveElement, Window,
};

use crate::{InteractionState, InteractionTracker, Ripple, Shape};

#[derive(Default)]
pub struct FeedbackState {
    hovered: bool,
    pressed: bool,
    dragged: bool,
    pointer_cancelled: bool,
    keyboard_focus: Option<FocusHandle>,
    pub shape: Option<Shape>,
    pub ripple: Option<Ripple>,
    tracker: Option<(InteractionTracker, FocusHandle)>,
}

impl FeedbackState {
    pub fn interaction(
        &self,
        enabled: bool,
        focus: &FocusHandle,
        window: &Window,
    ) -> InteractionState {
        let mut interaction = InteractionState::default();
        interaction.disabled = !enabled;
        interaction.hovered = self.hovered;
        interaction.focus_visible = focus.is_focused(window) && window.last_input_was_keyboard();
        interaction.pressed =
            enabled && self.pressed && (self.hovered || interaction.focus_visible);
        interaction.dragged = enabled && self.dragged;
        interaction
    }

    fn record(&self, window: &Window, cx: &mut App) {
        if let Some((tracker, focus)) = &self.tracker {
            tracker.record(self.interaction(true, focus, window), cx);
        }
    }

    pub fn cancel_unavailable(&mut self, enabled: bool, window: &Window) {
        if !enabled || !window.is_window_active() {
            self.hovered = false;
            self.cancel();
        } else if self
            .keyboard_focus
            .as_ref()
            .is_some_and(|focus| !focus.is_focused(window))
        {
            // Key-up is delivered to the new focus owner. Like Base's pending
            // keyboard click, presentation must cancel when its owner blurs.
            self.cancel();
        }
    }

    fn cancel(&mut self) {
        self.pressed = false;
        self.keyboard_focus = None;
        self.ripple = None;
    }

    pub fn clear(&mut self) {
        self.cancel();
        self.hovered = false;
        self.tracker = None;
    }

    pub fn is_dragged(&self) -> bool {
        self.dragged
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn set_dragged(&mut self, dragged: bool) {
        if dragged {
            self.pointer_cancelled = true;
            self.cancel();
        }
        self.dragged = dragged;
    }

    pub fn track(
        &mut self,
        enabled: bool,
        focus: &FocusHandle,
        tracker: Option<InteractionTracker>,
        window: &Window,
        cx: &mut App,
    ) -> InteractionState {
        let state = self.interaction(enabled, focus, window);
        self.track_snapshot(state, focus, tracker, cx);
        state
    }

    pub fn track_snapshot(
        &mut self,
        state: InteractionState,
        focus: &FocusHandle,
        tracker: Option<InteractionTracker>,
        cx: &mut App,
    ) {
        if let Some(tracker) = &tracker {
            tracker.record(state, cx);
        }
        self.tracker = tracker.map(|tracker| (tracker, focus.clone()));
    }

    pub fn blocks_activation(&self, event: &ClickEvent) -> bool {
        self.dragged
            || self.pointer_cancelled
                && matches!(event, ClickEvent::Mouse(_) | ClickEvent::Touch(_))
    }
}

pub fn activate_feedback(
    event: &ClickEvent,
    state: &RefCell<FeedbackState>,
    window: &mut Window,
    cx: &mut App,
) {
    let mut state = state.borrow_mut();
    match event {
        // Activation stops the bubbling click. Complete the pointer feedback
        // here even if the separate mouse-up listener does not run.
        ClickEvent::Mouse(_) => release_ripple(&mut state, cx),
        event if !state.pressed => {
            let pointer = match event {
                ClickEvent::Touch(event) => Some(event.position),
                _ => None,
            };
            start_ripple(&mut state, pointer, cx);
            release_ripple(&mut state, cx);
        }
        _ => {}
    }
    state.record(window, cx);
    window.refresh();
}

fn release_ripple(state: &mut FeedbackState, cx: &App) {
    state.pressed = false;
    state.keyboard_focus = None;
    if let Some(ripple) = &mut state.ripple {
        ripple.release(cx.background_executor().now());
    }
}

fn start_ripple(state: &mut FeedbackState, pointer: Option<Point<Pixels>>, cx: &App) {
    state.ripple = state.shape.as_ref().and_then(|shape| {
        // The accessible hit target can extend beyond the painted container.
        // Its activation still needs feedback, centered in the visible shape.
        let pointer = pointer.filter(|pointer| shape.contains(*pointer));
        Ripple::press(shape, pointer, cx.background_executor().now())
    });
}

/// Observe presentation events without consuming them: Base still owns activation,
/// focus, disabled semantics and controlled value changes. Action controls also
/// read hover from this state; selection and display controls style hover themselves.
pub fn feedback_target<E>(
    control: E,
    state: Rc<RefCell<FeedbackState>>,
    track_hover: bool,
    keyboard_owner: Option<FocusHandle>,
) -> E
where
    E: StatefulInteractiveElement,
{
    let press = state.clone();
    let release = state.clone();
    let outside = state.clone();
    let hover = state.clone();
    let keyboard = state.clone();
    control
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let mut state = press.borrow_mut();
            if state.dragged {
                return;
            }
            state.pointer_cancelled = false;
            state.pressed = true;
            state.keyboard_focus = None;
            if track_hover {
                state.hovered = true;
            }
            start_ripple(&mut state, Some(event.position), cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            let mut state = release.borrow_mut();
            release_ripple(&mut state, cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            let mut state = outside.borrow_mut();
            release_ripple(&mut state, cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_hover(move |hovering, window, cx| {
            let mut state = hover.borrow_mut();
            let was_pressed = state.pressed;
            if track_hover {
                state.hovered = *hovering;
            }
            if !*hovering && (track_hover || was_pressed) {
                release_ripple(&mut state, cx);
            }
            if track_hover || !*hovering && was_pressed {
                state.record(window, cx);
                window.refresh();
            }
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" {
                let mut state = keyboard.borrow_mut();
                state.cancel();
                state.record(window, cx);
                window.refresh();
            } else if matches!(event.keystroke.key.as_str(), "space" | "enter")
                && !event.is_held
                && keyboard_owner
                    .as_ref()
                    .is_none_or(|owner| owner.is_focused(window))
            {
                let mut state = keyboard.borrow_mut();
                if state.dragged {
                    return;
                }
                state.pressed = true;
                state.keyboard_focus = window.focused(cx);
                start_ripple(&mut state, None, cx);
                state.record(window, cx);
                window.refresh();
            }
        })
        .capture_key_up(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                let mut state = state.borrow_mut();
                release_ripple(&mut state, cx);
                state.record(window, cx);
                window.refresh();
            }
        })
}

pub fn ripple_state(
    id: &ElementId,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> Rc<RefCell<FeedbackState>> {
    let state = window
        .use_keyed_state((id.clone(), "ripple"), cx, |_, _| {
            Rc::new(RefCell::new(FeedbackState::default()))
        })
        .read(cx)
        .clone();
    state.borrow_mut().cancel_unavailable(enabled, window);
    state
}
