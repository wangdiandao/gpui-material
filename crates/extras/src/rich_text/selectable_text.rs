use std::{
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use gpui::{
    App, Bounds, ElementId, FontWeight, HighlightStyle, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    Styled, TextAlign, TextStyleRefinement, WhiteSpace, Window, div, px,
};
use gpui_base::{SelectableText as BaseSelectableText, TestSupportExt};
use gpui_material_foundation::{color, current_material_theme, font_for_style};
use gpui_material_theme::{ColorRole, Direction, LayoutPolicy, TypeRole, TypographyEmphasis};

static NEXT_GROUP_ORDER_BASE: AtomicU64 = AtomicU64::new(1);

/// Shared reading-order group coordinating multiple [`SelectableText`] elements
/// that form one continuous selectable document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectableTextGroup {
    order_base: u64,
    label: SharedString,
}

impl SelectableTextGroup {
    /// Creates a new shared selection group with a unique document-order namespace.
    pub fn new(label: impl Into<SharedString>, _cx: &mut App) -> Self {
        let slot = NEXT_GROUP_ORDER_BASE.fetch_add(1, Ordering::Relaxed);
        Self {
            order_base: slot.saturating_mul(1_000_000),
            label: label.into(),
        }
    }

    /// Returns the group label supplied at construction.
    pub fn label(&self) -> &SharedString {
        &self.label
    }
}

type RevealCallback = Box<dyn Fn(Bounds<Pixels>, &mut Window, &mut App)>;

/// Typography-aware Material 3 text supporting pointer drag selection,
/// `Ctrl/Cmd+C` clipboard copy (through `MaterialRoot`), and Material
/// `PrimaryContainer` selection highlights.
#[derive(IntoElement)]
pub struct SelectableText {
    id: ElementId,
    value: SharedString,
    role: TypeRole,
    emphasis: TypographyEmphasis,
    font_scale: f32,
    color: Option<ColorRole>,
    color_override: Option<Hsla>,
    selection_color: Option<Hsla>,
    align: Option<TextAlign>,
    wrap: bool,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    group: Option<SelectableTextGroup>,
    document_order: u64,
    reveal_range: Option<Range<usize>>,
    on_reveal: Option<RevealCallback>,
    accessible_label: Option<SharedString>,
}

impl SelectableText {
    /// Creates selectable text with the default [`TypeRole::BodyLarge`] role and wrapping enabled.
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            role: TypeRole::BodyLarge,
            emphasis: TypographyEmphasis::Baseline,
            font_scale: 1.0,
            color: None,
            color_override: None,
            selection_color: None,
            align: None,
            wrap: true,
            highlights: Vec::new(),
            group: None,
            document_order: 0,
            reveal_range: None,
            on_reveal: None,
            accessible_label: None,
        }
    }

    /// Creates selectable text styled with [`TypeRole::DisplayLarge`].
    pub fn display_large(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::DisplayLarge)
    }

    /// Creates selectable text styled with [`TypeRole::DisplayMedium`].
    pub fn display_medium(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::DisplayMedium)
    }

    /// Creates selectable text styled with [`TypeRole::DisplaySmall`].
    pub fn display_small(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::DisplaySmall)
    }

    /// Creates selectable text styled with [`TypeRole::HeadlineLarge`].
    pub fn headline_large(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::HeadlineLarge)
    }

    /// Creates selectable text styled with [`TypeRole::HeadlineMedium`].
    pub fn headline_medium(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::HeadlineMedium)
    }

    /// Creates selectable text styled with [`TypeRole::HeadlineSmall`].
    pub fn headline_small(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::HeadlineSmall)
    }

    /// Creates selectable text styled with [`TypeRole::TitleLarge`].
    pub fn title_large(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::TitleLarge)
    }

    /// Creates selectable text styled with [`TypeRole::TitleMedium`].
    pub fn title_medium(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::TitleMedium)
    }

    /// Creates selectable text styled with [`TypeRole::TitleSmall`].
    pub fn title_small(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::TitleSmall)
    }

    /// Creates selectable text styled with [`TypeRole::BodyLarge`].
    pub fn body_large(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::BodyLarge)
    }

    /// Creates selectable text styled with [`TypeRole::BodyMedium`].
    pub fn body_medium(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::BodyMedium)
    }

    /// Creates selectable text styled with [`TypeRole::BodySmall`].
    pub fn body_small(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::BodySmall)
    }

    /// Creates selectable text styled with [`TypeRole::LabelLarge`].
    pub fn label_large(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::LabelLarge)
    }

    /// Creates selectable text styled with [`TypeRole::LabelMedium`].
    pub fn label_medium(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::LabelMedium)
    }

    /// Creates selectable text styled with [`TypeRole::LabelSmall`].
    pub fn label_small(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::new(id, value).role(TypeRole::LabelSmall)
    }

    /// Sets the Material typography scale role. Defaults to [`TypeRole::BodyLarge`].
    pub fn role(mut self, role: TypeRole) -> Self {
        self.role = role;
        self
    }

    /// Selects emphasized typography weight when `true`, or baseline weight when `false`.
    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasis = if emphasized {
            TypographyEmphasis::Emphasized
        } else {
            TypographyEmphasis::Baseline
        };
        self
    }

    /// Applies an additional font-size multiplier on top of the active theme's typography scale.
    /// Non-finite or non-positive scales are ignored.
    pub fn font_scale(mut self, scale: f32) -> Self {
        if scale.is_finite() && scale > 0.0 {
            self.font_scale = scale;
        }
        self
    }

    /// Resolves the text color from the active theme's semantic color role.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self.color_override = None;
        self
    }

    /// Sets a literal text color. A later call to [`Self::color`] restores theme-role lookup.
    pub fn color_override(mut self, color: impl Into<Hsla>) -> Self {
        self.color_override = Some(color.into());
        self
    }

    /// Sets the text color to [`ColorRole::OnSurfaceVariant`].
    pub fn muted(self) -> Self {
        self.color(ColorRole::OnSurfaceVariant)
    }

    /// Overrides the selection highlight background. Defaults to Material
    /// `PrimaryContainer` with translucent wash for readability.
    pub fn selection_color(mut self, color: impl Into<Hsla>) -> Self {
        self.selection_color = Some(color.into());
        self
    }

    /// Sets explicit horizontal text alignment. Defaults to the theme's logical start edge.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
        self
    }

    /// Sets whether text wraps at the available width (defaults to `true`).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Applies ordered, non-overlapping UTF-8 byte-range highlights (such as search matches).
    pub fn highlights(
        mut self,
        highlights: impl IntoIterator<Item = (Range<usize>, HighlightStyle)>,
    ) -> Self {
        self.highlights = highlights.into_iter().collect();
        self
    }

    /// Joins a shared [`SelectableTextGroup`] so drag selection can span multiple elements.
    pub fn group(mut self, group: &SelectableTextGroup) -> Self {
        self.group = Some(group.clone());
        self
    }

    /// Sets the reading order of this run within its [`SelectableTextGroup`].
    pub fn document_order(mut self, order: u64) -> Self {
        self.document_order = order;
        self
    }

    /// Requests that the line containing the start of `range` (UTF-8 byte offset) scroll into view.
    pub fn reveal_range(mut self, range: Range<usize>) -> Self {
        self.reveal_range = Some(range);
        self
    }

    /// Receives the bounds of a revealed line for custom scroll containers.
    pub fn on_reveal(
        mut self,
        handler: impl Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reveal = Some(Box::new(handler));
        self
    }

    /// Overrides the accessible label (defaults to the text value itself).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

