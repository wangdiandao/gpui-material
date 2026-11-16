use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, Corners, ElementId, Entity,
    FocusHandle, Hsla, IntoElement, ParentElement, Pixels, StatefulInteractiveElement, Styled,
    Window, canvas, div, prelude::FluentBuilder, px, quad,
};
use gpui_base::{Button, Toggle};
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, InteractionTracker, Shape, animate,
    color, elevation_shadows, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

use crate::button_style::{ButtonShapes, ButtonStyle, finite_length};

pub(crate) use gpui_material_foundation::control_feedback::{
    FeedbackState, activate_feedback, feedback_target, ripple_state,
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
    pub(crate) dragged: bool,
    pub(crate) dragged_elevation: Option<Elevation>,
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
        state.clear();
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
        let tracker =
            interaction_source.map(|source| InteractionSource::track(source, &self.id, window, cx));
        self.render_with_tracker(scale, layout, tracker, window, cx)
    }

    pub(crate) fn interaction_snapshot(
        &self,
        dragged: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> InteractionState {
        let (feedback, focus) = self.feedback(window, cx);
        let mut state = feedback.borrow_mut();
        let enabled = !self.disabled;
        state.cancel_unavailable(enabled, window);
        state.set_dragged(enabled && dragged);
        state.interaction(enabled, &focus, window)
    }

    pub(crate) fn render_with_tracker(
        self,
        scale: f32,
        layout: ActionLayout,
        tracker: Option<InteractionTracker>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (feedback, focus) = self.feedback(window, cx);
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
            state.set_dragged(enabled && layout.dragged);
            let interaction = if layout.decorative {
                let mut interaction = InteractionState::default();
                interaction.disabled = self.disabled;
                interaction
            } else {
                state.interaction(enabled, &focus, window)
            };
            state.track_snapshot(interaction, &focus, tracker, cx);
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
            (if interaction.dragged {
                layout.dragged_elevation
            } else {
                layout.appearance.elevations.for_state(interaction)
            })
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
                        let semantic_feedback = feedback.clone();
                        let control = control.on_a11y_action(
                            AccessibleAction::Click,
                            move |_, window, cx| {
                                if !semantic_feedback.borrow().is_dragged() {
                                    invoke(window, cx);
                                }
                            },
                        );
                        control.when(!layout.semantic_only, |control| {
                            control.on_click(move |event, window, cx| {
                                cx.stop_propagation();
                                if !feedback.borrow().blocks_activation(event) {
                                    activate_feedback(event, &feedback, window, cx);
                                    on_activate(window, cx);
                                }
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
                        let semantic_feedback = feedback.clone();
                        control
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                if !semantic_feedback.borrow().is_dragged() {
                                    toggle(!selected, window, cx);
                                }
                            })
                            .on_change(move |selected, event, window, cx| {
                                cx.stop_propagation();
                                if !feedback.borrow().blocks_activation(event) {
                                    activate_feedback(event, &feedback, window, cx);
                                    on_change(selected, window, cx);
                                }
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

pub(crate) fn interpolated_corners(
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
