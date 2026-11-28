//! Material 3 disclosure and pagination controls built on `gpui-base`.

mod accordion;
mod collapsible;
mod pagination;

use std::{cell::RefCell, f32::consts::FRAC_PI_2, f32::consts::PI, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, FocusHandle, Hsla, IntoElement, Styled, Window, canvas, radians,
};
use gpui_material_components::{Icon, IconName};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionFeedback, InteractionState, MotionSample, animate_with_status,
    control_feedback::FeedbackState, current_material_theme,
};
use gpui_material_theme::{CornerShape, Direction, MotionKind, MotionSpeed};

pub use accordion::{Accordion, AccordionColors, AccordionItem, AccordionMode, AccordionVariant};
pub use collapsible::{Collapsible, CollapsibleColors};
pub use pagination::{Pagination, PaginationColors};

pub(crate) fn disclosure_motion(
    id: impl Into<ElementId>,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> MotionSample {
    let theme = current_material_theme(window, cx);
    let spec = theme
        .motion
        .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
    animate_with_status(id, if open { 1.0 } else { 0.0 }, spec, window, cx)
}

pub(crate) fn top_only_corners(shape: CornerShape) -> CornerShape {
    match shape {
        CornerShape::Radii([tl, tr, _, _]) => CornerShape::Radii([tl, tr, 0.0, 0.0]),
        CornerShape::Full => CornerShape::Radii([16.0, 16.0, 0.0, 0.0]),
    }
}

pub(crate) fn bottom_only_corners(shape: CornerShape) -> CornerShape {
    match shape {
        CornerShape::Radii([_, _, br, bl]) => CornerShape::Radii([0.0, 0.0, br, bl]),
        CornerShape::Full => CornerShape::Radii([0.0, 0.0, 16.0, 16.0]),
    }
}

pub(crate) fn disclosure_chevron(progress: f32) -> Icon {
    // `ChevronRight` rotated 90 deg points down (collapsed) and rotates 180 deg
    // to point up (expanded), matching Material Symbols `expand_more` -> `expand_less`.
    Icon::new(IconName::ChevronRight)
        .size(gpui::px(20.))
        .rotate(radians(FRAC_PI_2 + PI * progress.clamp(0.0, 1.0)))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn feedback_overlay(
    feedback: Rc<RefCell<FeedbackState>>,
    corners: CornerShape,
    direction: Direction,
    interaction: InteractionState,
    ink: Hsla,
    focus_ring: Hsla,
    placement: FocusRingPlacement,
    reduced_motion: bool,
) -> AnyElement {
    let prepaint_state = feedback.clone();
    canvas(
        move |bounds, window, _| {
            prepaint_state
                .borrow_mut()
                .set_rounded_shape(bounds, corners, direction, window);
        },
        move |_, _, window, cx| {
            let now = cx.background_executor().now();
            let state = feedback.borrow();
            if let Some(shape) = state.shape() {
                InteractionFeedback::new(interaction, ink, focus_ring)
                    .focus_ring_placement(placement)
                    .paint(shape, state.ripple(), now, reduced_motion, window);
            }
        },
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

pub(crate) fn keyed_focus(
    scope: &ElementId,
    key: impl Into<ElementId>,
    provided: Option<FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    match provided {
        Some(handle) => handle,
        None => window.with_id(ElementId::from((scope.clone(), "focus")), |window| {
            window
                .use_keyed_state(key.into(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        }),
    }
}

pub(crate) fn keyed_feedback(
    scope: &ElementId,
    key: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Rc<RefCell<FeedbackState>> {
    window.with_id(ElementId::from((scope.clone(), "feedback")), |window| {
        window
            .use_keyed_state(key.into(), cx, |_, _| {
                Rc::new(RefCell::new(FeedbackState::default()))
            })
            .read(cx)
            .clone()
    })
}
