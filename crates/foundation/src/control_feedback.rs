//! Shared presentation-event adapter for interactive Material controls.
//!
//! `gpui-base` owns activation, focus, disabled semantics, and controlled values;
//! this module tracks hover, press, drag, ripple, and prepared shape geometry for
//! Material state layers and focus rings without holding `RefMut` borrows across
//! GPUI notification callbacks.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, Bounds, ClickEvent, ElementId, EntityId, FocusHandle, MouseButton, Pixels, Point,
    StatefulInteractiveElement, Window,
};
use gpui_material_theme::{CornerShape, Direction};
use web_time::Instant;

use crate::{InteractionState, InteractionTracker, Ripple, Shape, ShapeFit, ShapePolygon};

struct FeedbackUpdate {
    tracker: Option<(InteractionTracker, InteractionState)>,
    view: Option<EntityId>,
    changed: bool,
}

impl FeedbackUpdate {
    fn apply(self, window: &mut Window, cx: &mut App) {
        if let Some((tracker, state)) = self.tracker {
            tracker.record(state, cx);
        }
        if self.changed {
            if let Some(view) = self.view {
                cx.notify(view);
            } else {
                window.refresh();
            }
        }
    }
}

/// Retained interaction, ripple, and geometry state for one interactive Material control.
#[derive(Default)]
pub struct FeedbackState {
    hovered: bool,
    pressed: bool,
    dragged: bool,
    pointer_cancelled: bool,
    shape_hit_test: bool,
    keyboard_focus: Option<FocusHandle>,
    rounded_key: Option<(Bounds<Pixels>, CornerShape, Direction)>,
    polygon_key: Option<(Bounds<Pixels>, ShapePolygon, ShapeFit)>,
    shape: Option<Shape>,
    ripple: Option<Ripple>,
    view: Option<EntityId>,
    tracker: Option<(InteractionTracker, FocusHandle)>,
}

impl FeedbackState {
    /// Returns the last prepared [`Shape`] for this control, if laid out.
    pub fn shape(&self) -> Option<&Shape> {
        self.shape.as_ref()
    }

