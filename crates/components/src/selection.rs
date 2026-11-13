use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Axis, Div, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, PathBuilder, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, canvas, div, point, prelude::FluentBuilder, px,
};
use gpui_base::{
    Checkbox as BaseCheckbox, Radio as BaseRadio, RadioGroup as BaseRadioGroup,
    Switch as BaseSwitch, TestSupportExt,
};
use gpui_material_foundation::{animate, color, move_roving_focus, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpeed, StateLayer};

use crate::action::Change;

pub use gpui_base::CheckboxState;

type CheckboxChange = Rc<dyn Fn(CheckboxState, &mut Window, &mut App)>;
type RadioChange = Rc<dyn Fn(&ElementId, &mut Window, &mut App)>;

fn tone(role: ColorRole, window: &Window, cx: &App) -> Hsla {
    color(window_material_theme(window, cx).colors.color(role))
}

fn selection_tone(selected: bool, disabled: bool, window: &Window, cx: &App) -> Hsla {
    if disabled {
        tone(ColorRole::OnSurface, window, cx).opacity(0.38)
    } else if selected {
        tone(ColorRole::Primary, window, cx)
    } else {
        tone(ColorRole::OnSurfaceVariant, window, cx)
    }
}

fn row(direction: Direction) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
}

fn selection_target(
    id: SharedString,
    indicator: impl IntoElement,
    accent: Hsla,
    enabled: bool,
) -> Div {
    div()
        .size(px(48.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("selection-indicator")
                .size(px(40.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .when(enabled, |target| {
                    let hover = id.clone();
                    target
                        .group_hover(hover, |style| {
                            style.bg(accent.opacity(StateLayer::Hover.opacity()))
                        })
                        .group_active(id, |style| {
                            style.bg(accent.opacity(StateLayer::Pressed.opacity()))
                        })
                })
                .child(indicator),
        )
}

fn focus_style(color: Hsla) -> impl FnOnce(gpui::StyleRefinement) -> gpui::StyleRefinement {
    move |style| style.border_color(color)
}

/// Controlled Material checkbox. Mixed activation requests Checked.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    state: CheckboxState,
    error: bool,
    disabled: bool,
    on_change: Option<CheckboxChange>,
}

impl Checkbox {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: CheckboxState,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state,
            error: false,
            disabled: false,
            on_change: None,
        }
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Derives a parent value from its children. An empty set is unchecked.
    pub fn aggregate(children: impl IntoIterator<Item = CheckboxState>) -> CheckboxState {
        let mut children = children.into_iter();
        let first = children.next().unwrap_or_default();
        if children.all(|child| child == first) {
            first
        } else {
            CheckboxState::Indeterminate
        }
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.state != CheckboxState::Unchecked;
        let accent = if self.error && !self.disabled {
            tone(ColorRole::Error, window, cx)
        } else {
            selection_tone(checked, self.disabled, window, cx)
        };
        let on_accent = tone(
            if self.disabled {
                ColorRole::Surface
            } else if self.error {
                ColorRole::OnError
            } else {
                ColorRole::OnPrimary
            },
            window,
            cx,
        );
        let mark = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let at = |x, y| bounds.origin + point(px(x), px(y));
                let mut path = PathBuilder::stroke(px(2.));
                if self.state == CheckboxState::Indeterminate {
                    path.move_to(at(4., 9.));
                    path.line_to(at(14., 9.));
                } else {
                    path.move_to(at(4., 9.));
                    path.line_to(at(7.5, 12.5));
                    path.line_to(at(14., 5.5));
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, on_accent);
                }
            },
        )
        .size_full();
        let indicator = div()
            .size(px(18.))
            .rounded(px(2.))
            .when(checked, |indicator| indicator.bg(accent).child(mark))
            .when(!checked, |indicator| {
                indicator.border_2().border_color(accent)
            });
        let group: SharedString = format!("material-checkbox/{}", self.id).into();
        let label = self.label;
        let control = BaseCheckbox::new(self.id)
            .state(self.state)
            .when(self.error, |control| {
                control.a11y_synthetic_children(|node| {
                    node.parent_node()
                        .set_invalid(gpui::accesskit::Invalid::True);
                })
            })
            .disabled(self.disabled)
            .accessibility_label(label.clone())
            .group(group.clone())
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                row(window_material_theme(window, cx).layout.direction)
                    .child(selection_target(group, indicator, accent, !self.disabled))
                    .child(label),
            );
        match self.on_change {
            Some(handler) if !self.disabled => {
                let change = handler.clone();
                let next = if self.state == CheckboxState::Checked {
                    CheckboxState::Unchecked
                } else {
                    CheckboxState::Checked
                };
                control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        change(next, window, cx)
                    })
                    .on_change(move |value, _, window, cx| handler(value, window, cx))
            }
            _ => control,
        }
    }
}

