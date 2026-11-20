use crate::{
    LayoutPolicy,
    generated::{BASELINE_TYPE_STYLES, EMPHASIZED_TYPE_STYLES},
};

/// The two Material typeface families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Typeface {
    Brand,
    Plain,
}

/// Family preference and ordered fallback for one typeface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontFamilies {
    pub primary: String,
    pub fallback: Vec<String>,
}

impl Default for FontFamilies {
    fn default() -> Self {
        Self {
            primary: "Roboto".into(),
            fallback: ["Segoe UI", "Microsoft YaHei UI", "Noto Sans CJK SC"]
                .map(str::to_owned)
                .into(),
        }
    }
}

/// One of the fifteen Material text roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeRole {
    BodyLarge,
    BodyMedium,
    BodySmall,
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
}

impl TypeRole {
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

    pub const fn style(self, emphasis: TypographyEmphasis) -> TypeStyle {
        match emphasis {
            TypographyEmphasis::Baseline => BASELINE_TYPE_STYLES[self as usize],
            TypographyEmphasis::Emphasized => EMPHASIZED_TYPE_STYLES[self as usize],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypographyEmphasis {
    #[default]
    Baseline,
    Emphasized,
}

/// Material type metrics in scale-independent units (sp).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub typeface: Typeface,
    pub size: f32,
    pub line_height: f32,
    pub tracking: f32,
    pub weight: u16,
}

/// Family names for the Material scale. The host resolves available fonts
/// and measures the resulting text.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Typography {
    pub brand: FontFamilies,
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
