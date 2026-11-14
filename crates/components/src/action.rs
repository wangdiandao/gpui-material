use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, ClickEvent, Corners, ElementId,
    FocusHandle, Hsla, IntoElement, MouseButton, ParentElement, Pixels, Point,
    StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px, quad,
};
use gpui_base::{Button, Toggle, box_shadow};
use gpui_material_foundation::{
    InteractionFeedback, InteractionState, Ripple, Shape, animate, color, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

pub(crate) type Activate = Rc<dyn Fn(&mut Window, &mut App)>;
pub(crate) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

pub(crate) enum Action {
    Invoke(Option<Activate>),
    Toggle {
        selected: bool,
        on_change: Option<Change>,
    },
}

#[derive(Clone, Copy)]
pub(crate) struct ActionStyle {
    pub(crate) background: Option<ColorRole>,
    pub(crate) background_color: Option<Hsla>,
    pub(crate) foreground: ColorRole,
    pub(crate) height: f32,
    pub(crate) min_width: f32,
    pub(crate) corner: CornerRole,
    pub(crate) pressed_corner: CornerRole,
    pub(crate) selected_corner: Option<CornerRole>,
    pub(crate) corner_motion: (MotionKind, MotionSpeed),
    pub(crate) outline: f32,
    pub(crate) elevation: Elevation,
    pub(crate) hover_elevation: Elevation,
}

pub(crate) struct ActionControl {
    pub(crate) id: ElementId,
    pub(crate) label: gpui::SharedString,
    pub(crate) focus: Option<FocusHandle>,
    pub(crate) disabled: bool,
    pub(crate) action: Action,
    pub(crate) style: ActionStyle,
    pub(crate) content: AnyElement,
}

#[derive(Default)]
pub(crate) struct FeedbackState {
    hovered: bool,
    pressed: bool,
    shape: Option<Shape>,
    ripple: Option<Ripple>,
}

impl ActionControl {
    pub(crate) fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        self.render_scaled(1., window, cx)
    }

    pub(crate) fn render_scaled(self, scale: f32, window: &mut Window, cx: &mut App) -> AnyElement {
        let feedback_id = (self.id.clone(), "material-feedback");
        let (feedback, focus) = window
            .use_keyed_state(feedback_id.clone(), cx, |_, cx| {
                (
                    Rc::new(RefCell::new(FeedbackState::default())),
                    cx.focus_handle(),
                )
            })
            .read(cx)
            .clone();
        let focus = self.focus.unwrap_or(focus);
        let style = self.style;
        let theme = window_material_theme(window, cx);
        let density = theme.layout.density;
        let direction = theme.layout.direction;
        let reduced = cx.reduce_motion();
        let (corner_kind, corner_speed) = style.corner_motion;
        let motion = theme.motion.spec(corner_kind, corner_speed, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let selected = matches!(&self.action, Action::Toggle { selected: true, .. });
        let foreground = color(theme.colors.color(style.foreground));
        let background = style.background.map(|role| color(theme.colors.color(role)));
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let shadow = color(theme.colors.color(ColorRole::Shadow));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let visual_height = px(density.visual_size(style.height) * scale);
        let visual_width = px(density.visual_size(style.min_width) * scale);
        let target_height = px(density.interactive_size(style.height) * scale);
        let target_width = px(density.interactive_size(style.min_width.max(1.)) * scale);

        let enabled = !self.disabled;
        let (hovered, pressed) = {
            let mut state = feedback.borrow_mut();
            if !enabled || !window.is_window_active() {
                state.pressed = false;
                state.ripple = None;
            }
            (
                state.hovered,
                enabled
                    && state.pressed
                    && (state.hovered
                        || focus.is_focused(window) && window.last_input_was_keyboard()),
            )
        };
        let progress = animate(
            (self.id.clone(), "press-shape"),
            f32::from(pressed),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        let selection = if style.selected_corner.is_some() {
            animate(
                (self.id.clone(), "selected-shape"),
                f32::from(selected),
                motion,
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let focus_visible = focus.is_focused(window) && window.last_input_was_keyboard();
        let mut interaction = InteractionState::default();
        interaction.disabled = !enabled;
        interaction.hovered = hovered;
        interaction.pressed = pressed;
        interaction.focus_visible = focus_visible;
        let layer = animate(
            (self.id.clone(), "state-layer"),
            interaction.layer().map_or(0., |layer| layer.opacity()),
            if enabled {
                effects
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .clamp(0., 1.);
        let outline_width = animate(
            (self.id.clone(), "outline-width"),
            style.outline,
            effects,
            window,
            cx,
        )
        .max(0.)
            * scale;
        let (foreground, background) = if enabled {
            (
                foreground,
                style.background_color.or(background).unwrap_or_default(),
            )
        } else {
            (
                on_surface.opacity(0.38),
                background.map_or_else(Hsla::default, |_| on_surface.opacity(0.12)),
            )
        };
        let elevation = if !enabled {
            0.
        } else if hovered && !pressed {
            style.hover_elevation.dp()
        } else {
            style.elevation.dp()
        };
        let elevation = animate(
            (self.id.clone(), "elevation"),
            elevation,
            effects,
            window,
            cx,
        )
        .max(0.)
            * scale;
        let mesh = feedback.clone();
        let paint_feedback = feedback.clone();
        let visual = canvas(
            move |bounds, _, _| {
                let corners = interpolated_corners(bounds, style, selection, progress, scale);
                let shape = Shape::rounded(bounds, corners, direction);
                mesh.borrow_mut().shape = shape;
                corners
                    .radii(
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                        direction,
                    )
                    .map(px)
            },
            move |bounds, radii, window, cx| {
                let [top_left, top_right, bottom_right, bottom_left] = radii;
                let radii = Corners {
                    top_left,
                    top_right,
                    bottom_left,
                    bottom_right,
                };
                if elevation > 0. {
                    window.paint_drop_shadows(
                        bounds,
                        radii,
                        &[
                            box_shadow(0., elevation * 0.4, elevation, 0., shadow.opacity(0.18)),
                            box_shadow(0., scale, elevation * 0.5, 0., shadow.opacity(0.13)),
                        ],
                    );
                }
                let outline = if enabled {
                    outline
                } else {
                    on_surface.opacity(0.12)
                };
                // A zero-width outline is still visible through antialiasing on some
                // backends if its pigment is opaque.
                if background.a > 0. || outline_width > 0. {
                    window.paint_quad(quad(
                        bounds,
                        radii,
                        background,
                        px(outline_width),
                        if outline_width > 0. {
                            outline
                        } else {
                            gpui::transparent_black()
                        },
                        BorderStyle::Solid,
                    ));
                }

                let feedback = paint_feedback.borrow();
                if let Some(shape) = &feedback.shape {
                    InteractionFeedback::new(interaction, foreground, ring).paint_with_opacity(
                        shape,
                        feedback.ripple.as_ref(),
                        layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
                }
            },
        )
        .absolute()
        .inset_0()
        .size_full();

        // The visual may be smaller than the interactive target. GPUI and Base
        // retain a single target and semantic node as the shape changes.
        let content = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .min_w(visual_width)
            .max_w_full()
            .min_h(visual_height)
            .child(visual)
            .child(self.content);
        let id = self.id;
        let label = self.label;
        let disabled = self.disabled;
        match self.action {
            Action::Invoke(on_activate) => {
                let control = Button::new(id)
                    .track_focus(&focus)
                    .disabled(disabled)
                    .accessibility_label(label);
                let control = match on_activate {
                    Some(on_activate) if enabled => {
                        let feedback = feedback.clone();
                        let invoke = on_activate.clone();
                        control
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                invoke(window, cx)
                            })
                            .on_click(move |event, window, cx| {
                                activate_feedback(event, &feedback, window, cx);
                                cx.stop_propagation();
                                on_activate(window, cx);
                            })
                    }
                    _ => control,
                };
                decorate(
                    control,
                    content,
                    feedback,
                    foreground,
                    target_height,
                    target_width,
                    enabled,
                )
            }
            Action::Toggle {
                selected,
                on_change,
            } => {
                let control = Toggle::new(id)
                    .track_focus(&focus)
                    .pressed(selected)
                    .disabled(disabled)
                    .accessibility_label(label);
                let control = match on_change {
                    Some(on_change) if enabled => {
                        let feedback = feedback.clone();
                        let toggle = on_change.clone();
                        control
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                toggle(!selected, window, cx)
                            })
                            .on_change(move |selected, event, window, cx| {
                                activate_feedback(event, &feedback, window, cx);
                                cx.stop_propagation();
                                on_change(selected, window, cx);
                            })
                    }
                    _ => control,
                };
                decorate(
                    control,
                    content,
                    feedback,
                    foreground,
                    target_height,
                    target_width,
                    enabled,
                )
            }
        }
    }
}

fn decorate(
    control: impl Styled + gpui::StatefulInteractiveElement + ParentElement + IntoElement,
    content: impl IntoElement,
    feedback: Rc<RefCell<FeedbackState>>,
    foreground: Hsla,
    height: Pixels,
    width: Pixels,
    enabled: bool,
) -> AnyElement {
    let hover = feedback.clone();
    let press = feedback.clone();
    let release = feedback.clone();
    let cancel = feedback.clone();
    let keys = feedback.clone();
    let key_up = feedback.clone();
    control
        .min_h(height)
        .min_w(width)
        .max_w_full()
        .text_color(foreground)
        .child(content)
        .when(enabled, |control| {
            control
                .on_hover(move |hovered, window, cx| {
                    let mut state = hover.borrow_mut();
                    state.hovered = *hovered;
                    if !hovered {
                        release_ripple(&mut state, cx);
                    }
                    window.refresh();
                })
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    let mut state = press.borrow_mut();
                    state.pressed = true;
                    state.hovered = true;
                    start_ripple(&mut state, Some(event.position), cx);
                    window.refresh();
                })
                .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                    release_ripple(&mut release.borrow_mut(), cx);
                    window.refresh();
                })
                .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                    release_ripple(&mut cancel.borrow_mut(), cx);
                    window.refresh();
                })
                .on_key_down(move |event, window, cx| {
                    let mut state = keys.borrow_mut();
                    match event.keystroke.key.as_str() {
                        "space" | "enter" if !event.is_held => {
                            state.pressed = true;
                            start_ripple(&mut state, None, cx);
                        }
                        "escape" => {
                            state.pressed = false;
                            state.ripple = None;
                        }
                        _ => return,
                    }
                    window.refresh();
                })
                .on_key_up(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        release_ripple(&mut key_up.borrow_mut(), cx);
                        window.refresh();
                    }
                })
        })
        .into_any_element()
}

