use std::time::Duration;

use gpui::{SpringConfig, SpringState};
use gpui_base::motion::Easing;
use gpui_material_theme::{ColorScheme, MotionSpec};
use web_time::Instant;

use crate::motion::{RetargetTransition, spring_config};

enum PaletteCurve {
    Spring {
        config: SpringConfig,
        state: SpringState,
        updated_at: Instant,
    },
    Tween(RetargetTransition),
}

pub(super) struct PaletteTransition {
    pub(super) from: ColorScheme,
    pub(super) to: ColorScheme,
    curve: PaletteCurve,
    pub(super) spec: MotionSpec,
}

impl PaletteTransition {
    pub(super) fn new(
        from: ColorScheme,
        to: ColorScheme,
        spec: MotionSpec,
        now: Instant,
    ) -> Option<Self> {
        let curve = match spec {
            MotionSpec::Instant => return None,
            MotionSpec::Spring(spring) => PaletteCurve::Spring {
                config: spring_config(spring)?,
                state: SpringState {
                    position: 0.0,
                    velocity: 0.0,
                },
                updated_at: now,
            },
            MotionSpec::Tween {
                duration_ms,
                easing,
            } => {
                let duration = Duration::from_millis(u64::from(duration_ms));
                let curve =
                    Easing::cubic_bezier(easing.x1, easing.y1, easing.x2, easing.y2).ok()?;
                if duration.is_zero() {
                    return None;
                }
                let mut transition = RetargetTransition::settled(0.0);
                transition.animate_to(1.0, duration, curve, now, false);
                PaletteCurve::Tween(transition)
            }
        };
        Some(Self {
            from,
            to,
            curve,
            spec,
        })
    }

    pub(super) fn advance(&mut self, now: Instant) -> (f32, bool) {
        match &mut self.curve {
            PaletteCurve::Spring {
                config,
                state,
                updated_at,
            } => {
                let elapsed = now.saturating_duration_since(*updated_at).as_secs_f32();
                let stepped = config.step(*state, 1.0, elapsed);
                *state = stepped;
                *updated_at = now;
                if config.is_settled(stepped, 1.0, 0.001) {
                    (1.0, false)
                } else {
                    (stepped.position.clamp(0.0, 1.0), true)
                }
            }
            PaletteCurve::Tween(transition) => {
                let (value, running) = transition.sample(now, false);
                (value.clamp(0.0, 1.0), running)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_material_theme::{Appearance, CubicBezier, Spring};

    #[test]
    fn palette_transition_instant_returns_none_and_tween_advances_to_completion() {
        let light = ColorScheme::baseline(Appearance::Light);
        let dark = ColorScheme::baseline(Appearance::Dark);
        let t0 = Instant::now();

        assert!(
            PaletteTransition::new(light.clone(), dark.clone(), MotionSpec::Instant, t0).is_none()
        );

        let tween = MotionSpec::Tween {
            duration_ms: 200,
            easing: CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        };
        let mut transition =
            PaletteTransition::new(light.clone(), dark.clone(), tween, t0).unwrap();
        let (mid, running) = transition.advance(t0 + Duration::from_millis(100));
        assert!(running);
        assert!((mid - 0.5).abs() < 1e-3);
        let (end, running) = transition.advance(t0 + Duration::from_millis(200));
        assert!(!running);
        assert_eq!(end, 1.0);

        let spring = MotionSpec::Spring(Spring {
            damping_ratio: 1.0,
            stiffness: 800.0,
        });
        let mut spring_transition = PaletteTransition::new(light, dark, spring, t0).unwrap();
        let (p1, running1) = spring_transition.advance(t0 + Duration::from_millis(16));
        assert!(running1);
        assert!(p1 > 0.0 && p1 < 1.0);
        let (p_end, running_end) = spring_transition.advance(t0 + Duration::from_secs(2));
        assert!(!running_end);
        assert_eq!(p_end, 1.0);
    }
}
