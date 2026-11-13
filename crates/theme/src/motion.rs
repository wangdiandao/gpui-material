use crate::generated::*;

/// Material spring parameters for unit mass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping_ratio: f32,
    pub stiffness: f32,
}

/// The two control points of a cubic B?zier easing from (0, 0) to (1, 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionSpec {
    Instant,
    Spring(Spring),
    Tween {
        duration_ms: u16,
        easing: CubicBezier,
    },
}

/// Spatial springs move or reshape an element. Effects springs drive
/// properties such as color and opacity and use critical damping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionKind {
    Spatial,
    Effects,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionSpeed {
    Fast,
    Default,
    Slow,
}

/// Standard and Expressive schemes from the fixed Material 3 reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MotionScheme {
    #[default]
    Standard,
    Expressive,
}

impl MotionScheme {
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
