use gpui::{
    AnyElement, App, Bounds, Element, ElementId, FocusHandle, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, TouchDragEvent, TouchPhase, WeakEntity, Window, point, px,
};
use gpui_material_foundation::DragCapture;

use super::state::{SWIPE_SLOP, SwipeInput, SwipeToDismissState, SwipeToDismissValue};

// Keep layout and clipping at the resting position. Only prepaint the
// foreground at its translated position so hit testing and rendering agree.
pub(super) struct SwipeContent {
    pub(super) content: AnyElement,
    pub(super) state: WeakEntity<SwipeToDismissState>,
    pub(super) focus: FocusHandle,
}

impl Element for SwipeContent {
    type RequestLayoutState = ();
    type PrepaintState = (bool, Option<Hitbox>);

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.content.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (offset, visible) = self
            .state
            .update(cx, |state, cx| {
                if state.width != bounds.size.width {
                    state.width = bounds.size.width;
                    if state.position != 0. || state.moving {
                        cx.notify();
                    }
                }
                (state.offset(), state.shows_foreground())
            })
            .unwrap_or((px(0.), true));
        if visible {
            window.set_focus_handle(&self.focus, cx);
            window.with_element_offset(point(offset, px(0.)), |window| {
                self.content.prepaint(window, cx);
            });
        }
        let hitbox = self.state.upgrade().and_then(|state| {
            let state = state.read(cx);
            (state.can_start() || !matches!(state.input, SwipeInput::None))
                .then(|| window.insert_hitbox(bounds, HitboxBehavior::Normal))
        });
        (visible, hitbox)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        (visible, hitbox): &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if *visible {
            self.content.paint(window, cx);
        }
        if let Some(hitbox) = hitbox {
            listen_for_swipes(self.state.clone(), hitbox.clone(), window);
        }
    }
}

fn listen_for_swipes(state: WeakEntity<SwipeToDismissState>, hitbox: Hitbox, window: &mut Window) {
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if phase.bubble() && event.button == MouseButton::Left && hitbox.is_hovered(window) {
                let _ = state.update(cx, |state, _| {
                    if state.can_start()
                        && matches!(state.input, SwipeInput::None | SwipeInput::SuppressMouseUp)
                    {
                        state.input = SwipeInput::Pressed(event.position);
                    }
                });
            }
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            if !phase.bubble() {
                return;
            }
            let _ = state.update(cx, |swipe, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    if !matches!(swipe.input, SwipeInput::Touch)
                        && swipe.cancel_input(window, false)
                    {
                        swipe.move_to(SwipeToDismissValue::Settled, window, cx);
                    }
                    return;
                }
                match swipe.input {
                    SwipeInput::Pressed(origin) => {
                        let delta = event.position - origin;
                        if delta.x.abs().max(delta.y.abs()) >= SWIPE_SLOP {
                            if swipe.options.can_start(delta)
                                && !cx.has_active_drag()
                                && window.captured_hitbox().is_none()
                            {
                                window.capture_pointer(hitbox.id);
                                swipe.input = watch_mouse_swipe(state.clone(), origin, window, cx);
                                swipe.drag_to(delta.x, window, cx);
                                cx.stop_propagation();
                            } else {
                                swipe.input = SwipeInput::None;
                            }
                        }
                    }
                    SwipeInput::Mouse { origin, .. } => {
                        swipe.drag_to(event.position.x - origin.x, window, cx);
                        cx.stop_propagation();
                    }
                    SwipeInput::SuppressMouseUp => cx.stop_propagation(),
                    _ => {}
                }
            });
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        move |event: &MouseUpEvent, phase, window, cx| {
            if !phase.bubble() || event.button != MouseButton::Left {
                return;
            }
            let _ = state.update(cx, |state, cx| match state.input {
                SwipeInput::Mouse { origin, .. } => {
                    state.release(event.position.x - origin.x, window, cx);
                    cx.stop_propagation();
                }
                SwipeInput::SuppressMouseUp => {
                    state.input = SwipeInput::None;
                    cx.stop_propagation();
                }
                SwipeInput::Pressed(_) => state.input = SwipeInput::None,
                _ => {}
            });
        }
    });
    window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
        if !phase.bubble() {
            return;
        }
        let _ = state.update(cx, |state, cx| {
            let delta = event.position - event.start_position;
            match event.phase {
                TouchPhase::Started
                    if hitbox.is_hovered_at(event.start_position, window)
                        && state.can_start()
                        && matches!(state.input, SwipeInput::None)
                        && state.options.can_start(delta) =>
                {
                    state.input = SwipeInput::Touch;
                    state.drag_to(delta.x, window, cx);
                    window.prevent_default();
                    cx.stop_propagation();
                }
                TouchPhase::Moved | TouchPhase::Ended
                    if matches!(state.input, SwipeInput::Touch) =>
                {
                    if event.phase == TouchPhase::Ended {
                        state.release(delta.x, window, cx);
                    } else {
                        state.drag_to(delta.x, window, cx);
                    }
                    cx.stop_propagation();
                }
                TouchPhase::Cancelled if matches!(state.input, SwipeInput::Touch) => {
                    state.cancel_interaction(window, cx);
                    cx.stop_propagation();
                }
                _ => {}
            }
        });
    });
}

fn watch_mouse_swipe(
    state: WeakEntity<SwipeToDismissState>,
    origin: Point<Pixels>,
    window: &Window,
    cx: &mut App,
) -> SwipeInput {
    let window_id = window.window_handle().window_id();
    let escape = cx.intercept_keystrokes({
        let state = state.clone();
        move |event, window, cx| {
            if event.keystroke.key == "escape"
                && !event.keystroke.modifiers.modified()
                && window.window_handle().window_id() == window_id
            {
                let cancelled = state.update(cx, |state, cx| state.cancel_interaction(window, cx));
                if cancelled.unwrap_or(false) {
                    cx.stop_propagation();
                }
            }
        }
    });
    let capture = DragCapture::watch_pointer(window, cx, move |window, cx| {
        let _ = state.update(cx, |state, cx| {
            state.cancel_interaction(window, cx);
        });
    });
    SwipeInput::Mouse {
        origin,
        _capture: capture,
        _escape: escape,
    }
}