/// A radio option. A group owns the tab stop and the caller owns selection.
#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    label: SharedString,
    selected: bool,
    disabled: bool,
    on_select: Option<crate::action::Activate>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
            on_select: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    fn render_control(
        self,
        focus: Option<&FocusHandle>,
        position: Option<(usize, usize)>,
        tab_stop: bool,
        window: &Window,
        cx: &App,
    ) -> BaseRadio {
        let accent = selection_tone(self.selected, self.disabled, window, cx);
        let indicator = div()
            .size(px(20.))
            .rounded_full()
            .border_2()
            .border_color(accent)
            .flex()
            .items_center()
            .justify_center()
            .when(self.selected, |indicator| {
                indicator.child(div().size(px(10.)).rounded_full().bg(accent))
            });
        let group: SharedString = format!("material-radio/{}", self.id).into();
        let radio = BaseRadio::new(self.id)
            .checked(self.selected)
            .disabled(self.disabled)
            .tab_stop(tab_stop)
            .accessibility_label(self.label.clone())
            .group(group.clone())
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                row(window_material_theme(window, cx).layout.direction)
                    .child(selection_target(group, indicator, accent, !self.disabled))
                    .child(div().min_w_0().truncate().child(self.label)),
            )
            .when_some(focus, |radio, focus| radio.track_focus(focus))
            .when_some(position, |radio, (position, size)| {
                radio.set_position(position, size)
            });
        match self.on_select {
            Some(handler) if !self.disabled => {
                let select = handler.clone();
                radio
                    .when(!self.selected, |radio| {
                        radio.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            select(window, cx)
                        })
                    })
                    .on_change(move |_, _, window, cx| handler(window, cx))
            }
            _ => radio,
        }
    }
}

impl RenderOnce for Radio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.render_control(None, None, true, window, cx)
    }
}

/// A controlled single-selection group with one Tab stop and directional navigation.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    label: SharedString,
    selected: Option<ElementId>,
    options: Vec<Radio>,
    axis: Axis,
    on_select: Option<RadioChange>,
}

