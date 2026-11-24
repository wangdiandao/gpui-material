use std::rc::Rc;

use gpui::{
    App, Context, FocusHandle, HitboxId, Pixels, Point, SharedString, Size, Subscription, Window,
    px,
};
use gpui_material_foundation::{DragCapture, InteractionState, InteractionTracker};
use gpui_material_theme::Direction;

const DRAG_SLOP: Pixels = px(4.);

pub(super) type ResizeRequest = Rc<dyn Fn(Pixels, &mut Window, &mut App)>;

pub(super) struct ResizeBinding {
    pub(super) label: SharedString,
    pub(super) width: Pixels,
    pub(super) min: Pixels,
    pub(super) max: Pixels,
    pub(super) request: ResizeRequest,
}

impl ResizeBinding {
    pub(super) fn request(&self, width: Pixels, window: &mut Window, cx: &mut App) {
        let width = width.clamp(self.min, self.max);
        if width != self.width {
            (self.request)(width, window, cx);
        }
    }
}

pub(super) struct PointerPress {
    mouse_capture: Option<(HitboxId, DragCapture)>,
    _escape: Subscription,
    origin: Point<Pixels>,
    width_at_press: Option<Pixels>,
    last_request: Option<Pixels>,
    dragged: bool,
}

impl PointerPress {
    pub(super) fn new(
        origin: Point<Pixels>,
        width: Option<Pixels>,
        mouse_capture: Option<(HitboxId, DragCapture)>,
        escape: Subscription,
    ) -> Self {
        Self {
            mouse_capture,
            _escape: escape,
            origin,
            width_at_press: width,
            last_request: width,
            dragged: false,
        }
    }
}

pub(super) struct HandleState {
    pub(super) focus: FocusHandle,
    pub(super) press: Option<PointerPress>,
    pub(super) hovered: bool,
    pub(super) direction: Direction,
    pub(super) adjustable: bool,
    pub(super) resize_context: Option<Size<Pixels>>,
    pub(super) tracker: Option<InteractionTracker>,
    _blur: Subscription,
    _activation: Subscription,
}

impl HandleState {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle().tab_stop(true);
        let blur = cx.on_blur(&focus, window, |state, window, cx| {
            state.end(window);
            state.publish(window, cx);
        });
        let activation = cx.observe_window_activation(window, |state, window, cx| {
            if !window.is_window_active() {
                state.end(window);
                state.hovered = false;
                state.publish(window, cx);
            }
        });
        let handle = window.window_handle();
        cx.on_release(move |state: &mut Self, cx| {
            if let Some((hitbox, _)) = state.press.take().and_then(|press| press.mouse_capture) {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, _| release_pointer(hitbox, window));
                });
            }
        })
        .detach();
        Self {
            focus,
            press: None,
            hovered: false,
            direction: Direction::Ltr,
            adjustable: false,
            resize_context: None,
            tracker: None,
            _blur: blur,
            _activation: activation,
        }
    }

    pub(super) fn snapshot(&self, window: &Window) -> InteractionState {
        let mut state = InteractionState::default();
        state.hovered = self.hovered;
        state.pressed = self.press.is_some();
        state.dragged = self.press.as_ref().is_some_and(|press| press.dragged);
        state.focus_visible =
            self.adjustable && self.focus.is_focused(window) && window.last_input_was_keyboard();
        state
    }

    pub(super) fn record(&self, window: &Window, cx: &mut App) {
        if let Some(tracker) = &self.tracker {
            tracker.record(self.snapshot(window), cx);
        }
    }

    pub(super) fn publish(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.record(window, cx);
        cx.notify();
        window.refresh();
    }

    pub(super) fn end(&mut self, window: &mut Window) {
        if let Some((hitbox, _)) = self.press.take().and_then(|press| press.mouse_capture) {
            release_pointer(hitbox, window);
        }
    }

    pub(super) fn final_pointer_width(&self) -> Option<Pixels> {
        self.press
            .as_ref()
            .filter(|press| press.dragged)?
            .last_request
    }

    pub(super) fn mouse_pressed(&self) -> bool {
        self.press
            .as_ref()
            .is_some_and(|press| press.mouse_capture.is_some())
    }

    pub(super) fn move_pointer(
        &mut self,
        position: Point<Pixels>,
        resize: Option<&ResizeBinding>,
    ) -> Option<Pixels> {
        let press = self.press.as_mut()?;
        let delta = position - press.origin;
        if !press.dragged {
            press.dragged = starts_horizontal_drag(delta);
        }
        if !press.dragged {
            return None;
        }
        let resize = resize?;
        let origin = press.width_at_press?;
        // Use the physical press origin even when the owner moves the handle
        // or rejects a request while it is being dragged.
        let value = (origin + logical_delta(delta.x, self.direction)).clamp(resize.min, resize.max);
        (press.last_request.replace(value) != Some(value)).then_some(value)
    }
}

pub(super) fn starts_horizontal_drag(delta: Point<Pixels>) -> bool {
    delta.x.abs() >= DRAG_SLOP && delta.x.abs() > delta.y.abs()
}

pub(super) fn release_pointer(id: HitboxId, window: &mut Window) {
    if window.captured_hitbox() == Some(id) {
        window.release_pointer();
    }
}

pub(super) fn logical_delta(delta: Pixels, direction: Direction) -> Pixels {
    if direction == Direction::Rtl {
        -delta
    } else {
        delta
    }
}
