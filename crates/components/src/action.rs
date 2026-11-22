use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, Corners, ElementId, Entity,
    FocusHandle, Hsla, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px, quad,
};
use gpui_base::{Button, Toggle};
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, InteractionTracker, ThemeScope,
    animate, color, current_material_theme, elevation_shadow_pair,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

use crate::button_style::{ButtonShapes, ButtonStyle, finite_length};

#[cfg(any(feature = "selection", feature = "surfaces"))]
pub(crate) use gpui_material_foundation::control_feedback::ripple_state;
pub(crate) use gpui_material_foundation::control_feedback::{
    FeedbackState, activate_feedback, feedback_target,
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
    pub(crate) outline_role: ColorRole,
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
        let colors = &current_material_theme(window, cx).colors;
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
    pub(crate) loading: bool,
    pub(crate) appearance: ButtonStyle,
    pub(crate) presentation_selected: Option<bool>,
    pub(crate) dragged: bool,
    pub(crate) dragged_elevation: Option<Elevation>,
}

#[derive(Clone, Copy)]
pub(crate) struct SegmentShape {
    pub(crate) leading: bool,
    pub(crate) trailing: bool,
    pub(crate) outer: CornerRole,
    pub(crate) inner: CornerRole,
    pub(crate) pressed: CornerRole,
}

pub(crate) struct ActionControl {
    pub(crate) id: ElementId,
    pub(crate) label: SharedString,
    pub(crate) focus: Option<FocusHandle>,
    pub(crate) disabled: bool,
    pub(crate) action: Action,
    pub(crate) style: ActionStyle,
    pub(crate) content: AnyElement,
}

#[derive(Clone, Copy)]
struct ActionPalette {
    foreground: Hsla,
    background: Hsla,
    outline: Hsla,
    ring: Hsla,
    shadow: Hsla,
}

#[derive(Clone, Copy)]
struct ActionVisuals {
    progress: f32,
    selection: f32,
    layer: f32,
    outline_width: f32,
    elevation: f32,
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

    #[cfg(feature = "selection")]
    pub(crate) fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.feedback(window, cx).1
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
        let theme = current_material_theme(window, cx);
        let density = theme.layout.density;
        let direction = theme.layout.direction;
        let reduced = cx.reduce_motion();
        let (corner_kind, corner_speed) = style.corner_motion;
        let motion = theme.motion.spec(corner_kind, corner_speed, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let selected = layout.presentation_selected.unwrap_or(matches!(
            &self.action,
            Action::Toggle { selected: true, .. }
        ));
        let palette = resolve_palette(style, layout, self.disabled, selected, window, cx);
        let visual_height = px(density.visual_size(style.height) * scale);
        let visual_width = layout
            .width
            .unwrap_or_else(|| px(density.visual_size(style.min_width) * scale));
        let target_height = px(density.interactive_size(style.height) * scale);
        let target_width = layout
            .width
            .unwrap_or_else(|| px(density.interactive_size(style.min_width.max(1.)) * scale));

        let interactive = !self.disabled && !layout.loading;
        let interaction = record_interaction(
            &feedback,
            &focus,
            tracker,
            self.disabled,
            layout.loading,
            layout.dragged,
            layout.decorative,
            window,
            cx,
        );
        let visuals = animate_visuals(
            &self.id,
            style,
            layout,
            interaction,
            selected,
            interactive,
            scale,
            motion,
            effects,
            reduced,
            window,
            cx,
        );
        let visual = render_visual_surface(
            style,
            layout,
            direction,
            scale,
            palette,
            visuals,
            interaction,
            reduced,
            &feedback,
        );

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
                .text_color(palette.foreground)
                .child(content)
                .into_any_element();
        }
        let width = (target_width, layout.width.is_some());
        match self.action {
            Action::Invoke(on_activate) => render_invoke_control(
                self.id,
                self.label,
                self.disabled,
                focus,
                layout,
                on_activate,
                content,
                feedback,
                palette.foreground,
                target_height,
                width,
                window,
                cx,
            ),
            Action::Toggle {
                selected,
                on_change,
            } => render_toggle_control(
                self.id,
                self.label,
                self.disabled,
                focus,
                layout,
                selected,
                on_change,
                content,
                feedback,
                palette.foreground,
                target_height,
                width,
                window,
                cx,
            ),
        }
    }
}

