use crate::{Direction, generated::*};

/// Logical rounded corners in top-start, top-end, bottom-end, bottom-start order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerShape {
    /// Explicit logical corner radii `[top_start, top_end, bottom_end, bottom_start]` in `dp`.
    Radii([f32; 4]),
    /// Pill / stadium rounding using half of the shorter side.
    Full,
}

impl CornerShape {
    /// Creates a rounded rectangle with the same logical radius on all four corners.
    pub const fn uniform(radius: f32) -> Self {
        Self::Radii([radius; 4])
    }

    /// Rounds the top-start and top-end logical corners.
    pub const fn top(radius: f32) -> Self {
        Self::Radii([radius, radius, 0.0, 0.0])
    }

    /// Rounds the bottom-start and bottom-end logical corners.
    pub const fn bottom(radius: f32) -> Self {
        Self::Radii([0.0, 0.0, radius, radius])
    }

    /// Rounds the top-start and bottom-start logical corners.
    pub const fn start(radius: f32) -> Self {
        Self::Radii([radius, 0.0, 0.0, radius])
    }

    /// Rounds the top-end and bottom-end logical corners.
    pub const fn end(radius: f32) -> Self {
        Self::Radii([0.0, radius, radius, 0.0])
    }

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

/// Semantic Material 3 corner-radius scale tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerRole {
    /// Sharp corners (`0 dp`).
    None,
    /// Extra-small uniform rounding (`4 dp`).
    ExtraSmall,
    /// Extra-small rounding on the top corners only (`4 dp` top, `0 dp` bottom).
    ExtraSmallTop,
    /// Small uniform rounding (`8 dp`).
    Small,
    /// Medium uniform rounding (`12 dp`).
    Medium,
    /// Large uniform rounding (`16 dp`).
    Large,
    /// Large rounding on the logical start corners only.
    LargeStart,
    /// Large rounding on the logical end corners only.
    LargeEnd,
    /// Large rounding on the top corners only.
    LargeTop,
    /// Increased large uniform rounding (`20 dp`).
    LargeIncreased,
    /// Extra-large uniform rounding (`28 dp`).
    ExtraLarge,
    /// Extra-large rounding on the top corners only.
    ExtraLargeTop,
    /// Increased extra-large uniform rounding (`32 dp`).
    ExtraLargeIncreased,
    /// Extra-extra-large uniform rounding (`48 dp`).
    ExtraExtraLarge,
    /// Full pill/circle rounding.
    Full,
}

impl CornerRole {
    /// Resolves this corner role to its Material [`CornerShape`] token value.
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
    /// The final circle radius, as in AndroidX `RoundedPolygon.circle`.
    Circle {
        /// Number of polygon vertices approximating the circle (`>= 3`).
        vertices: u8,
        /// Target circle radius in source units.
        radius: f32,
    },
    /// A rounded rectangle in source coordinates.
    Rectangle {
        /// Rectangle width in source units.
        width: f32,
        /// Rectangle height in source units.
        height: f32,
        /// Corner rounding radius in source units.
        rounding_radius: f32,
        /// Corner smoothing factor in `0.0..=1.0`.
        smoothing: f32,
    },
    /// The circumradius of the unrounded regular polygon.
    Regular {
        /// Number of polygon vertices (`>= 3`).
        vertices: u8,
        /// Circumradius of the unrounded polygon.
        radius: f32,
        /// Corner rounding radius in source units.
        rounding_radius: f32,
        /// Corner smoothing factor in `0.0..=1.0`.
        smoothing: f32,
        /// Rotation angle in degrees.
        rotation_degrees: f32,
    },
}

/// Basic semantic polygon tokens bundled in `tokens/material.json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonRole {
    /// Circular polygon token.
    Circle,
    /// Rounded square polygon token.
    Square,
    /// Rounded triangle polygon token.
    Triangle,
}

impl PolygonRole {
    /// Resolves this polygon role to its source [`PolygonSpec`].
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
    /// Circle shape (`circle`).
    Circle,
    /// Rounded square shape (`square`).
    Square,
    /// Slanted rounded square shape (`slanted`).
    Slanted,
    /// Arch shape (`arch`).
    Arch,
    /// Quarter-circle fan shape (`fan`).
    Fan,
    /// Rounded arrow / chevron shape (`arrow`).
    Arrow,
    /// Semi-circle shape (`semi-circle`).
    SemiCircle,
    /// Oval / ellipse shape (`oval`).
    Oval,
    /// Stadium / pill shape (`pill`).
    Pill,
    /// Rounded triangle shape (`triangle`).
    Triangle,
    /// Rounded diamond shape (`diamond`).
    Diamond,
    /// Clam-shell shape (`clam-shell`).
    ClamShell,
    /// Rounded pentagon shape (`pentagon`).
    Pentagon,
    /// Six-sided gem shape (`gem`).
    Gem,
    /// Eight-pointed sunny shape (`sunny`).
    Sunny,
    /// Twelve-pointed very-sunny shape (`very-sunny`).
    VerySunny,
    /// Four-sided scalloped cookie shape (`cookie-4-sided`).
    Cookie4Sided,
    /// Six-sided scalloped cookie shape (`cookie-6-sided`).
    Cookie6Sided,
    /// Seven-sided scalloped cookie shape (`cookie-7-sided`).
    Cookie7Sided,
    /// Nine-sided scalloped cookie shape (`cookie-9-sided`).
    Cookie9Sided,
    /// Twelve-sided scalloped cookie shape (`cookie-12-sided`).
    Cookie12Sided,
    /// Ghost-like organic shape (`ghostish`).
    Ghostish,
    /// Four-leaf clover shape (`clover-4-leaf`).
    Clover4Leaf,
    /// Eight-leaf clover shape (`clover-8-leaf`).
    Clover8Leaf,
    /// Sharp starburst shape (`burst`).
    Burst,
    /// Softened starburst shape (`soft-burst`).
    SoftBurst,
    /// Comic-style boom shape (`boom`).
    Boom,
    /// Softened boom shape (`soft-boom`).
    SoftBoom,
    /// Scalloped flower shape (`flower`).
    Flower,
    /// Puffy cloud-like shape (`puffy`).
    Puffy,
    /// Puffy diamond shape (`puffy-diamond`).
    PuffyDiamond,
    /// Stepped pixel-art circle shape (`pixel-circle`).
    PixelCircle,
    /// Stepped pixel-art triangle shape (`pixel-triangle`).
    PixelTriangle,
    /// Stacked bun shape (`bun`).
    Bun,
    /// Rounded heart shape (`heart`).
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
