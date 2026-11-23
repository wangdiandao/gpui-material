use crate::{TypeStyle, generated::*};

/// Logical horizontal reading and layout direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    /// Left-to-right layout direction (default).
    #[default]
    Ltr,
    /// Right-to-left layout direction.
    Rtl,
}

/// Material window width classes, based on available logical width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthClass {
    /// Compact width (`< 600 dp`).
    Compact,
    /// Medium width (`600..840 dp`).
    Medium,
    /// Expanded width (`840..1200 dp`).
    Expanded,
    /// Large desktop width (`1200..1600 dp`).
    Large,
    /// Extra-large desktop width (`>= 1600 dp`).
    ExtraLarge,
}

impl WidthClass {
    /// Thresholds are inclusive. Non-finite and negative widths resolve to Compact.
    pub fn from_width(width: f32) -> Self {
        if !width.is_finite() {
            return Self::Compact;
        }
        if width >= MD_SYS_LAYOUT_WINDOW_WIDTH_EXTRA_LARGE {
            Self::ExtraLarge
        } else if width >= MD_SYS_LAYOUT_WINDOW_WIDTH_LARGE {
            Self::Large
        } else if width >= MD_SYS_LAYOUT_WINDOW_WIDTH_EXPANDED {
            Self::Expanded
        } else if width >= MD_SYS_LAYOUT_WINDOW_WIDTH_MEDIUM {
            Self::Medium
        } else {
            Self::Compact
        }
    }
}

/// Desktop density adjustment for a component's visual size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Density {
    /// Tighter visual spacing for dense desktop layouts.
    Compact,
    /// Default Material visual density.
    #[default]
    Standard,
    /// Roomier visual spacing.
    Comfortable,
}

impl Density {
    /// Applies this density's delta to a baseline visual size in logical units.
    ///
    /// Non-finite or non-positive baselines return `0.0`.
    pub fn visual_size(self, baseline: f32) -> f32 {
        let delta = match self {
            Self::Compact => MD_SYS_LAYOUT_DENSITY_COMPACT,
            Self::Standard => MD_SYS_LAYOUT_DENSITY_STANDARD,
            Self::Comfortable => MD_SYS_LAYOUT_DENSITY_COMFORTABLE,
        };
        if !baseline.is_finite() || baseline <= 0.0 {
            return 0.0;
        }
        (baseline + delta).max(0.0)
    }

    /// The target stays at least 48 logical units even for dense visuals.
    pub fn interactive_size(self, baseline: f32) -> f32 {
        self.visual_size(baseline)
            .max(MD_SYS_LAYOUT_MINIMUM_INTERACTIVE_SIZE)
    }
}

/// Pure layout inputs. Widths and geometry are already in logical units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayoutPolicy {
    /// Visual density setting for component sizing.
    pub density: Density,
    /// Logical horizontal layout direction (`Ltr` or `Rtl`).
    pub direction: Direction,
}

impl LayoutPolicy {
    /// Converts text metrics using the host's user font scale. Invalid or
    /// overflowing scale factors use the baseline metrics.
    pub fn scale_text(style: TypeStyle, font_scale: f32) -> TypeStyle {
        if !font_scale.is_finite() || font_scale <= 0.0 {
            return style;
        }
        let scaled = TypeStyle {
            size: style.size * font_scale,
            line_height: style.line_height * font_scale,
            tracking: style.tracking * font_scale,
            ..style
        };
        if [scaled.size, scaled.line_height, scaled.tracking]
            .into_iter()
            .all(f32::is_finite)
            && scaled.size > 0.0
            && scaled.line_height > 0.0
        {
            scaled
        } else {
            style
        }
    }

    /// Maps logical start and end to physical left and right.
    pub const fn horizontal<T: Copy>(self, start: T, end: T) -> (T, T) {
        match self.direction {
            Direction::Ltr => (start, end),
            Direction::Rtl => (end, start),
        }
    }
}