fn activate_feedback(
    event: &ClickEvent,
    state: &RefCell<FeedbackState>,
    window: &mut Window,
    cx: &App,
) {
    let pointer = match event {
        ClickEvent::Mouse(_) => return,
        ClickEvent::Keyboard(_) => None,
        ClickEvent::Touch(event) => Some(event.position),
    };
    let mut state = state.borrow_mut();
    if !state.pressed {
        start_ripple(&mut state, pointer, cx);
        release_ripple(&mut state, cx);
    }
    window.refresh();
}

fn release_ripple(state: &mut FeedbackState, cx: &App) {
    state.pressed = false;
    if let Some(ripple) = &mut state.ripple {
        ripple.release(cx.background_executor().now());
    }
}

fn start_ripple(state: &mut FeedbackState, pointer: Option<Point<Pixels>>, cx: &App) {
    state.ripple = state
        .shape
        .as_ref()
        .and_then(|shape| Ripple::press(shape, pointer, cx.background_executor().now()));
}

fn interpolated_corners(
    bounds: Bounds<Pixels>,
    style: ActionStyle,
    selection: f32,
    press: f32,
    scale: f32,
) -> CornerShape {
    let width = f32::from(bounds.size.width) / scale;
    let height = f32::from(bounds.size.height) / scale;
    let direction = gpui_material_theme::Direction::Ltr;
    let rest = style.corner.shape().radii(width, height, direction);
    let selected = style
        .selected_corner
        .unwrap_or(style.corner)
        .shape()
        .radii(width, height, direction);
    let pressed = style.pressed_corner.shape().radii(width, height, direction);
    CornerShape::Radii(std::array::from_fn(|ix| {
        let resting = rest[ix] + (selected[ix] - rest[ix]) * selection;
        (resting + (pressed[ix] - resting) * press) * scale
    }))
}

