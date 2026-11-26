use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, color,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    current_material_theme,
};
use gpui_material_theme::{ColorRole, Direction};

use crate::Badge;

use super::{
    super::{
        rail::{NavigationRailArrangement, RailPresentation},
        tabs::{TabDecoration, TabIndicatorTarget, TabOptions, decorative_slot},
    },
    NavigationItem, NavigationItemColors, NavigationItemIconPosition, NavigationItems,
    Presentation, Select,
    state::{next_item, resolve_tab_stop, sync_focus_and_scroll, update_reveal_state},
    visual::ItemVisual,
};

#[allow(clippy::too_many_arguments)]
fn build_tab_decoration(
    row_id: &ElementId,
    items: &[NavigationItem],
    tabs: &mut Option<TabOptions>,
    presentation: Presentation,
    direction: Direction,
    selected: Option<&SharedString>,
    scrollable: bool,
    scroll: &ScrollHandle,
    indicator_color: Hsla,
    custom_indicator: bool,
    has_custom_divider: bool,
) -> Option<(usize, TabDecoration)> {
    let tabs = tabs.as_mut()?;
    let Presentation::Tabs(variant, _) = presentation else {
        return None;
    };
    let (index, item) = items
        .iter()
        .enumerate()
        .find(|(_, item)| selected == Some(&item.id))?;
    if !custom_indicator {
        return None;
    }
    let style = tabs.indicator_style.unwrap_or_default();
    let tint = item
        .colors
        .selected_indicator
        .or(style.color)
        .unwrap_or(indicator_color);
    Some((
        index,
        TabDecoration {
            row: row_id.clone(),
            selected: item.id.clone(),
            variant,
            direction,
            target: Rc::new(TabIndicatorTarget::default()),
            scroll: scrollable.then(|| scroll.clone()),
            style,
            content: tabs.indicator_content.take(),
            color: tint.opacity(if item.disabled { 0.38 } else { 1. }),
            has_default_divider: !has_custom_divider,
        },
    ))
}

struct DestinationRowContext<'a> {
    row_id: &'a ElementId,
    presentation: Presentation,
    options: Option<RailPresentation>,
    bar_colors: Option<super::NavigationBarColors>,
    tab_content: Option<Hsla>,
    selected: Option<&'a SharedString>,
    tab_stop: Option<&'a SharedString>,
    focus: &'a [FocusHandle],
    on_select: Option<&'a Select>,
    height: Pixels,
    min_tab_width: Pixels,
    centered_widths: Option<(gpui::DefiniteLength, gpui::DefiniteLength)>,
    font_scale: f32,
    rtl: bool,
    rail: bool,
    tab: bool,
    custom_indicator: bool,
    indicator_target: Option<(usize, Rc<TabIndicatorTarget>)>,
    count: usize,
    ring: Hsla,
}