fn resolve_palette(
    style: ActionStyle,
    layout: ActionLayout,
    disabled: bool,
    selected: bool,
    window: &Window,
    cx: &App,
) -> ActionPalette {
    let theme = current_material_theme(window, cx);
    let ring = color(theme.colors.color(ColorRole::Secondary));
    let shadow = color(theme.colors.color(ColorRole::Shadow));
    let foreground = layout
        .appearance
        .colors
        .content(disabled, selected)
        .unwrap_or_else(|| style.foreground(disabled, window, cx));
    let background = layout
        .appearance
        .colors
        .container(disabled, selected)
        .unwrap_or_else(|| {
            if disabled {
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
            if disabled {
                color(theme.colors.color(style.disabled.outline.0))
                    .opacity(style.disabled.outline.1)
            } else {
                color(theme.colors.color(style.outline_role))
            }
        },
        |border| border.color,
    );
    ActionPalette {
        foreground,
        background,
        outline,
        ring,
        shadow,
    }
}

#[allow(clippy::too_many_arguments)]
fn record_interaction(
    feedback: &RefCell<FeedbackState>,
    focus: &FocusHandle,
    tracker: Option<InteractionTracker>,
    disabled: bool,
    loading: bool,
    dragged: bool,
    decorative: bool,
    window: &Window,
    cx: &mut App,
) -> InteractionState {
    let interactive = !disabled && !loading;
    let mut state = feedback.borrow_mut();
    state.cancel_unavailable(interactive, window);
    state.set_dragged(interactive && dragged);
    let interaction = if decorative || loading {
        let mut interaction = InteractionState::default();
        interaction.disabled = disabled;
        interaction
    } else {
        state.interaction(interactive, focus, window)
    };
    state.track_snapshot(interaction, focus, tracker, cx);
    interaction
}

#[allow(clippy::too_many_arguments)]
fn animate_visuals(
    id: &ElementId,
    style: ActionStyle,
    layout: ActionLayout,
    interaction: InteractionState,
    selected: bool,
    enabled: bool,
    scale: f32,
    motion: MotionSpec,
    effects: MotionSpec,
    reduced: bool,
    window: &mut Window,
    cx: &mut App,
) -> ActionVisuals {
    let hovered = interaction.hovered;
    let pressed = interaction.pressed;
    let progress = animate(
        (id.clone(), "press-shape"),
        f32::from(pressed),
        motion,
        window,
        cx,
    )
    .clamp(0., 1.);
    let selection = if style.selected_corner.is_some() {
        animate(
            (id.clone(), "selected-shape"),
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
        (id.clone(), "state-layer"),
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
        (id.clone(), "outline-width"),
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
        (id.clone(), "elevation"),
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
    ActionVisuals {
        progress,
        selection,
        layer,
        outline_width,
        elevation,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_visual_surface(
    style: ActionStyle,
    layout: ActionLayout,
    direction: Direction,
    scale: f32,
    palette: ActionPalette,
    visuals: ActionVisuals,
    interaction: InteractionState,
    reduced: bool,
    feedback: &Rc<RefCell<FeedbackState>>,
) -> impl IntoElement {
    let mesh = feedback.clone();
    let paint_feedback = feedback.clone();
    canvas(
        move |bounds, window, _| {
            let corners = interpolated_corners(
                bounds,
                style,
                visuals.selection,
                visuals.progress,
                scale,
                layout,
            );
            mesh.borrow_mut()
                .set_rounded_shape(bounds, corners, direction, window);
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
            if let Some(shadows) = elevation_shadow_pair(visuals.elevation, scale, palette.shadow) {
                window.paint_drop_shadows(bounds, radii, &shadows);
            }
            // A zero-width outline is still visible through antialiasing on some
            // backends if its pigment is opaque.
            if palette.background.a > 0. || visuals.outline_width > 0. {
                window.paint_quad(quad(
                    bounds,
                    radii,
                    palette.background,
                    px(visuals.outline_width),
                    if visuals.outline_width > 0. {
                        palette.outline
                    } else {
                        gpui::transparent_black()
                    },
                    BorderStyle::Solid,
                ));
            }

            let feedback = paint_feedback.borrow();
            if let Some(shape) = &feedback.shape {
                InteractionFeedback::new(interaction, palette.foreground, palette.ring)
                    .paint_with_opacity(
                        shape,
                        feedback.ripple.as_ref(),
                        visuals.layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

#[allow(clippy::too_many_arguments)]
fn render_invoke_control(
    id: ElementId,
    label: SharedString,
    disabled: bool,
    focus: FocusHandle,
    layout: ActionLayout,
    on_activate: Option<Activate>,
    content: impl IntoElement,
    feedback: Rc<RefCell<FeedbackState>>,
    foreground: Hsla,
    target_height: Pixels,
    width: (Pixels, bool),
    window: &Window,
    cx: &App,
) -> AnyElement {
    let interactive = !disabled && !layout.loading;
    let control = Button::new(id)
        .track_focus(&focus)
        .tab_stop(layout.tab_stop)
        .disabled(disabled)
        .accessibility_label(label)
        .when(layout.loading, |control| {
            control.a11y_synthetic_children(|tree| {
                tree.parent_node().set_busy();
            })
        });
    let control = match on_activate {
        Some(on_activate) if interactive => {
            let on_activate = Rc::new(ThemeScope::bind_action(
                move |window, cx| on_activate(window, cx),
                window,
                cx,
            ));
            let feedback = feedback.clone();
            let invoke = on_activate.clone();
            let semantic_feedback = feedback.clone();
            let control = control.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                if !semantic_feedback.borrow().is_dragged() {
                    invoke(window, cx);
                }
            });
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
        width,
        interactive,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_toggle_control(
    id: ElementId,
    label: SharedString,
    disabled: bool,
    focus: FocusHandle,
    layout: ActionLayout,
    selected: bool,
    on_change: Option<Change>,
    content: impl IntoElement,
    feedback: Rc<RefCell<FeedbackState>>,
    foreground: Hsla,
    target_height: Pixels,
    width: (Pixels, bool),
    window: &Window,
    cx: &App,
) -> AnyElement {
    let interactive = !disabled && !layout.loading;
    let control = Toggle::new(id)
        .track_focus(&focus)
        .tab_stop(layout.tab_stop)
        .pressed(selected)
        .disabled(disabled)
        .accessibility_label(label)
        .when(layout.loading, |control| {
            control.a11y_synthetic_children(|tree| {
                tree.parent_node().set_busy();
            })
        });
    let control = match on_change {
        Some(on_change) if interactive => {
            let pending = Rc::new(Cell::new(!selected));
            let pending_arg = pending.clone();
            let on_change = Rc::new(ThemeScope::bind_action(
                move |window, cx| on_change(pending_arg.get(), window, cx),
                window,
                cx,
            ));
            let feedback = feedback.clone();
            let toggle = on_change.clone();
            let semantic_feedback = feedback.clone();
            let semantic_pending = pending.clone();
            control
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    if !semantic_feedback.borrow().is_dragged() {
                        semantic_pending.set(!selected);
                        toggle(window, cx);
                    }
                })
                .on_change(move |selected, event, window, cx| {
                    cx.stop_propagation();
                    if !feedback.borrow().blocks_activation(event) {
                        activate_feedback(event, &feedback, window, cx);
                        pending.set(selected);
                        on_change(window, cx);
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
        width,
        interactive,
    )
}

fn decorate(
    control: impl Styled + StatefulInteractiveElement + ParentElement + IntoElement,
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
    let radii = |shape: CornerShape| shape.radii(width, height, Direction::Ltr);
    let ButtonShapes {
        resting,
        pressed: pressed_shape,
        checked,
    } = layout.appearance.shapes;
    let mut rest = radii(resting.unwrap_or_else(|| style.corner.shape()));
    let mut selected =
        radii(checked.unwrap_or_else(|| style.selected_corner.unwrap_or(style.corner).shape()));
    let mut pressed = radii(pressed_shape.unwrap_or_else(|| style.pressed_corner.shape()));
    if let Some(segment) = layout.segment {
        let inner = radii(segment.inner.shape())[0];
        let inner_pressed = radii(segment.pressed.shape())[0];
        let outer = radii(segment.outer.shape())[0];
        let outside = [
            segment.leading,
            segment.trailing,
            segment.trailing,
            segment.leading,
        ];
        if segment.inner == CornerRole::None {
            rest = std::array::from_fn(|ix| if outside[ix] { rest[ix] } else { inner });
            selected = std::array::from_fn(|ix| if outside[ix] { selected[ix] } else { inner });
            pressed = std::array::from_fn(|ix| {
                if outside[ix] {
                    pressed[ix]
                } else {
                    inner_pressed
                }
            });
        } else {
            if resting.is_none() {
                rest = outside.map(|outer_edge| if outer_edge { outer } else { inner });
            }
            if checked.is_none() {
                selected = radii(CornerRole::Full.shape());
            }
            if pressed_shape.is_none() {
                pressed = outside.map(|outer_edge| if outer_edge { outer } else { inner_pressed });
            }
        }
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
    direction: Direction,
) -> impl IntoElement {
    let geometry = state.clone();
    canvas(
        move |bounds, window, _| {
            geometry
                .borrow_mut()
                .set_rounded_shape(bounds, corners, direction, window);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_segment_zero_inner_clips_interior_seam_in_all_states() {
        let style = ActionStyle {
            background: None,
            background_color: None,
            foreground: ColorRole::OnSurface,
            disabled: DisabledStyle {
                foreground: ColorRole::OnSurface,
                background: None,
                outline: (ColorRole::OnSurface, 0.12),
            },
            height: 32.,
            min_width: 32.,
            corner: CornerRole::Small,
            pressed_corner: CornerRole::Small,
            selected_corner: Some(CornerRole::Medium),
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: 0.,
            outline_role: ColorRole::Outline,
            elevation: Elevation::Level0,
            hover_elevation: Elevation::Level0,
        };
        let bounds = Bounds::new(gpui::point(px(0.), px(0.)), gpui::size(px(80.), px(32.)));
        let custom_shapes = ButtonShapes {
            resting: Some(CornerShape::Radii([10., 12., 14., 16.])),
            pressed: Some(CornerShape::Radii([6., 7., 8., 9.])),
            checked: Some(CornerRole::Full.shape()),
        };

        for (leading, trailing) in [(true, false), (false, true)] {
            let layout = ActionLayout {
                segment: Some(SegmentShape {
                    leading,
                    trailing,
                    outer: style.corner,
                    inner: CornerRole::None,
                    pressed: CornerRole::None,
                }),
                appearance: ButtonStyle {
                    shapes: custom_shapes,
                    ..Default::default()
                },
                ..ActionLayout::default()
            };
            for (selection, press) in [(0., 0.), (1., 0.), (0., 1.), (1., 1.)] {
                let CornerShape::Radii(radii) =
                    interpolated_corners(bounds, style, selection, press, 1., layout)
                else {
                    panic!("interpolated_corners must return explicit radii");
                };
                if leading {
                    assert!(radii[0] > 0. && radii[3] > 0.);
                    assert_eq!(radii[1], 0.);
                    assert_eq!(radii[2], 0.);
                } else {
                    assert_eq!(radii[0], 0.);
                    assert_eq!(radii[3], 0.);
                    assert!(radii[1] > 0. && radii[2] > 0.);
                }
            }
        }
    }
}
