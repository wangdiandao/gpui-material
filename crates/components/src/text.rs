use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, TextAlign, Window, canvas, div, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{color, shape_text, window_material_theme, wrapped_text};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypeStyle, TypographyEmphasis};

/// Single-line Material text. Measurement and painting share Foundation's
/// shaped glyphs, including font fallback and letter spacing.
#[derive(IntoElement)]
pub struct Text {
    value: SharedString,
    id: Option<ElementId>,
    role: TypeRole,
    emphasis: TypographyEmphasis,
    font_scale: f32,
    color: Option<ColorRole>,
    max_width: Option<gpui::Pixels>,
    wrap: bool,
}

impl Text {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            id: None,
            role: TypeRole::BodyLarge,
            emphasis: TypographyEmphasis::Baseline,
            font_scale: 1.,
            color: None,
            max_width: None,
            wrap: false,
        }
    }

    /// Supplies a stable ID for this label within its parent.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(mut self, role: TypeRole) -> Self {
        self.role = role;
        self
    }

    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasis = if emphasized {
            TypographyEmphasis::Emphasized
        } else {
            TypographyEmphasis::Baseline
        };
        self
    }

    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self
    }

    pub fn truncate(mut self, max_width: gpui::Pixels) -> Self {
        self.max_width = Some(max_width.max(px(0.)));
        self.wrap = false;
        self
    }

    /// Wraps at the available width and preserves explicit line breaks.
    /// This uses GPUI's native Unicode layout, whose pinned version does not
    /// expose custom tracking for wrapped text. Single-line text retains tracking.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self.max_width = None;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tint = self.color.map_or_else(
            || window.text_style().color,
            |role| color(window_material_theme(window, cx).colors.color(role)),
        );
        let text = if self.wrap {
            wrapped_text(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                window,
                cx,
            )
        } else {
            single_line(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                (self.max_width, 1., 1.),
                window,
                cx,
            )
        };
        text.id(self.id.unwrap_or_else(|| "material-text".into()))
            .test_support()
            .role(Role::Label)
            .aria_label(self.value)
    }
}

pub(crate) fn single_line(
    value: SharedString,
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    (max_width, expansion, opacity): (Option<Pixels>, f32, f32),
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let clean = if value.contains(['\r', '\n']) {
        SharedString::from(value.replace(['\r', '\n'], " "))
    } else {
        value
    };
    let theme = window_material_theme(window, cx);
    let typography = theme.typography.clone();
    let align = if theme.layout.direction == Direction::Rtl {
        TextAlign::Right
    } else {
        TextAlign::Left
    };
    let shaped = shape_text(
        clean.clone(),
        style,
        &typography,
        font_scale,
        tint,
        window,
        cx,
    )
    .expect("single-line text");
    let width = shaped.line().width.ceil();
    let height = shaped.line_height();
    div()
        .min_w_0()
        .w(max_width.map_or(width, |max| width.min(max)) * expansion)
        .max_w_full()
        .h(height)
        .overflow_hidden()
        .opacity(opacity)
        .child(
            canvas(
                move |bounds, window, cx| {
                    let maximum = bounds.size.width;
                    if shaped.line().width <= maximum || expansion < 1. {
                        return Some(shaped);
                    }
                    let shape = |value: SharedString| {
                        shape_text(value, style, &typography, font_scale, tint, window, cx)
                            .expect("single-line text")
                    };
                    let ellipsis = shape("…".into());
                    if ellipsis.line().width > maximum {
                        return None;
                    }
                    // Use shaped cluster boundaries to preserve ligatures and combining text.
                    let mut ends = shaped
                        .line()
                        .runs
                        .iter()
                        .flat_map(|run| &run.glyphs)
                        .map(|glyph| glyph.index)
                        .filter(|&index| clean.is_char_boundary(index))
                        .collect::<Vec<_>>();
                    ends.sort_unstable();
                    ends.dedup();
                    ends.into_iter()
                        .rev()
                        .filter(|&end| end > 0)
                        .map(|end| shape(format!("{}…", &clean[..end]).into()))
                        .find(|line| line.line().width <= maximum)
                        .or(Some(ellipsis))
                },
                move |bounds, shaped, window, cx| {
                    if let Some(shaped) = shaped {
                        let _ =
                            shaped.paint(bounds.origin, align, Some(bounds.size.width), window, cx);
                    }
                },
            )
            .size_full(),
        )
}
