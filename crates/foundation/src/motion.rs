use std::time::Duration;

use gpui::{App, Bounds, ElementId, Pixels, SpringConfig, SpringState, Window, px};
pub use gpui_base::motion::MotionStatus;
use gpui_base::motion::{Easing, TransitionId};
use gpui_material_theme::{CubicBezier, MotionSpec, Spring};
use web_time::Instant;

const SPRING_EPSILON: f32 = 0.001;
const TARGET_EPSILON: f32 = 1e-5;

/// Sample returned by [`animate_with_status`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSample {
    /// Current interpolated scalar value.
    pub value: f32,
    /// Lifecycle status of the underlying animation.
    pub status: MotionStatus,
}

impl MotionSample {
    /// Returns `true` while the animation is delayed or actively advancing.
    pub const fn is_running(self) -> bool {
        matches!(self.status, MotionStatus::Delayed | MotionStatus::Running)
    }
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

/// Entity-owned transition supporting mid-flight interruption, proportional
/// reverse-duration shortening, and immediate reduced-motion settlement.
#[derive(Clone)]
pub struct RetargetTransition {
    value: f32,
    target: f32,
    active: Option<ActiveTween>,
}

impl RetargetTransition {
    /// Creates a settled transition already resting at `value`.
    pub fn settled(value: f32) -> Self {
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

    /// Immediately settles the transition at `value` and cancels any active tween.
    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.active = None;
    }

