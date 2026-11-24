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

/// Sanitizes a logical scalar dimension to a finite non-negative value.
///
/// `NaN`, infinities, negative values, and `-0.0` normalize to `0.0`.
pub const fn non_negative_f32(value: f32) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        0.0
    } else {
        value
    }
}

/// Direction-aware logical edge insets in density-independent units (`dp`).
///
/// Shared across components to avoid duplicating per-component padding structs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicalEdges {
    /// Top edge inset in logical units.
    pub top: f32,
    /// Bottom edge inset in logical units.
    pub bottom: f32,
    /// Inline-start edge inset (left in LTR, right in RTL).
    pub start: f32,
    /// Inline-end edge inset (right in LTR, left in RTL).
    pub end: f32,
}

impl LogicalEdges {
    /// Zero insets on all four sides.
    pub const ZERO: Self = Self {
        top: 0.0,
        bottom: 0.0,
        start: 0.0,
        end: 0.0,
    };

    /// Creates uniform insets on all four sides, clamping invalid or negative values to `0.0`.
    pub const fn all(value: f32) -> Self {
        let value = non_negative_f32(value);
        Self {
            top: value,
            bottom: value,
            start: value,
            end: value,
        }
    }

    /// Creates symmetric insets from `horizontal` (start/end) and `vertical` (top/bottom).
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        let horizontal = non_negative_f32(horizontal);
        let vertical = non_negative_f32(vertical);
        Self {
            top: vertical,
            bottom: vertical,
            start: horizontal,
            end: horizontal,
        }
    }

    /// Creates explicit logical insets in CSS-like `(top, end, bottom, start)` order.
    pub const fn new(top: f32, end: f32, bottom: f32, start: f32) -> Self {
        Self {
            top: non_negative_f32(top),
            bottom: non_negative_f32(bottom),
            start: non_negative_f32(start),
            end: non_negative_f32(end),
        }
    }

    /// Sets the top inset.
    pub const fn top(mut self, value: f32) -> Self {
        self.top = non_negative_f32(value);
        self
    }

    /// Sets the bottom inset.
    pub const fn bottom(mut self, value: f32) -> Self {
        self.bottom = non_negative_f32(value);
        self
    }

    /// Sets the inline-start inset.
    pub const fn start(mut self, value: f32) -> Self {
        self.start = non_negative_f32(value);
        self
    }

    /// Sets the inline-end inset.
    pub const fn end(mut self, value: f32) -> Self {
        self.end = non_negative_f32(value);
        self
    }

    /// Sets both `start` and `end` insets.
    pub const fn horizontal(mut self, value: f32) -> Self {
        let value = non_negative_f32(value);
        self.start = value;
        self.end = value;
        self
    }

    /// Sets both `top` and `bottom` insets.
    pub const fn vertical(mut self, value: f32) -> Self {
        let value = non_negative_f32(value);
        self.top = value;
        self.bottom = value;
        self
    }

    /// Returns a copy with every side sanitized via [`non_negative_f32`].
    pub const fn sanitize(self) -> Self {
        Self {
            top: non_negative_f32(self.top),
            bottom: non_negative_f32(self.bottom),
            start: non_negative_f32(self.start),
            end: non_negative_f32(self.end),
        }
    }

    /// Resolves sanitized insets to physical `[top, right, bottom, left]` order for `direction`.
    pub const fn resolve(self, direction: Direction) -> [f32; 4] {
        let clean = self.sanitize();
        let (left, right) = match direction {
            Direction::Ltr => (clean.start, clean.end),
            Direction::Rtl => (clean.end, clean.start),
        };
        [clean.top, right, clean.bottom, left]
    }
}
