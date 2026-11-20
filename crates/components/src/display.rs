use gpui::{
    AnyElement, App, Axis, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_material_foundation::{color, current_material_theme, text_style};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

/// Decorative Material divider, with logical start and end insets.
#[derive(IntoElement)]
pub struct Divider {
    axis: Axis,
    thickness: gpui::Pixels,
    color_role: ColorRole,
    color_override: Option<gpui::Hsla>,
    start: gpui::Pixels,
    end: gpui::Pixels,
}

impl Divider {
    const DEFAULT_THICKNESS: gpui::Pixels = px(1.);

    pub fn horizontal() -> Self {
        Self {
            axis: Axis::Horizontal,
            thickness: Self::DEFAULT_THICKNESS,
            color_role: ColorRole::OutlineVariant,
            color_override: None,
            start: px(0.),
            end: px(0.),
        }
    }

    pub fn vertical() -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::horizontal()
        }
    }

    /// Sets the stroke thickness in logical pixels. Zero requests a one-device-pixel hairline.
    /// Negative or non-finite values use the default thickness of one logical pixel.
    pub fn thickness(mut self, thickness: gpui::Pixels) -> Self {
        self.thickness = if f32::from(thickness).is_finite() && thickness >= px(0.) {
            thickness
        } else {
            Self::DEFAULT_THICKNESS
        };
        self
    }

    /// Resolves the stroke color from the current window theme.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color_role = role;
        self.color_override = None;
        self
    }

    /// Sets a literal stroke color. A later call to `color` restores theme-role lookup.
    pub fn color_override(mut self, color: impl Into<gpui::Hsla>) -> Self {
        self.color_override = Some(color.into());
        self
    }

    pub fn inset(mut self, start: gpui::Pixels, end: gpui::Pixels) -> Self {
        self.start = start.max(px(0.));
        self.end = end.max(px(0.));
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = self
            .color_override
            .unwrap_or_else(|| tone(self.color_role, window, cx));
        let thickness = if self.thickness == px(0.) {
            px(window.scale_factor().recip())
        } else {
            self.thickness
        };
        let (left, right) = current_material_theme(window, cx)
            .layout
            .horizontal(self.start, self.end);
        if self.axis == Axis::Vertical {
            div()
                .w(thickness)
                .h_full()
                .pt(self.start)
                .pb(self.end)
                .child(div().size_full().bg(fill))
        } else {
            div()
                .w_full()
                .h(thickness)
                .pl(left)
                .pr(right)
                .child(div().size_full().bg(fill))
        }
    }
}

/// A standalone or anchored Material badge. For anchored badges, include the
/// badge's information in the host's accessible name.
#[derive(IntoElement)]
pub struct Badge {
    host: Option<AnyElement>,
    content: BadgeContent,
    container_color: Option<Hsla>,
    content_color: Option<Hsla>,
    accessible_label: Option<(ElementId, SharedString)>,
}

enum BadgeContent {
    Dot,
    Text(SharedString),
    Custom(ElementId, AnyElement),
}

impl Badge {
    /// A small badge that participates in the surrounding layout.
    pub fn standalone() -> Self {
        Self {
            host: None,
            content: BadgeContent::Dot,
            container_color: None,
            content_color: None,
            accessible_label: None,
        }
    }

    /// Anchors a small badge to the host's upper trailing edge.
    pub fn dot(host: impl IntoElement) -> Self {
        Self {
            host: Some(host.into_any_element()),
            ..Self::standalone()
        }
    }

    pub fn label(host: impl IntoElement, text: impl Into<SharedString>) -> Self {
        Self {
            content: BadgeContent::Text(text.into()),
            ..Self::dot(host)
        }
    }

    pub fn count(host: impl IntoElement, count: usize, maximum: usize) -> Self {
        Self::label(
            host,
            if count > maximum {
                format!("{maximum}+")
            } else {
                count.to_string()
            },
        )
    }

    /// Uses large badge styling with caller-supplied visual content.
    ///
    /// Use a stable ID within the parent and keep the content noninteractive.
    /// The badge or its host supplies the accessible name.
    pub fn content(mut self, id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        self.content = BadgeContent::Custom(id.into(), content.into_any_element());
        self
    }

    /// Sets the background color, which defaults to the window theme's `Error` role.
    pub fn container_color(mut self, color: impl Into<Hsla>) -> Self {
        self.container_color = Some(color.into());
        self
    }

    /// Sets the inherited content color, which defaults to the window theme's `OnError` role.
    pub fn content_color(mut self, color: impl Into<Hsla>) -> Self {
        self.content_color = Some(color.into());
        self
    }

    /// Names a badge for assistive technology without adding a focus stop.
    /// Anchored badges normally use their host's accessible name instead.
    pub fn accessible_label(
        mut self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.accessible_label = Some((id.into(), label.into()));
        self
    }

    pub(crate) fn take_accessible_label(&mut self) -> Option<SharedString> {
        self.accessible_label.take().map(|(_, label)| label)
    }

    pub(crate) fn attach_to(mut self, host: impl IntoElement) -> Self {
        self.host = Some(host.into_any_element());
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let large = !matches!(self.content, BadgeContent::Dot);
        let side = px(if large { 16. } else { 6. });
        let container_color = self
            .container_color
            .unwrap_or_else(|| tone(ColorRole::Error, window, cx));
        let content_color = self
            .content_color
            .unwrap_or_else(|| tone(ColorRole::OnError, window, cx));
        let typography = TypeRole::LabelSmall.style(TypographyEmphasis::Baseline);
        let overlap = px(if large { 12. } else { 6. });
        let mark = div()
            .min_w(side)
            .rounded_full()
            .when(large, |badge| badge.px(px(4.)))
            .when(self.host.is_some(), |badge| {
                badge
                    .absolute()
                    .top(if large { px(-2.) } else { px(0.) })
                    .when_else(
                        current_material_theme(window, cx).layout.direction == Direction::Rtl,
                        |badge| badge.right(gpui::relative(1.)).mr(-overlap),
                        |badge| badge.left(gpui::relative(1.)).ml(-overlap),
                    )
            })
            .bg(container_color)
            .text_color(content_color)
            .flex()
            .items_center()
            .justify_center();
        let mark = match self.content {
            BadgeContent::Dot => mark.h(side).w(side),
            BadgeContent::Text(text) => mark.h(side).child(crate::text::single_line(
                text,
                typography,
                1.,
                content_color,
                (None, 1., 1.),
                window,
                cx,
            )),
            BadgeContent::Custom(id, content) => {
                text_style(mark.min_h(side), typography, 1., window, cx).child(
                    div()
                        .id(id)
                        .role(Role::Group)
                        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                        .child(content),
                )
            }
        };
        let mark = if let Some((id, label)) = self.accessible_label {
            mark.id(id)
                .role(Role::Label)
                .aria_label(label)
                .into_any_element()
        } else {
            mark.into_any_element()
        };
        if let Some(host) = self.host {
            div().relative().child(host).child(mark)
        } else {
            div().flex().items_center().child(mark)
        }
    }
}

fn tone(role: ColorRole, window: &Window, cx: &App) -> gpui::Hsla {
    color(current_material_theme(window, cx).colors.color(role))
}
