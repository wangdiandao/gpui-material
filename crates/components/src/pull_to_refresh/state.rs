use std::{rc::Rc, time::Duration};

use gpui::{
    App, Context, ElementId, Pixels, ScrollHandle, ScrollSource, Window, WindowId, point, px,
};
use gpui_material_foundation::RetargetTransition;
use gpui_material_theme::CubicBezier;

const DRAG_MULTIPLIER: f32 = 0.5;
const SETTLE: Duration = Duration::from_millis(180);

pub(super) type ScrollPosition = Rc<dyn Fn(&Window, &App) -> Pixels>;

/// Readback supplied to a caller's indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PullToRefreshProgress {
    /// Pull progress relative to the refresh threshold (`0.0` hidden, `1.0` threshold, >`1.0` overshoot).
    pub distance_fraction: f32,
    /// Current downward translation of the indicator in logical pixels.
    pub offset: Pixels,
    /// Whether the caller-owned refreshing state is currently active.
    pub refreshing: bool,
    /// Whether a refresh request has been emitted and is awaiting caller confirmation or rejection.
    pub refresh_pending: bool,
    /// Whether the indicator is currently animating toward its resting or threshold position.
    pub animating: bool,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct ViewportKey {
    pub(super) window: WindowId,
    pub(super) id: ElementId,
}

pub(super) struct PullGesture {
    pub(super) source: ScrollSource,
    pub(super) distance: Pixels,
    pub(super) before_scroll: Pixels,
}

/// Retained gesture and indicator state. The application owns refreshing.
pub struct PullToRefreshState {
    scroll: ScrollHandle,
    pub(super) viewport: Option<ViewportKey>,
    pub(super) gesture: Option<PullGesture>,
    pub(super) fraction: f32,
    motion: RetargetTransition,
    pub(super) refreshing: bool,
    pub(super) pending: bool,
    enabled: bool,
    pub(super) visible: bool,
}

impl Default for PullToRefreshState {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            viewport: None,
            gesture: None,
            fraction: 0.,
            motion: RetargetTransition::settled(0.),
            refreshing: false,
            pending: false,
            enabled: true,
            visible: true,
        }
    }
}

impl PullToRefreshState {
    /// Creates an idle pull-to-refresh state with its own internal scroll handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// The scroll handle used when the container owns its viewport.
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.scroll.clone()
    }

    /// Zero is hidden and one is the refresh threshold. Pull overshoot can exceed one.
    pub fn distance_fraction(&self) -> f32 {
        self.fraction
    }

    /// Returns `true` while the indicator is transitioning between positions.
    pub fn is_animating(&self) -> bool {
        self.motion.is_animating()
    }

    /// Returns `true` when a refresh callback has been triggered and awaits a state update or [`Self::reject_refresh`].
    pub fn is_refresh_pending(&self) -> bool {
        self.pending
    }

    /// Sets the displayed distance immediately and cancels an active gesture.
    pub fn snap_to(&mut self, fraction: f32, cx: &mut Context<Self>) {
        let target = if fraction.is_finite() {
            fraction.max(0.)
        } else {
            0.
        };
        let had_gesture = self.gesture.take().is_some();
        let was_animating = self.motion.is_animating();
        let changed = had_gesture || self.fraction != target || was_animating;
        self.fraction = target;
        self.motion.snap_to(self.fraction);
        if changed {
            cx.notify();
        }
    }

    /// Cancels any active gesture and animates the indicator to the refresh threshold (`1.0`).
    pub fn animate_to_threshold(&mut self, cx: &mut Context<Self>) {
        self.animate_to_fraction(1., cx);
    }

    /// Cancels any active gesture and animates the indicator back to hidden (`0.0`).
    pub fn animate_to_hidden(&mut self, cx: &mut Context<Self>) {
        self.animate_to_fraction(0., cx);
    }

    fn animate_to_fraction(&mut self, target: f32, cx: &mut Context<Self>) {
        let had_gesture = self.gesture.take().is_some();
        let was_animating = self.motion.is_animating();
        let before = self.fraction;
        self.settle_to(target, cx);
        if had_gesture || was_animating || before != self.fraction || self.motion.is_animating() {
            cx.notify();
        }
    }

    /// Clears the current pull and any unanswered request, keeping the controlled busy value.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.pending = false;
        self.snap_to(if self.refreshing { 1. } else { 0. }, cx);
    }

    /// Rearms refresh after the owner declines a pending request.
    pub fn reject_refresh(&mut self, cx: &mut Context<Self>) {
        if self.pending {
            self.pending = false;
            self.animate_to_hidden(cx);
        }
    }

    pub(super) fn can_request(&self) -> bool {
        self.enabled && !self.refreshing && !self.pending
    }

    pub(super) fn configure(
        &mut self,
        viewport: &ViewportKey,
        refreshing: bool,
        enabled: bool,
        window: &mut Window,
        cx: &App,
    ) {
        let retargeted = self.viewport.as_ref() != Some(viewport);
        if retargeted && self.viewport.replace(viewport.clone()).is_some() {
            self.gesture = None;
            self.fraction = f32::from(refreshing || self.pending);
            self.motion.snap_to(self.fraction);
        }
        if refreshing != self.refreshing {
            self.refreshing = refreshing;
            self.pending = false;
            self.gesture = None;
            self.settle_to(f32::from(refreshing), cx);
        } else if self.enabled && !enabled && self.gesture.take().is_some() {
            self.settle_to(0., cx);
        }
        self.enabled = enabled;
        self.sample_motion(window, cx);
    }

    pub(super) fn settle_to(&mut self, target: f32, cx: &App) {
        let now = cx.background_executor().now();
        self.motion.animate_to(
            target,
            SETTLE,
            CubicBezier::EASE_OUT,
            now,
            cx.reduce_motion(),
        );
        self.fraction = self.motion.value();
    }

    fn sample_motion(&mut self, window: &mut Window, cx: &App) {
        if self.motion.is_animating() {
            let now = cx.background_executor().now();
            let (value, running) = self.motion.sample(now, cx.reduce_motion());
            self.fraction = value;
            if running && window.is_visible() && self.visible {
                window.request_animation_frame();
            }
        }
    }

    pub(super) fn set_visible(&mut self, visible: bool, window: &mut Window) {
        let resumed = !self.visible && visible;
        self.visible = visible;
        if resumed && self.motion.is_animating() && window.is_visible() {
            window.request_animation_frame();
        }
    }

    pub(super) fn pull_by(&mut self, amount: Pixels, threshold: Pixels, cx: &mut Context<Self>) {
        if let Some(gesture) = &mut self.gesture {
            let next_distance = (gesture.distance + amount).max(px(0.));
            let next_fraction = pull_fraction(next_distance, threshold);
            if next_distance != gesture.distance
                || next_fraction != self.fraction
                || self.motion.is_animating()
            {
                gesture.distance = next_distance;
                self.fraction = next_fraction;
                self.motion.snap_to(self.fraction);
                cx.notify();
            }
        }
    }
}

