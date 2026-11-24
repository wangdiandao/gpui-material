use std::mem;

use gpui::{
    Context, ElementId, EntityId, EventEmitter, FocusHandle, Pixels, Point, Subscription, Window,
    px,
};
use gpui_material_foundation::{DragCapture, animate, current_material_theme};
use gpui_material_theme::{Direction, MotionKind, MotionSpec, MotionSpeed};

const DEFAULT_THRESHOLD: Pixels = px(56.);
pub(super) const SWIPE_SLOP: Pixels = px(6.);

/// The resting anchor or a logical dismissal direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwipeToDismissValue {
    /// Resting, undismissed position.
    #[default]
    Settled,
    /// Dismissed from leading edge toward trailing edge (left-to-right in LTR, right-to-left in RTL).
    StartToEnd,
    /// Dismissed from trailing edge toward leading edge (right-to-left in LTR, left-to-right in RTL).
    EndToStart,
}

impl SwipeToDismissValue {
    fn position(self) -> f32 {
        match self {
            Self::Settled => 0.,
            Self::StartToEnd => 1.,
            Self::EndToStart => -1.,
        }
    }

    fn from_position(position: f32) -> Self {
        if position > 0. {
            Self::StartToEnd
        } else if position < 0. {
            Self::EndToStart
        } else {
            Self::Settled
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SwipeOptions {
    pub(super) direction: Direction,
    pub(super) start_to_end: bool,
    pub(super) end_to_start: bool,
    pub(super) gestures_enabled: bool,
}

impl Default for SwipeOptions {
    fn default() -> Self {
        Self {
            direction: Direction::Ltr,
            start_to_end: true,
            end_to_start: true,
            gestures_enabled: true,
        }
    }
}

impl SwipeOptions {
    fn allows(self, value: SwipeToDismissValue) -> bool {
        match value {
            SwipeToDismissValue::Settled => true,
            SwipeToDismissValue::StartToEnd => self.start_to_end,
            SwipeToDismissValue::EndToStart => self.end_to_start,
        }
    }

    fn sign(self) -> f32 {
        match self.direction {
            Direction::Ltr => 1.,
            Direction::Rtl => -1.,
        }
    }

    pub(super) fn can_start(self, delta: Point<Pixels>) -> bool {
        delta.x.abs() >= SWIPE_SLOP
            && delta.x.abs() > delta.y.abs()
            && self.allows(SwipeToDismissValue::from_position(
                f32::from(delta.x) * self.sign(),
            ))
    }
}

#[derive(Default)]
pub(super) enum SwipeInput {
    #[default]
    None,
    Pressed(Point<Pixels>),
    Mouse {
        origin: Point<Pixels>,
        _capture: DragCapture,
        _escape: Subscription,
    },
    Touch,
    SuppressMouseUp,
}

pub(super) struct Dismissed(pub(super) SwipeToDismissValue);

/// Retained swipe state. Give each mounted swipe container its own entity.
pub struct SwipeToDismissState {
    pub(super) width: Pixels,
    pub(super) position: f32,
    target: SwipeToDismissValue,
    pub(super) settled: SwipeToDismissValue,
    pub(super) moving: bool,
    rebase_motion: bool,
    pub(super) options: SwipeOptions,
    threshold: Option<Box<dyn Fn(Pixels) -> Pixels>>,
    pub(super) input: SwipeInput,
    pub(super) mount: Option<(EntityId, FocusHandle)>,
}

impl Default for SwipeToDismissState {
    fn default() -> Self {
        Self {
            width: px(0.),
            position: 0.,
            target: SwipeToDismissValue::Settled,
            settled: SwipeToDismissValue::Settled,
            moving: false,
            rebase_motion: false,
            options: SwipeOptions::default(),
            threshold: None,
            input: SwipeInput::None,
            mount: None,
        }
    }
}

impl EventEmitter<Dismissed> for SwipeToDismissState {}

impl SwipeToDismissState {
    /// Creates a new swipe state settled at [`SwipeToDismissValue::Settled`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects the initial anchor before mounting the state.
    pub fn initial_value(mut self, value: SwipeToDismissValue) -> Self {
        self.position = value.position();
        self.target = value;
        self.settled = value;
        self
    }

    /// Computes the release threshold from the current width in logical pixels.
    /// The default is 56 logical pixels, limited by the available travel.
    pub fn positional_threshold(mut self, threshold: impl Fn(Pixels) -> Pixels + 'static) -> Self {
        self.threshold = Some(Box::new(threshold));
        self
    }

    /// Returns the nearest anchor to the current foreground position.
    pub fn current_value(&self) -> SwipeToDismissValue {
        SwipeToDismissValue::from_position(self.position.round())
    }

    /// Returns the anchor where the current interaction would land.
    pub fn target_value(&self) -> SwipeToDismissValue {
        self.target
    }

    /// Returns the last fully reached anchor.
    pub fn settled_value(&self) -> SwipeToDismissValue {
        self.settled
    }

    /// Returns the logical direction of the current offset.
    pub fn dismiss_direction(&self) -> SwipeToDismissValue {
        SwipeToDismissValue::from_position(self.position)
    }

    /// Returns the fraction of the width uncovered by the foreground.
    pub fn progress(&self) -> f32 {
        self.position.abs()
    }

    /// Returns the signed horizontal offset in window coordinates.
    pub fn offset(&self) -> Pixels {
        self.width * (self.position * self.options.sign())
    }

    /// Animates to an enabled dismissal direction. Returns whether it was accepted.
    pub fn dismiss(
        &mut self,
        direction: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if direction == SwipeToDismissValue::Settled || !self.options.allows(direction) {
            return false;
        }
        self.cancel_input(window, true);
        self.move_to(direction, window, cx);
        true
    }

    /// Animates the foreground back to its resting position.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_input(window, true);
        self.move_to(SwipeToDismissValue::Settled, window, cx);
    }

    /// Moves immediately to an enabled anchor. Returns whether it was accepted.
    pub fn snap_to(
        &mut self,
        value: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.options.allows(value) {
            return false;
        }
        let cancelled = self.cancel_input(window, true);
        let settled = self.settle(value, window, cx);
        if cancelled || settled {
            window.refresh();
        }
        true
    }

    pub(crate) fn shows_foreground(&self) -> bool {
        self.moving || self.settled == SwipeToDismissValue::Settled
    }

    pub(crate) fn is_dragging(&self) -> bool {
        matches!(self.input, SwipeInput::Mouse { .. } | SwipeInput::Touch)
    }

    pub(super) fn move_to(
        &mut self,
        value: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.moving && self.target == value {
            return;
        }
        let changed = if cx.reduce_motion() || self.position == value.position() {
            self.settle(value, window, cx)
        } else {
            if !self.moving {
                self.rebase_motion = true;
            }
            self.target = value;
            self.moving = true;
            cx.notify();
            true
        };
        if changed {
            window.refresh();
        }
    }

    pub(super) fn settle(
        &mut self,
        value: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let changed = self.position != value.position() || self.target != value || self.moving;
        self.position = value.position();
        self.target = value;
        self.moving = false;
        self.rebase_motion = false;
        if self.settled != value {
            self.settled = value;
            if value != SwipeToDismissValue::Settled {
                if self
                    .mount
                    .as_ref()
                    .is_some_and(|(_, focus)| focus.contains_focused(window, cx))
                {
                    window.blur(cx);
                }
                cx.emit(Dismissed(value));
            }
            cx.notify();
            true
        } else if changed {
            cx.notify();
            true
        } else {
            false
        }
    }

    pub(super) fn sample_motion(
        &mut self,
        id: &ElementId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.moving {
            return;
        }
        let motion_id = (id.clone(), "swipe-offset");
        if mem::take(&mut self.rebase_motion) {
            animate(
                motion_id.clone(),
                self.position,
                MotionSpec::Instant,
                window,
                cx,
            );
        }
        let target = self.target.position();
        let spec = current_material_theme(window, cx).motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Fast,
            cx.reduce_motion(),
        );
        let position = animate(motion_id, target, spec, window, cx);
        if position == target {
            self.settle(self.target, window, cx);
        } else {
            let position = position.clamp(-1., 1.);
            if self.position != position {
                self.position = position;
                cx.notify();
            }
        }
    }

    pub(super) fn configure(
        &mut self,
        options: SwipeOptions,
        mount: (EntityId, FocusHandle),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mount = Some(mount);
        let previous = mem::replace(&mut self.options, options);
        if previous != options {
            if !options.allows(self.target) || !options.allows(self.settled) {
                self.cancel_input(window, true);
                self.settle(SwipeToDismissValue::Settled, window, cx);
            } else if !options.gestures_enabled
                || previous.direction != options.direction
                || !options.allows(self.dismiss_direction())
            {
                self.cancel_interaction(window, cx);
            }
            cx.notify();
        }
    }

    pub(super) fn can_start(&self) -> bool {
        self.options.gestures_enabled
            && (self.options.start_to_end || self.options.end_to_start)
            && self.width > px(0.)
            && self.settled == SwipeToDismissValue::Settled
            && !self.moving
    }

    pub(super) fn drag_to(&mut self, delta: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        let position = if self.width > px(0.) {
            (delta / self.width * self.options.sign()).clamp(-1., 1.)
        } else {
            0.
        };
        let direction = SwipeToDismissValue::from_position(position);
        let position = if self.options.allows(direction) {
            position
        } else {
            0.
        };
        let threshold = self
            .threshold
            .as_ref()
            .map_or(DEFAULT_THRESHOLD, |f| f(self.width));
        let threshold = if f32::from(threshold).is_finite() {
            threshold
        } else {
            DEFAULT_THRESHOLD
        };
        let threshold = threshold.clamp(px(0.), self.width);
        let target = if self.width * position.abs() >= threshold {
            SwipeToDismissValue::from_position(position)
        } else {
            SwipeToDismissValue::Settled
        };
        if self.position != position || self.target != target {
            self.position = position;
            self.target = target;
            cx.notify();
            window.refresh();
        }
    }

    pub(super) fn cancel_input(&mut self, window: &mut Window, suppress_mouse_up: bool) -> bool {
        match mem::take(&mut self.input) {
            SwipeInput::Mouse { .. } => {
                window.release_pointer();
                if suppress_mouse_up {
                    self.input = SwipeInput::SuppressMouseUp;
                }
                true
            }
            SwipeInput::Touch => true,
            SwipeInput::SuppressMouseUp if suppress_mouse_up => {
                self.input = SwipeInput::SuppressMouseUp;
                false
            }
            _ => false,
        }
    }

    pub(super) fn cancel_interaction(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let cancelled = self.cancel_input(window, true);
        if cancelled {
            self.move_to(SwipeToDismissValue::Settled, window, cx);
        }
        cancelled
    }

    pub(super) fn release(&mut self, delta: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        self.drag_to(delta, window, cx);
        self.cancel_input(window, false);
        self.move_to(self.target, window, cx);
    }
}
