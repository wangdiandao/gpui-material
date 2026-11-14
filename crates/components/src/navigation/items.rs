use std::{collections::HashMap, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, TextAlign, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_foundation::{animate, color, text_style, window_material_theme};
use gpui_material_theme::{
    ColorRole, Direction, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis,
};

use crate::Icon;

/// One destination or tab with an ID unique within its collection.
pub struct NavigationItem {
    id: SharedString,
    label: SharedString,
    icon: Option<Icon>,
    disabled: bool,
}

impl NavigationItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabVariant {
    #[default]
    Primary,
    Secondary,
}

/// A fixed Material tab row by default. Enable scrolling for long tab lists.
#[derive(IntoElement)]
pub struct Tabs {
    items: NavigationItems,
    variant: TabVariant,
    scrollable: bool,
}

impl Tabs {
    pub fn new(items: NavigationItems) -> Self {
        Self {
            items,
            variant: TabVariant::Primary,
            scrollable: false,
        }
    }

    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.items.render_row(
            Presentation::Tabs(self.variant, self.scrollable),
            window,
            cx,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarLayout {
    #[default]
    Vertical,
    Horizontal,
}

/// Material Expressive short navigation bar.
#[derive(IntoElement)]
pub struct NavigationBar {
    items: NavigationItems,
    layout: NavigationBarLayout,
}

impl NavigationBar {
    pub fn new(items: NavigationItems) -> Self {
        Self {
            items,
            layout: NavigationBarLayout::Vertical,
        }
    }

    pub fn layout(mut self, layout: NavigationBarLayout) -> Self {
        self.layout = layout;
        self
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.items
            .render_row(Presentation::Bar(self.layout), window, cx)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Presentation {
    Tabs(TabVariant, bool),
    Bar(NavigationBarLayout),
    Rail(bool),
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
            Self::Bar(NavigationBarLayout::Horizontal) | Self::Rail(true)
        )
    }

    fn is_scrollable(self) -> bool {
        !matches!(self, Self::Tabs(_, false))
    }

    fn minimum_height(self, has_icons: bool) -> f32 {
        match self {
            Self::Tabs(..) if has_icons => 64.,
            Self::Tabs(..) => 48.,
            Self::Bar(_) | Self::Rail(_) => 64.,
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
}

impl NavigationItems {
    pub(super) fn render_row(
        self,
        presentation: Presentation,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let tone = |role| color(theme.colors.color(role));
        let ring = tone(ColorRole::Secondary);
        let state_layer = tone(ColorRole::OnSurface);
        let divider = tone(ColorRole::SurfaceVariant);
        let background = tone(if matches!(presentation, Presentation::Bar(_)) {
            ColorRole::SurfaceContainer
        } else {
            ColorRole::Surface
        });
        let rail = presentation.is_rail();
        let tab = presentation.is_tab();
        let scrollable = presentation.is_scrollable();
        let has_icons = self.items.iter().any(|item| item.icon.is_some());
        let height = px(density.interactive_size(presentation.minimum_height(has_icons)));
        let state = window.use_keyed_state((self.id.clone(), "navigation-state"), cx, |_, _| {
            RowState::default()
        });
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
        let reveal_target = if state.read(cx).selection.as_ref() != selected {
            selected.or_else(|| focused.map(|(id, _)| id))
        } else {
            focused.map(|(id, _)| id).or(selected)
        };
        let reveal = reveal_target.and_then(|id| {
            items
                .iter()
                .position(|item| &item.id == id)
                .map(|index| (presentation, direction, id.clone(), index))
        });
        state.update(cx, |state, _| {
            if scrollable
                && state.reveal != reveal
                && let Some((_, _, _, index)) = &reveal
            {
                scroll.scroll_to_item(*index);
            }
            state.reveal = reveal;
            state.selection = self.selected.clone();
        });
        let count = items.len();
        let row = div()
            .id(self.id.clone())
            .test_support()
            .role(if tab { Role::TabList } else { Role::Navigation })
            .aria_label(self.label)
            .flex()
            .w_full()
            .min_w_0()
            .when_else(
                rail,
                |row| {
                    row.flex_col()
                        .gap(px(4.))
                        .min_h_0()
                        .flex_1()
                        .overflow_y_scroll()
                        .occlude()
                },
                |row| row.flex_row().flex_none().min_h(height).bg(background),
            )
            .when(tab, |row| row.border_b_1().border_color(divider))
            .when(scrollable && !rail, |row| row.overflow_x_scroll())
            .when(matches!(presentation, Presentation::Tabs(_, true)), |row| {
                row.px(px(52.))
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
        row.children(items.into_iter().enumerate().map(|(index, item)| {
            let selected = selected == Some(&item.id);
            let visual = ItemVisual {
                presentation,
                selected,
                height,
                rtl,
                font_scale: self.font_scale,
            };
            let handle = &focus[&item.id];
            let is_tab_stop = tab_stop.as_ref() == Some(&item.id);
            let id = (self.id.clone(), item.id.clone()).into();
            let content = visual.render(&id, item.icon, item.label.clone(), window, cx);
            let position = if rtl && !rail {
                count - index
            } else {
                index + 1
            };
            Button::new(item.id.clone())
                .role(if tab { Role::Tab } else { Role::Button })
                .aria_selected(selected)
                .aria_position_in_set(position)
                .aria_size_of_set(count)
                .accessibility_label(item.label)
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
                            |button| button.flex_none().min_w(px(90.)),
                        )
                    },
                )
                .border_color(gpui::transparent_black())
                .border_2()
                .when(!item.disabled, |button| {
                    button
                        .hover(|style| style.bg(state_layer.opacity(0.08)))
                        .active(|style| style.bg(state_layer.opacity(0.1)))
                        .focus_visible(|style| style.border_color(ring))
                        .when_some(self.on_select.clone(), |button, activate| {
                            let id = item.id.clone();
                            let on_accessibility_action = activate.clone();
                            button
                                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                    on_accessibility_action(&id, window, cx);
                                })
                                .on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    activate(&item.id, window, cx);
                                })
                        })
                })
                .opacity(if item.disabled { 0.38 } else { 1. })
                .child(content)
        }))
        .into_any_element()
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
}

impl ItemVisual {
    fn render(
        self,
        id: &ElementId,
        icon: Option<Icon>,
        label: SharedString,
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
        let label_color = tone(foreground);
        let icon_color = if self.selected && !self.presentation.is_tab() {
            ColorRole::OnSecondaryContainer
        } else {
            foreground
        };
        let secondary_container = tone(ColorRole::SecondaryContainer);
        let indicator = tone(ColorRole::Primary);
        let motion = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion());
        let selection = animate(
            (id.clone(), "indicator"),
            f32::from(self.selected),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        let horizontal = self.presentation.horizontal_label();
        let rail = self.presentation.is_rail();
        let tab = self.presentation.is_tab();
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
        })
        .child(label);
        let icon = icon.map(|icon| {
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .when(!tab && !horizontal, |icon| {
                    icon.w(px(56.))
                        .h(px(32.))
                        .rounded_full()
                        .bg(secondary_container.opacity(selection))
                })
                .child(icon.color(icon_color))
        });
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
                        .gap(px(if rail { 8. } else { 4. }))
                        .when(self.rtl, Styled::flex_row_reverse)
                        .px(px(16.))
                        .min_h(px(if rail { 56. } else { 40. }))
                        .rounded_full()
                        .bg(secondary_container.opacity(selection))
                        .when(rail, |content| content.w_full())
                },
                |content| content.flex_col().justify_center().gap(px(4.)).px(px(4.)),
            )
            .children(icon)
            .child(label);
        div()
            .relative()
            .flex()
            .w_full()
            .min_w_0()
            .min_h(self.height - px(4.))
            .items_center()
            .justify_center()
            .child(content)
            .when(tab, |body| {
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
