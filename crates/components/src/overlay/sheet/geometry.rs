use std::{cell::RefCell, rc::Rc};

use gpui::{App, Edges, ElementId, Pixels, Size, Window, px};
use gpui_material_foundation::{animate, resolve_edges};
use gpui_material_theme::{
    Direction, LogicalEdges, MaterialTheme, MotionKind, MotionSpec, MotionSpeed,
};

use super::super::bottom_sheet::ExpandableGesture;
use super::{SheetEdge, SheetPlacement};

pub(super) struct SheetGeometry {
    pub(super) left: bool,
    pub(super) detached: bool,
    pub(super) rtl: bool,
    pub(super) margins: Edges<Pixels>,
    pub(super) extent: Pixels,
    pub(super) collapsed: Pixels,
    pub(super) travel: Pixels,
    pub(super) height: Pixels,
    pub(super) offscreen: Pixels,
}

fn fit_margin_pair(a: Pixels, b: Pixels, available: Pixels) -> (Pixels, Pixels) {
    let sanitize = |margin: Pixels| {
        if f32::from(margin).is_finite() {
            margin.clamp(px(0.), available)
        } else {
            px(0.)
        }
    };
    let (a, b) = (sanitize(a), sanitize(b));
    if a + b > available && a + b > px(0.) {
        let factor = available / (a + b);
        (a * factor, b * factor)
    } else {
        (a, b)
    }
}

pub(super) fn fit_logical_margins(
    margins: LogicalEdges,
    viewport: Size<Pixels>,
    direction: Direction,
) -> Edges<Pixels> {
    let resolved = resolve_edges(margins, direction);
    let (left, right) = fit_margin_pair(
        resolved.left,
        resolved.right,
        (viewport.width - px(48.)).max(px(0.)),
    );
    let (top, bottom) = fit_margin_pair(
        resolved.top,
        resolved.bottom,
        (viewport.height - px(48.)).max(px(0.)),
    );
    Edges {
        top,
        bottom,
        left,
        right,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_geometry(
    id: &ElementId,
    theme: &MaterialTheme,
    edge: SheetEdge,
    placement: SheetPlacement,
    margins: LogicalEdges,
    modal: bool,
    bottom: bool,
    viewport: Size<Pixels>,
    available: Size<Pixels>,
    requested_extent: Pixels,
    side_max_width: Pixels,
    expanded: bool,
    has_expansion: bool,
    retained_layout: Option<(Pixels, Pixels)>,
    visibility: f32,
    gesture: &Rc<RefCell<ExpandableGesture>>,
    window: &mut Window,
    cx: &mut App,
) -> SheetGeometry {
    let left = match edge {
        SheetEdge::Start => theme.layout.direction == Direction::Ltr,
        SheetEdge::End => theme.layout.direction == Direction::Rtl,
        SheetEdge::Bottom => false,
    };
    let detached = !bottom && placement == SheetPlacement::Detached;
    let rtl = theme.layout.direction == Direction::Rtl;
    let margins = if detached {
        fit_logical_margins(margins, viewport, theme.layout.direction)
    } else {
        Edges::default()
    };
    let available_width = (viewport.width - margins.left - margins.right).max(px(0.));
    let target = if bottom {
        requested_extent.min(available.height)
    } else {
        requested_extent.min(side_max_width).min(available_width)
    };
    let extent = if !bottom && !modal {
        px(animate(
            (id.clone(), "side-sheet-width"),
            f32::from(target),
            theme.motion.spec(
                MotionKind::Spatial,
                MotionSpeed::Default,
                cx.reduce_motion(),
            ),
            window,
            cx,
        ))
        .clamp(px(0.), available_width)
    } else {
        target
    };
    // The 48dp handle and 48dp header actions must remain operable at rest.
    let collapsed = px(48. + 8. + 48. + 16.).min(extent);
    let travel = extent - collapsed;
    let height = if let Some((height, _)) = retained_layout {
        height
    } else if has_expansion && bottom {
        let drag = gesture.borrow();
        let tracked = drag.capture.is_some();
        let target = if expanded { extent } else { collapsed } - drag.offset;
        let policy = if tracked {
            MotionSpec::Instant
        } else {
            theme.motion.spec(
                MotionKind::Spatial,
                MotionSpeed::Default,
                cx.reduce_motion(),
            )
        };
        px(animate(
            (id.clone(), "sheet-height"),
            f32::from(target),
            policy,
            window,
            cx,
        ))
        .clamp(collapsed, extent)
    } else {
        extent
    };
    let offscreen = if retained_layout.is_some() {
        px(0.)
    } else {
        -(if bottom { height } else { extent }) * (1. - visibility)
    };
    SheetGeometry {
        left,
        detached,
        rtl,
        margins,
        extent,
        collapsed,
        travel,
        height,
        offscreen,
    }
}

#[cfg(test)]
mod tests {
    use gpui::size;

    use super::*;

    #[test]
    fn detached_margins_resolve_direction_and_clamp_to_viewport() {
        let margins = LogicalEdges::new(12., 24., 16., 8.);
        let viewport = size(px(400.), px(300.));
        let ltr = fit_logical_margins(margins, viewport, Direction::Ltr);
        let rtl = fit_logical_margins(margins, viewport, Direction::Rtl);
        assert_eq!((ltr.left, ltr.right), (px(8.), px(24.)));
        assert_eq!((rtl.left, rtl.right), (px(24.), px(8.)));
        assert_eq!((ltr.top, ltr.bottom), (px(12.), px(16.)));

        let tight = fit_logical_margins(
            LogicalEdges::all(200.),
            size(px(148.), px(148.)),
            Direction::Ltr,
        );
        assert_eq!(tight.left + tight.right, px(100.));
        assert_eq!(tight.top + tight.bottom, px(100.));
    }
}
