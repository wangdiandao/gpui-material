use material_color_utils::{
    blend::blend_functions::Blend,
    contrast::contrast_utils::Contrast,
    dynamic::{color_spec::SpecVersion as McuSpec, contrast_curve::ContrastCurve},
    hct::hct_color::Hct,
    palettes::tonal_palette::TonalPalette,
    scheme::{
        SchemeContent, SchemeExpressive, SchemeFidelity, SchemeMonochrome, SchemeNeutral,
        SchemeTonalSpot, SchemeVibrant,
    },
    utils::color_utils::Argb as McuArgb,
};

use crate::{
    Appearance, Argb, ColorPair, ColorRole, ColorScheme, MaterialTheme, generated::dynamic_colors,
};

/// Resolved Material 3 four-role custom color group for one appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomColorGroup {
    /// Accent color for high-emphasis custom fills and outlines.
    pub color: Argb,
    /// Foreground color paired with [`Self::color`].
    pub on_color: Argb,
    /// Tonal container fill derived from the custom seed.
    pub color_container: Argb,
    /// Foreground color paired with [`Self::color_container`].
    pub on_color_container: Argb,
}

impl CustomColorGroup {
    /// Returns the primary `(on_color, color)` foreground/background pair.
    pub const fn pair(self) -> ColorPair {
        ColorPair {
            foreground: self.on_color,
            background: self.color,
        }
    }

    /// Returns the tonal `(on_color_container, color_container)` pair.
    pub const fn container_pair(self) -> ColorPair {
        ColorPair {
            foreground: self.on_color_container,
            background: self.color_container,
        }
    }
}

/// Shifts `design`'s hue toward `source` using Material Color Utilities harmonization.
/// Input alpha channels are ignored and the returned color is opaque.
pub fn harmonize_color(design: Argb, source: Argb) -> Argb {
    let design = McuArgb(design.to_u32() | 0xff00_0000);
    let source = McuArgb(source.to_u32() | 0xff00_0000);
    Argb::from_u32(Blend::harmonize(design, source).0)
}

/// The Material dynamic color palette to derive from a seed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynamicVariant {
    /// Default Material 3 balanced tonal palette with moderate chroma.
    #[default]
    TonalSpot,
    /// High-chroma accent palette with bold secondary and tertiary hues.
    Vibrant,
    /// Detached, playful hue rotation for expressive Material 3 palettes.
    Expressive,
    /// Preserves the seed color closely in the primary palette.
    Fidelity,
    /// Derives accents faithful to source content colors.
    Content,
    /// Achromatic grayscale palette.
    Monochrome,
    /// Low-chroma near-neutral palette.
    Neutral,
}

/// The requested Material Color Utilities specification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecVersion {
    /// Material Color Utilities 2021 specification (default).
    #[default]
    Spec2021,
    /// Material Color Utilities 2025 specification (applies to `TonalSpot`, `Vibrant`, `Expressive`, and `Neutral`).
    Spec2025,
}

impl ColorScheme {
    /// Generates a scheme for an opaque sRGB seed. Contrast clamps to −1–1,
    /// with NaN using 0. Fidelity, Content, and Monochrome resolve with Spec2021.
    pub fn from_seed_with(
        seed: Argb,
        appearance: Appearance,
        contrast: f64,
        variant: DynamicVariant,
        spec_version: SpecVersion,
    ) -> Self {
        let seed = McuArgb(seed.to_u32() | 0xff00_0000);
        let contrast = if contrast.is_nan() {
            0.0
        } else {
            contrast.clamp(-1.0, 1.0)
        };
        let dark = appearance == Appearance::Dark;
        let spec = match (spec_version, variant) {
            (
                SpecVersion::Spec2025,
                DynamicVariant::TonalSpot
                | DynamicVariant::Vibrant
                | DynamicVariant::Expressive
                | DynamicVariant::Neutral,
            ) => McuSpec::Spec2025,
            _ => McuSpec::Spec2021,
        };
        macro_rules! build {
            ($scheme:ident) => {
                $scheme::builder(seed, dark, contrast)
                    .spec_version(spec)
                    .build()
            };
        }
        let scheme = match variant {
            DynamicVariant::TonalSpot => build!(SchemeTonalSpot),
            DynamicVariant::Vibrant => build!(SchemeVibrant),
            DynamicVariant::Expressive => build!(SchemeExpressive),
            DynamicVariant::Fidelity => build!(SchemeFidelity),
            DynamicVariant::Content => build!(SchemeContent),
            DynamicVariant::Monochrome => build!(SchemeMonochrome),
            DynamicVariant::Neutral => build!(SchemeNeutral),
        };
        let mut colors = Self::baseline(appearance);
        for (role, color) in dynamic_colors(&scheme) {
            colors.set_color(role, color);
        }
        let surface_tone = highest_surface_tone(&colors);
        use ColorRole::*;
        for (status_seed, roles) in [
            (0xff00_61a4, [Info, OnInfo, InfoContainer, OnInfoContainer]),
            (
                0xff00_6c4e,
                [Success, OnSuccess, SuccessContainer, OnSuccessContainer],
            ),
            (
                0xff97_4800,
                [Warning, OnWarning, WarningContainer, OnWarningContainer],
            ),
        ] {
            let harmonized = Blend::harmonize(McuArgb(status_seed), seed);
            let group = resolve_custom_group(harmonized, surface_tone, dark, contrast);
            let values = [
                group.color,
                group.on_color,
                group.color_container,
                group.on_color_container,
            ];
            for (role, color) in roles.into_iter().zip(values) {
                colors.set_color(role, color);
            }
        }
        colors
    }

