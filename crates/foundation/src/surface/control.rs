use gpui::{
    AccessibleAction, AnyElement, App, Hsla, IntoElement, ParentElement, Role,
    StatefulInteractiveElement, Styled, Toggled, Window, canvas, prelude::FluentBuilder,
};
use gpui_base::{Button, StyledExt};

use super::{Control, PolygonPaint, ResolvedShape, Surface, SurfaceAction};
use crate::{
    InteractionFeedback, InteractionSource,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
};

impl Surface {
    pub(super) fn render_control(
        self,
        control: Control,
        resolved_shape: ResolvedShape,
        polygon_paint: Option<PolygonPaint>,
        default_focus: Hsla,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let focus = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state((control.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let state = ripple_state(&control.id, !self.disabled, window, cx);
        let shape_hit_test = self.shape_hit_test.unwrap_or(polygon_paint.is_some());
        state.borrow_mut().set_shape_hit_test(shape_hit_test);
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &control.id, window, cx));
        let interaction = state
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let appearance = self.feedback;
        let ink = appearance
            .ink
            .or(self.style.text.color)
            .unwrap_or_else(|| window.text_style().color);
        let ring = if appearance.focus_ring {
            appearance.focus_color.unwrap_or(default_focus)
        } else {
            gpui::transparent_black()
        };
        let clip_backdrop = self.clip_backdrop;
        let border = polygon_paint.as_ref().and_then(|paint| paint.border);
        let bg_geometry = state.clone();
        let bg_paint = state.clone();
        let background_layer = polygon_paint.map(|paint| {
            let spec = resolved_shape.clone();
            canvas(
                move |bounds, window, _| {
                    let mut state = bg_geometry.borrow_mut();
                    match &spec {
                        ResolvedShape::Corners(corners, direction) => {
                            state.set_rounded_shape(bounds, *corners, *direction, window);
                        }
                        ResolvedShape::Polygon(polygon, fit) => {
                            state.set_polygon_shape(bounds, polygon, *fit, window);
                        }
                    }
                },
                move |_, _, window, _| {
                    if let Some(shape) = bg_paint.borrow().shape() {
                        shape.paint_shadow(
                            paint.shadow_elevation,
                            paint.shadow_scale,
                            paint.shadow_color,
                            window,
                        );
                        shape.paint(paint.background, window);
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        let has_bg = background_layer.is_some();
        let geometry = state.clone();
        let paint = state.clone();
        let visual = canvas(
            move |bounds, window, _| {
                if !has_bg {
                    let mut state = geometry.borrow_mut();
                    match &resolved_shape {
                        ResolvedShape::Corners(corners, direction) => {
                            state.set_rounded_shape(bounds, *corners, *direction, window);
                        }
                        ResolvedShape::Polygon(polygon, fit) => {
                            state.set_polygon_shape(bounds, polygon, *fit, window);
                        }
                    }
                }
            },
            move |bounds, _, window, cx| {
                let state = paint.borrow();
                if let Some(shape) = state.shape() {
                    if let Some(backdrop) = clip_backdrop {
                        shape.paint_backdrop_mask(bounds, backdrop, window);
                    }
                    if let Some((width, color)) = border {
                        shape.paint_stroke(width, color, window);
                    }
                    let reduced = cx.reduce_motion();
                    let ripple = state.ripple().filter(|_| appearance.ripple);
                    let layer = interaction
                        .layer_with_ripple(reduced || ripple.is_none())
                        .filter(|_| appearance.state_layer)
                        .map_or(0., |layer| layer.opacity());
                    InteractionFeedback::new(interaction, ink, ring).paint_with_opacity(
                        shape,
                        ripple,
                        layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
                }
            },
        )
        .absolute()
        .inset_0();
        let button = Button::new(control.id)
            .block()
            .items_stretch()
            .justify_start()
            .when(clip_backdrop.is_some(), Styled::overflow_hidden)
            .line_height(window.text_style().line_height)
            .refine_style(&self.style)
            .accessibility_label(control.label)
            .track_focus(&focus)
            .disabled(self.disabled);
        let button = match control.action {
            SurfaceAction::Activate => button,
            SurfaceAction::Select(value) => button.role(Role::ListBoxOption).aria_selected(value),
            SurfaceAction::Check(value) => button.role(Role::CheckBox).aria_toggled(if value {
                Toggled::True
            } else {
                Toggled::False
            }),
        };
        let button = if let Some(activate) = control.activate.filter(|_| !self.disabled) {
            let accessible = activate.clone();
            let feedback = state.clone();
            button
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    accessible(window, cx);
                })
                .on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    if !feedback.borrow().blocks_activation(event) {
                        activate_feedback(event, &feedback, window, cx);
                        activate(window, cx);
                    }
                })
        } else {
            button
        };
        button
            .when(!self.disabled, |button| {
                feedback_target(button, state, true, Some(focus))
            })
            .children(background_layer)
            .children(self.children)
            .child(visual)
            .into_any_element()
    }
}
