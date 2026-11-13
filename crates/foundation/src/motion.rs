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
            let policy = (spec.stiffness.is_finite() && spec.stiffness > 0.)
                .then(|| Duration::from_secs_f32(std::f32::consts::TAU / spec.stiffness.sqrt()))
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
