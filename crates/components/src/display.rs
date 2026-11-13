use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Axis, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, Direction, Elevation, StateLayer, TypeRole};

use crate::{Text, action::Activate};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CardVariant {
    Elevated,
    #[default]
    Filled,
    Outlined,
}

/// A static surface or a keyboard-operable action with independent child controls.
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    variant: CardVariant,
    children: Vec<AnyElement>,
    activation: Option<(SharedString, Activate)>,
    disabled: bool,
}

impl Card {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: CardVariant::default(),
            children: Vec::new(),
            activation: None,
            disabled: false,
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(
        mut self,
        name: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.activation = Some((name.into(), Rc::new(handler)));
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (background, shadow, outline) = match self.variant {
            CardVariant::Elevated => (ColorRole::SurfaceContainerLow, Elevation::Level1, false),
            CardVariant::Filled => (ColorRole::SurfaceContainerHighest, Elevation::Level0, false),
            CardVariant::Outlined => (ColorRole::Surface, Elevation::Level0, true),
        };
        let border = tone(ColorRole::OutlineVariant, window, cx);
        let focus = tone(ColorRole::Secondary, window, cx);
        let background = tone(background, window, cx);
        let foreground = tone(ColorRole::OnSurface, window, cx);
        let state_group = SharedString::from(format!("{}-card-surface", self.id));
        let elevation = if self.disabled { 0. } else { shadow.dp() };
        let surface = div()
            .id("card-surface")
            .rounded(px(12.))
            .bg(background)
            .text_color(foreground)
            .when(elevation > 0., |surface| {
                let shadow = tone(ColorRole::Shadow, window, cx);
                surface.shadow(vec![
                    gpui_base::box_shadow(0., elevation * 0.4, elevation, 0., shadow.opacity(0.18)),
                    gpui_base::box_shadow(0., 1., elevation * 0.5, 0., shadow.opacity(0.13)),
                ])
            })
            .border_1()
            .border_color(if outline {
                border
            } else {
                gpui::transparent_black()
            })
            .when(self.disabled, |card| card.opacity(0.55))
            .w_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.children)
            .when(self.activation.is_some() && !self.disabled, |surface| {
                surface_feedback(surface, state_group.clone(), background, foreground)
            });
        match self.activation {
            Some((name, activate)) => {
                let invoke = activate.clone();
                gpui_base::Button::new(self.id)
                    .group(state_group)
                    .w_full()
                    .accessibility_label(name)
                    .disabled(self.disabled)
                    .rounded(px(12.))
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .focus_visible(move |style| style.border_color(focus))
                    .when(!self.disabled, |control| {
                        control.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            invoke(window, cx)
                        })
                    })
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        activate(window, cx);
                    })
                    .child(surface)
                    .into_any_element()
            }
            None => div().id(self.id).child(surface).into_any_element(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListVariant {
    #[default]
    Standard,
    Segmented,
}

/// A row with a stable domain ID, text and optional leading/trailing slots.
#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    label: SharedString,
    supporting: Option<SharedString>,
    overline: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    activation: Option<Activate>,
    selected: bool,
    disabled: bool,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            overline: None,
            leading: None,
            trailing: None,
            activation: None,
            selected: false,
            disabled: false,
        }
    }

    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(text.into());
        self
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.activation = Some(Rc::new(handler));
        self
    }

    fn row(self, corners: [f32; 4], window: &Window, cx: &App) -> AnyElement {
        let background = if self.selected {
            ColorRole::SecondaryContainer
        } else {
            ColorRole::Surface
        };
        let foreground = if self.selected {
            ColorRole::OnSecondaryContainer
        } else {
            ColorRole::OnSurface
        };
        let secondary = if self.selected {
            ColorRole::OnSecondaryContainer
        } else {
            ColorRole::OnSurfaceVariant
        };
        let height = match (self.overline.is_some(), self.supporting.is_some()) {
            (false, false) => 56.,
            (true, true) => 88.,
            _ => 72.,
        };
        let content = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .children(self.overline.map(|text| {
                Text::new(text)
                    .id("overline")
                    .role(TypeRole::LabelSmall)
                    .color(secondary)
            }))
            .child(Text::new(self.label.clone()).id("label").color(foreground))
            .children(self.supporting.map(|text| {
                Text::new(text)
                    .id("supporting")
                    .role(TypeRole::BodyMedium)
                    .color(secondary)
            }));
        let background = tone(background, window, cx);
        let state_layer = tone(foreground, window, cx);
        let focus = tone(ColorRole::Secondary, window, cx);
        let state_group = SharedString::from(format!("{}-list-surface", self.id));
        let row = div()
            .id("list-surface")
            .min_h(px(height))
            .w_full()
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py(px(10.))
            .when(
                window_material_theme(window, cx).layout.direction == Direction::Rtl,
                Styled::flex_row_reverse,
            )
            .rounded_tl(px(corners[0]))
            .rounded_tr(px(corners[1]))
            .rounded_br(px(corners[2]))
            .rounded_bl(px(corners[3]))
            .bg(background)
            .when(self.activation.is_some() && !self.disabled, |row| {
                surface_feedback(row, state_group.clone(), background, state_layer)
            })
            .when(self.disabled, |row| row.opacity(0.55))
            .children(self.leading)
            .child(content)
            .children(self.trailing);
        let row = match self.activation {
            Some(activate) => {
                let invoke = activate.clone();
                gpui_base::Button::new((self.id.clone(), "activate"))
                    .group(state_group)
                    .w_full()
                    .accessibility_label(self.label)
                    .disabled(self.disabled)
                    .selected(self.selected)
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .rounded_md()
                    .focus_visible(move |style| style.border_color(focus))
                    .when(!self.disabled, |control| {
                        control.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            invoke(window, cx)
                        })
                    })
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        activate(window, cx);
                    })
                    .child(row)
                    .into_any_element()
            }
            None => row.into_any_element(),
        };
        div()
            .id(self.id)
            .role(Role::ListItem)
            .aria_selected(self.selected)
            .child(row)
            .into_any_element()
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.row([0.; 4], window, cx)
    }
}

