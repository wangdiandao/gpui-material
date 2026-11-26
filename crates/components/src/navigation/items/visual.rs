use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, BorderStyle, Bounds, Corners, Edges, ElementId, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Role, StatefulInteractiveElement, Styled, TextAlign,
    Window, canvas, div, point, prelude::FluentBuilder, px, quad, size,
};
use gpui_material_foundation::{
    InteractionFeedback, InteractionState, animate, control_feedback::FeedbackState,
    current_material_theme, resolve_edges, text_style,
};
use gpui_material_theme::{
    CornerShape, Density, Direction, MotionKind, MotionSpec, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::Badge;

use super::{
    super::tabs::{TabIndicatorTarget, TabVariant, decorative_slot},
    NavigationItem, NavigationItemIconPosition, Presentation,
    palette::{ItemPalette, resolve_palette},
};

#[allow(clippy::too_many_arguments)]
pub(in crate::navigation) fn render_feedback_surface(
    feedback: Rc<RefCell<FeedbackState>>,
    interaction: InteractionState,
    corners: CornerShape,
    direction: Direction,
    disabled: bool,
    indicator: Option<(Hsla, f32, f32)>,
    state_layer_color: Hsla,
    ring: Hsla,
    layer: f32,
    reduced: bool,
) -> AnyElement {
    let geometry = feedback.clone();
    let paint_feedback = feedback;
    canvas(
        move |bounds, window, _| {
            if disabled {
                geometry.borrow_mut().set_shape(None, window);
            } else {
                geometry
                    .borrow_mut()
                    .set_rounded_shape(bounds, corners, direction, window);
            }
        },
        move |bounds, _, window, cx| {
            if let Some((color, alpha, expand)) = indicator {
                let alpha = alpha.clamp(0., 1.);
                let expand = expand.clamp(0., 1.);
                if alpha > 0.
                    && expand > 0.
                    && bounds.size.width > px(0.)
                    && bounds.size.height > px(0.)
                {
                    let indicator_width = bounds.size.width * expand;
                    let origin_x = bounds.origin.x + (bounds.size.width - indicator_width) * 0.5;
                    let indicator_bounds = Bounds::new(
                        point(origin_x, bounds.origin.y),
                        size(indicator_width, bounds.size.height),
                    );
                    let [top_left, top_right, bottom_right, bottom_left] = corners
                        .radii(
                            indicator_bounds.size.width.into(),
                            indicator_bounds.size.height.into(),
                            direction,
                        )
                        .map(px);
                    let fill = if alpha >= 1. {
                        color
                    } else {
                        color.opacity(alpha)
                    };
                    window.paint_quad(quad(
                        indicator_bounds,
                        Corners {
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                        },
                        fill,
                        px(0.),
                        gpui::transparent_black(),
                        BorderStyle::Solid,
                    ));
                }
            }
            if !disabled {
                let feedback = paint_feedback.borrow();
                if let Some(shape) = feedback.shape() {
                    // The outer navigation button owns the keyboard focus ring border.
                    let mut visual_interaction = interaction;
                    visual_interaction.focus_visible = false;
                    InteractionFeedback::new(visual_interaction, state_layer_color, ring)
                        .paint_with_opacity(
                            shape,
                            feedback.ripple(),
                            layer,
                            cx.background_executor().now(),
                            reduced,
                            window,
                        );
                }
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
    .into_any_element()
}

pub(super) struct ItemVisual {
    pub(super) presentation: Presentation,
    pub(super) selected: bool,
    pub(super) height: Pixels,
    pub(super) rtl: bool,
    pub(super) font_scale: f32,
    pub(super) tab_content: Option<Hsla>,
    pub(super) styled_item: bool,
    pub(super) custom_indicator: bool,
    pub(super) indicator_target: Option<Rc<TabIndicatorTarget>>,
    pub(super) feedback: Rc<RefCell<FeedbackState>>,
    pub(super) interaction: InteractionState,
    pub(super) ring: Hsla,
}

impl ItemVisual {
    #[allow(clippy::too_many_arguments)]
    fn build_label(
        &self,
        id: &ElementId,
        item: &mut NavigationItem,
        label_color: Hsla,
        horizontal: bool,
        tab: bool,
        rail: bool,
        window: &Window,
        cx: &App,
    ) -> gpui::Div {
        let text_role = if tab {
            TypeRole::TitleSmall
        } else if rail && horizontal {
            TypeRole::LabelLarge
        } else {
            TypeRole::LabelMedium
        };
        let label = text_style(
            div().min_w_0().truncate().text_color(label_color),
            text_role.style(TypographyEmphasis::Baseline),
            self.font_scale,
            window,
            cx,
        )
        .text_align(if !horizontal {
            TextAlign::Center
        } else if self.rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        });
        if let Some(content) = item.label_content.take() {
            if tab {
                label.child(decorative_slot(
                    (id.clone(), "visual-label").into(),
                    content,
                ))
            } else {
                label.child(
                    div()
                        .id((id.clone(), "visual-label"))
                        .role(Role::Group)
                        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                        .child(content),
                )
            }
        } else {
            label.child(item.label.clone())
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_icon(
        &self,
        item: &mut NavigationItem,
        badge: &mut Option<Badge>,
        palette: &ItemPalette,
        indicator_padding: Option<Edges<Pixels>>,
        pill_surface: Option<AnyElement>,
        horizontal: bool,
        tab: bool,
        density: Density,
    ) -> Option<gpui::Div> {
        let pill_height = px(density.visual_size(32.).max(24.));
        item.icon.take().map(|icon| {
            let icon = icon.inherit_color();
            let icon = if let Some(badge) = badge.take() {
                badge.attach_to(icon).into_any_element()
            } else {
                icon.into_any_element()
            };
            div()
                .relative()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_color(palette.icon_color)
                .when(!tab && !horizontal, |icon| {
                    icon.w(px(56.)).h(pill_height).rounded_full()
                })
                .when_some(indicator_padding.filter(|_| !horizontal), |icon, edges| {
                    icon.w(px(24.) + edges.left + edges.right)
                        .h(px(24.) + edges.top + edges.bottom)
                        .pl(edges.left)
                        .pr(edges.right)
                        .pt(edges.top)
                        .pb(edges.bottom)
                })
                .children(pill_surface)
                .child(icon)
        })
    }

    pub(super) fn render(
        self,
        id: &ElementId,
        item: &mut NavigationItem,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let theme = current_material_theme(window, cx);
        let density = theme.layout.density;
        let direction = theme.layout.direction;
        let reduced = cx.reduce_motion();
        let tab = self.presentation.is_tab();
        let rail = self.presentation.is_rail();
        let rail_padding = self.presentation.rail_options().and_then(|options| {
            item.indicator_padding.map(|(collapsed, expanded)| {
                if options.expanded {
                    expanded
                } else {
                    collapsed
                }
            })
        });
        let indicator_padding = rail_padding.map(|padding| resolve_edges(padding, direction));
        let horizontal = item.icon_position.map_or_else(
            || self.presentation.horizontal_label(),
            |position| position == NavigationItemIconPosition::Start,
        );
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let spatial = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let selection = animate(
            (id.clone(), "indicator"),
            f32::from(self.selected),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let expand = if tab {
            1.
        } else {
            animate(
                (id.clone(), "indicator-expand"),
                f32::from(self.selected),
                spatial,
                window,
                cx,
            )
            .clamp(0., 1.)
        };
        let layer = animate(
            (id.clone(), "state-layer"),
            self.interaction
                .layer_with_ripple(reduced)
                .map_or(0., |layer| layer.opacity()),
            if !item.disabled {
                effects
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .clamp(0., 1.);
        let palette = resolve_palette(
            self.presentation,
            selection,
            self.tab_content,
            self.styled_item,
            &theme,
            item,
            horizontal,
        );
        let has_icon = item.icon.is_some();
        let pill_on_icon = !tab && !horizontal && has_icon;
        let pill_on_content = !tab && (horizontal || !has_icon);
        let feedback_surface = render_feedback_surface(
            self.feedback.clone(),
            self.interaction,
            if tab {
                CornerShape::Radii([0.; 4])
            } else {
                CornerShape::Full
            },
            direction,
            item.disabled,
            (!tab).then_some((palette.selected_indicator, selection, expand)),
            palette.state_layer,
            self.ring,
            layer,
            reduced,
        );
        let mut feedback_surface = Some(feedback_surface);
        let label = self.build_label(
            id,
            item,
            palette.label_color,
            horizontal,
            tab,
            rail,
            window,
            cx,
        );
        let mut badge = item.badge.take();
        let icon = self.build_icon(
            item,
            &mut badge,
            &palette,
            indicator_padding,
            if pill_on_icon {
                feedback_surface.take()
            } else {
                None
            },
            horizontal,
            tab,
            density,
        );
        let label = if let Some(badge) = badge {
            div()
                .min_w_0()
                .max_w_full()
                .child(badge.attach_to(label))
                .into_any_element()
        } else {
            label.into_any_element()
        };
        let padding = px(if horizontal { 16. } else { 4. });
        let pill_height = px(density.visual_size(32.).max(24.));
        let horizontal_pill_height = if rail {
            px(density.interactive_size(56.))
        } else {
            px(density.visual_size(40.).max(32.))
        };
        let content = div()
            .relative()
            .min_w_0()
            .max_w_full()
            .flex()
            .items_center()
            .when_else(
                horizontal,
                |content| {
                    content
                        .gap(px(if rail || tab { 8. } else { 4. }))
                        .when(self.rtl, Styled::flex_row_reverse)
                        .when(!tab, |content| {
                            content.min_h(horizontal_pill_height).rounded_full()
                        })
                        .when(rail, |content| content.w_full())
                },
                |content| {
                    content
                        .flex_col()
                        .justify_center()
                        .gap(px(4.))
                        .when(pill_on_content, |content| {
                            content.min_h(pill_height).rounded_full()
                        })
                },
            )
            .px(padding)
            .when_some(
                indicator_padding.filter(|_| horizontal),
                |content, edges| {
                    content
                        .pl(edges.left)
                        .pr(edges.right)
                        .pt(edges.top)
                        .pb(edges.bottom)
                },
            )
            .children(if pill_on_content {
                feedback_surface.take()
            } else {
                None
            })
            .children(icon)
            .child(label)
            .children(self.indicator_target.map(|target| {
                canvas(
                    move |bounds, _, _| {
                        target
                            .content_width
                            .set((bounds.size.width - padding * 2.).max(px(0.)))
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0()
            }));
        div()
            .relative()
            .flex()
            .w_full()
            .min_w_0()
            .min_h(self.height - px(4.))
            .items_center()
            .justify_center()
            .children(feedback_surface)
            .child(content)
            .when(tab && !self.custom_indicator && selection > 0., |body| {
                body.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(
                            match self.presentation {
                                Presentation::Tabs(TabVariant::Primary, _) => div()
                                    .w_2_3()
                                    .h(px(3.))
                                    .rounded_tl(px(3.))
                                    .rounded_tr(px(3.)),
                                _ => div().w_full().h(px(2.)),
                            }
                            .bg(palette.tab_indicator.opacity(selection)),
                        ),
                )
            })
    }
}