    /// Resolves a Material 3 four-role [`CustomColorGroup`] for `seed` in this
    /// scheme's appearance at default contrast (`0.0`).
    ///
    /// When `harmonize` is `true`, `seed` is shifted toward this scheme's
    /// [`ColorRole::Primary`] role before tone derivation.
    pub fn custom_color(&self, seed: Argb, harmonize: bool) -> CustomColorGroup {
        self.custom_color_with_contrast(seed, 0.0, harmonize)
    }

    /// Resolves a Material 3 four-role [`CustomColorGroup`] for `seed` in this
    /// scheme's appearance and requested contrast (`-1.0..=1.0`, `NaN` -> `0.0`).
    ///
    /// Uses Material 3's error-role contrast curves against this scheme's
    /// highest-emphasis surface tone via [`TonalPalette`].
    pub fn custom_color_with_contrast(
        &self,
        seed: Argb,
        contrast: f64,
        harmonize: bool,
    ) -> CustomColorGroup {
        let target = if harmonize {
            McuArgb(harmonize_color(seed, self.color(ColorRole::Primary)).to_u32())
        } else {
            McuArgb(seed.to_u32() | 0xff00_0000)
        };
        let contrast = if contrast.is_nan() {
            0.0
        } else {
            contrast.clamp(-1.0, 1.0)
        };
        let dark = self.appearance() == Appearance::Dark;
        let surface_tone = highest_surface_tone(self);
        resolve_custom_group(target, surface_tone, dark, contrast)
    }
}

fn highest_surface_tone(scheme: &ColorScheme) -> f64 {
    let role = match scheme.appearance() {
        Appearance::Light => ColorRole::SurfaceDim,
        Appearance::Dark => ColorRole::SurfaceBright,
    };
    Hct::from_argb(McuArgb(scheme.color(role).to_u32() | 0xff00_0000)).tone()
}

fn adjust_tone_for_contrast(
    base: f64,
    background: f64,
    required: f64,
    prefer_lighter: bool,
) -> f64 {
    if Contrast::ratio_of_tones(base, background) >= required {
        return base;
    }
    let adjusted = if prefer_lighter {
        Contrast::lighter(background, required).or_else(|| Contrast::darker(background, required))
    } else {
        Contrast::darker(background, required).or_else(|| Contrast::lighter(background, required))
    };
    adjusted.unwrap_or(if background < 50.0 { 100.0 } else { 0.0 })
}

fn resolve_custom_group(
    target: McuArgb,
    surface_tone: f64,
    dark: bool,
    contrast: f64,
) -> CustomColorGroup {
    let palette = TonalPalette::from_hct(target.into());

    let color_tone = adjust_tone_for_contrast(
        if dark { 80.0 } else { 40.0 },
        surface_tone,
        ContrastCurve::new(3.0, 4.5, 7.0, 7.0).get(contrast),
        dark,
    );
    let on_color_tone = adjust_tone_for_contrast(
        if dark { 20.0 } else { 100.0 },
        color_tone,
        ContrastCurve::new(4.5, 7.0, 11.0, 21.0).get(contrast),
        !dark,
    );
    let container_tone = adjust_tone_for_contrast(
        if dark { 30.0 } else { 90.0 },
        surface_tone,
        ContrastCurve::new(1.0, 1.0, 3.0, 4.5).get(contrast),
        dark,
    );
    let on_container_tone = adjust_tone_for_contrast(
        if dark { 90.0 } else { 30.0 },
        container_tone,
        ContrastCurve::new(3.0, 4.5, 7.0, 11.0).get(contrast),
        dark,
    );

    let tone_argb =
        |tone: f64| Argb::from_u32(palette.tone(tone.round().clamp(0.0, 100.0) as i32).0);
    CustomColorGroup {
        color: tone_argb(color_tone),
        on_color: tone_argb(on_color_tone),
        color_container: tone_argb(container_tone),
        on_color_container: tone_argb(on_container_tone),
    }
}