pub(crate) fn ripple_target<E>(control: E, state: Rc<RefCell<FeedbackState>>) -> E
where
    E: StatefulInteractiveElement,
{
    let press = state.clone();
    let release = state.clone();
    let outside = state.clone();
    let hover = state.clone();
    let keyboard = state.clone();
    control
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let mut state = press.borrow_mut();
            state.pressed = true;
            let pointer = state
                .shape
                .as_ref()
                .filter(|shape| shape.contains(event.position))
                .map(|_| event.position);
            start_ripple(&mut state, pointer, cx);
            window.refresh();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            release_ripple(&mut release.borrow_mut(), cx);
            window.refresh();
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            release_ripple(&mut outside.borrow_mut(), cx);
            window.refresh();
        })
        .on_hover(move |hovering, window, cx| {
            if !*hovering && hover.borrow().pressed {
                release_ripple(&mut hover.borrow_mut(), cx);
                window.refresh();
            }
        })
        .on_key_down(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "space" | "enter") && !event.is_held {
                let mut state = keyboard.borrow_mut();
                state.pressed = true;
                start_ripple(&mut state, None, cx);
                window.refresh();
            }
        })
        .on_key_up(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                release_ripple(&mut state.borrow_mut(), cx);
                window.refresh();
            }
        })
}

pub(crate) fn ripple_state(
    id: &ElementId,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> Rc<RefCell<FeedbackState>> {
    let state = window
        .use_keyed_state((id.clone(), "ripple"), cx, |_, _| {
            Rc::new(RefCell::new(FeedbackState::default()))
        })
        .read(cx)
        .clone();
    if !enabled || !window.is_window_active() {
        state.borrow_mut().ripple = None;
        state.borrow_mut().pressed = false;
    }
    state
}

pub(crate) fn ripple_ink(
    state: Rc<RefCell<FeedbackState>>,
    tint: Hsla,
    corners: CornerShape,
    direction: gpui_material_theme::Direction,
) -> impl IntoElement {
    let geometry = state.clone();
    canvas(
        move |bounds, _, _| {
            geometry.borrow_mut().shape = Shape::rounded(bounds, corners, direction);
        },
        move |_, _, window, cx| {
            let state = state.borrow();
            if let (Some(shape), Some(ripple)) = (&state.shape, &state.ripple) {
                ripple.paint(
                    shape,
                    tint,
                    cx.background_executor().now(),
                    cx.reduce_motion(),
                    window,
                );
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

pub(crate) fn icon_slot(icon: AnyElement, side: f32) -> impl IntoElement {
    div()
        .size(px(side))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(side))
        .line_height(px(side))
        .child(icon)
}
