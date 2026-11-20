use material_color_utils::{
    blend::blend_functions::Blend,
    dynamic::color_spec::SpecVersion as McuSpec,
    palettes::tonal_palette::TonalPalette,
    scheme::{
        SchemeContent, SchemeExpressive, SchemeFidelity, SchemeMonochrome, SchemeNeutral,
        SchemeTonalSpot, SchemeVibrant,
    },
    utils::color_utils::Argb as McuArgb,
};

use crate::{Appearance, Argb, ColorRole, ColorScheme, MaterialTheme, generated::dynamic_colors};

/// The Material dynamic color palette to derive from a seed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynamicVariant {
    #[default]
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Monochrome,
    Neutral,
}

/// The requested Material Color Utilities specification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecVersion {
    #[default]
    Spec2021,
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
        let mut scheme = match variant {
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
            // Resolve each custom palette with the same tone and contrast
            // relationships as Error, against the generated surface palette.
            scheme.error_palette =
                TonalPalette::from_hct(Blend::harmonize(McuArgb(status_seed), seed).into());
            scheme.argb_cache.pin().clear();
            scheme.hct_cache.pin().clear();
            scheme.tone_cache.pin().clear();
            let values = [
                scheme.error(),
                scheme.on_error(),
                scheme.error_container(),
                scheme.on_error_container(),
            ];
            for (role, color) in roles.into_iter().zip(values) {
                colors.set_color(role, Argb::from_u32(color.0));
            }
        }
        colors
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
    #[default]
    Baseline,
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Monochrome,
    Neutral,
    HighContrast,
    Ocean,
    Forest,
    Spring,
    Sunset,
    Desert,
    Summer,
    BrownBeige,
    SageIvory,
    TealCaramel,
}

impl ThemePreset {
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

    pub fn theme(self, appearance: Appearance) -> MaterialTheme {
        MaterialTheme {
            colors: self.colors(appearance),
            ..MaterialTheme::default()
        }
    }
}
