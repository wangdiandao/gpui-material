use gpui::{AnyElement, Div, Hsla, SharedString, Styled, TextAlign, div, prelude::FluentBuilder};
use gpui_material_foundation::text_style;
use gpui_material_theme::{Direction, TypeRole, TypographyEmphasis};

/// Layout and typography variant for a Material top [`super::AppBar`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarVariant {
    /// Single-row bar with start-aligned title.
    #[default]
    Small,
    /// Single-row bar with centered title.
    Centered,
    /// Collapsible two-row bar with medium expanded headline typography.
    MediumFlexible,
    /// Collapsible two-row bar with large expanded display typography.
    LargeFlexible,
    /// Two-row bar supporting a custom second row below the expanded heading.
    TwoRows,
}

impl AppBarVariant {
    pub(super) fn two_rows(self) -> bool {
        matches!(
            self,
            Self::MediumFlexible | Self::LargeFlexible | Self::TwoRows
        )
    }

    pub(super) fn expansion(self, subtitle: bool) -> f32 {
        match (self, subtitle) {
            (Self::MediumFlexible | Self::TwoRows, false) => 48.,
            (Self::MediumFlexible | Self::TwoRows, true) => 72.,
            (Self::LargeFlexible, false) => 56.,
            (Self::LargeFlexible, true) => 88.,
            _ => 0.,
        }
    }

    pub(super) fn expanded_roles(self) -> (TypeRole, TypeRole) {
        if self == Self::LargeFlexible {
            (TypeRole::DisplaySmall, TypeRole::TitleMedium)
        } else {
            (TypeRole::HeadlineMedium, TypeRole::LabelLarge)
        }
    }
}

/// Logical alignment for a title or subtitle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarAlignment {
    /// Align to the logical start edge (left in LTR, right in RTL).
    #[default]
    Start,
    /// Center horizontally.
    Center,
    /// Align to the logical end edge (right in LTR, left in RTL).
    End,
}

impl AppBarAlignment {
    pub(super) fn resolve(self, direction: Direction) -> TextAlign {
        match (self, direction) {
            (Self::Center, _) => TextAlign::Center,
            (Self::Start, Direction::Ltr) | (Self::End, Direction::Rtl) => TextAlign::Left,
            _ => TextAlign::Right,
        }
    }
}

/// Per-instance pigments. Missing fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AppBarColors {
    /// Container background color when not scrolled.
    pub container: Option<Hsla>,
    /// Container background color when content is scrolled beneath the bar.
    pub scrolled_container: Option<Hsla>,
    /// Foreground tint for the leading navigation icon slot.
    pub navigation: Option<Hsla>,
    /// Foreground color for the title text.
    pub title: Option<Hsla>,
    /// Foreground color for the subtitle text.
    pub subtitle: Option<Hsla>,
    /// Foreground tint for trailing action controls.
    pub actions: Option<Hsla>,
}

pub(super) type HeadingContent = Box<dyn FnOnce(bool) -> AnyElement>;

#[derive(Default)]
pub(super) struct Heading {
    pub(super) text: Option<SharedString>,
    pub(super) content: Option<HeadingContent>,
    pub(super) alignment: Option<AppBarAlignment>,
    pub(super) max_lines: Option<usize>,
}

impl Heading {
    pub(super) fn has_content(&self) -> bool {
        self.text.is_some() || self.content.is_some()
    }

    pub(super) fn additional_height(&self, role: TypeRole) -> f32 {
        if self.content.is_some() || self.text.is_none() {
            return 0.;
        }
        self.max_lines.unwrap_or(1).saturating_sub(1) as f32
            * role.style(TypographyEmphasis::Baseline).line_height
    }

    pub(super) fn render(
        self,
        expanded: bool,
        role: TypeRole,
        style: HeadingStyle,
        window: &gpui::Window,
        cx: &gpui::App,
    ) -> Option<Div> {
        use gpui::ParentElement;
        let alignment = self
            .alignment
            .unwrap_or(style.alignment)
            .resolve(style.direction);
        let label = text_style(
            div().min_w_0().text_align(alignment),
            role.style(TypographyEmphasis::Baseline),
            style.font_scale,
            window,
            cx,
        )
        .text_color(style.color);
        if let Some(build) = self.content {
            return Some(aligned_content(alignment).child(label.child(build(expanded))));
        }
        let lines = self.max_lines.unwrap_or(1);
        self.text.map(|text| {
            label
                .when_else(lines == 1, Styled::truncate, |label| {
                    label.line_clamp(lines)
                })
                .child(text)
        })
    }
}

pub(super) fn aligned_content(alignment: TextAlign) -> Div {
    div()
        .flex()
        .min_w_0()
        .when(alignment == TextAlign::Center, Styled::justify_center)
        .when(alignment == TextAlign::Right, Styled::justify_end)
}

#[derive(Clone, Copy)]
pub(super) struct HeadingStyle {
    pub(super) direction: Direction,
    pub(super) alignment: AppBarAlignment,
    pub(super) font_scale: f32,
    pub(super) color: Hsla,
}
