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

/// Named Material 3 Expressive shapes matching pinned Compose `MaterialShapes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MaterialShape {
    Circle,
    Square,
    Slanted,
    Arch,
    Fan,
    Arrow,
    SemiCircle,
    Oval,
    Pill,
    Triangle,
    Diamond,
    ClamShell,
    Pentagon,
    Gem,
    Sunny,
    VerySunny,
    Cookie4Sided,
    Cookie6Sided,
    Cookie7Sided,
    Cookie9Sided,
    Cookie12Sided,
    Ghostish,
    Clover4Leaf,
    Clover8Leaf,
    Burst,
    SoftBurst,
    Boom,
    SoftBoom,
    Flower,
    Puffy,
    PuffyDiamond,
    PixelCircle,
    PixelTriangle,
    Bun,
    Heart,
}

impl MaterialShape {
    /// All 35 Material 3 Expressive shapes in Compose `MaterialShapes` order.
    pub const ALL: [Self; 35] = [
        Self::Circle,
        Self::Square,
        Self::Slanted,
        Self::Arch,
        Self::Fan,
        Self::Arrow,
        Self::SemiCircle,
        Self::Oval,
        Self::Pill,
        Self::Triangle,
        Self::Diamond,
        Self::ClamShell,
        Self::Pentagon,
        Self::Gem,
        Self::Sunny,
        Self::VerySunny,
        Self::Cookie4Sided,
        Self::Cookie6Sided,
        Self::Cookie7Sided,
        Self::Cookie9Sided,
        Self::Cookie12Sided,
        Self::Ghostish,
        Self::Clover4Leaf,
        Self::Clover8Leaf,
        Self::Burst,
        Self::SoftBurst,
        Self::Boom,
        Self::SoftBoom,
        Self::Flower,
        Self::Puffy,
        Self::PuffyDiamond,
        Self::PixelCircle,
        Self::PixelTriangle,
        Self::Bun,
        Self::Heart,
    ];

    /// Stable lowercase identifier for this shape.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Square => "square",
            Self::Slanted => "slanted",
            Self::Arch => "arch",
            Self::Fan => "fan",
            Self::Arrow => "arrow",
            Self::SemiCircle => "semi-circle",
            Self::Oval => "oval",
            Self::Pill => "pill",
            Self::Triangle => "triangle",
            Self::Diamond => "diamond",
            Self::ClamShell => "clam-shell",
            Self::Pentagon => "pentagon",
            Self::Gem => "gem",
            Self::Sunny => "sunny",
            Self::VerySunny => "very-sunny",
            Self::Cookie4Sided => "cookie-4-sided",
            Self::Cookie6Sided => "cookie-6-sided",
            Self::Cookie7Sided => "cookie-7-sided",
            Self::Cookie9Sided => "cookie-9-sided",
            Self::Cookie12Sided => "cookie-12-sided",
            Self::Ghostish => "ghostish",
            Self::Clover4Leaf => "clover-4-leaf",
            Self::Clover8Leaf => "clover-8-leaf",
            Self::Burst => "burst",
            Self::SoftBurst => "soft-burst",
            Self::Boom => "boom",
            Self::SoftBoom => "soft-boom",
            Self::Flower => "flower",
            Self::Puffy => "puffy",
            Self::PuffyDiamond => "puffy-diamond",
            Self::PixelCircle => "pixel-circle",
            Self::PixelTriangle => "pixel-triangle",
            Self::Bun => "bun",
            Self::Heart => "heart",
        }
    }
}

impl From<PolygonRole> for MaterialShape {
    fn from(role: PolygonRole) -> Self {
        match role {
            PolygonRole::Circle => Self::Circle,
            PolygonRole::Square => Self::Square,
            PolygonRole::Triangle => Self::Triangle,
        }
    }
}
