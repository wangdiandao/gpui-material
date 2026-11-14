use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Axis, BorderStyle, Div, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, PathBuilder, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, point, prelude::FluentBuilder, px,
    quad,
};
use gpui_base::{
    Checkbox as BaseCheckbox, Radio as BaseRadio, RadioGroup as BaseRadioGroup,
    Switch as BaseSwitch, TestSupportExt,
};
use gpui_material_foundation::{animate, color, move_roving_focus, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpec, MotionSpeed, StateLayer};

use crate::action::{Change, FeedbackState, ripple_ink, ripple_state, ripple_target};

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
    ripple: Rc<std::cell::RefCell<FeedbackState>>,
    direction: Direction,
    reduced_motion: bool,
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
                .relative()
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
                        .when(reduced_motion, |target| {
                            target.group_active(id, |style| {
                                style.bg(accent.opacity(StateLayer::Pressed.opacity()))
                            })
                        })
                })
                .child(ripple_ink(
                    ripple,
                    accent,
                    gpui_material_theme::CornerRole::Full.shape(),
                    direction,
                ))
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
        let motion = window_material_theme(window, cx).motion;
        let reduced = self.disabled || cx.reduce_motion();
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let draw = animate(
            (self.id.clone(), "check-draw"),
            f32::from(checked),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let mixed = animate(
            (self.id.clone(), "check-shape"),
            f32::from(self.state == CheckboxState::Indeterminate),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let fill = animate(
            (self.id.clone(), "check-fill"),
            f32::from(checked),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let inactive = if self.error && !self.disabled {
            accent
        } else {
            selection_tone(false, self.disabled, window, cx)
        };
        let border = crate::motion::mix(inactive, accent, fill);
        let indicator = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let stroke = px(2.);
                // The outline and mark share the outer box coordinates, as in Compose.
                window.paint_quad(quad(
                    bounds,
                    px(2.),
                    accent.opacity(fill),
                    stroke,
                    border,
                    BorderStyle::Solid,
                ));
                if draw == 0. {
                    return;
                }
                let side = bounds.size.width;
                let start = point(side * 0.2, side * 0.5);
                let knee = point(side * (0.4 + 0.1 * mixed), side * (0.7 - 0.2 * mixed));
                let end = point(side * 0.8, side * (0.3 + 0.2 * mixed));
                let first = f32::from(knee.x - start.x).hypot(f32::from(knee.y - start.y));
                let second = f32::from(end.x - knee.x).hypot(f32::from(end.y - knee.y));
                let distance = (first + second) * draw;
                let mut path = PathBuilder::stroke(stroke);
                path.move_to(bounds.origin + start);
                if distance <= first {
                    path.line_to(bounds.origin + start + (knee - start) * (distance / first));
                } else {
                    path.line_to(bounds.origin + knee);
                    path.line_to(
                        bounds.origin + knee + (end - knee) * ((distance - first) / second),
                    );
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, on_accent);
                }
            },
        )
        .size(px(18.))
        .flex_none();
        let direction = window_material_theme(window, cx).layout.direction;
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let group: SharedString = format!("material-checkbox/{}", self.id).into();
        let label = self.label;
        let control = BaseCheckbox::new(self.id)
            .when(!self.disabled, |control| {
                ripple_target(control, ripple.clone())
            })
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
                row(direction)
                    .child(selection_target(
                        group,
                        indicator,
                        accent,
                        !self.disabled,
                        ripple,
                        direction,
                        cx.reduce_motion(),
                    ))
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
        window: &mut Window,
        cx: &mut App,
    ) -> BaseRadio {
        let motion = window_material_theme(window, cx).motion;
        let reduced = self.disabled || cx.reduce_motion();
        let dot = animate(
            (self.id.clone(), "radio-dot"),
            f32::from(self.selected),
            motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let tint = animate(
            (self.id.clone(), "radio-color"),
            f32::from(self.selected),
            motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let accent = crate::motion::mix(
            selection_tone(false, self.disabled, window, cx),
            selection_tone(true, self.disabled, window, cx),
            tint,
        );
        let indicator = div()
            .size(px(20.))
            .rounded_full()
            .border_2()
            .border_color(accent)
            .flex()
            .items_center()
            .justify_center()
            .child(div().size(px(10. * dot)).rounded_full().bg(accent));
        let direction = window_material_theme(window, cx).layout.direction;
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let group: SharedString = format!("material-radio/{}", self.id).into();
        let radio = BaseRadio::new(self.id)
            .when(!self.disabled, |control| {
                ripple_target(control, ripple.clone())
            })
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
                row(direction)
                    .child(selection_target(
                        group,
                        indicator,
                        accent,
                        !self.disabled,
                        ripple,
                        direction,
                        cx.reduce_motion(),
                    ))
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
        let motion = theme.motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Fast,
            self.disabled || cx.reduce_motion(),
        );
        let tint = animate(
            (self.id.clone(), "switch-color"),
            f32::from(self.checked),
            window_material_theme(window, cx).motion.spec(
                MotionKind::Effects,
                MotionSpeed::Fast,
                self.disabled || cx.reduce_motion(),
            ),
            window,
            cx,
        )
        .clamp(0., 1.);
        let outline = tone(Outline, window, cx);
        // Compose precomposites disabled switch pigments over surface. Keeping
        // these translucent lets the track bleed through the handle and icon.
        let disabled_tone = |role, opacity| {
            tone(Surface, window, cx).blend(tone(role, window, cx).opacity(opacity))
        };
        let (track, thumb, outline) = if self.disabled {
            (
                disabled_tone(
                    if self.checked {
                        OnSurface
                    } else {
                        SurfaceContainerHighest
                    },
                    0.12,
                ),
                if self.checked {
                    tone(Surface, window, cx)
                } else {
                    disabled_tone(OnSurface, 0.38)
                },
                if self.checked {
                    gpui::transparent_black()
                } else {
                    disabled_tone(OnSurface, 0.12)
                },
            )
        } else {
            (
                crate::motion::mix(
                    tone(SurfaceContainerHighest, window, cx),
                    tone(Primary, window, cx),
                    tint,
                ),
                crate::motion::mix(outline, tone(OnPrimary, window, cx), tint),
                outline.opacity(1. - tint),
            )
        };
        let icon_color = if self.disabled {
            disabled_tone(
                if self.checked {
                    OnSurface
                } else {
                    SurfaceContainerHighest
                },
                0.38,
            )
        } else {
            tone(
                if self.checked {
                    OnPrimaryContainer
                } else {
                    SurfaceContainerHighest
                },
                window,
                cx,
            )
        };
        let icon = if self.checked {
            self.checked_icon
        } else {
            self.unchecked_icon
        };
        let normal_size = if self.checked || icon.is_some() {
            24.
        } else {
            16.
        };
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let active = ripple.borrow().pressed();
        let diameter = animate(
            (self.id.clone(), "thumb-diameter"),
            if active { 28. } else { normal_size },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        )
        .clamp(16., 28.);
        let offset = animate(
            (self.id.clone(), "thumb-offset"),
            match (active, self.checked) {
                (true, false) => 2.,
                (true, true) => 22.,
                (false, true) => 24.,
                (false, false) => (32. - normal_size) / 2.,
            },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        );
        // Offset and diameter are sampled independently, including a reversal on release.
        let offset = px(offset.clamp(2., 50. - diameter));
        let ink = ripple.clone();
        let group: SharedString = format!("material-switch/{}", self.id).into();
        let handle = div()
            .absolute()
            .top_0()
            .h_full()
            .w(px(diameter))
            .flex()
            .items_center()
            .justify_center()
            .when(direction == Direction::Ltr, |handle| handle.left(offset))
            .when(direction == Direction::Rtl, |handle| handle.right(offset))
            .child(
                div()
                    .id((self.id.clone(), "switch-thumb"))
                    .relative()
                    .test_support()
                    .size(px(diameter))
                    .rounded_full()
                    .bg(thumb)
                    .text_color(icon_color)
                    .text_size(px(16.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .id("switch-state-layer")
                            .absolute()
                            .size(px(40.))
                            .left(px((diameter - 40.) / 2.))
                            .top(px((diameter - 40.) / 2.))
                            .rounded_full()
                            .when(!self.disabled, |layer| {
                                let accent = tone(
                                    if self.checked { Primary } else { OnSurface },
                                    window,
                                    cx,
                                );
                                layer
                                    .group_hover(group.clone(), move |style| {
                                        style.bg(accent.opacity(StateLayer::Hover.opacity()))
                                    })
                                    .when(cx.reduce_motion(), |layer| {
                                        layer.group_active(group.clone(), move |style| {
                                            style.bg(accent.opacity(StateLayer::Pressed.opacity()))
                                        })
                                    })
                            })
                            .child(ripple_ink(
                                ink,
                                tone(if self.checked { Primary } else { OnSurface }, window, cx),
                                gpui_material_theme::CornerRole::Full.shape(),
                                direction,
                            )),
                    )
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
            .when(outline.a > 0., |track| {
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
            .group(group)
            .accessibility_label(self.label.clone())
            .when(!self.disabled, |control| ripple_target(control, ripple))
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(Secondary, window, cx)))
            .child(
                row(direction)
                    .id("switch-row")
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
