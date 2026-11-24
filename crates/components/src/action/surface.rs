use std::{cell::RefCell, rc::Rc};

use gpui::{
    BorderStyle, Bounds, Corners, Hsla, IntoElement, Pixels, Styled, Window, canvas, px, quad,
};
use gpui_material_foundation::{
    InteractionFeedback, InteractionState, control_feedback::FeedbackState, elevation_shadow_pair,
};
use gpui_material_theme::Direction;

use super::style::{ActionLayout, ActionPalette, ActionStyle, ActionVisuals, interpolated_corners};

/// Paints drop shadows, background fill, and outline stroke for a rounded control surface.
/// Shared by single-action controls and split surfaces (such as removable chips).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_container_surface(
    bounds: Bounds<Pixels>,
    radii: [Pixels; 4],
    elevation: f32,
    scale: f32,
    shadow: Hsla,
    background: Hsla,
    outline_width: f32,
    outline: Hsla,
    window: &mut Window,
) {
    let [top_left, top_right, bottom_right, bottom_left] = radii;
    let radii = Corners {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    };
    if let Some(shadows) = elevation_shadow_pair(elevation, scale, shadow) {
        window.paint_drop_shadows(bounds, radii, &shadows);
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
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_visual_surface(
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
            paint_container_surface(
                bounds,
                radii,
                visuals.elevation,
                scale,
                palette.shadow,
                palette.background,
                visuals.outline_width,
                palette.outline,
                window,
            );

            let feedback = paint_feedback.borrow();
            if let Some(shape) = feedback.shape() {
                InteractionFeedback::new(interaction, palette.foreground, palette.ring)
                    .paint_with_opacity(
                        shape,
                        feedback.ripple(),
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
