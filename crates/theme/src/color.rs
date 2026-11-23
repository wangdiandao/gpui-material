use crate::{ColorRole, generated::*};

/// A packed 8-bit sRGB color in alpha, red, green, blue order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Argb(u32);

impl Argb {
    /// Creates an [`Argb`] from a packed `0xAARRGGBB` integer.
    pub const fn from_u32(argb: u32) -> Self {
        Self(argb)
    }

    /// Creates an opaque (`alpha = 255`) sRGB color.
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(u32::from_be_bytes([255, red, green, blue]))
    }

    /// Creates an sRGB color from explicit red, green, blue, and alpha channels.
    pub const fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(u32::from_be_bytes([alpha, red, green, blue]))
    }

    /// Returns a copy of this color with its alpha channel replaced.
    pub const fn with_alpha(self, alpha: u8) -> Self {
        let [_, red, green, blue] = self.channels();
        Self::from_rgba(red, green, blue, alpha)
    }

    /// Returns the packed `0xAARRGGBB` representation.
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Returns the color channels in `[alpha, red, green, blue]` order.
    pub const fn channels(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Returns the 8-bit alpha channel.
    pub const fn alpha(self) -> u8 {
        self.channels()[0]
    }

    /// Returns the 8-bit red channel.
    pub const fn red(self) -> u8 {
        self.channels()[1]
    }

    /// Returns the 8-bit green channel.
    pub const fn green(self) -> u8 {
        self.channels()[2]
    }

    /// Returns the 8-bit blue channel.
    pub const fn blue(self) -> u8 {
        self.channels()[3]
    }
}

impl ColorRole {
    /// Returns the standard Material foreground role paired with this semantic
    /// background role, or `None` when this role is not a paired background.
    pub const fn on_role(self) -> Option<Self> {
        use ColorRole::*;

        Some(match self {
            Background => OnBackground,
            Error => OnError,
            ErrorContainer => OnErrorContainer,
            Info => OnInfo,
            InfoContainer => OnInfoContainer,
            InverseSurface => InverseOnSurface,
            Primary => OnPrimary,
            PrimaryContainer => OnPrimaryContainer,
            PrimaryFixed | PrimaryFixedDim => OnPrimaryFixed,
            Secondary => OnSecondary,
            SecondaryContainer => OnSecondaryContainer,
            SecondaryFixed | SecondaryFixedDim => OnSecondaryFixed,
            Success => OnSuccess,
            SuccessContainer => OnSuccessContainer,
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
            Warning => OnWarning,
            WarningContainer => OnWarningContainer,
            _ => return None,
        })
    }
}

/// The selected appearance. The host resolves any system preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Appearance {
    /// Light mode (default).
    #[default]
    Light,
    /// Dark mode.
    Dark,
}

/// Resolved foreground and background colors for a Material pairing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPair {
    /// Foreground content color (text, icons, indicators).
    pub foreground: Argb,
    /// Background container or surface color.
    pub background: Argb,
}

/// All semantic color roles for one appearance, with per-role overrides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorScheme {
    appearance: Appearance,
    colors: [Argb; ColorRole::ALL.len()],
}

#[cfg(feature = "serde")]
impl serde::Serialize for ColorScheme {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(serde::Serialize)]
        #[serde(rename = "ColorScheme")]
        struct SerializedColorScheme {
            appearance: Appearance,
            overrides: std::collections::BTreeMap<ColorRole, Argb>,
        }

        let mut overrides = std::collections::BTreeMap::new();
        for role in ColorRole::ALL {
            let color = self.colors[role as usize];
            if color != self.baseline_color(role) {
                overrides.insert(role, color);
            }
        }
        SerializedColorScheme {
            appearance: self.appearance,
            overrides,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ColorScheme {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(rename = "ColorScheme")]
        struct SerializedColorScheme {
            appearance: Appearance,
            // Keep the serialized form keyed by role so additions can use baseline defaults.
            #[serde(default)]
            overrides: std::collections::BTreeMap<ColorRole, Argb>,
        }

        let raw = SerializedColorScheme::deserialize(deserializer)?;
        let mut scheme = Self::baseline(raw.appearance);
        for (role, color) in raw.overrides {
            scheme.set_color(role, color);
        }
        Ok(scheme)
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::baseline(Appearance::Light)
    }
}

