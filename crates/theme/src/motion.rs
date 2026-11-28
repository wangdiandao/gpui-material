use crate::generated::*;

/// Material spring parameters for unit mass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Damping ratio (`< 1.0` underdamped/bouncy, `1.0` critically damped).
    pub damping_ratio: f32,
    /// Spring stiffness constant.
    pub stiffness: f32,
}

/// The two control points of a cubic Bézier easing from `(0, 0)` to `(1, 1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// First control point X coordinate in `0.0..=1.0`.
    pub x1: f32,
    /// First control point Y coordinate.
    pub y1: f32,
    /// Second control point X coordinate in `0.0..=1.0`.
    pub x2: f32,
    /// Second control point Y coordinate.
    pub y2: f32,
}

impl CubicBezier {
    /// Constant-rate linear interpolation (`(0.0, 0.0, 1.0, 1.0)`).
    pub const LINEAR: Self = Self {
        x1: 0.0,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };

    /// Standard CSS `ease` curve (`(0.25, 0.1, 0.25, 1.0)`).
    pub const EASE: Self = Self {
        x1: 0.25,
        y1: 0.1,
        x2: 0.25,
        y2: 1.0,
    };

    /// Standard CSS `ease-in` curve (`(0.42, 0.0, 1.0, 1.0)`).
    pub const EASE_IN: Self = Self {
        x1: 0.42,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };

    /// Standard CSS `ease-out` curve (`(0.0, 0.0, 0.58, 1.0)`).
    pub const EASE_OUT: Self = Self {
        x1: 0.0,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };

    /// Standard CSS `ease-in-out` curve (`(0.42, 0.0, 0.58, 1.0)`).
    pub const EASE_IN_OUT: Self = Self {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };
}

/// Resolved animation timing specification for a state transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionSpec {
    /// Complete the transition immediately without interpolation (used for reduced motion).
    Instant,
    /// Physics-based spring transition.
    Spring(Spring),
    /// Fixed-duration cubic Bézier tween.
    Tween {
        /// Tween duration in milliseconds.
        duration_ms: u16,
        /// Easing curve applied across the duration.
        easing: CubicBezier,
    },
}

/// Spatial springs move or reshape an element. Effects springs drive
/// properties such as color and opacity and use critical damping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionKind {
    /// Motion affecting position, size, or shape.
    Spatial,
    /// Motion affecting color, opacity, or elevation effects without overshoot.
    Effects,
}

/// Relative speed tier for a Material motion token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionSpeed {
    /// Quick transitions for small controls and immediate feedback.
    Fast,
    /// Balanced transitions for standard component state changes.
    Default,
    /// Slower transitions for large surfaces, sheets, or full-screen changes.
    Slow,
}

/// Standard and Expressive schemes from the fixed Material 3 reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MotionScheme {
    /// Utility-focused motion with subtle or no overshoot (default).
    #[default]
    Standard,
    /// Playful, energetic Material 3 Expressive spring physics.
    Expressive,
}

impl MotionScheme {
    /// Resolves the [`MotionSpec`] for the given kind and speed, collapsing to
    /// [`MotionSpec::Instant`] when `reduced_motion` is `true`.
    pub const fn spec(
        self,
        kind: MotionKind,
        speed: MotionSpeed,
        reduced_motion: bool,
    ) -> MotionSpec {
        use MotionKind::{Effects, Spatial};
        use MotionSpeed::{Default, Fast, Slow};

        if reduced_motion {
            return MotionSpec::Instant;
        }

        let spring = match (self, kind, speed) {
            (Self::Standard, Spatial, Fast) => MD_SYS_MOTION_STANDARD_SPRING_FAST_SPATIAL,
            (Self::Standard, Spatial, Default) => MD_SYS_MOTION_STANDARD_SPRING_DEFAULT_SPATIAL,
            (Self::Standard, Spatial, Slow) => MD_SYS_MOTION_STANDARD_SPRING_SLOW_SPATIAL,
            (Self::Expressive, Spatial, Fast) => MD_SYS_MOTION_EXPRESSIVE_SPRING_FAST_SPATIAL,
            (Self::Expressive, Spatial, Default) => MD_SYS_MOTION_EXPRESSIVE_SPRING_DEFAULT_SPATIAL,
            (Self::Expressive, Spatial, Slow) => MD_SYS_MOTION_EXPRESSIVE_SPRING_SLOW_SPATIAL,
            (Self::Standard, Effects, Fast) => MD_SYS_MOTION_STANDARD_SPRING_FAST_EFFECTS,
            (Self::Standard, Effects, Default) => MD_SYS_MOTION_STANDARD_SPRING_DEFAULT_EFFECTS,
            (Self::Standard, Effects, Slow) => MD_SYS_MOTION_STANDARD_SPRING_SLOW_EFFECTS,
            (Self::Expressive, Effects, Fast) => MD_SYS_MOTION_EXPRESSIVE_SPRING_FAST_EFFECTS,
            (Self::Expressive, Effects, Default) => MD_SYS_MOTION_EXPRESSIVE_SPRING_DEFAULT_EFFECTS,
            (Self::Expressive, Effects, Slow) => MD_SYS_MOTION_EXPRESSIVE_SPRING_SLOW_EFFECTS,
        };
        MotionSpec::Spring(spring)
    }
}
