use std::{collections::HashMap, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, TextAlign, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, animate, color,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    text_style, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, Direction, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis,
};

use super::tabs::{
    TabDecoration, TabIndicatorTarget, TabOptions, TabReveal, TabVariant, decorative_slot,
};
use crate::{Badge, Icon};

use super::rail::{NavigationRailArrangement, NavigationRailPadding, RailPresentation, length};

/// Placement of a navigation item's icon relative to its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationItemIconPosition {
    Top,
    Start,
}

/// Optional pigments for an Expressive navigation destination.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationItemColors {
    pub selected_icon: Option<Hsla>,
    pub selected_label_top: Option<Hsla>,
    pub selected_label_start: Option<Hsla>,
    pub selected_indicator: Option<Hsla>,
    pub unselected_icon: Option<Hsla>,
    pub unselected_label: Option<Hsla>,
    pub disabled_icon: Option<Hsla>,
    pub disabled_label: Option<Hsla>,
}

/// One destination or tab with an ID unique within its collection.
pub struct NavigationItem {
    id: SharedString,
    label: SharedString,
    icon: Option<Icon>,
    icon_position: Option<NavigationItemIconPosition>,
    label_content: Option<AnyElement>,
    badge: Option<Badge>,
    indicator_padding: Option<(NavigationRailPadding, NavigationRailPadding)>,
    colors: NavigationItemColors,
    interaction_source: Option<Entity<InteractionSource>>,
    disabled: bool,
}

impl NavigationItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            icon_position: None,
            label_content: None,
            badge: None,
            indicator_padding: None,
            colors: NavigationItemColors::default(),
            interaction_source: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides icon placement inherited from the tab, bar, or rail.
    pub fn icon_position(mut self, position: NavigationItemIconPosition) -> Self {
        self.icon_position = Some(position);
        self
    }

    /// Replaces the visible label while retaining the accessible name.
    pub fn label_content(mut self, content: impl IntoElement) -> Self {
        self.label_content = Some(content.into_any_element());
        self
    }

    /// Anchors a standalone badge to the icon, or to the label if there is no icon.
    /// An accessible badge label becomes the destination's description.
    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Sets the rail indicator's logical content padding without shrinking its target.
    pub fn indicator_padding(
        mut self,
        collapsed: NavigationRailPadding,
        expanded: NavigationRailPadding,
    ) -> Self {
        self.indicator_padding = Some((collapsed, expanded));
        self
    }

    /// Sets per-state pigments for a navigation or tab destination.
    pub fn colors(mut self, colors: NavigationItemColors) -> Self {
        self.colors = colors;
        self
    }

    /// Observes the destination using its collection ID and item ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }
}

type Select = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Controlled destinations. Use the same collection ID across adaptive
/// presentations to preserve focus on the destination that currently owns it.
pub struct NavigationItems {
    id: ElementId,
    label: SharedString,
    items: Vec<NavigationItem>,
    selected: Option<SharedString>,
    on_select: Option<Select>,
    font_scale: f32,
    state: Option<Entity<RowState>>,
    rail_dividers: HashMap<SharedString, AnyElement>,
}

