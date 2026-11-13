//! Material 3 theme values for a host toolkit.
//!
//! The caller owns the theme and supplies current window metrics and system preferences.
#![forbid(unsafe_code)]

mod color;
mod layout;
mod motion;
mod shape;
mod typography;

mod generated {
    use crate::{
        Argb, CornerShape, CubicBezier, PolygonSpec, Spring, TokenValue, TypeStyle, Typeface,
    };

    include!("generated.rs");
}

#[cfg(feature = "dynamic-color")]
pub use color::contrast_ratio_of_tones;
pub use color::{Appearance, Argb, ColorPair, ColorScheme, Elevation, StateLayer};
pub use generated::ColorRole;
pub use layout::{Density, Direction, LayoutPolicy, WidthClass};
pub use motion::{CubicBezier, MotionKind, MotionScheme, MotionSpec, MotionSpeed, Spring};
pub use shape::{CornerRole, CornerShape, PolygonRole, PolygonSpec};
pub use typography::{FontFamilies, TypeRole, TypeStyle, Typeface, Typography, TypographyEmphasis};

/// Application-owned configuration consumed by the Material rendering layer.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialTheme {
    pub colors: ColorScheme,
    pub typography: Typography,
    pub motion: MotionScheme,
    pub layout: LayoutPolicy,
}

impl MaterialTheme {
    /// Creates the baseline theme for an explicit appearance.
    pub fn baseline(appearance: Appearance) -> Self {
        Self {
            colors: ColorScheme::baseline(appearance),
            ..Self::default()
        }
    }

    /// Creates a Tonal Spot theme from an opaque sRGB seed. The seed's alpha is ignored.
    #[cfg(feature = "dynamic-color")]
    pub fn from_seed(seed: Argb, appearance: Appearance, contrast: f64) -> Self {
        Self {
            colors: ColorScheme::from_seed(seed, appearance, contrast),
            ..Self::default()
        }
    }
}

/// The typed value of a token from the bundled data. Semantic color roles
/// resolve through the current [ColorScheme].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenValue {
    Color(Argb),
    ColorRole(ColorRole),
    Corners(CornerShape),
    Dp(f32),
    DpOffset(f32),
    TypeStyle(TypeStyle),
    Easing(CubicBezier),
    Milliseconds(u16),
    Opacity(f32),
    Polygon(PolygonSpec),
    Sp(f32),
    Spring(Spring),
    Typeface(Typeface),
    Weight(u16),
}

/// Looks up an exact, case-sensitive token name from the bundled Material data.
pub fn token(name: &str) -> Option<TokenValue> {
    let index = generated::TOKENS
        .binary_search_by_key(&name, |(name, _)| *name)
        .ok()?;
    Some(generated::TOKENS[index].1)
}
