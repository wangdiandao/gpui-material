use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, TextAlign, Transformation, Window, canvas,
    div, prelude::FluentBuilder, px, size, svg,
};
use gpui_material_foundation::{color, material_theme, shape_text};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

/// Single-line Material text. Measurement and painting share Foundation's
/// shaped glyphs, including font fallback and letter spacing.
#[derive(IntoElement)]
pub struct Text {
    value: SharedString,
    id: Option<ElementId>,
    role: TypeRole,
    emphasis: TypographyEmphasis,
    font_scale: f32,
    color: ColorRole,
    max_width: Option<gpui::Pixels>,
}

impl Text {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            id: None,
            role: TypeRole::BodyLarge,
            emphasis: TypographyEmphasis::Baseline,
            font_scale: 1.,
            color: ColorRole::OnSurface,
            max_width: None,
        }
    }

    /// Gives this label a stable accessibility node. Omit this for text
    /// whose parent already supplies the same accessible name.
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
        self.color = role;
        self
    }

    pub fn truncate(mut self, max_width: gpui::Pixels) -> Self {
        self.max_width = Some(max_width.max(px(0.)));
        self
    }
}

impl RenderOnce for Text {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = material_theme(cx);
        let role = self.role.style(self.emphasis);
        let tint = color(theme.colors.color(self.color));
        let clean = if self.value.contains(['\r', '\n']) {
            SharedString::from(self.value.replace(['\r', '\n'], " "))
        } else {
            self.value.clone()
        };
        let shape = |value: SharedString| {
            shape_text(
                value,
                role,
                &theme.typography,
                self.font_scale,
                tint,
                window,
                cx,
            )
            .expect("one line")
        };
        let mut shaped = shape(clean.clone());
        if let Some(maximum) = self.max_width
            && shaped.line().width > maximum
        {
            let ellipsis = shape("…".into());
            if ellipsis.line().width > maximum {
                shaped = shape("".into());
            } else {
                // Use shaped cluster boundaries, so truncation cannot split a
                // ligature or a combining sequence.
                let mut ends = shaped
                    .line()
                    .runs
                    .iter()
                    .flat_map(|run| run.glyphs.iter())
                    .map(|glyph| glyph.index)
                    .filter(|&ix| clean.is_char_boundary(ix))
                    .collect::<Vec<_>>();
                ends.push(0);
                ends.sort_unstable();
                ends.dedup();
                let fits =
                    |end| shape(format!("{}…", &clean[..end]).into()).line().width <= maximum;
                let ix = ends.partition_point(|&end| fits(end)).saturating_sub(1);
                shaped = shape(format!("{}…", &clean[..ends[ix]]).into());
            }
        }
        let width = shaped.line().width;
        let height = shaped.line_height();
        let align = if theme.layout.direction == Direction::Rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        };
        let text = div()
            .w(width)
            .max_w_full()
            .h(height)
            .overflow_hidden()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let _ =
                            shaped.paint(bounds.origin, align, Some(bounds.size.width), window, cx);
                    },
                )
                .size_full(),
            );
        match self.id {
            Some(id) => text
                .id(id)
                .role(Role::Label)
                .aria_label(self.value)
                .into_any_element(),
            None => text.into_any_element(),
        }
    }
}

/// A caller-provided SVG from the GPUI asset source, or a custom element.
/// SVG mirroring is opt-in for icons whose meaning is directional.
#[derive(IntoElement)]
pub struct Icon {
    content: IconContent,
    name: Option<(ElementId, SharedString)>,
    side: gpui::Pixels,
    color: ColorRole,
    mirror: bool,
}

enum IconContent {
    Svg(SharedString),
    Element(AnyElement),
}

impl Icon {
    pub fn svg(path: impl Into<SharedString>) -> Self {
        Self {
            content: IconContent::Svg(path.into()),
            name: None,
            side: px(24.),
            color: ColorRole::OnSurfaceVariant,
            mirror: false,
        }
    }

    pub fn custom(element: impl IntoElement) -> Self {
        Self {
            content: IconContent::Element(element.into_any_element()),
            ..Self::svg("")
        }
    }

    pub fn label(mut self, id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        self.name = Some((id.into(), name.into()));
        self
    }

    pub fn size(mut self, side: gpui::Pixels) -> Self {
        self.side = side.max(px(0.));
        self
    }

    pub fn color(mut self, color: ColorRole) -> Self {
        self.color = color;
        self
    }

    pub fn mirror_in_rtl(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = material_theme(cx);
        let tint = color(theme.colors.color(self.color));
        let rtl = self.mirror && theme.layout.direction == Direction::Rtl;
        let icon = match self.content {
            IconContent::Svg(path) => svg()
                .path(path)
                .size_full()
                .text_color(tint)
                .when(rtl, |icon| {
                    icon.with_transformation(Transformation::scale(size(-1., 1.)))
                })
                .into_any_element(),
            IconContent::Element(element) => element,
        };
        let icon = div()
            .size(self.side)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .text_color(tint)
            .child(icon);
        match self.name {
            Some((id, label)) => icon
                .id(id)
                .role(Role::Image)
                .aria_label(label)
                .into_any_element(),
            None => icon.into_any_element(),
        }
    }
}