impl RadioGroup {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: None,
            options: Vec::new(),
            axis: Axis::Vertical,
            on_select: None,
        }
    }

    pub fn selected(mut self, selected: impl Into<ElementId>) -> Self {
        self.selected = Some(selected.into());
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = Radio>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&ElementId, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = window_material_theme(window, cx).layout.direction;
        let current = self
            .options
            .iter()
            .position(|item| !item.disabled && Some(&item.id) == self.selected.as_ref());
        let tab_stop = current.or_else(|| self.options.iter().position(|item| !item.disabled));
        let size = self.options.len();
        let focus: Vec<_> = self
            .options
            .iter()
            .map(|item| {
                let id = ElementId::from((self.id.clone(), "radio-focus"));
                window.with_id(id, |window| {
                    window
                        .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
            })
            .collect();
        let navigation: Vec<_> = self
            .options
            .iter()
            .zip(&focus)
            .map(|(item, focus)| (item.id.clone(), focus.clone(), !item.disabled))
            .collect();
        let change = self.on_select.clone();
        let selected = self.selected.clone();
        let axis = self.axis;
        let group = BaseRadioGroup::new(self.id.clone())
            .axis(axis)
            .aria_label(self.label)
            .flex()
            .gap_1()
            .when(axis == Axis::Vertical, Styled::flex_col)
            .when(axis == Axis::Horizontal, Styled::flex_wrap)
            .when(
                axis == Axis::Horizontal && direction == Direction::Rtl,
                Styled::flex_row_reverse,
            )
            .on_key_down(move |event, window, cx| {
                let items: Vec<_> = navigation
                    .iter()
                    .map(|(_, focus, enabled)| (focus, *enabled))
                    .collect();
                let Some(ix) =
                    move_roving_focus(&items, &event.keystroke.key, axis, direction, window, cx)
                else {
                    return;
                };
                cx.stop_propagation();
                if let Some(change) = &change
                    && Some(&navigation[ix].0) != selected.as_ref()
                {
                    change(&navigation[ix].0, window, cx);
                }
            });

        group.children(self.options.into_iter().zip(&focus).enumerate().map(
            |(ix, (mut radio, focus))| {
                let option_id = radio.id.clone();
                radio.id = ElementId::from((self.id.clone(), option_id.to_string()));
                radio.selected = Some(&option_id) == self.selected.as_ref();
                let handler = self.on_select.clone();
                radio.on_select = handler.map(|handler| {
                    let callback: crate::action::Activate =
                        Rc::new(move |window, cx| handler(&option_id, window, cx));
                    callback
                });
                radio.render_control(
                    Some(focus),
                    Some((ix + 1, size)),
                    Some(ix) == tab_stop,
                    window,
                    cx,
                )
            },
        ))
    }
}

/// Material switch. The caller owns the checked state and optional thumb icons.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    label: SharedString,
    checked: bool,
    disabled: bool,
    checked_icon: Option<AnyElement>,
    unchecked_icon: Option<AnyElement>,
    on_change: Option<Change>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            checked,
            disabled: false,
            checked_icon: None,
            unchecked_icon: None,
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn checked_icon(mut self, icon: impl IntoElement) -> Self {
        self.checked_icon = Some(icon.into_any_element());
        self
    }

    pub fn unchecked_icon(mut self, icon: impl IntoElement) -> Self {
        self.unchecked_icon = Some(icon.into_any_element());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use ColorRole::*;
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
        let progress = animate(
            ElementId::from((self.id.clone(), "thumb")),
            f32::from(self.checked),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        let (track, thumb, outline) = match (self.checked, self.disabled) {
            (true, false) => (tone(Primary, window, cx), tone(OnPrimary, window, cx), None),
            (false, false) => (
                tone(SurfaceContainerHighest, window, cx),
                tone(Outline, window, cx),
                Some(tone(Outline, window, cx)),
            ),
            (true, true) => (
                tone(OnSurface, window, cx).opacity(0.12),
                tone(Surface, window, cx),
                None,
            ),
            (false, true) => (
                tone(SurfaceContainerHighest, window, cx).opacity(0.12),
                tone(OnSurface, window, cx).opacity(0.38),
                Some(tone(OnSurface, window, cx).opacity(0.12)),
            ),
        };
        let icon = if self.checked {
            self.checked_icon
        } else {
            self.unchecked_icon
        };
        let group: SharedString = format!("material-switch/{}", self.id).into();
        let normal_size = if self.checked || icon.is_some() {
            24.
        } else {
            16.
        };
        let offset = px(24. * progress);
        let handle = div()
            .absolute()
            .top_0()
            .h_full()
            .w(px(28.))
            .flex()
            .items_center()
            .justify_center()
            .when(direction == Direction::Ltr, |handle| handle.left(offset))
            .when(direction == Direction::Rtl, |handle| handle.right(offset))
            .child(
                div()
                    .id((self.id.clone(), "switch-thumb"))
                    .test_support()
                    .size(px(normal_size))
                    .rounded_full()
                    .bg(thumb)
                    .text_color(tone(
                        if self.checked {
                            OnPrimaryContainer
                        } else {
                            SurfaceContainerHighest
                        },
                        window,
                        cx,
                    ))
                    .text_size(px(16.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(!self.disabled, |thumb| {
                        thumb.group_active(group.clone(), |style| style.size(px(28.)))
                    })
                    .children(icon),
            );
        let track = div()
            .id((self.id.clone(), "switch-track"))
            .test_support()
            .relative()
            .w(px(52.))
            .h(px(32.))
            .rounded_full()
            .bg(track)
            .when_some(outline, |track, outline| {
                track.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded_full()
                        .border_2()
                        .border_color(outline),
                )
            })
            .child(handle);
        let control = BaseSwitch::new(self.id)
            .checked(self.checked)
            .disabled(self.disabled)
            .accessibility_label(self.label.clone())
            .group(group)
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(Secondary, window, cx)))
            .child(
                row(direction)
                    .min_h(px(48.))
                    .px_1()
                    .child(track)
                    .child(self.label),
            );
        match self.on_change {
            Some(handler) if !self.disabled => {
                let change = handler.clone();
                control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        change(!self.checked, window, cx)
                    })
                    .on_change(move |checked, _, window, cx| handler(checked, window, cx))
            }
            _ => control,
        }
    }
}