impl ColorScheme {
    /// Returns the baseline Material 3 color scheme for the given [`Appearance`].
    pub const fn baseline(appearance: Appearance) -> Self {
        Self {
            appearance,
            colors: match appearance {
                Appearance::Light => LIGHT,
                Appearance::Dark => DARK,
            },
        }
    }

    /// Returns the [`Appearance`] (light or dark) associated with this scheme.
    pub const fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Resolves a semantic [`ColorRole`] to its packed [`Argb`] value.
    pub const fn color(&self, role: ColorRole) -> Argb {
        self.colors[role as usize]
    }

    /// Overrides one role. Callers overriding a background should also check its foreground.
    pub fn set_color(&mut self, role: ColorRole, color: Argb) {
        self.colors[role as usize] = color;
    }

    /// Fluent counterpart to [`Self::set_color`].
    pub fn with_color(mut self, role: ColorRole, color: Argb) -> Self {
        self.set_color(role, color);
        self
    }

    #[cfg(feature = "serde")]
    const fn baseline_color(&self, role: ColorRole) -> Argb {
        match self.appearance {
            Appearance::Light => LIGHT[role as usize],
            Appearance::Dark => DARK[role as usize],
        }
    }

    /// Resolves the standard foreground for a semantic background.
    pub fn pair(&self, background: ColorRole) -> Option<ColorPair> {
        let foreground = background.on_role()?;
        Some(ColorPair {
            foreground: self.color(foreground),
            background: self.color(background),
        })
    }

    /// Interpolates all resolved sRGB and alpha channels. The target appearance
    /// takes effect at the midpoint. Values outside 0–1 clamp, and NaN uses 0.
    pub fn blend(&self, target: &Self, t: f32) -> Self {
        if t.is_nan() || t <= 0.0 || self == target {
            return self.clone();
        }
        if t >= 1.0 {
            return target.clone();
        }
        let appearance = if t < 0.5 {
            self.appearance
        } else {
            target.appearance
        };
        let colors = std::array::from_fn(|index| {
            let from = self.colors[index].channels();
            let to = target.colors[index].channels();
            let channels = std::array::from_fn(|i| {
                let start = f32::from(from[i]);
                (start + (f32::from(to[i]) - start) * t).round() as u8
            });
            Argb::from_u32(u32::from_be_bytes(channels))
        });
        Self { appearance, colors }
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
        Self::from_seed_with(
            seed,
            appearance,
            contrast,
            crate::DynamicVariant::TonalSpot,
            crate::SpecVersion::Spec2021,
        )
    }
}

/// The six Material elevation levels. The renderer supplies the shadow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Elevation {
    /// Resting elevation (`0 dp`, default).
    #[default]
    Level0,
    /// Low elevation (`1 dp`).
    Level1,
    /// Medium-low elevation (`3 dp`).
    Level2,
    /// Medium elevation (`6 dp`).
    Level3,
    /// Medium-high elevation (`8 dp`).
    Level4,
    /// Highest elevation (`12 dp`).
    Level5,
}

impl Elevation {
    /// Returns the Material elevation height in density-independent pixels (`dp`).
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
    /// Pointer hover state layer.
    Hover,
    /// Keyboard or programmatic focus state layer.
    Focus,
    /// Active press state layer.
    Pressed,
    /// Active drag state layer.
    Dragged,
}

impl StateLayer {
    /// Returns the normalized Material state-layer opacity (`0.0..=1.0`).
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
