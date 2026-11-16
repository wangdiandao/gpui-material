use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    animate, color, move_roving_focus, text_style, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, MotionKind, MotionSpeed, StateLayer, TypeRole,
    TypographyEmphasis,
};

use crate::{
    Text,
    action::{Activate, ripple_ink, ripple_state, ripple_target},
};

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
    focus: Option<(FocusHandle, bool)>,
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
            focus: None,
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

    /// Uses a retained focus handle and roving tab stop supplied by a virtual
    /// collection. The collection owns navigation across unrendered rows.
    pub fn track_focus(mut self, focus: &FocusHandle, tab_stop: bool) -> Self {
        self.focus = Some((focus.clone(), tab_stop));
        self
    }

    fn row(
        self,
        corners: [f32; 4],
        focus: Option<(&FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let motion = window_material_theme(window, cx).motion;
        let reduced = cx.reduce_motion();
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Default, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let selection = animate(
            (self.id.clone(), "list-selection"),
            f32::from(self.selected),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let corners = std::array::from_fn::<_, 4, _>(|idx| {
            let id = gpui::ElementId::from((self.id.clone(), "list-corners"));
            animate(
                gpui::ElementId::NamedChild(id.into(), ["tl", "tr", "br", "bl"][idx].into()),
                corners[idx],
                spatial,
                window,
                cx,
            )
            .max(0.)
        });
        let background = crate::motion::mix(
            tone(ColorRole::Surface, window, cx),
            if self.disabled {
                tone(ColorRole::OnSurface, window, cx).opacity(0.38)
            } else {
                tone(ColorRole::SecondaryContainer, window, cx)
            },
            selection,
        );
        let foreground = if self.selected && !self.disabled {
            ColorRole::OnSecondaryContainer
        } else {
            ColorRole::OnSurface
        };
        let secondary = if self.disabled {
            ColorRole::OnSurface
        } else if self.selected {
            ColorRole::OnSecondaryContainer
        } else {
            ColorRole::OnSurfaceVariant
        };
        let text_height = window
            .use_keyed_state((self.id.clone(), "list-text-height"), cx, |_, _| {
                Rc::new(Cell::new(px(0.)))
            })
            .read(cx)
            .clone();
        let measure = text_height.clone();
        let three_lines = self.overline.is_some() && self.supporting.is_some()
            || self
                .supporting
                .as_ref()
                .is_some_and(|text| text.contains('\n'))
            || text_height.get() > px(44.);
        let height = match (self.overline.is_some(), self.supporting.is_some()) {
            _ if three_lines => 88.,
            (false, false) => 56.,
            _ => 72.,
        };
        let content = div()
            .relative()
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
                    .wrap()
            }))
            .child(
                canvas(
                    move |bounds, window, _| {
                        if measure.replace(bounds.size.height) != bounds.size.height {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
        let state_layer = tone(foreground, window, cx);
        let focus_ring = tone(ColorRole::Secondary, window, cx);
        let (feedback, opacity) = surface_feedback(
            &self.id,
            self.activation.is_some() && !self.disabled,
            window,
            cx,
        );
        let interactive = self.activation.is_some() && !self.disabled;
        let ripple = ripple_state(&self.id, interactive, window, cx);
        let direction = window_material_theme(window, cx).layout.direction;
        let row = div()
            .id("list-surface")
            .relative()
            .min_h(px(height))
            .w_full()
            .flex()
            .when_else(three_lines, Styled::items_start, Styled::items_center)
            .gap(px(12.))
            .px(px(16.))
            .py(px(if three_lines { 12. } else { 10. }))
            .when(
                window_material_theme(window, cx).layout.direction == Direction::Rtl,
                Styled::flex_row_reverse,
            )
            .rounded_tl(px(corners[0]))
            .rounded_tr(px(corners[1]))
            .rounded_br(px(corners[2]))
            .rounded_bl(px(corners[3]))
            .bg(background.blend(state_layer.opacity(opacity)))
            .when(interactive, |row| {
                row.child(ripple_ink(
                    ripple.clone(),
                    state_layer,
                    gpui_material_theme::CornerShape::Radii(corners),
                    direction,
                ))
            })
            .when(interactive, |row| ripple_target(row, ripple))
            .children(self.leading.map(|leading| {
                div()
                    .when(self.disabled, |part| {
                        part.text_color(tone(ColorRole::OnSurface, window, cx).opacity(0.38))
                    })
                    .child(leading)
            }))
            .child(content.when(self.disabled, |content| content.opacity(0.38)))
            .children(self.trailing.map(|trailing| {
                div()
                    .when(self.disabled, |part| {
                        part.text_color(tone(ColorRole::OnSurface, window, cx).opacity(0.38))
                    })
                    .child(trailing)
            }));
        let row = match self.activation {
            Some(activate) => {
                let invoke = activate.clone();
                gpui_base::Button::new((self.id.clone(), "activate"))
                    .when_some(focus, |button, (focus, tab_stop)| {
                        button.track_focus(focus).tab_stop(tab_stop)
                    })
                    .when(!self.disabled, |button| monitor_surface(button, feedback))
                    .w_full()
                    .accessibility_label(self.label)
                    .disabled(self.disabled)
                    .selected(self.selected)
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .rounded_md()
                    .focus_visible(move |style| style.border_color(focus_ring))
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
            .test_support()
            .aria_selected(self.selected)
            .child(row)
            .into_any_element()
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.clone();
        self.row(
            [0.; 4],
            focus.as_ref().map(|(handle, tab)| (handle, *tab)),
            window,
            cx,
        )
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
        let direction = window_material_theme(window, cx).layout.direction;
        let focus: Vec<_> = self
            .items
            .iter()
            .map(|item| {
                window.with_id((self.id.clone(), "list-focus"), |window| {
                    window
                        .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
            })
            .collect();
        let enabled: Vec<_> = self
            .items
            .iter()
            .map(|item| !item.disabled && item.activation.is_some())
            .collect();
        let tab_stop = focus
            .iter()
            .enumerate()
            .position(|(ix, focus)| enabled[ix] && focus.is_focused(window))
            .or_else(|| {
                self.items
                    .iter()
                    .enumerate()
                    .position(|(ix, item)| enabled[ix] && item.selected)
            })
            .or_else(|| enabled.iter().position(|enabled| *enabled));
        let navigation = focus.clone();
        div()
            .id(self.id)
            .role(Role::List)
            .flex()
            .flex_col()
            .w_full()
            .on_key_down(move |event, window, cx| {
                // Nested controls keep their own arrow-key behavior.
                if !navigation.iter().any(|focus| focus.is_focused(window)) {
                    return;
                }
                let items: Vec<_> = navigation
                    .iter()
                    .zip(&enabled)
                    .map(|(focus, enabled)| (focus, *enabled))
                    .collect();
                if move_roving_focus(
                    &items,
                    &event.keystroke.key,
                    Axis::Vertical,
                    direction,
                    window,
                    cx,
                )
                .is_some()
                {
                    cx.stop_propagation();
                    window.refresh();
                }
            })
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
                item.row(
                    corners,
                    Some((&focus[ix], Some(ix) == tab_stop)),
                    window,
                    cx,
                )
            }))
    }
}

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
        let (left, right) = window_material_theme(window, cx)
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
                        window_material_theme(window, cx).layout.direction == Direction::Rtl,
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
    color(window_material_theme(window, cx).colors.color(role))
}

fn surface_feedback(
    id: &ElementId,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Rc<Cell<Option<StateLayer>>>, f32) {
    let state = window
        .use_keyed_state((id.clone(), "surface-interaction"), cx, |_, _| {
            Rc::new(Cell::new(None::<StateLayer>))
        })
        .read(cx)
        .clone();
    if !enabled || !window.is_window_active() {
        state.set(None);
    }
    let alpha = animate(
        (id.clone(), "surface-opacity"),
        state.get().map_or(0., StateLayer::opacity),
        window_material_theme(window, cx).motion.spec(
            MotionKind::Effects,
            MotionSpeed::Fast,
            cx.reduce_motion() || !enabled,
        ),
        window,
        cx,
    )
    .clamp(0., 1.);
    (state, alpha)
}

fn monitor_surface(
    button: gpui_base::Button,
    state: Rc<Cell<Option<StateLayer>>>,
) -> gpui_base::Button {
    let hover = state.clone();
    let down = state.clone();
    let up = state.clone();
    button
        .on_hover(move |hovering, window, _| {
            hover.set(hovering.then_some(StateLayer::Hover));
            window.refresh();
        })
        .on_mouse_down(MouseButton::Left, move |_, window, _| {
            down.set(Some(StateLayer::Pressed));
            window.refresh();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, _| {
            up.set(Some(StateLayer::Hover));
            window.refresh();
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, _| {
            state.set(None);
            window.refresh();
        })
}
