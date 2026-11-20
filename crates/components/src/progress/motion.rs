use gpui_base::motion::Easing;
use gpui_material_theme::{TokenValue, token};
use web_time::Instant;

pub(super) struct ProgressMotion {
    pub(super) last_measured: f32,
    pub(super) arc_origin: f32,
    last_time: Option<Instant>,
    loop_running: bool,
    loop_ms: f64,
    wave_hz: f64,
    wave_phase: f64,
    pub(super) visible: bool,
}

impl Default for ProgressMotion {
    fn default() -> Self {
        Self {
            last_measured: 0.,
            arc_origin: 0.,
            last_time: None,
            loop_running: false,
            loop_ms: 0.,
            wave_hz: 0.,
            wave_phase: 0.,
            visible: true,
        }
    }
}

impl ProgressMotion {
    pub(super) fn new(last_measured: f32) -> Self {
        Self {
            last_measured,
            ..Self::default()
        }
    }

    pub(super) fn set_visible(&mut self, visible: bool, now: Instant) {
        if self.visible != visible {
            self.visible = visible;
            if !visible {
                self.loop_running = false;
                self.wave_hz = 0.;
                self.last_time = Some(now);
            } else {
                // Re-anchor timestamp on resume so off-screen idle duration is skipped.
                self.last_time = Some(now);
            }
        }
    }

    pub(super) fn sample(
        &mut self,
        now: Instant,
        animate_loop: bool,
        wave_hz: f64,
        reduced: bool,
        settled_loop: bool,
    ) -> (f32, f32) {
        if reduced {
            self.last_time = None;
            self.loop_ms = 0.;
            self.wave_phase = 0.;
        }
        let elapsed = self
            .last_time
            .replace(now)
            .map_or(0., |last| now.saturating_duration_since(last).as_secs_f64());
        if animate_loop && self.loop_running {
            self.loop_ms += elapsed * 1000.;
        }
        if settled_loop {
            // 42 seconds is a multiple of both the linear and circular cycles.
            self.loop_ms %= 42_000.;
        }
        if wave_hz > 0. && self.wave_hz > 0. {
            self.wave_phase = (self.wave_phase + elapsed * self.wave_hz).fract();
        }
        self.loop_running = animate_loop;
        self.wave_hz = wave_hz;
        (self.loop_ms as f32, self.wave_phase as f32)
    }
}

// The keyframe timings match ProgressIndicator.kt in Material 3 1.5.0-rc01.
pub(super) fn linear_indeterminate(elapsed_ms: f32) -> [(f32, f32); 2] {
    let clock = elapsed_ms % 1750.;
    let sample = |delay, duration| {
        easing_sample(
            "md.sys.motion.easing-emphasized-accelerate",
            (clock - delay) / duration,
        )
    };
    [
        (sample(250., 1000.), sample(0., 1000.)),
        (sample(900., 850.), sample(650., 850.)),
    ]
}

pub(super) fn circular_indeterminate(elapsed_ms: f32) -> (f32, f32) {
    let clock = elapsed_ms % 6000.;
    let step = (clock / 1500.).floor();
    let turn = easing_sample(
        "md.sys.motion.easing-emphasized-decelerate",
        (clock % 1500.) / 300.,
    );
    let phase = clock / 3000.;
    let length = if phase < 1. {
        0.1 + (0.87 - 0.1) * phase
    } else {
        0.87 - (0.87 - 0.1) * easing_sample("md.sys.motion.easing-standard", phase - 1.)
    };
    // Keep the angle unwrapped during a mode change, then paint it periodically.
    (
        (elapsed_ms / 6000.) * 3. + (elapsed_ms / 6000.).floor() + (step + turn) / 4.,
        length,
    )
}

pub(super) fn easing_sample(name: &str, progress: f32) -> f32 {
    let progress = progress.clamp(0., 1.);
    let TokenValue::Easing(curve) = token(name).expect("bundled progress easing") else {
        unreachable!("progress easing token must contain a cubic Bézier");
    };
    let easing =
        Easing::cubic_bezier(curve.x1, curve.y1, curve.x2, curve.y2).expect("valid Material curve");
    easing.sample(progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indeterminate_keyframes_use_both_linear_segments_and_periodic_rotation() {
        assert_eq!(linear_indeterminate(0.), [(0., 0.); 2]);
        let [(start, end), (second_start, second_end)] = linear_indeterminate(800.);
        assert!(0. < start && start < end && end < 1.);
        assert_eq!(second_start, 0.);
        assert!(second_end > 0.);
        assert_eq!(linear_indeterminate(2550.), linear_indeterminate(800.));
        assert_eq!(circular_indeterminate(0.), (0., 0.1));
        assert_eq!(circular_indeterminate(6000.), (4., 0.1));
        assert_eq!(circular_indeterminate(3000.).1, 0.87);
    }

    #[test]
    fn progress_motion_holds_phase_while_paused_and_resumes_without_elapsed_idle_time() {
        use std::time::Duration;
        let now = Instant::now();
        let at = |millis| now + Duration::from_millis(millis);
        let mut motion = ProgressMotion::default();
        assert_eq!(motion.sample(at(0), true, 2., false, true), (0., 0.));
        assert_eq!(motion.sample(at(125), true, 2., false, true), (125., 0.25));
        assert_eq!(
            motion.sample(at(125), false, 0., false, false),
            (125., 0.25)
        );
        assert_eq!(motion.sample(at(1000), true, 1., false, true), (125., 0.25));
        assert_eq!(motion.sample(at(1250), true, 1., false, true), (375., 0.5));
        assert_eq!(motion.sample(at(1300), false, 0., true, false), (0., 0.));
    }
}
