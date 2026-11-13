use std::rc::Rc;

use gpui::{
    AnyElement, App, Axis, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Role, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_material_foundation::{Surface, color, material_theme};
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (background, shadow, outline) = match self.variant {
            CardVariant::Elevated => (ColorRole::SurfaceContainerLow, Elevation::Level1, false),
            CardVariant::Filled => (ColorRole::SurfaceContainerHighest, Elevation::Level0, false),
            CardVariant::Outlined => (ColorRole::Surface, Elevation::Level0, true),
        };
        let border = tone(ColorRole::OutlineVariant, cx);
        let focus = tone(ColorRole::Secondary, cx);
        let surface = Surface::new(background)
            .rounded(px(12.))
            .shadow_elevation(if self.disabled {
                Elevation::Level0
            } else {
                shadow
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
            .children(self.children);
        match self.activation {
            Some((name, activate)) => gpui_base::Button::new(self.id)
                .w_full()
                .accessibility_label(name)
                .disabled(self.disabled)
                .rounded(px(12.))
                .border_2()
                .border_color(gpui::transparent_black())
                .focus_visible(move |style| style.border_color(focus))
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    activate(window, cx);
                })
                .child(surface)
                .into_any_element(),
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

    fn row(self, corners: [f32; 4], cx: &App) -> AnyElement {
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
            .children(
                self.overline
                    .map(|text| Text::new(text).role(TypeRole::LabelSmall).color(secondary)),
            )
            .child(Text::new(self.label.clone()).color(foreground))
            .children(
                self.supporting
                    .map(|text| Text::new(text).role(TypeRole::BodyMedium).color(secondary)),
            );
        let background = tone(background, cx);
        let state_layer = tone(foreground, cx);
        let focus = tone(ColorRole::Secondary, cx);
        let row = div()
            .min_h(px(height))
            .w_full()
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py(px(10.))
            .when(
                material_theme(cx).layout.direction == Direction::Rtl,
                Styled::flex_row_reverse,
            )
            .rounded_tl(px(corners[0]))
            .rounded_tr(px(corners[1]))
            .rounded_br(px(corners[2]))
            .rounded_bl(px(corners[3]))
            .bg(background)
            .when(self.disabled, |row| row.opacity(0.55))
            .children(self.leading)
            .child(content)
            .children(self.trailing);
        let row = match self.activation {
            Some(activate) => gpui_base::Button::new((self.id.clone(), "activate"))
                .w_full()
                .accessibility_label(self.label)
                .disabled(self.disabled)
                .selected(self.selected)
                .border_2()
                .border_color(gpui::transparent_black())
                .rounded_md()
                .focus_visible(move |style| style.border_color(focus))
                .when(!self.disabled, |element| {
                    element
                        .hover(move |style| {
                            style.bg(state_layer.opacity(StateLayer::Hover.opacity()))
                        })
                        .active(move |style| {
                            style.bg(state_layer.opacity(StateLayer::Pressed.opacity()))
                        })
                })
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    activate(window, cx);
                })
                .child(row)
                .into_any_element(),
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.row([0.; 4], cx)
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
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
                item.row(corners, cx)
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = tone(ColorRole::OutlineVariant, cx);
        let (left, right) = material_theme(cx).layout.horizontal(self.start, self.end);
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let large = self.text.is_some();
        let side = px(if large { 16. } else { 6. });
        let mark = div()
            .absolute()
            .top(px(-3.))
            .min_w(side)
            .h(side)
            .rounded_full()
            .when(
                material_theme(cx).layout.direction == Direction::Rtl,
                |badge| badge.left(px(-3.)),
            )
            .when(
                material_theme(cx).layout.direction == Direction::Ltr,
                |badge| badge.right(px(-3.)),
            )
            .when(large, |badge| badge.px_1())
            .bg(tone(ColorRole::Error, cx))
            .text_color(tone(ColorRole::OnError, cx))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(11.))
            .children(self.text);
        div().relative().child(self.host).child(mark)
    }
}

fn tone(role: ColorRole, cx: &App) -> gpui::Hsla {
    color(material_theme(cx).colors.color(role))
}
