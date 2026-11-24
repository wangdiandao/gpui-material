use gpui::StatefulInteractiveElement;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, DragMoveEvent, Entity, FocusHandle, HitboxBehavior,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Render, Size,
    Stateful, Styled, TouchDragEvent, TouchPhase, Window, canvas, div,
};
use gpui_base::slider::{SliderEvent, SliderState, SliderValue};
use gpui_material_foundation::{
    DragCapture, InteractionSource, InteractionState, InteractionTracker, RangeThumb,
};

use super::geometry::{Geometry, choose_thumb};
use super::{Slider, thumb_id, thumb_index};

#[derive(Default)]
pub(super) struct Gesture {
    pub(super) thumb: Option<RangeThumb>,
    pub(super) origin: Option<Point<Pixels>>,
    pub(super) initial_value: Option<SliderValue>,
    pub(super) capture: Option<DragCapture>,
    pub(super) dragged: bool,
    pub(super) disabled: bool,
    pub(super) feedback: [ThumbFeedback; 2],
}

#[derive(Default)]
pub(super) struct ThumbFeedback {
    pub(super) focus: Option<FocusHandle>,
    pub(super) tracker: Option<InteractionTracker>,
    pub(super) hovered: bool,
}

impl Gesture {
    pub(super) fn record(&self, window: &Window, cx: &mut App) {
        for (which, feedback) in [RangeThumb::Start, RangeThumb::End]
            .into_iter()
            .zip(&self.feedback)
        {
            let Some(tracker) = &feedback.tracker else {
                continue;
            };
            let mut snapshot = InteractionState::default();
            snapshot.disabled = self.disabled;
            if !self.disabled && window.is_window_active() {
                snapshot.hovered = feedback.hovered;
                snapshot.focus_visible = window.last_input_was_keyboard()
                    && feedback
                        .focus
                        .as_ref()
                        .is_some_and(|focus| focus.is_focused(window));
                snapshot.pressed = self.thumb == Some(which);
                snapshot.dragged = snapshot.pressed && self.dragged;
            }
            tracker.record(snapshot, cx);
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum FinishMode {
    Commit,
    CancelCapture,
    Rollback,
}

struct SliderDrag(Entity<SliderState>);

impl Render for SliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(super) fn sync_gesture_feedback(
    slider: &Slider,
    focus_handles: [&FocusHandle; 2],
    gesture: &RefCell<Gesture>,
    range: bool,
    window: &mut Window,
    cx: &mut App,
) {
    {
        let mut gesture = gesture.borrow_mut();
        gesture.disabled = slider.disabled;
        for which in [RangeThumb::Start, RangeThumb::End] {
            let index = thumb_index(which);
            let feedback = &mut gesture.feedback[index];
            feedback.focus = Some(focus_handles[index].clone());
            feedback.tracker = slider.thumbs[index]
                .source
                .as_ref()
                .filter(|_| range || which == RangeThumb::End)
                .map(|source| {
                    let default_key;
                    let key = match &slider.thumbs[index].source_key {
                        Some(id) => id,
                        None => {
                            default_key = thumb_id(&slider.id, which);
                            &default_key
                        }
                    };
                    InteractionSource::track(source, key, window, cx)
                });
            if slider.disabled || !window.is_window_active() || !range && which == RangeThumb::Start
            {
                feedback.hovered = false;
            }
        }
    }
    gesture.borrow().record(window, cx);
}

fn render_touch_layer(
    state: Entity<SliderState>,
    gesture: Rc<RefCell<Gesture>>,
    focus: [FocusHandle; 2],
    geometry: Geometry,
    range: bool,
    handle_sizes: [Size<Pixels>; 2],
) -> impl IntoElement {
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, _| {
            let gesture = gesture.clone();
            let state = state.clone();
            let focus = focus.clone();
            window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                if !phase.bubble() {
                    return;
                }
                if event.phase == TouchPhase::Started {
                    if window.default_prevented()
                        || !hitbox.is_hovered_at(event.start_position, window)
                    {
                        return;
                    }
                    let slider = state.read(cx);
                    let bounds = slider.bounds();
                    if !geometry.has_positive_extent(bounds) {
                        return;
                    }
                    let current = slider.percentage();
                    let initial_value = slider.value();
                    let which = if range && current.start == current.end {
                        let fraction = geometry.fraction(bounds, event.start_position);
                        choose_thumb(
                            true,
                            fraction,
                            fraction,
                            geometry.fraction(bounds, event.position),
                        )
                    } else {
                        geometry.choose_at(
                            range,
                            current,
                            handle_sizes,
                            bounds,
                            event.start_position,
                        )
                    };
                    window.prevent_default();
                    {
                        let mut gesture = gesture.borrow_mut();
                        gesture.origin = Some(event.start_position);
                        gesture.thumb = Some(which);
                        gesture.initial_value = Some(initial_value);
                    }
                    focus[thumb_index(which)].focus(window, cx);
                }
                let which = {
                    let gesture = gesture.borrow();
                    if gesture.origin != Some(event.start_position) || gesture.capture.is_some() {
                        return;
                    }
                    let Some(which) = gesture.thumb else {
                        return;
                    };
                    which
                };
                match event.phase {
                    TouchPhase::Started | TouchPhase::Moved => {
                        {
                            let mut gesture = gesture.borrow_mut();
                            gesture.dragged = event.position != event.start_position;
                        }
                        gesture.borrow().record(window, cx);
                        update(&state, geometry, which, event.position, window, cx);
                    }
                    TouchPhase::Ended => {
                        finish(&state, &gesture, window, cx, FinishMode::Commit);
                    }
                    TouchPhase::Cancelled => {
                        finish(&state, &gesture, window, cx, FinishMode::Rollback);
                    }
                }
                cx.stop_propagation();
            });
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_pointer_handlers(
    root: Stateful<gpui::Div>,
    state: Entity<SliderState>,
    gesture: Rc<RefCell<Gesture>>,
    focus: [FocusHandle; 2],
    geometry: Geometry,
    range: bool,
    start: f32,
    end: f32,
    handle_sizes: [Size<Pixels>; 2],
) -> AnyElement {
    let touch_layer = render_touch_layer(
        state.clone(),
        gesture.clone(),
        focus.clone(),
        geometry,
        range,
        handle_sizes,
    );
    let [start_focus, end_focus] = focus.clone();
    let down = gesture.clone();
    let begin = gesture.clone();
    let drag = gesture.clone();
    let release = gesture.clone();
    let release_out = gesture.clone();
    let scroll = gesture.clone();
    let escape = gesture;
    let down_state = state.clone();
    let begin_state = state.clone();
    let drag_state = state.clone();
    let drag_source = state.clone();
    let drag_focus = focus;
    let release_state = state.clone();
    let outside_state = state.clone();
    let escape_state = state;
    root.child(touch_layer)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let slider = down_state.read(cx);
            let bounds = slider.bounds();
            if !geometry.has_positive_extent(bounds) {
                return;
            }
            let initial_value = slider.value();
            let which = geometry.choose_at(
                range,
                slider.percentage(),
                handle_sizes,
                bounds,
                event.position,
            );
            {
                let mut down = down.borrow_mut();
                down.thumb = Some(which);
                down.origin = Some(event.position);
                down.initial_value = Some(initial_value);
            }
            if which == RangeThumb::Start {
                start_focus.focus(window, cx);
            } else {
                end_focus.focus(window, cx);
            }
            down.borrow().record(window, cx);
            update(&down_state, geometry, which, event.position, window, cx);
        })
        .on_drag(SliderDrag(drag_source), move |payload, _, window, cx| {
            let gesture = Rc::downgrade(&begin);
            let state = begin_state.clone();
            let capture = DragCapture::watch(window, cx, move |window, cx| {
                if let Some(gesture) = gesture.upgrade() {
                    finish(&state, &gesture, window, cx, FinishMode::Commit);
                }
            });
            {
                let mut begin = begin.borrow_mut();
                begin.capture = Some(capture);
                begin.dragged = true;
            }
            begin.borrow().record(window, cx);
            cx.stop_propagation();
            cx.new(|_| SliderDrag(payload.0.clone()))
        })
        .on_drag_move(move |event: &DragMoveEvent<SliderDrag>, window, cx| {
            if event.drag(cx).0.entity_id() != drag_state.entity_id() {
                return;
            }
            let mut gesture = drag.borrow_mut();
            let Some(mut which) = gesture.thumb else {
                return;
            };
            let mut focus_thumb = None;
            if range
                && start == end
                && let Some(origin) = gesture.origin
            {
                let bounds = drag_state.read(cx).bounds();
                if !geometry.has_positive_extent(bounds) {
                    drop(gesture);
                    cx.stop_propagation();
                    return;
                }
                let current = geometry.fraction(bounds, event.event.position);
                let start_pos = geometry.fraction(bounds, origin);
                if (current - start_pos).abs() <= f32::EPSILON {
                    drop(gesture);
                    cx.stop_propagation();
                    return;
                }
                gesture.origin = None;
                which = if current < start_pos {
                    RangeThumb::Start
                } else {
                    RangeThumb::End
                };
                gesture.thumb = Some(which);
                focus_thumb = Some(which);
            }
            drop(gesture);
            if let Some(which) = focus_thumb {
                drag_focus[thumb_index(which)].focus(window, cx);
            }
            drag.borrow().record(window, cx);
            update(
                &drag_state,
                geometry,
                which,
                event.event.position,
                window,
                cx,
            );
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            finish(&release_state, &release, window, cx, FinishMode::Commit)
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            finish(&outside_state, &release_out, window, cx, FinishMode::Commit)
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && !event.keystroke.modifiers.modified()
                && escape.borrow().thumb.is_some()
            {
                finish(&escape_state, &escape, window, cx, FinishMode::Rollback);
                cx.stop_propagation();
            }
        })
        .on_scroll(move |_, _, cx| {
            // Wheel events over the active range gesture stay with it.
            if scroll.borrow().thumb.is_some() {
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

pub(super) fn update(
    state: &Entity<SliderState>,
    geometry: Geometry,
    thumb: RangeThumb,
    pointer: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        let bounds = state.bounds();
        if !geometry.has_positive_extent(bounds) {
            return;
        }
        let position = geometry.position(bounds, pointer);
        state.update_value_by_position(
            geometry.axis,
            position,
            thumb == RangeThumb::Start,
            window,
            cx,
        );
    });
}

pub(super) fn finish(
    state: &Entity<SliderState>,
    gesture: &RefCell<Gesture>,
    window: &mut Window,
    cx: &mut App,
    mode: FinishMode,
) {
    let (capture, rollback) = {
        let mut gesture = gesture.borrow_mut();
        if gesture.thumb.take().is_none() {
            return;
        }
        gesture.origin = None;
        gesture.dragged = false;
        let rollback = gesture.initial_value.take();
        let capture = gesture.capture.take();
        (capture, rollback)
    };
    gesture.borrow().record(window, cx);
    if let Some(capture) = capture
        && mode != FinishMode::Commit
    {
        capture.cancel(window, cx);
    }
    state.update(cx, |state, cx| {
        if mode == FinishMode::Rollback
            && let Some(initial) = rollback
            && state.value() != initial
        {
            state.set_value(initial, window, cx);
            cx.emit(SliderEvent::Change(initial));
        }
        state.handle_release(cx);
    });
    window.refresh();
}
