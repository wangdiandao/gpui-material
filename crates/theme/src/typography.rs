use crate::generated::{BASELINE_TYPE_STYLES, EMPHASIZED_TYPE_STYLES};

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Typography {
    pub brand: FontFamilies,
    pub plain: FontFamilies,
}

impl Typography {
    pub fn families(&self, typeface: Typeface) -> &FontFamilies {
        match typeface {
            Typeface::Brand => &self.brand,
            Typeface::Plain => &self.plain,
        }
    }
}
