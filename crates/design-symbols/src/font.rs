use std::{
    borrow::Cow,
    error::Error,
    fmt,
    fmt::Write as _,
    sync::{Arc, Mutex},
};

use ttf_parser::{Face, OutlineBuilder, Tag};

use crate::{Symbol, SymbolAxes};

const FILL: Tag = Tag::from_bytes(b"FILL");
const WEIGHT: Tag = Tag::from_bytes(b"wght");
const GRADE: Tag = Tag::from_bytes(b"GRAD");
const OPTICAL_SIZE: Tag = Tag::from_bytes(b"opsz");
const SVG_CACHE_LEN: usize = 256;

type SvgCacheSlot = Option<((u32, SymbolAxes), Arc<[u8]>)>;

/// A font load or glyph extraction error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SymbolError {
    InvalidFont,
    UnsupportedAxes,
    MissingGlyph(Symbol),
    InvalidOutline(Symbol),
    #[cfg(target_arch = "wasm32")]
    Fetch(String),
    #[cfg(target_arch = "wasm32")]
    HttpStatus(u16),
}

impl fmt::Display for SymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFont => f.write_str("invalid Material Symbols font"),
            Self::UnsupportedAxes => f.write_str("expected FILL 0–1 and wght 100–700"),
            Self::MissingGlyph(symbol) => write!(f, "font is missing {}", symbol.name()),
            Self::InvalidOutline(symbol) => write!(f, "invalid outline for {}", symbol.name()),
            #[cfg(target_arch = "wasm32")]
            Self::Fetch(message) => write!(f, "could not fetch Material Symbols font: {message}"),
            #[cfg(target_arch = "wasm32")]
            Self::HttpStatus(status) => {
                write!(f, "Material Symbols font request returned HTTP {status}")
            }
        }
    }
}

impl Error for SymbolError {}

/// Font bytes for one Material Symbols family, owned by the application.
///
/// The same font can generate SVGs at different fill and weight settings.
/// Rendered SVGs can be retained and reused independently of the font.
pub struct SymbolFont {
    bytes: Cow<'static, [u8]>,
    has_grade_opsz: bool,
    cache: Mutex<Vec<SvgCacheSlot>>,
}

impl SymbolFont {
    /// Load a Material Symbols TrueType font from application-supplied bytes.
    ///
    /// The font must cover the FILL and weight ranges. It may contain a subset
    /// of glyphs, provided it retains their upstream code points.
    pub fn from_bytes(bytes: impl Into<Cow<'static, [u8]>>) -> Result<Self, SymbolError> {
        let bytes = bytes.into();
        let face = Face::parse(&bytes, 0).map_err(|_| SymbolError::InvalidFont)?;
        let axes = face.variation_axes();
        let covers = |tag, min, max| {
            axes.into_iter()
                .any(|axis| axis.tag == tag && axis.min_value <= min && axis.max_value >= max)
        };
        if !covers(FILL, 0., 1.) || !covers(WEIGHT, 100., 700.) {
            return Err(SymbolError::UnsupportedAxes);
        }
        let has_grade_opsz = axes
            .into_iter()
            .any(|axis| axis.tag == GRADE || axis.tag == OPTICAL_SIZE);
        Ok(Self {
            bytes,
            has_grade_opsz,
            cache: Mutex::new(Vec::new()),
        })
    }

