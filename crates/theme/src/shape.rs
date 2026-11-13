use crate::{Direction, generated::*};

/// Logical rounded corners in top-start, top-end, bottom-end, bottom-start order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerShape {
    Radii([f32; 4]),
    Full,
}

impl CornerShape {
    /// Returns physical radii in top-left, top-right, bottom-right, bottom-left
    /// order. Scales all corners together when adjacent arcs would overlap.
    pub fn radii(self, width: f32, height: f32, direction: Direction) -> [f32; 4] {
        let width = nonnegative(width);
        let height = nonnegative(height);
        let [tl, tr, br, bl] = match self {
            Self::Full => return [width.min(height) * 0.5; 4],
            Self::Radii(radii) => radii.map(nonnegative),
        };
        let mut factor = 1.0_f32;
        for (edge, a, b) in [
            (width, tl, tr),
            (width, bl, br),
            (height, tl, bl),
            (height, tr, br),
        ] {
            // Use a wider sum so even two finite maximum radii can be scaled.
            let sum = f64::from(a) + f64::from(b);
            if sum > f64::from(edge) {
                factor = factor.min((f64::from(edge) / sum) as f32);
            }
        }
        let physical = match direction {
            Direction::Ltr => [tl, tr, br, bl],
            Direction::Rtl => [tr, tl, bl, br],
        };
        physical.map(|r| r * factor)
    }
}

fn nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerRole {
    None,
    ExtraSmall,
    ExtraSmallTop,
    Small,
    Medium,
    Large,
    LargeStart,
    LargeEnd,
    LargeTop,
    LargeIncreased,
    ExtraLarge,
    ExtraLargeTop,
    ExtraLargeIncreased,
    ExtraExtraLarge,
    Full,
}

impl CornerRole {
    pub const fn shape(self) -> CornerShape {
        match self {
            Self::None => MD_SYS_SHAPE_CORNER_NONE,
            Self::ExtraSmall => MD_SYS_SHAPE_CORNER_EXTRA_SMALL,
            Self::ExtraSmallTop => MD_SYS_SHAPE_CORNER_EXTRA_SMALL_TOP,
            Self::Small => MD_SYS_SHAPE_CORNER_SMALL,
            Self::Medium => MD_SYS_SHAPE_CORNER_MEDIUM,
            Self::Large => MD_SYS_SHAPE_CORNER_LARGE,
            Self::LargeStart => MD_SYS_SHAPE_CORNER_LARGE_START,
            Self::LargeEnd => MD_SYS_SHAPE_CORNER_LARGE_END,
            Self::LargeTop => MD_SYS_SHAPE_CORNER_LARGE_TOP,
            Self::LargeIncreased => MD_SYS_SHAPE_CORNER_LARGE_INCREASED,
            Self::ExtraLarge => MD_SYS_SHAPE_CORNER_EXTRA_LARGE,
            Self::ExtraLargeTop => MD_SYS_SHAPE_CORNER_EXTRA_LARGE_TOP,
            Self::ExtraLargeIncreased => MD_SYS_SHAPE_CORNER_EXTRA_LARGE_INCREASED,
            Self::ExtraExtraLarge => MD_SYS_SHAPE_CORNER_EXTRA_EXTRA_LARGE,
            Self::Full => MD_SYS_SHAPE_CORNER_FULL,
        }
    }
}

/// Inputs for the finite rounded polygons used as morph endpoints. These
/// are source coordinates before path generation and normalization.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolygonSpec {
    /// The final circle radius, as in AndroidX RoundedPolygon.circle.
    Circle { vertices: u8, radius: f32 },
    Rectangle {
        width: f32,
        height: f32,
        rounding_radius: f32,
        smoothing: f32,
    },
    /// The circumradius of the unrounded regular polygon.
    Regular {
        vertices: u8,
        radius: f32,
        rounding_radius: f32,
        smoothing: f32,
        rotation_degrees: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonRole {
    Circle,
    Square,
    Triangle,
}

impl PolygonRole {
    pub const fn spec(self) -> PolygonSpec {
        match self {
            Self::Circle => MD_SYS_SHAPE_POLYGON_CIRCLE,
            Self::Square => MD_SYS_SHAPE_POLYGON_SQUARE,
            Self::Triangle => MD_SYS_SHAPE_POLYGON_TRIANGLE,
        }
    }
}