fn build_destination_element(
    index: usize,
    mut item: NavigationItem,
    divider: Option<AnyElement>,
    ctx: &DestinationRowContext<'_>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = ctx.selected == Some(&item.id);
    let styled_item = ctx.bar_colors.is_some()
        || ctx.tab && (ctx.tab_content.is_some() || item.colors != NavigationItemColors::default())
        || ctx.rail
            && (ctx.options.is_some_and(|options| options.content.is_some())
                || item.colors != NavigationItemColors::default());
    let handle = &ctx.focus[index];
    let is_tab_stop = ctx.tab_stop == Some(&item.id);
    let id: ElementId = (ctx.row_id.clone(), item.id.clone()).into();
    let description = item.badge.as_mut().and_then(Badge::take_accessible_label);
    let feedback = ripple_state(&id, !item.disabled, window, cx);
    let tracker = item
        .interaction_source
        .as_ref()
        .map(|source| InteractionSource::track(source, &id, window, cx));
    let interaction = feedback
        .borrow_mut()
        .track(!item.disabled, handle, tracker, window, cx);
    let visual = ItemVisual {
        presentation: ctx.presentation,
        selected,
        height: ctx.height,
        rtl: ctx.rtl,
        font_scale: ctx.font_scale,
        tab_content: ctx.tab_content,
        styled_item,
        custom_indicator: ctx.custom_indicator,
        indicator_target: ctx
            .indicator_target
            .as_ref()
            .and_then(|(selected_index, target)| {
                (*selected_index == index).then(|| target.clone())
            }),
        feedback: feedback.clone(),
        interaction,
        ring: ctx.ring,
    };
    let content = visual.render(&id, &mut item, window, cx);
    let position = if ctx.rtl && !ctx.rail {
        ctx.count - index
    } else {
        index + 1
    };
    let slot_id = (id.clone(), "rail-destination");
    let divider_id = (id.clone(), "rail-divider");
    let ring = ctx.ring;
    let button = Button::new(id)
        .role(if ctx.tab {
            Role::Tab
        } else {
            Role::ListBoxOption
        })
        .aria_selected(selected)
        .aria_position_in_set(position)
        .aria_size_of_set(ctx.count)
        .accessibility_label(item.label)
        .when_some(description, |button, description| {
            button.aria_description(description)
        })
        .track_focus(handle)
        .tab_stop(is_tab_stop)
        .disabled(item.disabled)
        .min_h(ctx.height)
        .min_w(px(48.))
        .when_else(
            ctx.rail,
            |button| button.flex_none().w_full(),
            |button| {
                button.flex_1().when(
                    matches!(ctx.presentation, Presentation::Tabs(_, true)),
                    |button| button.flex_none().min_w(ctx.min_tab_width),
                )
            },
        )
        .when_some(ctx.centered_widths, |button, (minimum, maximum)| {
            button.flex_none().min_w(minimum).max_w(maximum)
        })
        .border_color(gpui::transparent_black())
        .border_2()
        .when(!item.disabled, |button| {
            let click_feedback = feedback.clone();
            let button = button
                .focus_visible(move |style| style.border_color(ring))
                .when_some(ctx.on_select.cloned(), |button, activate| {
                    let id = item.id.clone();
                    let on_accessibility_action = activate.clone();
                    button
                        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            on_accessibility_action(&id, window, cx);
                        })
                        .on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            activate_feedback(event, &click_feedback, window, cx);
                            activate(&item.id, window, cx);
                        })
                });
            feedback_target(button, feedback, true, Some(handle.clone()))
        })
        .opacity(if item.disabled && !styled_item {
            0.38
        } else {
            1.
        })
        .child(content);
    if let Some(options) = ctx.options {
        div()
            .id(slot_id)
            .flex_none()
            .when(
                index == 0
                    && matches!(
                        options.arrangement,
                        NavigationRailArrangement::Center | NavigationRailArrangement::End
                    ),
                |slot| slot.mt_auto(),
            )
            .when(
                index + 1 == ctx.count && options.arrangement == NavigationRailArrangement::Center,
                |slot| slot.mb_auto(),
            )
            .child(button)
            .children(divider.map(|divider| div().id(divider_id).child(divider)))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

impl NavigationItems {
    pub(in crate::navigation) fn render_row(
        self,
        presentation: Presentation,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row_with_tabs(presentation, None, window, cx)
    }

    pub(in crate::navigation) fn render_row_with_tabs(
        self,
        presentation: Presentation,
        mut tabs: Option<TabOptions>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let density = theme.layout.density;
        let tone = |role| color(theme.colors.color(role));
        let ring = tone(ColorRole::Secondary);
        let indicator_color = tone(ColorRole::Primary);
        let tab_colors = tabs.as_ref().map(|tabs| tabs.colors).unwrap_or_default();
        let divider = tab_colors
            .divider
            .unwrap_or_else(|| tone(ColorRole::SurfaceVariant));
        let bar_colors = presentation.bar_colors();
        let background = bar_colors
            .and_then(|colors| colors.container)
            .or(tab_colors.container)
            .unwrap_or_else(|| {
                tone(if bar_colors.is_some() {
                    ColorRole::SurfaceContainer
                } else {
                    ColorRole::Surface
                })
            });
        let options = presentation.rail_options();
        let rail = presentation.is_rail();
        let tab = presentation.is_tab();
        let scrollable = presentation.is_scrollable();
        let has_icons = self.items.iter().any(|item| {
            item.icon.is_some()
                && (!tab || item.icon_position != Some(NavigationItemIconPosition::Start))
        });
        let height = px(density.interactive_size(presentation.minimum_height(has_icons)));
        let state = self
            .state
            .clone()
            .unwrap_or_else(|| self.row_state(window, cx));
        let mut rail_dividers = self.rail_dividers;
        let (mut focus, scroll) = sync_focus_and_scroll(&state, &self.items, rail, cx);
        let scroll = tabs
            .as_ref()
            .filter(|_| scrollable)
            .and_then(|tabs| tabs.scroll.clone())
            .unwrap_or(scroll);
        let enabled: Vec<_> = self
            .items
            .iter()
            .zip(&focus)
            .filter(|(item, _)| !item.disabled)
            .map(|(item, handle)| (item.id.clone(), handle.clone()))
            .collect();
        let focused = enabled.iter().find(|(_, handle)| handle.is_focused(window));
        let selected = self.selected.as_ref();
        let tab_stop = resolve_tab_stop(
            &state,
            &enabled,
            focused,
            selected,
            self.retain_roving_stop,
            cx,
        );
        let mut items = self.items;
        if rtl && !rail {
            items.reverse();
            focus.reverse();
        }
        let focused_id = focused.map(|(id, _)| id);
        let reveal_index = update_reveal_state(
            &state,
            &items,
            presentation,
            direction,
            selected,
            focused_id,
            tab,
            scrollable,
            &scroll,
            cx,
        );
        let custom_indicator = tabs
            .as_ref()
            .is_some_and(|tabs| tabs.indicator_style.is_some() || tabs.indicator_content.is_some());
        let custom_divider = tabs.as_mut().and_then(|tabs| tabs.divider.take());
        let decoration = build_tab_decoration(
            &self.id,
            &items,
            &mut tabs,
            presentation,
            direction,
            selected,
            scrollable,
            &scroll,
            indicator_color,
            custom_indicator,
            custom_divider.is_some(),
        );
        let indicator_target = decoration
            .as_ref()
            .map(|(index, decoration)| (*index, decoration.target.clone()));
        let width = window.viewport_size().width;
        let edge_padding = tabs
            .as_ref()
            .map_or(px(52.), |tabs| tabs.edge_padding.min(width / 2.));
        let min_tab_width = tabs
            .as_ref()
            .map_or(px(90.), |tabs| tabs.min_tab_width.min(width.max(px(90.))));
        let count = items.len();
        let centered_widths = (presentation.centered() && count > 0).then(|| {
            // Compose reserves side space for short labels, while capping wider
            // destinations to their share of an equal-weight bar.
            let share = 1. / count as f32;
            let group = ((count.min(7) + 3) as f32 * 0.1).min(1.);
            (gpui::relative(share * group), gpui::relative(share))
        });
        let row = div()
            .when(tab && (scrollable || indicator_target.is_some()), |row| {
                let scroll = scroll.clone();
                let indicator_target = indicator_target.clone();
                row.on_children_prepainted(move |bounds, window, cx| {
                    if let Some((index, target)) = &indicator_target {
                        target.tab_bounds.set(if scrollable {
                            scroll.bounds_for_item(*index)
                        } else {
                            bounds.get(*index).copied()
                        });
                    }
                    if let Some(index) = reveal_index {
                        state.update(cx, |state, _| {
                            state.tab_reveal.observe_geometry(index, &scroll, window)
                        });
                    }
                })
            })
            .id(self.id.clone())
            .test_support()
            .role(if tab { Role::TabList } else { Role::ListBox })
            .aria_label(self.label)
            .flex()
            .w_full()
            .min_w_0()
            .when_else(
                rail,
                |row| {
                    row.flex_col()
                        .gap(options.map_or(px(4.), |options| options.spacing))
                        .min_h_0()
                        .flex_1()
                        .overflow_y_scroll()
                        .occlude()
                },
                |row| {
                    row.flex_row()
                        .flex_none()
                        .min_h(height)
                        .when(custom_divider.is_none(), |row| row.bg(background))
                },
            )
            .when(tab && custom_divider.is_none(), |row| {
                row.border_b_1().border_color(divider)
            })
            .when_some(options, |row, options| match options.arrangement {
                NavigationRailArrangement::SpaceBetween => row.justify_between(),
                NavigationRailArrangement::SpaceAround => row.justify_around(),
                NavigationRailArrangement::SpaceEvenly => row.justify_evenly(),
                _ => row,
            })
            .when(centered_widths.is_some(), Styled::justify_center)
            .when(scrollable && !rail, |row| row.overflow_x_scroll())
            .when(matches!(presentation, Presentation::Tabs(_, true)), |row| {
                row.px(edge_padding)
            })
            .when(scrollable, |row| row.track_scroll(&scroll))
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let Some(current) = enabled
                    .iter()
                    .position(|(_, handle)| handle.is_focused(window))
                else {
                    return;
                };
                if let Some(next) =
                    next_item(&event.keystroke.key, current, enabled.len(), rail, rtl)
                {
                    enabled[next].1.focus(window, cx);
                    window.refresh();
                    cx.stop_propagation();
                }
            });
        let destination_ctx = DestinationRowContext {
            row_id: &self.id,
            presentation,
            options,
            bar_colors,
            tab_content: tab_colors.content,
            selected,
            tab_stop: tab_stop.as_ref(),
            focus: &focus,
            on_select: self.on_select.as_ref(),
            height,
            min_tab_width,
            centered_widths,
            font_scale: self.font_scale,
            rtl,
            rail,
            tab,
            custom_indicator,
            indicator_target,
            count,
            ring,
        };
        let row = row.children(items.into_iter().enumerate().map(|(index, item)| {
            let divider = rail_dividers.remove(&item.id);
            build_destination_element(index, item, divider, &destination_ctx, window, cx)
        }));
        if custom_divider.is_some() || decoration.is_some() {
            div()
                .relative()
                .w_full()
                .min_w_0()
                .when(custom_divider.is_some(), |host| host.bg(background))
                .children(custom_divider.map(|divider| {
                    decorative_slot((self.id.clone(), "custom-divider").into(), divider)
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                }))
                .child(row)
                .children(decoration.map(|(_, decoration)| decoration.element()))
                .into_any_element()
        } else {
            row.into_any_element()
        }
    }
}
