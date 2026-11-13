use crate::{ColorRole, generated::*};

/// A packed 8-bit sRGB color in alpha, red, green, blue order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Argb(u32);

impl Argb {
    pub const fn from_u32(argb: u32) -> Self {
        Self(argb)
    }

    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(u32::from_be_bytes([255, red, green, blue]))
    }

    pub const fn to_u32(self) -> u32 {
        self.0
    }

    pub const fn channels(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

/// The selected appearance. The host resolves any system preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Appearance {
    #[default]
    Light,
    Dark,
}

/// Resolved foreground and background colors for a Material pairing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPair {
    pub foreground: Argb,
    pub background: Argb,
}

/// All semantic color roles for one appearance, with per-role overrides.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorScheme {
    appearance: Appearance,
    // Keep the serialized form keyed by role so additions can use baseline defaults.
    #[cfg_attr(feature = "serde", serde(default))]
    overrides: std::collections::BTreeMap<ColorRole, Argb>,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::baseline(Appearance::Light)
    }
}

impl ColorScheme {
    pub fn baseline(appearance: Appearance) -> Self {
        Self {
            appearance,
            overrides: Default::default(),
        }
    }

    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    pub fn color(&self, role: ColorRole) -> Argb {
        self.overrides
            .get(&role)
            .copied()
            .unwrap_or_else(|| self.baseline_color(role))
    }

    /// Overrides one role. Callers overriding a background should also check its foreground.
    pub fn set_color(&mut self, role: ColorRole, color: Argb) {
        if color == self.baseline_color(role) {
            self.overrides.remove(&role);
        } else {
            self.overrides.insert(role, color);
        }
    }

    fn baseline_color(&self, role: ColorRole) -> Argb {
        match self.appearance {
            Appearance::Light => LIGHT[role as usize],
            Appearance::Dark => DARK[role as usize],
        }
    }

    /// Resolves the standard foreground for a semantic background.
    pub fn pair(&self, background: ColorRole) -> Option<ColorPair> {
        use ColorRole::*;

        let foreground = match background {
            Background => OnBackground,
            Error => OnError,
            ErrorContainer => OnErrorContainer,
            InverseSurface => InverseOnSurface,
            Primary => OnPrimary,
            PrimaryContainer => OnPrimaryContainer,
            PrimaryFixed | PrimaryFixedDim => OnPrimaryFixed,
            Secondary => OnSecondary,
            SecondaryContainer => OnSecondaryContainer,
            SecondaryFixed | SecondaryFixedDim => OnSecondaryFixed,
            Surface
            | SurfaceBright
            | SurfaceDim
            | SurfaceContainerLowest
            | SurfaceContainerLow
            | SurfaceContainer
            | SurfaceContainerHigh
            | SurfaceContainerHighest => OnSurface,
            SurfaceVariant => OnSurfaceVariant,
            Tertiary => OnTertiary,
            TertiaryContainer => OnTertiaryContainer,
            TertiaryFixed | TertiaryFixedDim => OnTertiaryFixed,
            _ => return None,
        };
        Some(ColorPair {
            foreground: self.color(foreground),
            background: self.color(background),
        })
    }

    /// Applies the Compose tonal-elevation tint to the surface role.
    pub fn surface_at_elevation(&self, elevation: Elevation) -> Argb {
        let surface = self.color(ColorRole::Surface);
        let dp = elevation.dp();
        if dp == 0.0 {
            return surface;
        }

        let overlay_alpha = (4.5 * dp.ln_1p() + 2.0) / 100.0;
        let [alpha, r, g, b] = surface.channels();
        let [_, tint_r, tint_g, tint_b] = self.color(ColorRole::SurfaceTint).channels();
        let under_alpha = f32::from(alpha) / 255.0 * (1.0 - overlay_alpha);
        let alpha = overlay_alpha + under_alpha;
        let composite = |under, over| {
            ((f32::from(over) * overlay_alpha + f32::from(under) * under_alpha) / alpha).round()
                as u8
        };
        Argb::from_u32(u32::from_be_bytes([
            (alpha * 255.0).round() as u8,
            composite(r, tint_r),
            composite(g, tint_g),
            composite(b, tint_b),
        ]))
    }

    /// Generates Tonal Spot roles using MCU Spec2021. Contrast is clamped to
    /// -1 through 1. NaN selects the default contrast (0).
    #[cfg(feature = "dynamic-color")]
    pub fn from_seed(seed: Argb, appearance: Appearance, contrast: f64) -> Self {
        use material_color_utils::{
            dynamic::color_spec::SpecVersion, scheme::SchemeTonalSpot, utils::color_utils,
        };

        let seed = color_utils::Argb(seed.to_u32() | 0xff00_0000);
        let contrast = if contrast.is_nan() {
            0.0
        } else {
            contrast.clamp(-1.0, 1.0)
        };
        let scheme = SchemeTonalSpot::builder(seed, appearance == Appearance::Dark, contrast)
            .spec_version(SpecVersion::Spec2021)
            .build();
        let mut colors = Self::baseline(appearance);
        for (role, color) in ColorRole::ALL.into_iter().zip(dynamic_colors(&scheme)) {
            colors.set_color(role, color);
        }
        colors
    }
}

/// The six Material elevation levels. The renderer supplies the shadow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Elevation {
    #[default]
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl Elevation {
    pub const fn dp(self) -> f32 {
        match self {
            Self::Level0 => MD_SYS_ELEVATION_LEVEL_0,
            Self::Level1 => MD_SYS_ELEVATION_LEVEL_1,
            Self::Level2 => MD_SYS_ELEVATION_LEVEL_2,
            Self::Level3 => MD_SYS_ELEVATION_LEVEL_3,
            Self::Level4 => MD_SYS_ELEVATION_LEVEL_4,
            Self::Level5 => MD_SYS_ELEVATION_LEVEL_5,
        }
    }
}

/// Material state layer opacities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateLayer {
    Hover,
    Focus,
    Pressed,
    Dragged,
}

impl StateLayer {
    pub const fn opacity(self) -> f32 {
        match self {
            Self::Hover => MD_SYS_STATE_HOVER_STATE_LAYER_OPACITY,
            Self::Focus => MD_SYS_STATE_FOCUS_STATE_LAYER_OPACITY,
            Self::Pressed => MD_SYS_STATE_PRESSED_STATE_LAYER_OPACITY,
            Self::Dragged => MD_SYS_STATE_DRAGGED_STATE_LAYER_OPACITY,
        }
    }
}
/// Returns the contrast ratio for two CIE L* tones, clamped to 0 through 100.
/// Returns `None` if either tone is NaN.
#[cfg(feature = "dynamic-color")]
pub fn contrast_ratio_of_tones(first: f64, second: f64) -> Option<f64> {
    use material_color_utils::contrast::contrast_utils::Contrast;

    if first.is_nan() || second.is_nan() {
        return None;
    }
    Some(Contrast::ratio_of_tones(
        first.clamp(0.0, 100.0),
        second.clamp(0.0, 100.0),
    ))
}
