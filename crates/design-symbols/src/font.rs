use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
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
const SVG_CACHE_CAPACITY: usize = 256;

type CacheKey = (u32, SymbolAxes);

#[derive(Default)]
struct SvgCache {
    entries: HashMap<CacheKey, Arc<[u8]>>,
    order: VecDeque<CacheKey>,
}

impl SvgCache {
    fn get(&self, key: &CacheKey) -> Option<Arc<[u8]>> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: CacheKey, data: Arc<[u8]>) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) = self.entries.entry(key) {
            entry.insert(data);
            return;
        }
        while self.entries.len() >= SVG_CACHE_CAPACITY {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }
        self.entries.insert(key, data);
        self.order.push_back(key);
    }
}

/// A font load or glyph extraction error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SymbolError {
    /// The supplied bytes could not be parsed as a valid TrueType/OpenType font face.
    InvalidFont,
    /// The font does not cover the required `FILL` (`0..=1`) and `wght` (`100..=700`) variation axes.
    UnsupportedAxes,
    /// The requested symbol's Unicode code point is not present in the loaded font.
    MissingGlyph(Symbol),
    /// The font contained the glyph index but produced no outline path.
    InvalidOutline(Symbol),
    /// Fetching the font asset from the browser failed.
    #[cfg(target_arch = "wasm32")]
    Fetch(String),
    /// The browser fetch request returned a non-OK HTTP status code.
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
    cache: Mutex<SvgCache>,
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
            cache: Mutex::new(SvgCache::default()),
        })
    }

    #[cfg(all(
        any(
            feature = "embed-outlined",
            feature = "embed-rounded",
            feature = "embed-sharp"
        ),
        not(target_arch = "wasm32")
    ))]
    fn from_embedded_bytes(bytes: &'static [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            has_grade_opsz: false,
            cache: Mutex::new(SvgCache::default()),
        }
    }

    /// Use the bundled Outlined font on desktop. Requires the `embed-outlined` feature.
    #[cfg(all(feature = "embed-outlined", not(target_arch = "wasm32")))]
    pub fn embedded_outlined() -> Self {
        Self::from_embedded_bytes(include_bytes!("../assets/material-symbols-outlined.ttf"))
    }

    /// Use the bundled Rounded font on desktop. Requires the `embed-rounded` feature.
    #[cfg(all(feature = "embed-rounded", not(target_arch = "wasm32")))]
    pub fn embedded_rounded() -> Self {
        Self::from_embedded_bytes(include_bytes!("../assets/material-symbols-rounded.ttf"))
    }

    /// Use the bundled Sharp font on desktop. Requires the `embed-sharp` feature.
    #[cfg(all(feature = "embed-sharp", not(target_arch = "wasm32")))]
    pub fn embedded_sharp() -> Self {
        Self::from_embedded_bytes(include_bytes!("../assets/material-symbols-sharp.ttf"))
    }

    /// Use a bundled font on desktop. Requires the `embed-assets` feature.
    #[cfg(all(feature = "embed-assets", not(target_arch = "wasm32")))]
    pub fn embedded(style: crate::SymbolStyle) -> Self {
        use crate::SymbolStyle::{Outlined, Rounded, Sharp};

        match style {
            Outlined => Self::embedded_outlined(),
            Rounded => Self::embedded_rounded(),
            Sharp => Self::embedded_sharp(),
        }
    }

    /// Generate a monochrome 24×24 SVG with the selected fill and weight.
    ///
    /// The design grid uses grade 0 and optical size 24. A font subset returns
    /// [SymbolError::MissingGlyph] when it omits the requested symbol.
    pub fn svg_data(&self, symbol: Symbol, axes: SymbolAxes) -> Result<Arc<[u8]>, SymbolError> {
        let codepoint = symbol.codepoint();
        let key = (codepoint, axes);
        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(svg) = cache.get(&key) {
            return Ok(svg);
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
        let _ = write!(
            buf,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 -{em} {em} {em}"><path transform="scale(1 -1)" d=""#,
        );
        let mut svg = SvgPath(buf);
        face.outline_glyph(glyph, &mut svg)
            .ok_or(SymbolError::InvalidOutline(symbol))?;
        svg.0.push_str(r#""/></svg>"#);
        let data: Arc<[u8]> = Arc::from(svg.0.into_bytes());
        cache.insert(key, data.clone());
        Ok(data)
    }
}

fn push_coord(out: &mut String, value: f32) {
    if !value.is_finite() {
        out.push('0');
        return;
    }
    let scaled = (f64::from(value) * 100.0).round() as i64;
    if scaled == 0 {
        out.push('0');
        return;
    }
    if scaled < 0 {
        out.push('-');
    }
    let abs = scaled.unsigned_abs();
    let int_part = abs / 100;
    let frac_part = abs % 100;
    if frac_part == 0 {
        let _ = write!(out, "{int_part}");
    } else if frac_part.is_multiple_of(10) {
        let _ = write!(out, "{int_part}.{}", frac_part / 10);
    } else {
        let _ = write!(out, "{int_part}.{frac_part:02}");
    }
}

struct SvgPath(String);

impl OutlineBuilder for SvgPath {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push('M');
        push_coord(&mut self.0, x);
        self.0.push(' ');
        push_coord(&mut self.0, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push('L');
        push_coord(&mut self.0, x);
        self.0.push(' ');
        push_coord(&mut self.0, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push('Q');
        push_coord(&mut self.0, x1);
        self.0.push(' ');
        push_coord(&mut self.0, y1);
        self.0.push(' ');
        push_coord(&mut self.0, x);
        self.0.push(' ');
        push_coord(&mut self.0, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push('C');
        push_coord(&mut self.0, x1);
        self.0.push(' ');
        push_coord(&mut self.0, y1);
        self.0.push(' ');
        push_coord(&mut self.0, x2);
        self.0.push(' ');
        push_coord(&mut self.0, y2);
        self.0.push(' ');
        push_coord(&mut self.0, x);
        self.0.push(' ');
        push_coord(&mut self.0, y);
    }

    fn close(&mut self) {
        self.0.push('Z');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_font_bytes_and_formats_errors() {
        assert!(matches!(
            SymbolFont::from_bytes(Vec::new()),
            Err(SymbolError::InvalidFont)
        ));
        assert!(SymbolError::InvalidFont.to_string().contains("invalid"));
        assert!(SymbolError::UnsupportedAxes.to_string().contains("FILL"));
        assert!(
            SymbolError::MissingGlyph(Symbol::Favorite)
                .to_string()
                .contains("favorite")
        );
        assert!(
            SymbolError::InvalidOutline(Symbol::Favorite)
                .to_string()
                .contains("favorite")
        );
    }

    #[test]
    fn push_coord_formats_compactly_without_trailing_zeros() {
        let fmt = |v| {
            let mut s = String::new();
            push_coord(&mut s, v);
            s
        };
        assert_eq!(fmt(0.0), "0");
        assert_eq!(fmt(-0.001), "0");
        assert_eq!(fmt(f32::NAN), "0");
        assert_eq!(fmt(12.0), "12");
        assert_eq!(fmt(-12.50), "-12.5");
        assert_eq!(fmt(12.345), "12.35");
        assert_eq!(fmt(0.05), "0.05");
    }

    #[test]
    fn svg_cache_hits_and_evicts_fifo_at_capacity() {
        let mut cache = SvgCache::default();
        let axes = SymbolAxes::default();
        let first: Arc<[u8]> = Arc::from(b"first".as_slice());
        cache.insert((0, axes), first.clone());
        assert!(Arc::ptr_eq(&cache.get(&(0, axes)).unwrap(), &first));

        for i in 1..=(SVG_CACHE_CAPACITY as u32) {
            cache.insert((i, axes), Arc::from(format!("v{i}").into_bytes()));
        }
        assert!(cache.get(&(0, axes)).is_none());
        assert!(cache.get(&(1, axes)).is_some());
        assert!(cache.get(&(SVG_CACHE_CAPACITY as u32, axes)).is_some());
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