    /// Starts or retargets an eased transition toward `target`.
    ///
    /// Reversing mid-flight proportionally shortens the duration; `reduced_motion` or a
    /// zero duration snaps immediately to `target`.
    pub fn animate_to(
        &mut self,
        target: f32,
        duration: Duration,
        easing: Easing,
        now: Instant,
        reduced_motion: bool,
    ) {
        let (_, _) = self.sample(now, reduced_motion);
        if reduced_motion || duration.is_zero() || (self.value - target).abs() <= TARGET_EPSILON {
            self.snap_to(target);
            return;
        }
        if (self.target - target).abs() <= TARGET_EPSILON && self.active.is_some() {
            return;
        }
        let current = self.value;
        let (reversing_adjusted_start, reversing_factor, effective_duration) =
            if let Some(prev) = &self.active {
                let elapsed = now.saturating_duration_since(prev.started_at);
                let raw = if prev.duration.is_zero() {
                    1.0
                } else {
                    (elapsed.as_secs_f32() / prev.duration.as_secs_f32()).clamp(0.0, 1.0)
                };
                let eased = prev.easing.sample(raw);
                let reversing = (target - prev.reversing_adjusted_start).abs() <= TARGET_EPSILON;
                if reversing {
                    let factor = (eased * prev.reversing_factor + (1.0 - prev.reversing_factor))
                        .abs()
                        .clamp(0.0, 1.0);
                    (self.target, factor, duration.mul_f64(f64::from(factor)))
                } else {
                    (current, 1.0, duration)
                }
            } else {
                (current, 1.0, duration)
            };

        if effective_duration.is_zero() {
            self.snap_to(target);
            return;
        }
        self.target = target;
        self.active = Some(ActiveTween {
            from: current,
            reversing_adjusted_start,
            started_at: now,
            duration: effective_duration,
            reversing_factor,
            easing,
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
        Self {
            value: target,
            target,
            kind: ChannelKind::Settled,
        }
    }

    fn settle(&mut self, target: f32) -> MotionSample {
        self.value = target;
        self.target = target;
        self.kind = ChannelKind::Settled;
        MotionSample {
            value: target,
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
        let (current, reversing_adjusted_start, reversing_factor, effective_duration) = match &self
            .kind
        {
            ChannelKind::Tween(prev) => {
                let elapsed = now.saturating_duration_since(prev.started_at);
                let raw = if prev.duration.is_zero() {
                    1.0
                } else {
                    (elapsed.as_secs_f32() / prev.duration.as_secs_f32()).clamp(0.0, 1.0)
                };
                let eased = prev.easing.sample(raw);
                let current = prev.from + (self.target - prev.from) * eased;
                let reversing =
                    raw < 1.0 && (target - prev.reversing_adjusted_start).abs() <= TARGET_EPSILON;
                if reversing {
                    let factor = (eased * prev.reversing_factor + (1.0 - prev.reversing_factor))
                        .abs()
                        .clamp(0.0, 1.0);
                    (
                        current,
                        self.target,
                        factor,
                        base_duration.mul_f64(f64::from(factor)),
                    )
                } else {
                    (current, current, 1.0, base_duration)
                }
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
/// under reduced motion, and suppressing frame requests while the window is hidden.
pub fn animate_with_status(
    id: impl Into<TransitionId>,
    target: f32,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> MotionSample {
    let id = ElementId::NamedChild(
        ElementId::from(id.into()).into(),
        "__material-motion".into(),
    );
    let state = window.use_keyed_state(id, cx, |_, _| ChannelState::settled(target));
    let now = cx.background_executor().now();
    let reduced = cx.reduce_motion();

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
    id: impl Into<TransitionId>,
    target: f32,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    animate_with_status(id, target, spec, window, cx).value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_config_rejects_invalid_and_overflowing_parameters() {
        for (stiffness, damping) in [
            (0., 1.),
            (-1., 1.),
            (f32::NAN, 1.),
            (f32::INFINITY, 1.),
            (f32::from_bits(1), 1.),
            (400., -0.5),
            (400., f32::NAN),
        ] {
            assert!(
                spring_config(Spring {
                    damping_ratio: damping,
                    stiffness,
                })
                .is_none()
            );
        }
        assert!(
            spring_config(Spring {
                damping_ratio: 0.9,
                stiffness: 400.,
            })
            .is_some()
        );
    }

    #[test]
    fn loop_clock_holds_phase_while_paused_and_resets_under_reduced_motion() {
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        let mut clock = LoopClock::default();

        assert_eq!(clock.advance(at(0), true, false), Duration::ZERO);
        assert_eq!(
            clock.advance(at(120), true, false),
            Duration::from_millis(120)
        );
        // Pause while clipped off-screen or inactive.
        assert_eq!(
            clock.advance(at(120), false, false),
            Duration::from_millis(120)
        );
        // Long idle period does not advance elapsed time.
        assert_eq!(
            clock.advance(at(5_000), true, false),
            Duration::from_millis(120)
        );
        assert_eq!(
            clock.advance(at(5_080), true, false),
            Duration::from_millis(200)
        );
        // Reduced motion resets cleanly.
        assert_eq!(clock.advance(at(5_100), true, true), Duration::ZERO);
        assert_eq!(clock.elapsed(), Duration::ZERO);
    }

    #[test]
    fn retarget_transition_shortens_mid_flight_reversals_and_settles_on_reduced_motion() {
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        let linear = Easing::Linear;
        let mut transition = RetargetTransition::settled(0.0);

        transition.animate_to(
            1.0,
            Duration::from_millis(200),
            linear.clone(),
            at(0),
            false,
        );
        assert!(transition.is_animating());

        // Advance to 25% (50 ms of 200 ms).
        let (v, running) = transition.sample(at(50), false);
        assert!(running);
        assert!((v - 0.25).abs() < 1e-4);

        // Reverse back to 0.0: duration must shorten to 25% (50 ms), not full 200 ms.
        transition.animate_to(
            0.0,
            Duration::from_millis(200),
            linear.clone(),
            at(50),
            false,
        );
        let (mid, running) = transition.sample(at(75), false);
        assert!(running);
        assert!((mid - 0.125).abs() < 1e-3, "mid={mid}");

        let (end, running) = transition.sample(at(100), false);
        assert!(!running);
        assert_eq!(end, 0.0);

        // Mid-flight reduced motion settles immediately to target.
        transition.animate_to(1.0, Duration::from_millis(200), linear, at(100), false);
        let (_, running) = transition.sample(at(120), false);
        assert!(running);
        let (settled, running) = transition.sample(at(130), true);
        assert!(!running);
        assert_eq!(settled, 1.0);
    }

    #[test]
    fn spring_channel_preserves_velocity_across_reversal_and_hands_off_to_tween() {
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        let spring = MotionSpec::Spring(Spring {
            damping_ratio: 0.9,
            stiffness: 700.,
        });
        let tween = MotionSpec::Tween {
            duration_ms: 200,
            easing: CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        };

        let mut channel = ChannelState::settled(0.0);
        let s0 = channel.step(1.0, spring, at(0), false);
        assert_eq!(s0.status, MotionStatus::Running);
        let s1 = channel.step(1.0, spring, at(32), false);
        assert!(s1.value > 0.05 && s1.value < 1.0);

        // Reverse mid-flight: positive velocity carries slightly forward before turning back,
        // never snapping back to 0 or jumping discontinuously.
        let rev0 = channel.step(0.0, spring, at(32), false);
        assert!((rev0.value - s1.value).abs() < 1e-4);
        let rev1 = channel.step(0.0, spring, at(40), false);
        assert!(rev1.status == MotionStatus::Running);

        // Switch mid-flight from spring to tween without jumping.
        let handoff = channel.step(1.0, tween, at(40), false);
        assert!((handoff.value - rev1.value).abs() < 1e-4);
        assert_eq!(handoff.status, MotionStatus::Running);

        // Mid-flight reduced motion snaps cleanly to target and stops running.
        let reduced = channel.step(1.0, tween, at(60), true);
        assert_eq!(reduced.value, 1.0);
        assert_eq!(reduced.status, MotionStatus::Finished);
    }
}
