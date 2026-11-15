use std::time::Duration;

use gpui::{App, Window};
use gpui_base::motion::{Easing, Spring, Transition, TransitionId, spring, transition};
use gpui_material_theme::MotionSpec;

/// Samples a stable motion channel during rendering. Base preserves velocity
/// for springs, retargets from the current value, and handles system reduced motion.
pub fn animate(
    id: impl Into<TransitionId>,
    target: f32,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let id = id.into();
    // Instant updates both keyed state types so either policy can resume at the new value.
    let instant = || Transition::new(Duration::ZERO);
    match spec {
        MotionSpec::Instant => {
            transition(id.clone(), target, instant(), window, cx);
            spring(id, target, Spring::new(Duration::ZERO), window, cx)
        }
        MotionSpec::Spring(spec) => {
            let policy = spring_response(spec.stiffness)
                .and_then(|response| {
                    Spring::new(response)
                        .try_with_damping(spec.damping_ratio)
                        .ok()
                })
                .unwrap_or_else(|| Spring::new(Duration::ZERO));
            let value = spring(id.clone(), target, policy, window, cx);
            transition(id, value, instant(), window, cx);
            value
        }
        MotionSpec::Tween {
            duration_ms,
            easing,
        } => {
            let policy = Easing::cubic_bezier(easing.x1, easing.y1, easing.x2, easing.y2)
                .map(|easing| {
                    Transition::new(Duration::from_millis(u64::from(duration_ms))).easing(easing)
                })
                .unwrap_or_else(|_| instant());
            let value = transition(id.clone(), target, policy, window, cx);
            spring(id, value, Spring::new(Duration::ZERO), window, cx);
            value
        }
    }
}

fn spring_response(stiffness: f32) -> Option<Duration> {
    (stiffness.is_finite() && stiffness > 0.)
        .then(|| std::f32::consts::TAU / stiffness.sqrt())
        .and_then(|seconds| Duration::try_from_secs_f32(seconds).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_response_rejects_invalid_and_overflowing_durations() {
        for stiffness in [0., -1., f32::NAN, f32::INFINITY, f32::from_bits(1)] {
            assert_eq!(spring_response(stiffness), None);
        }
        assert_eq!(
            spring_response(400.),
            Some(Duration::from_secs_f32(std::f32::consts::TAU / 20.))
        );
    }
}
