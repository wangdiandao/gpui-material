//! The complete Material Symbols catalog and variable glyph outlines.
//!
//! Use [SymbolFont::from_bytes] to inject a font. On desktop the
//! `embed-assets` feature also provides bundled fonts. Browser apps can
//! asynchronously fetch a TrueType asset with `SymbolFont::fetch`.
//! Pass the generated SVG bytes to `gpui_material::Icon::svg_data` or GPUI's SVG element.

mod font;
#[cfg(target_arch = "wasm32")]
mod web;

pub use font::{SymbolError, SymbolFont};

macro_rules! catalog {
    ($($variant:ident => $name:literal, $codepoint:literal;)+) => {
        /// A Material Symbols name from the pinned Google catalog, including aliases.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        #[repr(u16)]
        pub enum Symbol {
            $(
                #[doc = $name]
                $variant,
            )+
        }

        impl Symbol {
            /// Every name, in alphabetical order by its upstream spelling.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            const NAMES: &'static [&'static str] = &[$($name),+];
            const CODEPOINTS: &'static [u32] = &[$($codepoint),+];

            /// The exact upstream name, suitable for storage and interchange.
            pub const fn name(self) -> &'static str {
                Self::NAMES[self as usize]
            }

            /// Resolve a case-sensitive upstream name.
            pub fn from_name(name: &str) -> Option<Self> {
                let index = Self::NAMES.binary_search(&name).ok()?;
                Some(Self::ALL[index])
            }

            /// The font's Unicode code point. Multiple names can share a glyph.
            pub const fn codepoint(self) -> u32 {
                Self::CODEPOINTS[self as usize]
            }
        }
    };
}

include!("catalog.rs");

/// The three Material Symbols font families.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SymbolStyle {
    #[default]
    Outlined,
    Rounded,
    Sharp,
}

/// A Material Symbols weight in the inclusive range 100–700.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Weight(u16);

impl Weight {
    pub const THIN: Self = Self(100);
    pub const REGULAR: Self = Self(400);
    pub const BOLD: Self = Self(700);

    /// Create a weight, returning `None` for values outside 100–700.
    pub const fn new(value: u16) -> Option<Self> {
        if value >= Self::THIN.0 && value <= Self::BOLD.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self::REGULAR
    }
}

/// Fill and weight for one glyph. Defaults to FILL=0 and weight 400.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SymbolAxes {
    filled: bool,
    weight: Weight,
}

impl SymbolAxes {
    /// Default unfilled (FILL=0), regular-weight (400) axes for `const` contexts.
    pub const fn new() -> Self {
        Self {
            filled: false,
            weight: Weight::REGULAR,
        }
    }

    /// Select FILL=1 when true and FILL=0 otherwise.
    pub const fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub const fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    pub const fn is_filled(self) -> bool {
        self.filled
    }

    pub const fn get_weight(self) -> Weight {
        self.weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_resolves_every_upstream_name() {
        assert_eq!(Symbol::ALL.len(), 4304);
        for &symbol in Symbol::ALL {
            assert_eq!(Symbol::from_name(symbol.name()), Some(symbol));
            assert!(char::from_u32(symbol.codepoint()).is_some());
        }
        assert_eq!(Symbol::from_name("10k"), Some(Symbol::N10k));
        assert_eq!(Symbol::Favorite.codepoint(), 0xe87e);
        assert_eq!(
            Symbol::FavoriteBorder.codepoint(),
            Symbol::Favorite.codepoint()
        );
        assert_eq!(Symbol::from_name("not_a_symbol"), None);
    }

    #[test]
    fn weight_accepts_the_entire_axis_range() {
        const CONST_AXES: SymbolAxes = SymbolAxes::new().filled(true).weight(Weight::BOLD);
        assert!(CONST_AXES.is_filled());
        assert_eq!(CONST_AXES.get_weight(), Weight::BOLD);
        assert_eq!(SymbolAxes::new(), SymbolAxes::default());
        assert_eq!(Weight::new(99), None);
        assert_eq!(Weight::new(100), Some(Weight::THIN));
        assert_eq!(Weight::new(550).unwrap().get(), 550);
        assert_eq!(Weight::new(700), Some(Weight::BOLD));
        assert_eq!(Weight::new(701), None);
    }
}