impl MaterialTheme {
    /// Generates colors while keeping the default typography, motion, and layout.
    pub fn from_seed_with(
        seed: Argb,
        appearance: Appearance,
        contrast: f64,
        variant: DynamicVariant,
        spec_version: SpecVersion,
    ) -> Self {
        Self {
            colors: ColorScheme::from_seed_with(seed, appearance, contrast, variant, spec_version),
            ..Self::default()
        }
    }
}

/// Bundled palette recipes, with seed colors inspired by md3.io.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThemePreset {
    /// Standard Material 3 baseline purple palette (default).
    #[default]
    Baseline,
    /// Tonal Spot dynamic scheme derived from the baseline seed.
    TonalSpot,
    /// Vibrant dynamic scheme derived from the baseline seed.
    Vibrant,
    /// Expressive dynamic scheme derived from the baseline seed.
    Expressive,
    /// Fidelity dynamic scheme derived from the baseline seed.
    Fidelity,
    /// Content dynamic scheme derived from the baseline seed.
    Content,
    /// Monochrome grayscale dynamic scheme.
    Monochrome,
    /// Low-chroma neutral dynamic scheme.
    Neutral,
    /// High-contrast (`contrast = 1.0`) Tonal Spot scheme derived from the baseline seed.
    HighContrast,
    /// Deep ocean blue Tonal Spot palette.
    Ocean,
    /// Forest green Tonal Spot palette.
    Forest,
    /// Bright spring green Tonal Spot palette.
    Spring,
    /// Warm sunset orange Tonal Spot palette.
    Sunset,
    /// Earthy desert ochre Tonal Spot palette.
    Desert,
    /// Clear summer sky blue Tonal Spot palette.
    Summer,
    /// Warm brown and beige Tonal Spot palette.
    BrownBeige,
    /// Muted sage and ivory Tonal Spot palette.
    SageIvory,
    /// Deep teal and caramel Tonal Spot palette.
    TealCaramel,
}

impl ThemePreset {
    /// All bundled presets in display order.
    pub const ALL: [Self; 18] = [
        Self::Baseline,
        Self::TonalSpot,
        Self::Vibrant,
        Self::Expressive,
        Self::Fidelity,
        Self::Content,
        Self::Monochrome,
        Self::Neutral,
        Self::HighContrast,
        Self::Ocean,
        Self::Forest,
        Self::Spring,
        Self::Sunset,
        Self::Desert,
        Self::Summer,
        Self::BrownBeige,
        Self::SageIvory,
        Self::TealCaramel,
    ];

    /// Resolves the preset for the requested appearance.
    pub fn colors(self, appearance: Appearance) -> ColorScheme {
        use DynamicVariant as Variant;
        const BASELINE_SEED: u32 = 0xff67_50a4;
        let (seed, variant) = match self {
            Self::Baseline => return ColorScheme::baseline(appearance),
            Self::TonalSpot | Self::HighContrast => (BASELINE_SEED, Variant::TonalSpot),
            Self::Vibrant => (BASELINE_SEED, Variant::Vibrant),
            Self::Expressive => (BASELINE_SEED, Variant::Expressive),
            Self::Fidelity => (BASELINE_SEED, Variant::Fidelity),
            Self::Content => (BASELINE_SEED, Variant::Content),
            Self::Monochrome => (BASELINE_SEED, Variant::Monochrome),
            Self::Neutral => (BASELINE_SEED, Variant::Neutral),
            Self::Ocean => (0xff00_6c9c, Variant::TonalSpot),
            Self::Forest => (0xff4b_5d3a, Variant::TonalSpot),
            Self::Spring => (0xff76_d275, Variant::TonalSpot),
            Self::Sunset => (0xffd3_5400, Variant::TonalSpot),
            Self::Desert => (0xff9a_7a3e, Variant::TonalSpot),
            Self::Summer => (0xff41_96cb, Variant::TonalSpot),
            Self::BrownBeige => (0xff3e_2723, Variant::TonalSpot),
            Self::SageIvory => (0xff7d_8c73, Variant::TonalSpot),
            Self::TealCaramel => (0xff2b_4d56, Variant::TonalSpot),
        };
        ColorScheme::from_seed_with(
            Argb::from_u32(seed),
            appearance,
            f64::from(self == Self::HighContrast),
            variant,
            SpecVersion::Spec2021,
        )
    }

    /// Builds a complete [`MaterialTheme`] using this preset's colors and default typography, motion, and layout.
    pub fn theme(self, appearance: Appearance) -> MaterialTheme {
        MaterialTheme {
            colors: self.colors(appearance),
            ..MaterialTheme::default()
        }
    }
}
