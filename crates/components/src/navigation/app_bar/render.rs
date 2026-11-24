use gpui::{
    App, Div, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{animate, color, current_material_theme, resolve_edges};
use gpui_material_theme::{
    ColorRole, Direction, LogicalEdges, MotionKind, MotionSpec, MotionSpeed, TypeRole,
};

use super::{
    AppBar, AppBarAlignment, AppBarScrollState, AppBarVariant,
    scroll::length,
    types::{HeadingStyle, aligned_content},
};

fn apply_edges(row: Div, edges: LogicalEdges, direction: Direction) -> Div {
    let resolved = resolve_edges(edges, direction);
    row.pl(resolved.left)
        .pr(resolved.right)
        .pt(resolved.top)
        .pb(resolved.bottom)
}

impl AppBar {
    fn scroll_position(&self, limit: f32, window: &mut Window, cx: &mut App) -> (f32, f32) {
        if self.scroll.is_none() && self.scroll_state.is_none() {
            return (0., 0.);
        }
        let state = self.scroll_state.clone().unwrap_or_else(|| {
            window.use_keyed_state((self.id.clone(), "app-bar-scroll"), cx, |_, _| {
                AppBarScrollState::new()
            })
        });
        let content_offset = self
            .scroll
            .as_ref()
            .map(|handle| length(-handle.offset().y).unwrap_or(0.));
        let position = state.update(cx, |state, _| {
            state.prepare(
                content_offset,
                limit,
                self.scroll_behavior,
                self.scroll_enabled,
            )
        });
        let motion = if position.settling && !cx.reduce_motion() {
            self.scroll_motion.unwrap_or_else(|| {
                current_material_theme(window, cx).motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    false,
                )
            })
        } else {
            MotionSpec::Instant
        };
        let fraction = animate(
            (self.id.clone(), "app-bar-height"),
            position.target,
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        if state.update(cx, |state, _| state.display(position, fraction))
            && self.scroll_state.is_some()
        {
            let state = state.downgrade();
            window.defer(cx, move |_, cx| {
                let _ = state.update(cx, |_, cx| cx.notify());
            });
        }
        (fraction, position.overlap)
    }
}

impl RenderOnce for AppBar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let collapsed = self
            .collapsed_height
            .unwrap_or_else(|| theme.layout.density.interactive_size(64.));
        let two_rows = self.variant.two_rows();
        let (expanded_title_role, expanded_subtitle_role) = self.variant.expanded_roles();
        let text_scale = theme.typography.text_scale();
        let default_expansion = if two_rows {
            (self.variant.expansion(self.subtitle.has_content())
                + self.title.additional_height(expanded_title_role)
                + self.subtitle.additional_height(expanded_subtitle_role))
                * (self.font_scale * text_scale).max(1.)
        } else {
            0.
        };
        let limit = self
            .expanded_height
            .map_or(default_expansion, |height| (height - collapsed).max(0.));
        let (fraction, overlap) = self.scroll_position(limit, window, cx);
        let remaining = limit * (1. - fraction);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let centered = self.variant == AppBarVariant::Centered;
        let has_search = self.search.is_some();
        let expanded_row = two_rows && remaining > 0. && !has_search;
        let expanded_heading = expanded_row && remaining > limit / 2.;
        let default_alignment = if centered {
            AppBarAlignment::Center
        } else {
            AppBarAlignment::Start
        };
        let role = |role| color(theme.colors.color(role));
        let surface = self
            .colors
            .container
            .unwrap_or_else(|| role(ColorRole::Surface));
        let scrolled = self
            .colors
            .scrolled_container
            .unwrap_or_else(|| role(ColorRole::SurfaceContainer));
        let background = match overlap {
            0. => surface,
            1. => scrolled,
            _ => crate::control_style::mix_color(surface, scrolled, overlap),
        };
        let title_color = self
            .colors
            .title
            .unwrap_or_else(|| role(ColorRole::OnSurface));
        let subtitle_color = self
            .colors
            .subtitle
            .unwrap_or_else(|| role(ColorRole::OnSurfaceVariant));
        let mut navigation = div()
            .flex()
            .items_center()
            .min_w_0()
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse);
        if let Some(navigation_color) = self.colors.navigation {
            navigation = navigation.text_color(navigation_color);
        }
        let search_logo = has_search.then(|| self.logo.take()).flatten();
        navigation = navigation.children(self.navigation).children(search_logo);
        let mut actions = div()
            .flex()
            .justify_end()
            .min_w_0()
            .when(rtl, Styled::flex_row_reverse)
            .children(self.actions);
        if let Some(actions_color) = self.colors.actions {
            actions = actions.text_color(actions_color);
        }
        let label = self.title.text.clone().unwrap_or_default();
        let title_alignment = self
            .title
            .alignment
            .unwrap_or(default_alignment)
            .resolve(direction);
        let heading =
            if has_search {
                None
            } else if !two_rows && self.logo.is_some() {
                Some(
                    aligned_content(title_alignment)
                        .text_color(title_color)
                        .children(self.logo.take()),
                )
            } else {
                let heading_style = HeadingStyle {
                    direction,
                    alignment: default_alignment,
                    font_scale: self.font_scale,
                    color: title_color,
                };
                let (title_role, subtitle_role) = if expanded_heading {
                    (expanded_title_role, expanded_subtitle_role)
                } else {
                    (TypeRole::TitleLarge, TypeRole::LabelMedium)
                };
                Some(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .children(expanded_heading.then(|| self.logo.take()).flatten().map(
                            |logo| {
                                aligned_content(title_alignment)
                                    .text_color(title_color)
                                    .child(logo)
                            },
                        ))
                        .children(self.title.render(
                            expanded_heading,
                            title_role,
                            heading_style,
                            window,
                            cx,
                        ))
                        .children(self.subtitle.render(
                            expanded_heading,
                            subtitle_role,
                            HeadingStyle {
                                color: subtitle_color,
                                ..heading_style
                            },
                            window,
                            cx,
                        )),
                )
            };
        let (top_heading, lower_heading) = if expanded_heading {
            (None, heading)
        } else {
            (heading, None)
        };
        let center_slots = centered && !has_search;
        let row = div()
            .flex()
            .items_center()
            .min_h(px(if two_rows || has_search {
                collapsed
            } else {
                collapsed + remaining
            }))
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse)
            .child(navigation.when(center_slots, |lane| lane.flex_1()))
            .child(
                div()
                    .min_w_0()
                    .when_else(center_slots, |lane| lane.w_1_2(), |lane| lane.flex_1())
                    .children(self.search)
                    .children(top_heading),
            )
            .child(actions.when(center_slots, |lane| lane.flex_1()));
        let primary = apply_edges(
            row,
            self.padding.unwrap_or(LogicalEdges::symmetric(4., 0.)),
            direction,
        );
        let lower = expanded_row.then(|| {
            let body = div()
                .h(px(remaining))
                .overflow_hidden()
                .flex()
                .flex_col()
                .justify_end()
                .children(lower_heading)
                .children(
                    self.second_row
                        .filter(|_| self.variant == AppBarVariant::TwoRows && expanded_heading),
                );
            apply_edges(
                body,
                self.padding.unwrap_or(LogicalEdges::new(0., 16., 4., 16.)),
                direction,
            )
        });
        div()
            .id(self.id)
            .test_support()
            .role(Role::Toolbar)
            .aria_label(label)
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(background)
            .child(primary)
            .children(lower)
    }
}
