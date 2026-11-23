use crate::{
    LayoutPolicy,
    generated::{BASELINE_TYPE_STYLES, EMPHASIZED_TYPE_STYLES},
};

/// The two Material typeface families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Typeface {
    /// Expressive display and headline typeface (`brand`).
    Brand,
    /// Neutral body and UI label typeface (`plain`).
    Plain,
}

/// Family preference and ordered fallback for one typeface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontFamilies {
    /// Preferred primary font family name (defaults to `"Roboto"`).
    pub primary: String,
    /// Ordered fallback font family names tried when [`Self::primary`] is unavailable.
    pub fallback: Vec<String>,
}

impl Default for FontFamilies {
    fn default() -> Self {
        Self {
            primary: "Roboto".into(),
            fallback: [
                "SF Pro Text",
                "Segoe UI",
                "Noto Sans",
                "PingFang SC",
                "Microsoft YaHei UI",
                "Noto Sans CJK SC",
            ]
            .map(str::to_owned)
            .into(),
        }
    }
}

/// One of the fifteen Material text roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeRole {
    /// Large body copy (`16 sp`).
    BodyLarge,
    /// Default body copy (`14 sp`).
    BodyMedium,
    /// Small body or supporting copy (`12 sp`).
    BodySmall,
    /// Largest display headline (`57 sp`).
    DisplayLarge,
    /// Medium display headline (`45 sp`).
    DisplayMedium,
    /// Small display headline (`36 sp`).
    DisplaySmall,
    /// Large section headline (`32 sp`).
    HeadlineLarge,
    /// Medium section headline (`28 sp`).
    HeadlineMedium,
    /// Small section headline (`24 sp`).
    HeadlineSmall,
    /// Prominent control label (`14 sp`).
    LabelLarge,
    /// Standard control label (`12 sp`).
    LabelMedium,
    /// Compact label or badge text (`11 sp`).
    LabelSmall,
    /// Large title (`22 sp`).
    TitleLarge,
    /// Medium title (`16 sp`).
    TitleMedium,
    /// Small title (`14 sp`).
    TitleSmall,
}

impl TypeRole {
    /// All 15 Material text roles in stable token order.
    pub const ALL: [Self; 15] = [
        Self::BodyLarge,
        Self::BodyMedium,
        Self::BodySmall,
        Self::DisplayLarge,
        Self::DisplayMedium,
        Self::DisplaySmall,
        Self::HeadlineLarge,
        Self::HeadlineMedium,
        Self::HeadlineSmall,
        Self::LabelLarge,
        Self::LabelMedium,
        Self::LabelSmall,
        Self::TitleLarge,
        Self::TitleMedium,
        Self::TitleSmall,
    ];

    /// Returns the unscaled Material [`TypeStyle`] for this role and emphasis level.
    pub const fn style(self, emphasis: TypographyEmphasis) -> TypeStyle {
        match emphasis {
            TypographyEmphasis::Baseline => BASELINE_TYPE_STYLES[self as usize],
            TypographyEmphasis::Emphasized => EMPHASIZED_TYPE_STYLES[self as usize],
        }
    }
}

/// Emphasis tier for Material 3 Expressive typography.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypographyEmphasis {
    /// Standard Material 3 type scale weights (default).
    #[default]
    Baseline,
    /// Heavier Material 3 Expressive emphasized type scale weights.
    Emphasized,
}

/// Material type metrics in scale-independent units (sp).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    /// Logical typeface slot (`Brand` or `Plain`).
    pub typeface: Typeface,
    /// Font size in `sp`.
    pub size: f32,
    /// Line height in `sp`.
    pub line_height: f32,
    /// Letter spacing (tracking) in `sp`.
    pub tracking: f32,
    /// Numeric font weight (`100..=900`).
    pub weight: u16,
}

/// Family names for the Material scale. The host resolves available fonts
/// and measures the resulting text.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Typography {
    /// Font family stack used for [`Typeface::Brand`] roles.
    pub brand: FontFamilies,
    /// Font family stack used for [`Typeface::Plain`] roles.
    pub plain: FontFamilies,
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_text_scale",
            deserialize_with = "deserialize_text_scale"
        )
    )]
    text_scale: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            brand: FontFamilies::default(),
            plain: FontFamilies::default(),
            text_scale: default_text_scale(),
        }
    }
}

impl Typography {
    /// Sets the theme's text scale. The range is 1.0–2.0, with NaN using 1.0.
    pub fn with_text_scale(mut self, scale: f32) -> Self {
        self.text_scale = normalize_text_scale(scale);
        self
    }

    /// Returns the active text scale multiplier in `1.0..=2.0`.
    pub fn text_scale(&self) -> f32 {
        self.text_scale
    }

    /// Resolves a Material role using the theme's text scale.
    pub fn style(&self, role: TypeRole, emphasis: TypographyEmphasis) -> TypeStyle {
        self.scale_style(role.style(emphasis))
    }

    /// Scales an unscaled style's size, line height and tracking.
    pub fn scale_style(&self, style: TypeStyle) -> TypeStyle {
        LayoutPolicy::scale_text(style, self.text_scale)
    }

    /// Returns the configured [`FontFamilies`] for a logical [`Typeface`] slot.
    pub fn families(&self, typeface: Typeface) -> &FontFamilies {
        match typeface {
            Typeface::Brand => &self.brand,
            Typeface::Plain => &self.plain,
        }
    }
}

fn default_text_scale() -> f32 {
    1.0
}

fn normalize_text_scale(scale: f32) -> f32 {
    if scale.is_nan() {
        default_text_scale()
    } else {
        scale.clamp(1.0, 2.0)
    }
}

#[cfg(feature = "serde")]
fn deserialize_text_scale<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<f32, D::Error> {
    <f32 as serde::Deserialize>::deserialize(deserializer).map(normalize_text_scale)
}
