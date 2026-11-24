use std::time::Duration;

use gpui::{Bounds, point, px, size};
use gpui_base::motion::Easing;
use gpui_material_theme::{CubicBezier, MotionSpec, Spring};
use web_time::Instant;

use super::{
    ChannelState, LoopClock, MotionStatus, OverlayMotion, RetargetTransition, ScaledOverlay,
    sanitize_overlay_scale, spring_config,
};

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

#[test]
fn non_finite_targets_settle_immediately_without_running() {
    let t0 = Instant::now();
    let spring = MotionSpec::Spring(Spring {
        damping_ratio: 0.9,
        stiffness: 700.,
    });

    let mut channel = ChannelState::settled(0.5);
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let sample = channel.step(invalid, spring, t0, false);
        assert_eq!(sample.value, 0.5);
        assert!(!sample.is_running());
    }

    let mut transition = RetargetTransition::settled(0.25);
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        transition.animate_to(
            invalid,
            Duration::from_millis(200),
            Easing::Linear,
            t0,
            false,
        );
        assert!(!transition.is_animating());
        assert_eq!(transition.value(), 0.25);
    }
}

#[test]
fn overlay_motion_and_scaled_overlay_protect_bounds_and_zero_scale() {
    assert!(sanitize_overlay_scale(0.0) > 0.0);
    assert!(sanitize_overlay_scale(-1.0) > 0.0);
    assert!(sanitize_overlay_scale(f32::NAN) > 0.0);
    assert_eq!(sanitize_overlay_scale(0.85), 0.85);

    let zero_motion = OverlayMotion {
        scale: 0.0,
        alpha: 0.5,
    };
    assert!(zero_motion.safe_scale() > 0.0);
    assert!(f32::from(zero_motion.px(100.0)).is_finite());

    let bounds = Bounds::new(point(px(10.), px(20.)), size(px(100.), px(50.)));
    let half_motion = OverlayMotion {
        scale: 0.5,
        alpha: 1.0,
    };
    assert_eq!(
        half_motion.origin(bounds, point(0.5, 0.5)),
        point(px(35.), px(32.5))
    );

    let scaled = ScaledOverlay::new(gpui::div(), zero_motion, |_| point(0.5, 0.5));
    assert!(scaled.safe_scale() > 0.0);
}
