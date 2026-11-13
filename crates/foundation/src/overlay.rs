use gpui::{
    Anchor, Div, ElementId, InteractiveElement, ScrollHandle, Stateful, StatefulInteractiveElement,
    Styled, div,
};
use gpui_material_theme::Direction;

/// Mirrors the horizontal part of an overlay anchor for logical direction.
/// Base retains placement, fit, focus scope and dismissal.
pub fn logical_anchor(anchor: Anchor, direction: Direction) -> Anchor {
    if direction == Direction::Ltr {
        return anchor;
    }
    match anchor {
        Anchor::TopLeft => Anchor::TopRight,
        Anchor::TopRight => Anchor::TopLeft,
        Anchor::BottomLeft => Anchor::BottomRight,
        Anchor::BottomRight => Anchor::BottomLeft,
        Anchor::LeftCenter => Anchor::RightCenter,
        Anchor::RightCenter => Anchor::LeftCenter,
        Anchor::TopCenter | Anchor::BottomCenter => anchor,
    }
}

/// Creates an inner vertical scroll viewport that keeps wheel input from
/// reaching scroll owners behind it. The caller gives it a bounded height.
pub fn isolated_scroll(id: impl Into<ElementId>, handle: &ScrollHandle) -> Stateful<Div> {
    div()
        .id(id)
        .min_h_0()
        .overflow_y_scroll()
        .occlude()
        .track_scroll(handle)
}