/// A list of identified rows. Applications can supply a visible window of rows
/// from their own virtualized collection.
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    items: Vec<ListItem>,
    variant: ListVariant,
}

impl List {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            variant: ListVariant::default(),
        }
    }

    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ListItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let last = self.items.len().saturating_sub(1);
        div()
            .id(self.id)
            .role(Role::List)
            .flex()
            .flex_col()
            .w_full()
            .when(self.variant == ListVariant::Segmented, |list| {
                list.gap(px(2.))
            })
            .children(self.items.into_iter().enumerate().map(|(ix, item)| {
                let corners = if self.variant == ListVariant::Standard {
                    [0.; 4]
                } else if item.selected {
                    CornerRole::Large.shape().radii(100., 100., Direction::Ltr)
                } else {
                    let top = if ix == 0 { 16. } else { 4. };
                    let bottom = if ix == last { 16. } else { 4. };
                    [top, top, bottom, bottom]
                };
                item.row(corners, window, cx)
            }))
    }
}

/// Decorative Material divider, with logical start and end insets.
#[derive(IntoElement)]
pub struct Divider {
    axis: Axis,
    start: gpui::Pixels,
    end: gpui::Pixels,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            axis: Axis::Horizontal,
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

    pub fn inset(mut self, start: gpui::Pixels, end: gpui::Pixels) -> Self {
        self.start = start.max(px(0.));
        self.end = end.max(px(0.));
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = tone(ColorRole::OutlineVariant, window, cx);
        let (left, right) = window_material_theme(window, cx)
            .layout
            .horizontal(self.start, self.end);
        if self.axis == Axis::Vertical {
            div()
                .w(px(1.))
                .h_full()
                .pt(self.start)
                .pb(self.end)
                .child(div().size_full().bg(fill))
        } else {
            div()
                .w_full()
                .h(px(1.))
                .pl(left)
                .pr(right)
                .child(div().size_full().bg(fill))
        }
    }
}

/// An anchored badge. Pass its accessible name to the host control so the host
/// announces the count once and keeps a single focus stop.
#[derive(IntoElement)]
pub struct Badge {
    host: AnyElement,
    text: Option<SharedString>,
}

impl Badge {
    pub fn dot(host: impl IntoElement) -> Self {
        Self {
            host: host.into_any_element(),
            text: None,
        }
    }

    pub fn label(host: impl IntoElement, text: impl Into<SharedString>) -> Self {
        Self {
            host: host.into_any_element(),
            text: Some(text.into()),
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
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let large = self.text.is_some();
        let side = px(if large { 16. } else { 6. });
        let mark = div()
            .absolute()
            .top(px(-3.))
            .min_w(side)
            .h(side)
            .rounded_full()
            .when(
                window_material_theme(window, cx).layout.direction == Direction::Rtl,
                |badge| badge.left(px(-3.)),
            )
            .when(
                window_material_theme(window, cx).layout.direction == Direction::Ltr,
                |badge| badge.right(px(-3.)),
            )
            .when(large, |badge| badge.px_1())
            .bg(tone(ColorRole::Error, window, cx))
            .text_color(tone(ColorRole::OnError, window, cx))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(11.))
            .children(self.text);
        div().relative().child(self.host).child(mark)
    }
}

fn tone(role: ColorRole, window: &Window, cx: &App) -> gpui::Hsla {
    color(window_material_theme(window, cx).colors.color(role))
}

fn surface_feedback(
    surface: gpui::Stateful<gpui::Div>,
    group: SharedString,
    background: gpui::Hsla,
    foreground: gpui::Hsla,
) -> gpui::Stateful<gpui::Div> {
    surface
        .group_hover(group.clone(), move |style| {
            style.bg(background.blend(foreground.opacity(StateLayer::Hover.opacity())))
        })
        .group_active(group, move |style| {
            style.bg(background.blend(foreground.opacity(StateLayer::Pressed.opacity())))
        })
}
