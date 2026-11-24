mod control;
mod style;
mod surface;

use std::{cell::RefCell, rc::Rc};

use gpui::{AnyElement, Hsla, IntoElement, ParentElement, Styled, canvas, div, px};
use gpui_material_theme::{CornerShape, Direction};

#[cfg(any(feature = "selection", feature = "surfaces"))]
pub(crate) use gpui_material_foundation::control_feedback::ripple_state;
pub(crate) use gpui_material_foundation::control_feedback::{
    FeedbackState, activate_feedback, feedback_target,
};

pub(crate) use control::{Action, ActionControl, Activate, Change, InvokeAction};
pub(crate) use style::{
    ActionLayout, ActionStyle, DisabledStyle, SegmentShape, interpolated_corners,
};
#[cfg(feature = "selection")]
pub(crate) use surface::paint_container_surface;

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
            if let (Some(shape), Some(ripple)) = (state.shape(), state.ripple()) {
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
