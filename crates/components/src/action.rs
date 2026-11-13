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
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Elevation, MotionKind, MotionSpeed};

pub(crate) type Activate = Rc<dyn Fn(&mut Window, &mut App)>;
pub(crate) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

pub(crate) enum Action {
    Invoke(Option<Activate>),
    Toggle {
        selected: bool,
        on_change: Option<Change>,
    },
}

pub(crate) struct ActionStyle {
    pub(crate) background: Option<ColorRole>,
    pub(crate) foreground: ColorRole,
    pub(crate) height: f32,
    pub(crate) min_width: f32,
    pub(crate) corner: CornerRole,
    pub(crate) pressed_corner: CornerRole,
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
struct FeedbackState {
    hovered: bool,
    pressed: bool,
    shape: Option<Shape>,
    ripple: Option<Ripple>,
}

impl ActionControl {
    pub(crate) fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
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
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let foreground = color(theme.colors.color(style.foreground));
        let background = style.background.map(|role| color(theme.colors.color(role)));
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let shadow = color(theme.colors.color(ColorRole::Shadow));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let visual_height = px(density.visual_size(style.height));
        let visual_width = px(density.visual_size(style.min_width));
        let target_height = px(density.interactive_size(style.height));
        let target_width = px(density.interactive_size(style.min_width.max(1.)));

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
        let progress = animate(feedback_id, f32::from(pressed), motion, window, cx).clamp(0., 1.);
        let (foreground, background) = if enabled {
            (foreground, background.unwrap_or_default())
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
        let mesh = feedback.clone();
        let paint_feedback = feedback.clone();
        let paint_focus = focus.clone();
        let visual = canvas(
            move |bounds, _, _| {
                let corners =
                    interpolated_corners(bounds, style.corner, style.pressed_corner, progress);
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
                            box_shadow(0., 1., elevation * 0.5, 0., shadow.opacity(0.13)),
                        ],
                    );
                }
                let outline = if enabled {
                    outline
                } else {
                    on_surface.opacity(0.12)
                };
                window.paint_quad(quad(
                    bounds,
                    radii,
                    background,
                    px(style.outline),
                    outline,
                    BorderStyle::Solid,
                ));

                let feedback = paint_feedback.borrow();
                if let Some(shape) = &feedback.shape {
                    let mut state = InteractionState::default();
                    state.disabled = !enabled;
                    state.hovered = hovered;
                    state.pressed = pressed;
                    state.focus_visible =
                        paint_focus.is_focused(window) && window.last_input_was_keyboard();
                    InteractionFeedback::new(state, foreground, ring).paint(
                        shape,
                        feedback.ripple.as_ref(),
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
                .on_hover(move |hovered, window, _| {
                    let mut state = hover.borrow_mut();
                    state.hovered = *hovered;
                    if !hovered {
                        state.pressed = false;
                        state.ripple = None;
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
                .on_mouse_up(MouseButton::Left, move |_, window, _| {
                    release.borrow_mut().pressed = false;
                    window.refresh();
                })
                .on_mouse_up_out(MouseButton::Left, move |_, window, _| {
                    let mut state = cancel.borrow_mut();
                    state.pressed = false;
                    state.ripple = None;
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
                .on_key_up(move |event, window, _| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        key_up.borrow_mut().pressed = false;
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
    start_ripple(&mut state.borrow_mut(), pointer, cx);
    window.refresh();
}

fn start_ripple(state: &mut FeedbackState, pointer: Option<Point<Pixels>>, cx: &App) {
    state.ripple = state
        .shape
        .as_ref()
        .and_then(|shape| Ripple::start(shape, pointer, cx.background_executor().now()));
}

fn interpolated_corners(
    bounds: Bounds<Pixels>,
    rest: CornerRole,
    pressed: CornerRole,
    progress: f32,
) -> CornerShape {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let direction = gpui_material_theme::Direction::Ltr;
    let from = rest.shape().radii(width, height, direction);
    let to = pressed.shape().radii(width, height, direction);
    CornerShape::Radii(std::array::from_fn(|ix| {
        from[ix] + (to[ix] - from[ix]) * progress
    }))
}

pub(crate) fn icon_slot(icon: AnyElement, side: f32) -> impl IntoElement {
    div()
        .size(px(side))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .child(icon)
}
