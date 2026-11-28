mod overlay;
#[cfg(test)]
mod tests;

use std::time::Duration;

use gpui::{App, Bounds, ElementId, Pixels, SpringConfig, SpringState, Window, px};
use gpui_base::motion::Easing;
use gpui_material_theme::{CubicBezier, MotionSpec, Spring};
use web_time::Instant;

pub use overlay::{OverlayMotion, ScaledOverlay, sanitize_overlay_scale};

const SPRING_EPSILON: f32 = 0.001;
const TARGET_EPSILON: f32 = 1e-5;

/// Lifecycle status of a Material motion channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionStatus {
    /// Channel is already settled at its target with no motion this frame.
    Idle,
    /// Channel is actively advancing toward its target.
    Running,
    /// Channel reached its target and settled on this sample.
    Finished,
}

impl MotionStatus {
    /// Returns `true` while the animation is actively advancing.
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` when the animation is not advancing (`Idle` or `Finished`).
    pub const fn is_settled(self) -> bool {
        !self.is_running()
    }
}

/// Sample returned by [`animate_with_status`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSample {
    /// Current interpolated scalar value.
    pub value: f32,
    /// Lifecycle status of the underlying animation.
    pub status: MotionStatus,
}

impl MotionSample {
    /// Returns `true` while the animation is actively advancing.
    pub const fn is_running(self) -> bool {
        self.status.is_running()
    }
}

fn bezier_to_easing(easing: CubicBezier) -> Option<Easing> {
    if easing == CubicBezier::LINEAR {
        return Some(Easing::Linear);
    }
    Easing::cubic_bezier(easing.x1, easing.y1, easing.x2, easing.y2).ok()
}

/// Returns whether `bounds` is visible in `window` and intersects the active
/// paint or prepaint content mask.
pub fn is_bounds_visible(bounds: Bounds<Pixels>, window: &Window) -> bool {
    if !window.is_visible() || bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return false;
    }
    let clipped = bounds.intersect(&window.content_mask().bounds);
    clipped.size.width > px(0.) && clipped.size.height > px(0.)
}

/// Unified pause/resume clock for continuous loop animations.
///
/// Holds elapsed phase across inactive, hidden, or off-screen intervals without
/// jumping on resume, and resets to zero when reduced motion is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoopClock {
    epoch: Option<Instant>,
    elapsed: Duration,
}

impl LoopClock {
    /// Advances the clock to `now` when `running`, pauses it when not running, or resets
    /// it to zero when `reduced_motion` is `true`, returning the current elapsed duration.
    pub fn advance(&mut self, now: Instant, running: bool, reduced_motion: bool) -> Duration {
        if reduced_motion {
            *self = Self::default();
            return Duration::ZERO;
        }
        if running {
            self.elapsed + now.saturating_duration_since(*self.epoch.get_or_insert(now))
        } else {
            self.pause(now);
            self.elapsed
        }
    }

    /// Commits elapsed time up to `now` and suspends further advancement until resumed.
    pub fn pause(&mut self, now: Instant) {
        if let Some(epoch) = self.epoch.take() {
            self.elapsed += now.saturating_duration_since(epoch);
        }
    }

    /// Resets the clock back to zero elapsed time and clears its active epoch.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Returns the accumulated paused elapsed duration.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[derive(Clone)]
struct ActiveTween {
    from: f32,
    reversing_adjusted_start: f32,
    started_at: Instant,
    duration: Duration,
    reversing_factor: f32,
    easing: Easing,
}

fn retarget_active_tween(
    prev: &ActiveTween,
    prev_target: f32,
    target: f32,
    base_duration: Duration,
    now: Instant,
) -> (f32, f32, f32, Duration) {
    let elapsed = now.saturating_duration_since(prev.started_at);
    let raw = if prev.duration.is_zero() {
        1.0
    } else {
        (elapsed.as_secs_f32() / prev.duration.as_secs_f32()).clamp(0.0, 1.0)
    };
    let eased = prev.easing.sample(raw);
    let current = prev.from + (prev_target - prev.from) * eased;
    let reversing = raw < 1.0 && (target - prev.reversing_adjusted_start).abs() <= TARGET_EPSILON;
    if reversing {
        let factor = (eased * prev.reversing_factor + (1.0 - prev.reversing_factor))
            .abs()
            .clamp(0.0, 1.0);
        (
            current,
            prev_target,
            factor,
            base_duration.mul_f64(f64::from(factor)),
        )
    } else {
        (current, current, 1.0, base_duration)
    }
}

/// Entity-owned transition supporting mid-flight interruption, proportional
/// reverse-duration shortening, and immediate reduced-motion settlement.
#[derive(Clone)]
pub struct RetargetTransition {
    value: f32,
    target: f32,
    active: Option<ActiveTween>,
}

impl RetargetTransition {
    /// Creates a settled transition already resting at `value` (non-finite values fall back to `0.0`).
    pub fn settled(value: f32) -> Self {
        let value = if value.is_finite() { value } else { 0.0 };
        Self {
            value,
            target: value,
            active: None,
        }
    }

    /// Returns the last sampled scalar value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns the current target scalar value.
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Returns `true` when an interpolation tween is currently in flight.
    pub fn is_animating(&self) -> bool {
        self.active.is_some()
    }

    /// Immediately settles the transition at `value` (keeping current value if non-finite)
    /// and cancels any active tween.
    pub fn snap_to(&mut self, value: f32) {
        if value.is_finite() {
            self.value = value;
            self.target = value;
        } else {
            self.target = self.value;
        }
        self.active = None;
    }

    /// Starts or retargets an eased transition toward `target`.
    ///
    /// Non-finite targets settle immediately at the current value. Reversing mid-flight
    /// proportionally shortens the duration; `reduced_motion`, an invalid cubic Bézier,
    /// or a zero duration snaps immediately to `target`.
    pub fn animate_to(
        &mut self,
        target: f32,
        duration: Duration,
        easing: CubicBezier,
        now: Instant,
        reduced_motion: bool,
    ) {
        let (_, _) = self.sample(now, reduced_motion);
        if !target.is_finite() {
            self.snap_to(self.value);
            return;
        }
        let Some(curve) = bezier_to_easing(easing) else {
            self.snap_to(target);
            return;
        };
        if reduced_motion || duration.is_zero() || (self.value - target).abs() <= TARGET_EPSILON {
            self.snap_to(target);
            return;
        }
        if (self.target - target).abs() <= TARGET_EPSILON && self.active.is_some() {
            return;
        }
        let (current, reversing_adjusted_start, reversing_factor, effective_duration) =
            if let Some(prev) = &self.active {
                retarget_active_tween(prev, self.target, target, duration, now)
            } else {
                (self.value, self.value, 1.0, duration)
            };

        if effective_duration.is_zero() {
            self.snap_to(target);
            return;
        }
        self.value = current;
        self.target = target;
        self.active = Some(ActiveTween {
            from: current,
            reversing_adjusted_start,
            started_at: now,
            duration: effective_duration,
            reversing_factor,
            easing: curve,
        });
    }

    /// Samples the transition at `now`, returning `(current_value, still_animating)`.
    pub fn sample(&mut self, now: Instant, reduced_motion: bool) -> (f32, bool) {
        let Some(active) = &self.active else {
            return (self.value, false);
        };
        if reduced_motion {
            self.value = self.target;
            self.active = None;
            return (self.value, false);
        }
        let elapsed = now.saturating_duration_since(active.started_at);
        if active.duration.is_zero() || elapsed >= active.duration {
            self.value = self.target;
            self.active = None;
            return (self.value, false);
        }
        let raw = (elapsed.as_secs_f32() / active.duration.as_secs_f32()).clamp(0.0, 1.0);
        let eased = active.easing.sample(raw);
        self.value = active.from + (self.target - active.from) * eased;
        (self.value, true)
    }
}

#[derive(Clone)]
enum ChannelKind {
    Settled,
    Spring {
        state: SpringState,
        updated_at: Instant,
    },
    Tween(ActiveTween),
}

#[derive(Clone)]
struct ChannelState {
    value: f32,
    target: f32,
    kind: ChannelKind,
}

impl ChannelState {
    fn settled(target: f32) -> Self {
        let target = if target.is_finite() { target } else { 0.0 };
        Self {
            value: target,
            target,
            kind: ChannelKind::Settled,
        }
    }

    fn settle(&mut self, target: f32) -> MotionSample {
        let settled = if target.is_finite() {
            target
        } else {
            self.value
        };
        self.value = settled;
        self.target = settled;
        self.kind = ChannelKind::Settled;
        MotionSample {
            value: settled,
            status: MotionStatus::Finished,
        }
    }

    fn step(
        &mut self,
        target: f32,
        spec: MotionSpec,
        now: Instant,
        reduced_motion: bool,
    ) -> MotionSample {
        if !target.is_finite() {
            return self.settle(self.value);
        }
        if reduced_motion || matches!(spec, MotionSpec::Instant) {
            return self.settle(target);
        }

        // Common settled fast path: no allocation, no state churn, no frame request.
        if matches!(self.kind, ChannelKind::Settled) && self.target == target {
            return MotionSample {
                value: target,
                status: MotionStatus::Idle,
            };
        }

        match spec {
            MotionSpec::Instant => self.settle(target),
            MotionSpec::Spring(spec) => self.step_spring(target, spec, now),
            MotionSpec::Tween {
                duration_ms,
                easing,
            } => self.step_tween(target, duration_ms, easing, now),
        }
    }

    fn step_spring(&mut self, target: f32, spec: Spring, now: Instant) -> MotionSample {
        let Some(config) = spring_config(spec) else {
            return self.settle(target);
        };

        let (prev_state, prev_target, elapsed) = match &self.kind {
            ChannelKind::Spring { state, updated_at } => (
                *state,
                self.target,
                now.saturating_duration_since(*updated_at).as_secs_f32(),
            ),
            ChannelKind::Tween(tween) => {
                // Smoothly hand off from current tween position without jumping.
                let current = sample_tween_value(tween, self.target, now);
                (
                    SpringState {
                        position: current,
                        velocity: 0.0,
                    },
                    target,
                    0.0,
                )
            }
            ChannelKind::Settled => (
                SpringState {
                    position: self.value,
                    velocity: 0.0,
                },
                target,
                0.0,
            ),
        };

        // Advance over the elapsed frame under the previous target so continuous
        // velocity carries across mid-flight interruptions and reversals.
        let stepped = config.step(prev_state, prev_target, elapsed);
        if config.is_settled(stepped, target, SPRING_EPSILON) {
            return self.settle(target);
        }

        self.value = stepped.position;
        self.target = target;
        self.kind = ChannelKind::Spring {
            state: stepped,
            updated_at: now,
        };
        MotionSample {
            value: stepped.position,
            status: MotionStatus::Running,
        }
    }

    fn step_tween(
        &mut self,
        target: f32,
        duration_ms: u16,
        easing: CubicBezier,
        now: Instant,
    ) -> MotionSample {
        let base_duration = Duration::from_millis(u64::from(duration_ms));
        let Ok(curve) = Easing::cubic_bezier(easing.x1, easing.y1, easing.x2, easing.y2) else {
            return self.settle(target);
        };
        if base_duration.is_zero() {
            return self.settle(target);
        }

        if self.target == target
            && let ChannelKind::Tween(tween) = &self.kind
        {
            let elapsed = now.saturating_duration_since(tween.started_at);
            if tween.duration.is_zero() || elapsed >= tween.duration {
                return self.settle(target);
            }
            let raw = (elapsed.as_secs_f32() / tween.duration.as_secs_f32()).clamp(0.0, 1.0);
            let value = tween.from + (target - tween.from) * tween.easing.sample(raw);
            self.value = value;
            return MotionSample {
                value,
                status: MotionStatus::Running,
            };
        }

        // Retarget or hand off from current value.
        let (current, reversing_adjusted_start, reversing_factor, effective_duration) =
            match &self.kind {
                ChannelKind::Tween(prev) => {
                    retarget_active_tween(prev, self.target, target, base_duration, now)
                }
                ChannelKind::Spring { state, .. } => {
                    (state.position, state.position, 1.0, base_duration)
                }
                ChannelKind::Settled => (self.value, self.value, 1.0, base_duration),
            };

        if effective_duration.is_zero() || (current - target).abs() <= TARGET_EPSILON {
            return self.settle(target);
        }

        self.value = current;
        self.target = target;
        self.kind = ChannelKind::Tween(ActiveTween {
            from: current,
            reversing_adjusted_start,
            started_at: now,
            duration: effective_duration,
            reversing_factor,
            easing: curve,
        });
        MotionSample {
            value: current,
            status: MotionStatus::Running,
        }
    }
}

fn sample_tween_value(tween: &ActiveTween, target: f32, now: Instant) -> f32 {
    let elapsed = now.saturating_duration_since(tween.started_at);
    if tween.duration.is_zero() || elapsed >= tween.duration {
        target
    } else {
        let raw = (elapsed.as_secs_f32() / tween.duration.as_secs_f32()).clamp(0.0, 1.0);
        tween.from + (target - tween.from) * tween.easing.sample(raw)
    }
}

pub(crate) fn spring_config(spec: Spring) -> Option<SpringConfig> {
    if !spec.stiffness.is_finite()
        || spec.stiffness <= 0.
        || !spec.damping_ratio.is_finite()
        || spec.damping_ratio < 0.
    {
        return None;
    }
    let frequency = spec.stiffness.sqrt();
    let response = std::f32::consts::TAU / frequency;
    Duration::try_from_secs_f32(response).ok()?;
    Some(SpringConfig::new(
        spec.stiffness,
        2.0 * spec.damping_ratio * frequency,
        1.0,
    ))
}

/// Samples a unified motion channel and returns both its value and lifecycle status.
///
/// Uses one keyed state entity per channel, preserving velocity across spring
/// retargets, shortening tween reversals proportionally, settling immediately
/// under reduced motion or non-finite targets, and suppressing frame requests
/// while the window is hidden.
pub fn animate_with_status(
    id: impl Into<ElementId>,
    target: f32,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> MotionSample {
    let id = ElementId::NamedChild(id.into().into(), "__material-motion".into());
    let state = window.use_keyed_state(id, cx, |_, _| ChannelState::settled(target));
    let now = cx.background_executor().now();
    let reduced = cx.reduce_motion();

    // Guard non-finite targets to prevent permanent frame loops.
    if !target.is_finite() {
        return state.update(cx, |channel, _| channel.settle(channel.value));
    }

    // Fast path: avoid mutating/notifying the keyed entity when already settled on target.
    {
        let current = state.read(cx);
        if matches!(current.kind, ChannelKind::Settled) && current.target == target {
            return MotionSample {
                value: target,
                status: MotionStatus::Idle,
            };
        }
    }

    let sample = state.update(cx, |channel, _| channel.step(target, spec, now, reduced));
    if sample.is_running() && window.is_visible() {
        window.request_animation_frame();
    }
    sample
}

/// Samples a stable motion channel during rendering.
pub fn animate(
    id: impl Into<ElementId>,
    target: f32,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    animate_with_status(id, target, spec, window, cx).value
}