pub(super) fn pull_fraction(distance: Pixels, threshold: Pixels) -> f32 {
    if !f32::from(distance).is_finite()
        || !f32::from(threshold).is_finite()
        || threshold <= px(0.)
        || distance <= px(0.)
    {
        return 0.;
    }
    let progress = (distance / threshold) * DRAG_MULTIPLIER;
    let overshoot = (progress - 1.).clamp(0., 2.);
    progress.min(1.) + overshoot - overshoot.powi(2) * 0.25
}

pub(super) fn meets_refresh_threshold(fraction: f32) -> bool {
    fraction.is_finite() && fraction >= 1.
}

#[derive(Clone)]
pub(super) enum ScrollBinding {
    Handle(ScrollHandle),
    Position(ScrollPosition),
}

impl ScrollBinding {
    pub(super) fn distance_from_top(&self, window: &Window, cx: &App) -> Pixels {
        let distance = match self {
            Self::Handle(handle) => -handle.offset().y,
            Self::Position(position) => position(window, cx),
        };
        if f32::from(distance).is_finite() {
            distance.max(px(0.))
        } else {
            px(f32::MAX)
        }
    }

    pub(super) fn remove_overscroll(&self) {
        if let Self::Handle(handle) = self {
            let offset = handle.offset();
            if offset.y > px(0.) {
                handle.set_offset(point(offset.x, px(0.)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_fraction_reaches_threshold_and_dampens_overshoot() {
        assert_eq!(pull_fraction(px(0.), px(80.)), 0.);
        assert_eq!(pull_fraction(px(-10.), px(80.)), 0.);
        assert_eq!(pull_fraction(px(f32::NAN), px(80.)), 0.);
        assert_eq!(pull_fraction(px(80.), px(80.)), 0.5);
        assert_eq!(pull_fraction(px(160.), px(80.)), 1.0);
        assert!(meets_refresh_threshold(pull_fraction(px(160.), px(80.))));
        assert!(!meets_refresh_threshold(pull_fraction(px(159.), px(80.))));
        assert!(!meets_refresh_threshold(f32::NAN));
        assert_eq!(pull_fraction(px(320.), px(80.)), 1.75);
        assert_eq!(pull_fraction(px(640.), px(80.)), 2.0);
    }
}
