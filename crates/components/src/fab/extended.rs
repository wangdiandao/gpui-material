use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AvailableSpace, ElementId, Empty, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Size, Styled, Window, canvas, div, point, px, size,
};
use gpui_material_foundation::{OverlayMotion, animate, current_material_theme};
use gpui_material_theme::{Direction, MotionKind, MotionSpeed, TypographyEmphasis};

use crate::action::icon_slot;

use super::{
    fab_builders,
    types::{FabAppearance, FabContent},
};

/// An elevated action with a label or custom content and an optional icon.
/// All presentations share one action, accessible name, and focus handle.
#[derive(IntoElement)]
pub struct ExtendedFab {
    content: FabContent,
    icon: Option<AnyElement>,
    custom_content: Option<AnyElement>,
    expanded: bool,
    appearance: FabAppearance,
}

impl ExtendedFab {
    /// Creates an action with a leading icon and a label.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            icon: Some(icon.into_any_element()),
            ..Self::text(id, label)
        }
    }

    /// Creates a text-only action. The label also supplies its accessible name.
    pub fn text(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            content: FabContent::new(id, label),
            icon: None,
            custom_content: None,
            expanded: true,
            appearance: FabAppearance::default(),
        }
    }

    fab_builders!();

    /// Replaces the rendered label with decorative content. The constructor's
    /// accessible name and optional icon still belong to the action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.custom_content = Some(content.into_any_element());
        self
    }

    /// Applies the caller's expansion state. An icon-bearing action collapses
    /// to a square. Iconless actions retain their content in both states.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (height, icon, _, padding, gap, role) = self.content.size.metrics();
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let extent = theme.layout.density.interactive_size(height);
        let visual_height = theme.layout.density.visual_size(height);
        let scheme = theme.motion;
        let motion = self.appearance.motion(
            &self.content.id,
            extent.max(f32::from(window.viewport_size().width)),
            window,
            cx,
        );
        let action = self.content.action(Empty);
        let interactive = self.appearance.interactive(&action, motion, window, cx);
        if motion.alpha == 0. {
            return Empty.into_any_element();
        }
        let (progress, opacity) = if self.icon.is_some() {
            let reduced = cx.reduce_motion();
            let mut sample = |channel, kind| {
                animate(
                    (action.id.clone(), channel),
                    f32::from(self.expanded),
                    scheme.spec(kind, MotionSpeed::Fast, reduced),
                    window,
                    cx,
                )
                .clamp(0., 1.)
            };
            (
                sample("expansion", MotionKind::Spatial),
                sample("label-opacity", MotionKind::Effects),
            )
        } else {
            (1., 1.)
        };
        let (icon_size, padding, gap) = if self.icon.is_some() {
            let collapsed_padding = (height - icon) / 2.;
            (
                icon,
                collapsed_padding + (padding - collapsed_padding) * progress,
                gap * progress,
            )
        } else {
            (0., padding, 0.)
        };
        let ink = self
            .appearance
            .style
            .colors
            .content(action.disabled, false)
            .unwrap_or_else(|| action.style.foreground(action.disabled, window, cx));
        let typography = role.style(TypographyEmphasis::Baseline);
        let mut custom_content = self.custom_content;
        let mut custom_measured = None;
        let mut pre_shaped = None;
        let label_width = if progress == 0. {
            px(0.)
        } else if let Some(content) = &mut custom_content {
            let measured = window.with_text_style(
                Some(gpui::TextStyleRefinement {
                    color: Some(ink),
                    font_size: Some(px(typography.size).into()),
                    line_height: Some(px(typography.line_height).into()),
                    ..Default::default()
                }),
                |window| {
                    content.layout_as_root(
                        size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                        window,
                        cx,
                    )
                },
            );
            custom_measured = Some(measured);
            measured.width
        } else if let Some((clean, shaped)) =
            crate::text::shape_single_line(action.label.clone(), typography, 1., ink, window, cx)
        {
            let width = shaped.line().width.ceil();
            pre_shaped = Some((clean, shaped));
            width
        } else {
            px(0.)
        };
        let fixed_width = px(icon_size + gap + padding * 2.);
        let width = (fixed_width + label_width * progress).max(px(extent));
        // Keep the nominal width through visibility scaling. Only the resolved
        // action is prepainted, so measurement creates no hitboxes or semantics.
        let control = canvas(
            move |bounds, window, cx| {
                if motion.scale == 0. {
                    return None;
                }
                let scale = if f32::from(bounds.size.width * motion.scale).is_finite() {
                    motion.scale
                } else {
                    1.
                };
                let motion = OverlayMotion { scale, ..motion };
                let pivot = self.appearance.animation.alignment.pivot(direction);
                let assigned_width = bounds.size.width * scale;
                let mut row = div()
                    .flex()
                    .items_center()
                    .w(assigned_width)
                    .px(px(padding * scale))
                    .gap(px(gap * scale))
                    .when(direction == Direction::Rtl, Styled::flex_row_reverse);
                if let Some(icon) = self.icon {
                    row = row.child(icon_slot(icon, icon_size * scale));
                }
                let mut action = action;
                let available = (bounds.size.width - fixed_width).max(px(0.));
                let label_width = (label_width * progress).min(available) * scale;
                if label_width > px(0.) {
                    if let Some(content) = custom_content {
                        row = row.child(custom_label(
                            content,
                            custom_measured,
                            label_width,
                            px(visual_height * scale),
                            opacity,
                        ));
                    } else if scale == 1.
                        && let Some(shaped) = pre_shaped
                    {
                        row = row.child(crate::text::single_line_with_shaped(
                            shaped,
                            typography,
                            1.,
                            ink,
                            (Some(label_width / progress), progress, opacity),
                            window,
                            cx,
                        ));
                    } else {
                        row = row.child(crate::text::single_line(
                            action.label.clone(),
                            typography,
                            scale,
                            ink,
                            (Some(label_width / progress), progress, opacity),
                            window,
                            cx,
                        ));
                    }
                }
                action.content = row.into_any_element();
                let action = self
                    .appearance
                    .control(action, scale, interactive, window, cx);
                let mut target = div()
                    .w(assigned_width)
                    .opacity(motion.alpha)
                    .child(action)
                    .into_any_element();
                target.prepaint_as_root(
                    motion.origin(bounds, pivot),
                    bounds
                        .size
                        .map(|side| AvailableSpace::Definite(side * scale)),
                    window,
                    cx,
                );
                Some(target)
            },
            |_, target, window, cx| {
                if let Some(mut target) = target {
                    target.paint(window, cx);
                }
            },
        );
        div()
            .w(width)
            .h(px(extent))
            .min_w(px(extent))
            .min_h(px(extent))
            .max_w_full()
            .child(control.size_full())
            .into_any_element()
    }
}

fn custom_label(
    mut content: AnyElement,
    measured: Option<Size<Pixels>>,
    width: Pixels,
    height: Pixels,
    opacity: f32,
) -> gpui::Div {
    div()
        .w(width)
        .h(height)
        .min_w_0()
        .overflow_hidden()
        .opacity(opacity)
        .child(
            canvas(
                move |bounds, window, cx| {
                    // Reuse the initial MaxContent layout when it fits inside the allocated
                    // slot; only re-layout when constrained tighter by available bounds.
                    let actual_height = match measured {
                        Some(size)
                            if size.width <= bounds.size.width
                                && size.height <= bounds.size.height =>
                        {
                            size.height
                        }
                        _ => {
                            content
                                .layout_as_root(
                                    bounds.size.map(AvailableSpace::Definite),
                                    window,
                                    cx,
                                )
                                .height
                        }
                    };
                    let origin = point(
                        bounds.origin.x,
                        bounds.origin.y + ((bounds.size.height - actual_height) / 2.).max(px(0.)),
                    );
                    content.prepaint_at(origin, window, cx);
                    content
                },
                |_, mut content, window, cx| content.paint(window, cx),
            )
            .size_full(),
        )
}
