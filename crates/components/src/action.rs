use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, ClickEvent, Corners, ElementId, Entity,
    FocusHandle, Hsla, IntoElement, MouseButton, ParentElement, Pixels, Point,
    StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px, quad,
};
use gpui_base::{Button, Toggle};
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, InteractionTracker, Ripple, Shape,
    animate, color, elevation_shadows, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

use crate::button_style::{ButtonShapes, ButtonStyle, finite_length};

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
    pub(crate) disabled: DisabledStyle,
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

/// Disabled pigments are component tokens, not a universal opacity applied to
/// the enabled colors. In particular buttons and chips use different fills.
#[derive(Clone, Copy)]
pub(crate) struct DisabledStyle {
    pub(crate) foreground: ColorRole,
    pub(crate) background: Option<(ColorRole, f32)>,
    pub(crate) outline: (ColorRole, f32),
}

impl ActionStyle {
    pub(crate) fn foreground(self, disabled: bool, window: &Window, cx: &App) -> Hsla {
        let colors = &window_material_theme(window, cx).colors;
        if disabled {
            color(colors.color(self.disabled.foreground)).opacity(0.38)
        } else {
            color(colors.color(self.foreground))
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ActionLayout {
    pub(crate) segment: Option<SegmentShape>,
    pub(crate) width: Option<Pixels>,
    pub(crate) tab_stop: bool,
    pub(crate) semantic_only: bool,
    pub(crate) decorative: bool,
    pub(crate) appearance: ButtonStyle,
}

#[derive(Clone, Copy)]
pub(crate) struct SegmentShape {
    pub(crate) leading: bool,
    pub(crate) trailing: bool,
    pub(crate) inner: CornerRole,
    pub(crate) pressed: CornerRole,
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
    dragged: bool,
    pointer_cancelled: bool,
    keyboard_focus: Option<FocusHandle>,
    pub(crate) shape: Option<Shape>,
    pub(crate) ripple: Option<Ripple>,
    tracker: Option<(InteractionTracker, FocusHandle)>,
}

impl FeedbackState {
    fn interaction(&self, enabled: bool, focus: &FocusHandle, window: &Window) -> InteractionState {
        let mut interaction = InteractionState::default();
        interaction.disabled = !enabled;
        interaction.hovered = self.hovered;
        interaction.focus_visible = focus.is_focused(window) && window.last_input_was_keyboard();
        interaction.pressed =
            enabled && self.pressed && (self.hovered || interaction.focus_visible);
        interaction.dragged = enabled && self.dragged;
        interaction
    }

    fn record(&self, window: &Window, cx: &mut App) {
        if let Some((tracker, focus)) = &self.tracker {
            tracker.record(self.interaction(true, focus, window), cx);
        }
    }

    fn cancel_unavailable(&mut self, enabled: bool, window: &Window) {
        if !enabled || !window.is_window_active() {
            self.hovered = false;
            self.cancel();
        } else if self
            .keyboard_focus
            .as_ref()
            .is_some_and(|focus| !focus.is_focused(window))
        {
            // Key-up is delivered to the new focus owner. Like Base's pending
            // keyboard click, presentation must cancel when its owner blurs.
            self.cancel();
        }
    }

    fn cancel(&mut self) {
        self.pressed = false;
        self.keyboard_focus = None;
        self.ripple = None;
    }

    pub(crate) fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) fn set_dragged(&mut self, dragged: bool) {
        if dragged {
            self.pointer_cancelled = true;
            self.cancel();
        }
        self.dragged = dragged;
    }

    pub(crate) fn track(
        &mut self,
        enabled: bool,
        focus: &FocusHandle,
        tracker: Option<InteractionTracker>,
        window: &Window,
        cx: &mut App,
    ) -> InteractionState {
        let state = self.interaction(enabled, focus, window);
        if let Some(tracker) = &tracker {
            tracker.record(state, cx);
        }
        self.tracker = tracker.map(|tracker| (tracker, focus.clone()));
        state
    }

    pub(crate) fn blocks_activation(&self, event: &ClickEvent) -> bool {
        self.dragged || self.pointer_cancelled && matches!(event, ClickEvent::Mouse(_))
    }
}

impl ActionControl {
    fn feedback(
        &self,
        window: &mut Window,
        cx: &mut App,
    ) -> (Rc<RefCell<FeedbackState>>, FocusHandle) {
        let (feedback, focus) = window
            .use_keyed_state((self.id.clone(), "material-feedback"), cx, |_, cx| {
                (
                    Rc::new(RefCell::new(FeedbackState::default())),
                    cx.focus_handle(),
                )
            })
            .read(cx);
        (
            feedback.clone(),
            self.focus.as_ref().unwrap_or(focus).clone(),
        )
    }

    pub(crate) fn clear_interaction(&self, window: &mut Window, cx: &mut App) {
        let (feedback, focus) = self.feedback(window, cx);
        let mut state = feedback.borrow_mut();
        state.cancel();
        state.hovered = false;
        state.tracker = None;
        if focus.is_focused(window) {
            window.defer(cx, move |window, cx| {
                if focus.is_focused(window) {
                    window.blur(cx);
                }
            });
        }
    }

    pub(crate) fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        self.render_scaled(1., None, ButtonStyle::default(), window, cx)
    }

    pub(crate) fn render_scaled(
        self,
        scale: f32,
        interaction_source: Option<&Entity<InteractionSource>>,
        appearance: ButtonStyle,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_layout(
            scale,
            ActionLayout {
                tab_stop: true,
                appearance,
                ..Default::default()
            },
            interaction_source,
            window,
            cx,
        )
    }

    pub(crate) fn render_layout(
        self,
        scale: f32,
        layout: ActionLayout,
        interaction_source: Option<&Entity<InteractionSource>>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (feedback, focus) = self.feedback(window, cx);
        let tracker =
            interaction_source.map(|source| InteractionSource::track(source, &self.id, window, cx));
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
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let shadow = color(theme.colors.color(ColorRole::Shadow));
        let foreground = layout
            .appearance
            .colors
            .content(self.disabled, selected)
            .unwrap_or_else(|| style.foreground(self.disabled, window, cx));
        let background = layout
            .appearance
            .colors
            .container(self.disabled, selected)
            .unwrap_or_else(|| {
                if self.disabled {
                    style
                        .disabled
                        .background
                        .map_or_else(Hsla::default, |(role, opacity)| {
                            color(theme.colors.color(role)).opacity(opacity)
                        })
                } else {
                    style
                        .background_color
                        .or_else(|| style.background.map(|role| color(theme.colors.color(role))))
                        .unwrap_or_default()
                }
            });
        let outline = layout.appearance.border.map_or_else(
            || {
                if self.disabled {
                    color(theme.colors.color(style.disabled.outline.0))
                        .opacity(style.disabled.outline.1)
                } else {
                    color(theme.colors.color(ColorRole::OutlineVariant))
                }
            },
            |border| border.color,
        );
        let visual_height = px(density.visual_size(style.height) * scale);
        let visual_width = layout
            .width
            .unwrap_or_else(|| px(density.visual_size(style.min_width) * scale));
        let target_height = px(density.interactive_size(style.height) * scale);
        let target_width = layout
            .width
            .unwrap_or_else(|| px(density.interactive_size(style.min_width.max(1.)) * scale));

        let enabled = !self.disabled;
        let interaction = {
            let mut state = feedback.borrow_mut();
            state.cancel_unavailable(enabled, window);
            let interaction = if layout.decorative {
                let mut interaction = InteractionState::default();
                interaction.disabled = self.disabled;
                interaction
            } else {
                state.interaction(enabled, &focus, window)
            };
            if let Some(tracker) = &tracker {
                tracker.record(interaction, cx);
            }
            state.tracker = tracker.map(|tracker| (tracker, focus.clone()));
            interaction
        };
        let hovered = interaction.hovered;
        let pressed = interaction.pressed;
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
        let layer = animate(
            (self.id.clone(), "state-layer"),
            interaction
                .layer_with_ripple(reduced)
                .map_or(0., |layer| layer.opacity()),
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
            layout.appearance.border.map_or(style.outline, |border| {
                f32::from(finite_length(border.width))
            }),
            effects,
            window,
            cx,
        )
        .max(0.);
        let default_elevation = if !enabled {
            0.
        } else if hovered && !pressed {
            style.hover_elevation.dp()
        } else {
            style.elevation.dp()
        };
        let elevation = animate(
            (self.id.clone(), "elevation"),
            layout
                .appearance
                .elevations
                .for_state(interaction)
                .map_or(default_elevation, Elevation::dp),
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
                let corners =
                    interpolated_corners(bounds, style, selection, progress, scale, layout);
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
                        &elevation_shadows(elevation, scale, shadow),
                    );
                }
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
        if layout.decorative {
            return div()
                .min_h(target_height)
                .min_w(target_width)
                .max_w_full()
                .text_color(foreground)
                .child(content)
                .into_any_element();
        }
        let id = self.id;
        let label = self.label;
        let disabled = self.disabled;
        match self.action {
            Action::Invoke(on_activate) => {
                let control = Button::new(id)
                    .track_focus(&focus)
                    .tab_stop(layout.tab_stop)
                    .disabled(disabled)
                    .accessibility_label(label);
                let control = match on_activate {
                    Some(on_activate) if enabled => {
                        let feedback = feedback.clone();
                        let invoke = on_activate.clone();
                        let control = control
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                invoke(window, cx)
                            });
                        control.when(!layout.semantic_only, |control| {
                            control.on_click(move |event, window, cx| {
                                activate_feedback(event, &feedback, window, cx);
                                cx.stop_propagation();
                                on_activate(window, cx);
                            })
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
                    (target_width, layout.width.is_some()),
                    enabled,
                )
            }
            Action::Toggle {
                selected,
                on_change,
            } => {
                let control = Toggle::new(id)
                    .track_focus(&focus)
                    .tab_stop(layout.tab_stop)
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
                    (target_width, layout.width.is_some()),
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
    (width, fixed_width): (Pixels, bool),
    enabled: bool,
) -> AnyElement {
    let control = control
        .min_h(height)
        .min_w(width)
        .when(fixed_width, |control| control.w(width))
        .max_w_full()
        .text_color(foreground)
        .child(content);
    control
        .when(enabled, |control| {
            feedback_target(control, feedback, true, None)
        })
        .into_any_element()
}

pub(crate) fn activate_feedback(
    event: &ClickEvent,
    state: &RefCell<FeedbackState>,
    window: &mut Window,
    cx: &mut App,
) {
    let mut state = state.borrow_mut();
    match event {
        // Activation stops the bubbling click. Complete the pointer feedback
        // here even if the separate mouse-up listener does not run.
        ClickEvent::Mouse(_) => release_ripple(&mut state, cx),
        event if !state.pressed => {
            let pointer = match event {
                ClickEvent::Touch(event) => Some(event.position),
                _ => None,
            };
            start_ripple(&mut state, pointer, cx);
            release_ripple(&mut state, cx);
        }
        _ => {}
    }
    state.record(window, cx);
    window.refresh();
}

fn release_ripple(state: &mut FeedbackState, cx: &App) {
    state.pressed = false;
    state.keyboard_focus = None;
    if let Some(ripple) = &mut state.ripple {
        ripple.release(cx.background_executor().now());
    }
}

fn start_ripple(state: &mut FeedbackState, pointer: Option<Point<Pixels>>, cx: &App) {
    state.ripple = state.shape.as_ref().and_then(|shape| {
        // The accessible hit target can extend beyond the painted container.
        // Its activation still needs feedback, centered in the visible shape.
        let pointer = pointer.filter(|pointer| shape.contains(*pointer));
        Ripple::press(shape, pointer, cx.background_executor().now())
    });
}

fn interpolated_corners(
    bounds: Bounds<Pixels>,
    style: ActionStyle,
    selection: f32,
    press: f32,
    scale: f32,
    layout: ActionLayout,
) -> CornerShape {
    let width = f32::from(bounds.size.width) / scale;
    let height = f32::from(bounds.size.height) / scale;
    let radii =
        |shape: CornerShape| shape.radii(width, height, gpui_material_theme::Direction::Ltr);
    let ButtonShapes {
        resting,
        pressed,
        checked,
    } = layout.appearance.shapes;
    let mut rest = radii(resting.unwrap_or_else(|| style.corner.shape()));
    let mut selected =
        radii(checked.unwrap_or_else(|| style.selected_corner.unwrap_or(style.corner).shape()));
    let mut pressed = radii(pressed.unwrap_or_else(|| style.pressed_corner.shape()));
    if let Some(segment) = layout.segment {
        let inner = radii(segment.inner.shape())[0];
        let inner_pressed = radii(segment.pressed.shape())[0];
        let outer = width.min(height) * 0.5;
        rest = [
            segment.leading,
            segment.trailing,
            segment.trailing,
            segment.leading,
        ]
        .map(|outer_edge| if outer_edge { outer } else { inner });
        selected = [outer; 4];
        pressed = [
            segment.leading,
            segment.trailing,
            segment.trailing,
            segment.leading,
        ]
        .map(|outer_edge| if outer_edge { outer } else { inner_pressed });
    }
    CornerShape::Radii(std::array::from_fn(|ix| {
        let resting = rest[ix] + (selected[ix] - rest[ix]) * selection;
        (resting + (pressed[ix] - resting) * press) * scale
    }))
}

pub(crate) fn ripple_target<E>(control: E, state: Rc<RefCell<FeedbackState>>) -> E
where
    E: StatefulInteractiveElement,
{
    feedback_target(control, state, false, None)
}

/// Observe presentation events without consuming them: Base still owns activation,
/// focus, disabled semantics and controlled value changes. Action controls also
/// read hover from this state; selection and display controls style hover themselves.
pub(crate) fn feedback_target<E>(
    control: E,
    state: Rc<RefCell<FeedbackState>>,
    track_hover: bool,
    keyboard_owner: Option<FocusHandle>,
) -> E
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
            state.pointer_cancelled = false;
            state.pressed = true;
            state.keyboard_focus = None;
            if track_hover {
                state.hovered = true;
            }
            start_ripple(&mut state, Some(event.position), cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            let mut state = release.borrow_mut();
            release_ripple(&mut state, cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            let mut state = outside.borrow_mut();
            release_ripple(&mut state, cx);
            state.record(window, cx);
            window.refresh();
        })
        .on_hover(move |hovering, window, cx| {
            let mut state = hover.borrow_mut();
            let was_pressed = state.pressed;
            if track_hover {
                state.hovered = *hovering;
            }
            if !*hovering && (track_hover || was_pressed) {
                release_ripple(&mut state, cx);
            }
            if track_hover || !*hovering && was_pressed {
                state.record(window, cx);
                window.refresh();
            }
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" {
                let mut state = keyboard.borrow_mut();
                state.cancel();
                state.record(window, cx);
                window.refresh();
            } else if matches!(event.keystroke.key.as_str(), "space" | "enter")
                && !event.is_held
                && keyboard_owner
                    .as_ref()
                    .is_none_or(|owner| owner.is_focused(window))
            {
                let mut state = keyboard.borrow_mut();
                state.pressed = true;
                state.keyboard_focus = window.focused(cx);
                start_ripple(&mut state, None, cx);
                state.record(window, cx);
                window.refresh();
            }
        })
        .capture_key_up(move |event, window, cx| {
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                let mut state = state.borrow_mut();
                release_ripple(&mut state, cx);
                state.record(window, cx);
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
    state.borrow_mut().cancel_unavailable(enabled, window);
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
