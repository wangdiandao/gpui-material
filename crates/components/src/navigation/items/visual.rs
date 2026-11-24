use std::rc::Rc;

use gpui::{
    App, Edges, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, Role,
    StatefulInteractiveElement, Styled, TextAlign, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{animate, current_material_theme, resolve_edges, text_style};
use gpui_material_theme::{Density, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis};

use crate::Badge;

use super::{
    super::tabs::{TabIndicatorTarget, TabVariant, decorative_slot},
    NavigationItem, NavigationItemIconPosition, Presentation,
    palette::{ItemPalette, resolve_palette},
};

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
        selection: f32,
        horizontal: bool,
        tab: bool,
        density: Density,
    ) -> Option<gpui::Div> {
        let pill_height = px(density.visual_size(32.).max(24.));
        item.icon.take().map(|icon| {
            let icon = if self.styled_item {
                icon.inherit_color()
            } else {
                icon.color(palette.icon_role)
            };
            let icon = if let Some(badge) = badge.take() {
                badge.attach_to(icon).into_any_element()
            } else {
                icon.into_any_element()
            };
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_color(palette.icon_color)
                .when(!tab && !horizontal, |icon| {
                    icon.w(px(56.))
                        .h(pill_height)
                        .rounded_full()
                        .bg(palette.selected_indicator.opacity(selection))
                })
                .when_some(indicator_padding.filter(|_| !horizontal), |icon, edges| {
                    icon.w(px(24.) + edges.left + edges.right)
                        .h(px(24.) + edges.top + edges.bottom)
                        .pl(edges.left)
                        .pr(edges.right)
                        .pt(edges.top)
                        .pb(edges.bottom)
                })
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
        let indicator_padding =
            rail_padding.map(|padding| resolve_edges(padding, theme.layout.direction));
        let horizontal = item.icon_position.map_or_else(
            || self.presentation.horizontal_label(),
            |position| position == NavigationItemIconPosition::Start,
        );
        let palette = resolve_palette(
            self.presentation,
            self.selected,
            self.tab_content,
            self.styled_item,
            &theme,
            item,
            horizontal,
        );
        let motion = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion());
        let selection = if self.custom_indicator {
            0.
        } else {
            animate(
                (id.clone(), "indicator"),
                f32::from(self.selected),
                motion,
                window,
                cx,
            )
            .clamp(0., 1.)
        };
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
            selection,
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
                            content
                                .min_h(horizontal_pill_height)
                                .rounded_full()
                                .bg(palette.selected_indicator.opacity(selection))
                        })
                        .when(rail, |content| content.w_full())
                },
                |content| content.flex_col().justify_center().gap(px(4.)),
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