impl NavigationItems {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = NavigationItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            selected: None,
            on_select: None,
            font_scale: 1.,
            state: None,
            rail_dividers: HashMap::new(),
        }
    }

    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected = Some(id.into());
        self
    }

    /// Reports pointer, keyboard activation and accessibility activation.
    /// Arrow keys and Home/End move focus. The caller commits selection.
    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        self
    }

    pub(super) fn with_rail_dividers(
        mut self,
        dividers: HashMap<SharedString, AnyElement>,
    ) -> Self {
        self.rail_dividers = dividers;
        self
    }

    // Resolve the state before the adaptive scaffold puts it under a different
    // element path. Otherwise switching from its bar slot to its rail slot
    // replaces the destination focus handles.
    pub(super) fn retain(mut self, window: &mut Window, cx: &mut App) -> Self {
        if self.state.is_none() {
            self.state = Some(self.row_state(window, cx));
        }
        self
    }

    fn row_state(&self, window: &mut Window, cx: &mut App) -> Entity<RowState> {
        window.use_keyed_state((self.id.clone(), "navigation-state"), cx, |_, _| {
            RowState::default()
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarLayout {
    #[default]
    Vertical,
    Horizontal,
}

/// Distribution of the navigation items along the width of the bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarArrangement {
    #[default]
    EqualWeight,
    Centered,
}

/// Short navigation bar pigments. Item overrides take precedence over `content`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationBarColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
}

/// Material Expressive short navigation bar.
#[derive(IntoElement)]
pub struct NavigationBar {
    items: NavigationItems,
    layout: NavigationBarLayout,
    arrangement: NavigationBarArrangement,
    colors: NavigationBarColors,
}

impl NavigationBar {
    pub fn new(items: NavigationItems) -> Self {
        Self {
            items,
            layout: NavigationBarLayout::Vertical,
            arrangement: NavigationBarArrangement::default(),
            colors: NavigationBarColors::default(),
        }
    }

    pub fn layout(mut self, layout: NavigationBarLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn arrangement(mut self, arrangement: NavigationBarArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    pub fn colors(mut self, colors: NavigationBarColors) -> Self {
        self.colors = colors;
        self
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.items.render_row(
            Presentation::Bar {
                layout: self.layout,
                arrangement: self.arrangement,
                colors: self.colors,
            },
            window,
            cx,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Presentation {
    Tabs(TabVariant, bool),
    Bar {
        layout: NavigationBarLayout,
        arrangement: NavigationBarArrangement,
        colors: NavigationBarColors,
    },
    Rail(RailPresentation),
}

impl Presentation {
    fn is_tab(self) -> bool {
        matches!(self, Self::Tabs(..))
    }

    fn is_rail(self) -> bool {
        matches!(self, Self::Rail(_))
    }

    fn horizontal_label(self) -> bool {
        matches!(
            self,
            Self::Bar {
                layout: NavigationBarLayout::Horizontal,
                ..
            } | Self::Rail(RailPresentation { expanded: true, .. })
        )
    }

    fn bar_colors(self) -> Option<NavigationBarColors> {
        match self {
            Self::Bar { colors, .. } => Some(colors),
            _ => None,
        }
    }

    fn rail_options(self) -> Option<RailPresentation> {
        match self {
            Self::Rail(options) => Some(options),
            _ => None,
        }
    }

    fn item_colors(self) -> Option<NavigationBarColors> {
        match self {
            Self::Rail(options) => Some(NavigationBarColors {
                content: options.content,
                container: None,
            }),
            _ => self.bar_colors(),
        }
    }

    fn centered(self) -> bool {
        matches!(
            self,
            Self::Bar {
                arrangement: NavigationBarArrangement::Centered,
                ..
            }
        )
    }

    fn is_scrollable(self) -> bool {
        !matches!(self, Self::Tabs(_, false))
    }

    fn minimum_height(self, has_icons: bool) -> f32 {
        match self {
            Self::Tabs(..) if has_icons => 64.,
            Self::Tabs(..) => 48.,
            Self::Bar { .. } | Self::Rail(_) => 64.,
        }
    }
}

#[derive(Default)]
struct RowState {
    selection: Option<SharedString>,
    focus: HashMap<SharedString, FocusHandle>,
    horizontal_scroll: ScrollHandle,
    vertical_scroll: ScrollHandle,
    reveal: Option<(Presentation, Direction, SharedString, usize)>,
    tab_reveal: TabReveal,
}

impl NavigationItems {
    pub(super) fn render_row(
        self,
        presentation: Presentation,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row_with_tabs(presentation, None, window, cx)
    }

    pub(super) fn render_row_with_tabs(
        self,
        presentation: Presentation,
        mut tabs: Option<TabOptions>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let tone = |role| color(theme.colors.color(role));
        let ring = tone(ColorRole::Secondary);
        let indicator_color = tone(ColorRole::Primary);
        let state_layer = tone(ColorRole::OnSurface);
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
        let item_colors = presentation.item_colors();
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
        let (focus, scroll) = state.update(cx, |state, cx| {
            let mut previous = std::mem::take(&mut state.focus);
            state.focus = self
                .items
                .iter()
                .map(|item| {
                    let handle = previous
                        .remove(&item.id)
                        .unwrap_or_else(|| cx.focus_handle());
                    (item.id.clone(), handle)
                })
                .collect();
            let scroll = if rail {
                &state.vertical_scroll
            } else {
                &state.horizontal_scroll
            };
            (state.focus.clone(), scroll.clone())
        });
        let scroll = tabs
            .as_ref()
            .filter(|_| scrollable)
            .and_then(|tabs| tabs.scroll.clone())
            .unwrap_or(scroll);
        let enabled: Vec<_> = self
            .items
            .iter()
            .filter(|item| !item.disabled)
            .map(|item| (item.id.clone(), focus[&item.id].clone()))
            .collect();
        let focused = enabled.iter().find(|(_, handle)| handle.is_focused(window));
        let selected = self.selected.as_ref();
        let first = enabled.first().map(|(id, _)| id);
        let tab_stop = focused
            .map(|(id, _)| id)
            .or_else(|| selected.filter(|id| enabled.iter().any(|(item, _)| item == *id)))
            .or(first)
            .cloned();
        let mut items = self.items;
        if rtl && !rail {
            items.reverse();
        }
        let focused_id = focused.map(|(id, _)| id);
        let reveal_index = if tab && scrollable {
            state.update(cx, |state, _| {
                state.tab_reveal.choose(
                    selected,
                    focused_id,
                    items.iter().map(|item| item.id.clone()).collect(),
                    &scroll,
                )
            })
        } else {
            state.update(cx, |state, _| {
                let reveal_target = if state.selection.as_ref() != selected {
                    selected.or(focused_id)
                } else {
                    focused_id.or(selected)
                };
                let reveal = reveal_target.and_then(|id| {
                    items
                        .iter()
                        .position(|item| &item.id == id)
                        .map(|index| (presentation, direction, id.clone(), index))
                });
                if scrollable
                    && state.reveal != reveal
                    && let Some((_, _, _, index)) = &reveal
                {
                    scroll.scroll_to_item(*index);
                }
                state.reveal = reveal;
                state.selection = self.selected.clone();
                state.tab_reveal = TabReveal::default();
            });
            None
        };
        let custom_indicator = tabs
            .as_ref()
            .is_some_and(|tabs| tabs.indicator_style.is_some() || tabs.indicator_content.is_some());
        let custom_divider = tabs.as_mut().and_then(|tabs| tabs.divider.take());
        let decoration = tabs.as_mut().and_then(|tabs| {
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
                    row: self.id.clone(),
                    selected: item.id.clone(),
                    variant,
                    direction,
                    target: Rc::new(TabIndicatorTarget::default()),
                    scroll: scrollable.then(|| scroll.clone()),
                    style,
                    content: tabs.indicator_content.take(),
                    color: tint.opacity(if item.disabled { 0.38 } else { 1. }),
                    has_default_divider: custom_divider.is_none(),
                },
            ))
        });
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
        let row = row.children(items.into_iter().enumerate().map(|(index, mut item)| {
            let selected = selected == Some(&item.id);
            let styled_item = bar_colors.is_some()
                || tab
                    && (tab_colors.content.is_some()
                        || item.colors != NavigationItemColors::default())
                || rail
                    && (options.is_some_and(|options| options.content.is_some())
                        || item.colors != NavigationItemColors::default());
            let visual = ItemVisual {
                presentation,
                selected,
                height,
                rtl,
                font_scale: self.font_scale,
                tab_content: tab_colors.content,
                styled_item,
                custom_indicator,
                indicator_target: indicator_target
                    .as_ref()
                    .and_then(|(selected_index, target)| {
                        (*selected_index == index).then(|| target.clone())
                    }),
            };
            let handle = &focus[&item.id];
            let is_tab_stop = tab_stop.as_ref() == Some(&item.id);
            let id = (self.id.clone(), item.id.clone()).into();
            let customize_item = tab || item_colors.is_some();
            let description = item
                .badge
                .as_mut()
                .filter(|_| customize_item)
                .and_then(Badge::take_accessible_label);
            let content = visual.render(&id, &mut item, window, cx);
            let feedback = item
                .interaction_source
                .as_ref()
                .filter(|_| customize_item)
                .map(|source| {
                    let feedback = ripple_state(&id, !item.disabled, window, cx);
                    let tracker = InteractionSource::track(source, &id, window, cx);
                    feedback
                        .borrow_mut()
                        .track(!item.disabled, handle, Some(tracker), window, cx);
                    feedback
                });
            let position = if rtl && !rail {
                count - index
            } else {
                index + 1
            };
            let divider = rail_dividers.remove(&item.id);
            let slot_id = (id, "rail-destination");
            let button = Button::new(item.id.clone())
                .role(if tab { Role::Tab } else { Role::ListBoxOption })
                .aria_selected(selected)
                .aria_position_in_set(position)
                .aria_size_of_set(count)
                .accessibility_label(item.label)
                .when_some(description, |button, description| {
                    button.aria_description(description)
                })
                .track_focus(handle)
                .tab_stop(is_tab_stop)
                .disabled(item.disabled)
                .min_h(height)
                .min_w(px(48.))
                .when_else(
                    rail,
                    |button| button.flex_none().w_full(),
                    |button| {
                        button.flex_1().when(
                            matches!(presentation, Presentation::Tabs(_, true)),
                            |button| button.flex_none().min_w(min_tab_width),
                        )
                    },
                )
                .when_some(centered_widths, |button, (minimum, maximum)| {
                    button.flex_none().min_w(minimum).max_w(maximum)
                })
                .border_color(gpui::transparent_black())
                .border_2()
                .when(!item.disabled, |button| {
                    button
                        .hover(|style| style.bg(state_layer.opacity(0.08)))
                        .active(|style| style.bg(state_layer.opacity(0.1)))
                        .focus_visible(|style| style.border_color(ring))
                        .when_some(feedback.as_ref().cloned(), |button, feedback| {
                            feedback_target(button, feedback, true, Some(handle.clone()))
                        })
                        .when_some(self.on_select.clone(), |button, activate| {
                            let id = item.id.clone();
                            let on_accessibility_action = activate.clone();
                            button
                                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                    on_accessibility_action(&id, window, cx);
                                })
                                .on_click(move |event, window, cx| {
                                    cx.stop_propagation();
                                    if let Some(feedback) = &feedback {
                                        activate_feedback(event, feedback, window, cx);
                                    }
                                    activate(&item.id, window, cx);
                                })
                        })
                })
                .opacity(if item.disabled && !styled_item {
                    0.38
                } else {
                    1.
                })
                .child(content);
            if let Some(options) = options {
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
                        index + 1 == count
                            && options.arrangement == NavigationRailArrangement::Center,
                        |slot| slot.mb_auto(),
                    )
                    .child(button)
                    .children(divider.map(|divider| div().id("rail-divider").child(divider)))
                    .into_any_element()
            } else {
                button.into_any_element()
            }
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

fn next_item(key: &str, current: usize, len: usize, vertical: bool, rtl: bool) -> Option<usize> {
    let forward = match key {
        "home" => return Some(0),
        "end" => return Some(len - 1),
        "down" if vertical => true,
        "up" if vertical => false,
        "right" if !vertical => !rtl,
        "left" if !vertical => rtl,
        _ => return None,
    };
    Some((current + if forward { 1 } else { len - 1 }) % len)
}

struct ItemVisual {
    presentation: Presentation,
    selected: bool,
    height: gpui::Pixels,
    rtl: bool,
    font_scale: f32,
    tab_content: Option<Hsla>,
    styled_item: bool,
    custom_indicator: bool,
    indicator_target: Option<Rc<TabIndicatorTarget>>,
}

impl ItemVisual {
    fn render(
        self,
        id: &ElementId,
        item: &mut NavigationItem,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let theme = window_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        let foreground = if !self.selected {
            ColorRole::OnSurfaceVariant
        } else {
            match self.presentation {
                Presentation::Tabs(TabVariant::Primary, _) => ColorRole::Primary,
                Presentation::Tabs(TabVariant::Secondary, _) => ColorRole::OnSurface,
                _ => ColorRole::Secondary,
            }
        };
        let icon_role = if self.selected && !self.presentation.is_tab() {
            ColorRole::OnSecondaryContainer
        } else {
            foreground
        };
        let bar = self.presentation.item_colors();
        let tab = self.presentation.is_tab();
        let customize_item = bar.is_some() || tab;
        let fallback = bar.and_then(|colors| colors.content).or(self.tab_content);
        let rail_padding = self.presentation.rail_options().and_then(|options| {
            item.indicator_padding.map(|(collapsed, expanded)| {
                if options.expanded {
                    expanded
                } else {
                    collapsed
                }
            })
        });
        let indicator_padding = rail_padding.map(|padding| {
            let (left, right) = theme
                .layout
                .horizontal(length(padding.start), length(padding.end));
            (left, right, length(padding.top), length(padding.bottom))
        });
        let horizontal = item.icon_position.filter(|_| customize_item).map_or_else(
            || self.presentation.horizontal_label(),
            |position| position == NavigationItemIconPosition::Start,
        );
        let (icon_color, label_color, selected_indicator) = if self.styled_item {
            let colors = item.colors;
            let resolve = |selected: Option<Hsla>,
                           unselected: Option<Hsla>,
                           disabled: Option<Hsla>,
                           active_role| {
                let unselected = unselected
                    .or(fallback)
                    .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant));
                if item.disabled {
                    disabled.unwrap_or_else(|| unselected.opacity(0.38))
                } else if self.selected {
                    selected.or(fallback).unwrap_or_else(|| tone(active_role))
                } else {
                    unselected
                }
            };
            let icon = resolve(
                colors.selected_icon,
                colors.unselected_icon,
                colors.disabled_icon,
                icon_role,
            );
            let label = resolve(
                if horizontal {
                    colors.selected_label_start
                } else {
                    colors.selected_label_top
                },
                colors.unselected_label,
                colors.disabled_label,
                foreground,
            );
            let selected_indicator = colors
                .selected_indicator
                .unwrap_or_else(|| tone(ColorRole::SecondaryContainer));
            (
                icon,
                label,
                selected_indicator.opacity(if item.disabled { 0.38 } else { 1. }),
            )
        } else {
            (
                tone(icon_role),
                tone(foreground),
                tone(ColorRole::SecondaryContainer),
            )
        };
        let indicator = item
            .colors
            .selected_indicator
            .unwrap_or_else(|| tone(ColorRole::Primary))
            .opacity(if item.disabled && self.styled_item {
                0.38
            } else {
                1.
            });
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
        let rail = self.presentation.is_rail();
        let text_role = if tab {
            TypeRole::TitleSmall
        } else if rail && horizontal {
            TypeRole::LabelLarge
        } else {
            TypeRole::LabelMedium
        };
        let mut label = text_style(
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
        label = if let Some(content) = item.label_content.take().filter(|_| customize_item) {
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
        };
        let mut badge = item.badge.take().filter(|_| customize_item);
        let icon = item.icon.take().map(|icon| {
            let icon = if self.styled_item {
                icon.inherit_color()
            } else {
                icon.color(icon_role)
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
                .text_color(icon_color)
                .when(!tab && !horizontal, |icon| {
                    icon.w(px(56.))
                        .h(px(32.))
                        .rounded_full()
                        .bg(selected_indicator.opacity(selection))
                })
                .when_some(
                    indicator_padding.filter(|_| !horizontal),
                    |icon, (left, right, top, bottom)| {
                        icon.w(px(24.) + left + right)
                            .h(px(24.) + top + bottom)
                            .pl(left)
                            .pr(right)
                            .pt(top)
                            .pb(bottom)
                    },
                )
                .child(icon)
        });
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
                                .min_h(px(if rail { 56. } else { 40. }))
                                .rounded_full()
                                .bg(selected_indicator.opacity(selection))
                        })
                        .when(rail, |content| content.w_full())
                },
                |content| content.flex_col().justify_center().gap(px(4.)),
            )
            .px(padding)
            .when_some(
                indicator_padding.filter(|_| horizontal),
                |content, (left, right, top, bottom)| content.pl(left).pr(right).pt(top).pb(bottom),
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
            .when(tab && !self.custom_indicator, |body| {
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
                            .bg(indicator.opacity(selection)),
                        ),
                )
            })
    }
}
