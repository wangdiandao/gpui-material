//! Material 3 theme values for a host toolkit.
//!
//! The caller owns the theme and supplies current window metrics and system preferences.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod color;
#[cfg(feature = "dynamic-color")]
mod dynamic_color;
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
#[cfg(feature = "dynamic-color")]
pub use dynamic_color::{
    CustomColorGroup, DynamicVariant, SpecVersion, ThemePreset, harmonize_color,
};
pub use generated::ColorRole;
pub use layout::{Density, Direction, LayoutPolicy, WidthClass};
pub use motion::{CubicBezier, MotionKind, MotionScheme, MotionSpec, MotionSpeed, Spring};
pub use shape::{CornerRole, CornerShape, MaterialShape, PolygonRole, PolygonSpec};
pub use typography::{FontFamilies, TypeRole, TypeStyle, Typeface, Typography, TypographyEmphasis};

/// Application-owned configuration consumed by the Material rendering layer.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialTheme {
    /// Semantic color roles and appearance for this theme.
    pub colors: ColorScheme,
    /// Font families and type scale configuration.
    pub typography: Typography,
    /// Spring motion scheme (`Standard` or `Expressive`).
    pub motion: MotionScheme,
    /// Desktop density and layout direction policy.
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

    /// Replaces the theme's [`ColorScheme`].
    pub fn with_colors(mut self, colors: ColorScheme) -> Self {
        self.colors = colors;
        self
    }

    /// Replaces the theme's [`Typography`] configuration.
    pub fn with_typography(mut self, typography: Typography) -> Self {
        self.typography = typography;
        self
    }

    /// Replaces the theme's [`MotionScheme`].
    pub fn with_motion(mut self, motion: MotionScheme) -> Self {
        self.motion = motion;
        self
    }

    /// Replaces the theme's [`LayoutPolicy`].
    pub fn with_layout(mut self, layout: LayoutPolicy) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the desktop [`Density`] in [`Self::layout`].
    pub fn with_density(mut self, density: Density) -> Self {
        self.layout.density = density;
        self
    }

    /// Sets the layout [`Direction`] in [`Self::layout`].
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.layout.direction = direction;
        self
    }
}

/// The typed value of a token from the bundled data. Semantic color roles
/// resolve through the current [ColorScheme].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenValue {
    /// A literal packed sRGB color (`md.ref.palette.*` or resolved mode color).
    Color(Argb),
    /// A semantic color role resolved against the active [`ColorScheme`].
    ColorRole(ColorRole),
    /// A corner rounding specification (`CornerShape::Full` or four corner radii).
    Corners(CornerShape),
    /// A non-negative logical dimension in density-independent pixels (`dp`).
    Dp(f32),
    /// A signed logical offset in density-independent pixels (`dp`).
    DpOffset(f32),
    /// A complete Material typographic style.
    TypeStyle(TypeStyle),
    /// A cubic Bézier easing curve.
    Easing(CubicBezier),
    /// A motion duration in milliseconds.
    Milliseconds(u16),
    /// A normalized alpha or state-layer opacity in `0.0..=1.0`.
    Opacity(f32),
    /// A Material 3 Expressive polygon shape specification.
    Polygon(PolygonSpec),
    /// A typographic measurement in scale-independent pixels (`sp`).
    Sp(f32),
    /// A unit-mass spring physics specification.
    Spring(Spring),
    /// A logical typeface slot (`Brand` or `Plain`).
    Typeface(Typeface),
    /// A CSS-style numeric font weight (`1..=1000`).
    Weight(u16),
}

/// Looks up an exact, case-sensitive token name from the bundled Material data.
pub fn token(name: &str) -> Option<TokenValue> {
    let index = generated::TOKENS
        .binary_search_by_key(&name, |(name, _)| *name)
        .ok()?;
    Some(generated::TOKENS[index].1)
}