impl RenderOnce for SelectableText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let style = LayoutPolicy::scale_text(
            theme.typography.style(self.role, self.emphasis),
            self.font_scale,
        );
        let font = font_for_style(&theme.typography, style, cx);
        let tint = self.color_override.unwrap_or_else(|| {
            self.color.map_or_else(
                || color(theme.colors.color(ColorRole::OnSurface)),
                |role| color(theme.colors.color(role)),
            )
        });
        let selection_bg = self.selection_color.unwrap_or_else(|| {
            color(theme.colors.color(ColorRole::PrimaryContainer)).opacity(0.75)
        });
        let align = self
            .align
            .unwrap_or(if theme.layout.direction == Direction::Rtl {
                TextAlign::Right
            } else {
                TextAlign::Left
            });

        let text_style = TextStyleRefinement {
            color: Some(tint),
            font_family: Some(font.family),
            font_features: Some(font.features),
            font_fallbacks: font.fallbacks,
            font_size: Some(px(style.size).into()),
            line_height: Some(px(style.line_height).into()),
            font_weight: Some(FontWeight(f32::from(style.weight))),
            font_style: Some(font.style),
            text_align: Some(align),
            white_space: Some(if self.wrap {
                WhiteSpace::Normal
            } else {
                WhiteSpace::Nowrap
            }),
            ..Default::default()
        };

        let run_id = (self.id.clone(), "run");
        let order = self.group.as_ref().map_or(self.document_order, |group| {
            group.order_base.saturating_add(self.document_order)
        });
        let mut base = BaseSelectableText::new(run_id, self.value.clone())
            .document_order(order)
            .text_style(text_style)
            .selection_color(selection_bg);

        if !self.highlights.is_empty() {
            base = base.highlights(self.highlights);
        }
        if let Some(range) = self.reveal_range {
            base = base.reveal_range(range);
        }
        if let Some(handler) = self.on_reveal {
            base = base.on_reveal(handler);
        }

        let accessible_name = self.accessible_label.unwrap_or_else(|| self.value.clone());
        let focus_handle = window
            .use_keyed_state((self.id.clone(), "selection-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        let focus_on_press = focus_handle.clone();

        div()
            .id(self.id)
            .test_support()
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus_on_press.focus(window, cx);
            })
            .role(Role::Label)
            .aria_label(accessible_name)
            .aria_value(self.value)
            .min_w_0()
            .cursor_text()
            .child(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectable_text_shorthands_and_muted_set_expected_defaults() {
        let cases = [
            (
                SelectableText::display_large("a", "x"),
                TypeRole::DisplayLarge,
            ),
            (
                SelectableText::headline_medium("b", "x"),
                TypeRole::HeadlineMedium,
            ),
            (SelectableText::title_small("c", "x"), TypeRole::TitleSmall),
            (SelectableText::body_medium("d", "x"), TypeRole::BodyMedium),
            (SelectableText::label_small("e", "x"), TypeRole::LabelSmall),
        ];
        for (text, role) in cases {
            assert_eq!(text.role, role);
            assert!(text.wrap);
        }
        let muted = SelectableText::body_small("m", "hint").muted().wrap(false);
        assert_eq!(muted.color, Some(ColorRole::OnSurfaceVariant));
        assert!(!muted.wrap);
    }
}