    /// Use a bundled font on desktop. Requires the `embed-assets` feature.
    #[cfg(all(feature = "embed-assets", not(target_arch = "wasm32")))]
    pub fn embedded(style: crate::SymbolStyle) -> Self {
        use crate::SymbolStyle::{Outlined, Rounded, Sharp};

        let bytes: &'static [u8] = match style {
            Outlined => include_bytes!("../assets/material-symbols-outlined.ttf"),
            Rounded => include_bytes!("../assets/material-symbols-rounded.ttf"),
            Sharp => include_bytes!("../assets/material-symbols-sharp.ttf"),
        };
        Self {
            bytes: Cow::Borrowed(bytes),
            has_grade_opsz: false,
            cache: Mutex::new(Vec::new()),
        }
    }

    /// Generate a monochrome 24×24 SVG with the selected fill and weight.
    ///
    /// The design grid uses grade 0 and optical size 24. A font subset returns
    /// [SymbolError::MissingGlyph] when it omits the requested symbol.
    pub fn svg_data(&self, symbol: Symbol, axes: SymbolAxes) -> Result<Arc<[u8]>, SymbolError> {
        let codepoint = symbol.codepoint();
        let key = (codepoint, axes);
        let slot_index = ((codepoint as usize).wrapping_mul(0x9e37_79b9)
            ^ (usize::from(axes.filled) << 16)
            ^ usize::from(axes.weight.get()))
            & (SVG_CACHE_LEN - 1);
        if let Ok(cache) = self.cache.lock()
            && let Some(Some((cached_key, svg))) = cache.get(slot_index)
            && *cached_key == key
        {
            return Ok(svg.clone());
        }

        let mut face = Face::parse(&self.bytes, 0).map_err(|_| SymbolError::InvalidFont)?;
        face.set_variation(FILL, f32::from(axes.filled))
            .ok_or(SymbolError::UnsupportedAxes)?;
        face.set_variation(WEIGHT, f32::from(axes.weight.get()))
            .ok_or(SymbolError::UnsupportedAxes)?;
        if self.has_grade_opsz {
            // The derived fonts already use these values. The original upstream
            // fonts expose both axes and use the same design grid at these values.
            face.set_variation(GRADE, 0.);
            face.set_variation(OPTICAL_SIZE, 24.);
        }

        let glyph = char::from_u32(codepoint)
            .and_then(|codepoint| face.glyph_index(codepoint))
            .ok_or(SymbolError::MissingGlyph(symbol))?;
        let em = face.units_per_em();
        let mut buf = String::with_capacity(1024);
        write!(
            buf,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 -{em} {em} {em}"><path transform="scale(1 -1)" d=""#,
        )
        .unwrap();
        let mut svg = SvgPath(buf);
        face.outline_glyph(glyph, &mut svg)
            .ok_or(SymbolError::InvalidOutline(symbol))?;
        svg.0.push_str(r#""/></svg>"#);
        let data: Arc<[u8]> = Arc::from(svg.0.into_bytes());
        if let Ok(mut cache) = self.cache.lock() {
            if cache.is_empty() {
                cache.resize(SVG_CACHE_LEN, None);
            }
            cache[slot_index] = Some((key, data.clone()));
        }
        Ok(data)
    }
}

struct SvgPath(String);

impl OutlineBuilder for SvgPath {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(self.0, "M{x} {y}").unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(self.0, "L{x} {y}").unwrap();
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(self.0, "Q{x1} {y1} {x} {y}").unwrap();
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(self.0, "C{x1} {y1} {x2} {y2} {x} {y}").unwrap();
    }

    fn close(&mut self) {
        self.0.push('Z');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_font_bytes() {
        assert!(matches!(
            SymbolFont::from_bytes(Vec::new()),
            Err(SymbolError::InvalidFont)
        ));
    }

    #[cfg(all(feature = "embed-assets", not(target_arch = "wasm32")))]
    #[test]
    fn every_symbol_has_an_outline_in_each_family() {
        use crate::{SymbolStyle, Weight};

        for style in [
            SymbolStyle::Outlined,
            SymbolStyle::Rounded,
            SymbolStyle::Sharp,
        ] {
            let font = SymbolFont::embedded(style);
            SymbolFont::from_bytes(font.bytes.clone()).unwrap();
            for weight in [Weight::THIN, Weight::REGULAR, Weight::BOLD] {
                for filled in [false, true] {
                    let axes = SymbolAxes::default().weight(weight).filled(filled);
                    for &symbol in Symbol::ALL {
                        let svg = font
                            .svg_data(symbol, axes)
                            .unwrap_or_else(|error| panic!("{style:?} {axes:?}: {error}"));
                        assert!(
                            svg.windows(4).any(|bytes| bytes == b"d=\"M"),
                            "{style:?} {symbol:?}"
                        );
                    }
                }
            }
        }
    }

    #[cfg(all(feature = "embed-assets", not(target_arch = "wasm32")))]
    #[test]
    fn family_fill_and_weight_change_the_outline_independently() {
        use crate::{SymbolStyle, Weight};

        let outlined = SymbolFont::embedded(SymbolStyle::Outlined);
        let rounded = SymbolFont::embedded(SymbolStyle::Rounded);
        let axes = SymbolAxes::default();
        let glyph = Symbol::Favorite;
        let base = outlined.svg_data(glyph, axes).unwrap();
        let alias = outlined.svg_data(Symbol::FavoriteBorder, axes).unwrap();
        assert!(Arc::ptr_eq(&base, &alias));

        assert_ne!(base, rounded.svg_data(glyph, axes).unwrap());
        assert_ne!(base, outlined.svg_data(glyph, axes.filled(true)).unwrap());
        assert_ne!(
            base,
            outlined.svg_data(glyph, axes.weight(Weight::BOLD)).unwrap()
        );
    }
}