    /// Returns the active [`Ripple`] animation for this control, if any.
    pub fn ripple(&self) -> Option<&Ripple> {
        self.ripple.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn set_ripple_for_test(&mut self, ripple: Option<Ripple>) {
        self.ripple = ripple;
    }

    /// Updates the feedback geometry unconditionally and captures its owning view during prepaint.
    pub fn set_shape(&mut self, shape: Option<Shape>, window: &Window) {
        self.view = Some(window.current_view());
        self.rounded_key = None;
        self.polygon_key = None;
        self.shape = shape;
    }

    /// Updates rounded-rectangle feedback geometry only when the bounds, corners, or direction change.
    pub fn set_rounded_shape(
        &mut self,
        bounds: Bounds<Pixels>,
        corners: CornerShape,
        direction: Direction,
        window: &Window,
    ) {
        self.view = Some(window.current_view());
        self.polygon_key = None;
        let key = (bounds, corners, direction);
        if self.rounded_key != Some(key) {
            self.rounded_key = Some(key);
            self.shape = Shape::rounded(bounds, corners, direction);
        }
    }

    /// Updates polygon feedback geometry only when the bounds, polygon contour, or fit mode change.
    pub fn set_polygon_shape(
        &mut self,
        bounds: Bounds<Pixels>,
        polygon: &ShapePolygon,
        fit: ShapeFit,
        window: &Window,
    ) {
        self.view = Some(window.current_view());
        self.rounded_key = None;
        let matches =
            self.polygon_key
                .as_ref()
                .is_some_and(|(cached_bounds, cached_polygon, cached_fit)| {
                    *cached_bounds == bounds && *cached_fit == fit && cached_polygon == polygon
                });
        if !matches {
            self.polygon_key = Some((bounds, polygon.clone(), fit));
            self.shape = Shape::polygon(bounds, polygon, fit);
        }
    }

    pub(crate) fn prune_finished_ripple(&mut self, now: Instant) {
        if self
            .ripple
            .as_ref()
            .is_some_and(|ripple| ripple.is_finished(now))
        {
            self.ripple = None;
        }
    }

    /// Enables or disables shape-aware pointer hit-test gating against [`Self::shape`].
    pub fn set_shape_hit_test(&mut self, enabled: bool) {
        self.shape_hit_test = enabled;
    }

    /// Returns whether `position` falls within the interactive hit area.
    pub fn contains_pointer(&self, position: Point<Pixels>) -> bool {
        if !self.shape_hit_test {
            return true;
        }
        self.shape
            .as_ref()
            .is_none_or(|shape| shape.contains(position))
    }

    fn take_update(&self, changed: bool, window: &Window) -> FeedbackUpdate {
        let tracker = self
            .tracker
            .as_ref()
            .map(|(tracker, focus)| (tracker.clone(), self.interaction(true, focus, window)));
        FeedbackUpdate {
            tracker,
            view: self.view,
            changed,
        }
    }

    /// Computes the current [`InteractionState`] snapshot for this control.
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

    /// Cancels hover, press, and ripple state when the control becomes disabled,
    /// its window deactivates, or its keyboard focus owner blurs.
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

    fn cancel(&mut self) -> bool {
        let changed = self.pressed || self.ripple.is_some();
        self.pressed = false;
        self.keyboard_focus = None;
        self.ripple = None;
        changed
    }

    /// Clears all active pointer, keyboard, hover, and tracker state.
    pub fn clear(&mut self) {
        self.cancel();
        self.hovered = false;
        self.tracker = None;
    }

    /// Returns whether this control is currently in a dragged interaction state.
    pub fn is_dragged(&self) -> bool {
        self.dragged
    }

    /// Returns whether this control is currently pressed.
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Updates dragged state, cancelling any active press/ripple when entering drag.
    pub fn set_dragged(&mut self, dragged: bool) {
        if dragged {
            self.pointer_cancelled = true;
            self.cancel();
        }
        self.dragged = dragged;
    }

    /// Computes the current [`InteractionState`], forwards it to `tracker`, and stores `tracker`.
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

    /// Records a caller-computed [`InteractionState`] snapshot into `tracker` and retains `tracker`.
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

    /// Returns `true` when `event` should be suppressed due to drag, prior pointer cancellation,
    /// or shape-aware hit-test failure.
    pub fn blocks_activation(&self, event: &ClickEvent) -> bool {
        if self.dragged
            || self.pointer_cancelled
                && matches!(event, ClickEvent::Mouse(_) | ClickEvent::Touch(_))
        {
            return true;
        }
        if self.shape_hit_test {
            match event {
                ClickEvent::Mouse(mouse) => {
                    !self.contains_pointer(mouse.down.position)
                        || !self.contains_pointer(mouse.up.position)
                }
                ClickEvent::Touch(touch) => !self.contains_pointer(touch.position),
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Triggers or releases ripple feedback in response to a GPUI click activation event.
pub fn activate_feedback(
    event: &ClickEvent,
    state: &RefCell<FeedbackState>,
    window: &mut Window,
    cx: &mut App,
) {
    let update = {
        let mut state = state.borrow_mut();
        let changed = match event {
            // Activation stops the bubbling click. Complete the pointer feedback
            // here even if the separate mouse-up listener does not run.
            ClickEvent::Mouse(_) => release_ripple(&mut state, cx),
            event if !state.pressed => {
                state.prune_finished_ripple(cx.background_executor().now());
                let replaced_ripple = state.ripple.is_some();
                let pointer = match event {
                    ClickEvent::Touch(event) => Some(event.position),
                    _ => None,
                };
                start_ripple(&mut state, pointer, cx);
                release_ripple(&mut state, cx) || replaced_ripple
            }
            _ => false,
        };
        state.take_update(changed, window)
    };
    update.apply(window, cx);
}

fn release_ripple(state: &mut FeedbackState, cx: &App) -> bool {
    let pressed = std::mem::take(&mut state.pressed);
    state.keyboard_focus = None;
    let released = state
        .ripple
        .as_mut()
        .is_some_and(|ripple| ripple.release_if_held(cx.background_executor().now()));
    pressed || released
}

fn start_ripple(state: &mut FeedbackState, pointer: Option<Point<Pixels>>, cx: &App) {
    state.ripple = state.shape.as_ref().and_then(|shape| {
        // The accessible hit target can extend beyond the painted container.
        // Its activation still needs feedback, centered in the visible shape.
        let pointer = pointer.filter(|pointer| shape.contains(*pointer));
        Ripple::press(shape, pointer, cx.background_executor().now())
    });
}

/// Observes presentation events without consuming them: Base still owns activation,
/// focus, disabled semantics, and controlled value changes. Action controls also
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
    let track_move = track_hover && state.borrow().shape_hit_test;
    let press = state.clone();
    let release = state.clone();
    let outside = state.clone();
    let hover = state.clone();
    let move_hover = state.clone();
    let keyboard = state.clone();
    let control = control
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let update = {
                let mut state = press.borrow_mut();
                if state.dragged {
                    return;
                }
                if !state.contains_pointer(event.position) {
                    state.pointer_cancelled = true;
                    return;
                }
                state.pointer_cancelled = false;
                state.pressed = true;
                state.keyboard_focus = None;
                if track_hover {
                    state.hovered = true;
                }
                start_ripple(&mut state, Some(event.position), cx);
                state.take_update(true, window)
            };
            update.apply(window, cx);
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            let update = {
                let mut state = release.borrow_mut();
                let changed = release_ripple(&mut state, cx);
                state.take_update(changed, window)
            };
            update.apply(window, cx);
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            let update = {
                let mut state = outside.borrow_mut();
                let changed = release_ripple(&mut state, cx);
                state.take_update(changed, window)
            };
            update.apply(window, cx);
        })
        .on_hover(move |hovering, window, cx| {
            let update = {
                let mut state = hover.borrow_mut();
                let effective_hover = *hovering && state.contains_pointer(window.mouse_position());
                let was_pressed = state.pressed;
                let hover_changed = track_hover && state.hovered != effective_hover;
                if track_hover {
                    state.hovered = effective_hover;
                }
                let released = !effective_hover
                    && (track_hover || was_pressed)
                    && release_ripple(&mut state, cx);
                (track_hover || !effective_hover && was_pressed)
                    .then(|| state.take_update(hover_changed || released, window))
            };
            if let Some(update) = update {
                update.apply(window, cx);
            }
        });
    let control = if track_move {
        control.on_mouse_move(move |event, window, cx| {
            let update = {
                let mut state = move_hover.borrow_mut();
                if !state.shape_hit_test {
                    return;
                }
                let inside = state.contains_pointer(event.position);
                if state.hovered != inside {
                    state.hovered = inside;
                    if !inside && state.pressed {
                        release_ripple(&mut state, cx);
                    }
                    Some(state.take_update(true, window))
                } else {
                    None
                }
            };
            if let Some(update) = update {
                update.apply(window, cx);
            }
        })
    } else {
        control
    };
    control
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" {
                let update = {
                    let mut state = keyboard.borrow_mut();
                    state.prune_finished_ripple(cx.background_executor().now());
                    let changed = state.cancel();
                    state.take_update(changed, window)
                };
                update.apply(window, cx);
            } else if matches!(event.keystroke.key.as_str(), "space" | "enter")
                && !event.is_held
                && keyboard_owner
                    .as_ref()
                    .is_none_or(|owner| owner.is_focused(window))
            {
                let update = {
                    let mut state = keyboard.borrow_mut();
                    if state.dragged {
                        return;
                    }
                    state.pressed = true;
                    state.keyboard_focus = window.focused(cx);
                    start_ripple(&mut state, None, cx);
                    state.take_update(true, window)
                };
                update.apply(window, cx);
            }
        })
        .capture_key_up(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                let update = {
                    let mut state = state.borrow_mut();
                    let changed = release_ripple(&mut state, cx);
                    state.take_update(changed, window)
                };
                update.apply(window, cx);
            }
        })
}

/// Retrieves or creates the keyed [`FeedbackState`] for an interactive element,
/// pruning finished ripples and cancelling unavailable states.
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
    {
        let mut state = state.borrow_mut();
        state.prune_finished_ripple(cx.background_executor().now());
        state.cancel_unavailable(enabled, window);
    }
    state
}
