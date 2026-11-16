use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, BorderStyle, Corners, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, IsZero, ParentElement, Pixels, Point, Size, Styled,
    TextStyleRefinement, Window, canvas, div, prelude::FluentBuilder, px, quad, size,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{animate, elevation_shadows};
use gpui_material_theme::{CornerShape, Direction, Elevation, MotionSpec};

use super::{focus_handle, length};

/// How a horizontal docked toolbar distributes unused width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarArrangement {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Logical insets in a toolbar's content area.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToolbarPadding {
    pub top: Pixels,
    pub start: Pixels,
    pub bottom: Pixels,
    pub end: Pixels,
}

impl ToolbarPadding {
    pub(super) fn apply<E: Styled>(
        self,
        strip: E,
        direction: Direction,
        viewport: Size<Pixels>,
    ) -> E {
        let dimension = |side, extent| px(length(side).unwrap_or(0.)).min(extent / 2.);
        let (left, right) = match direction {
            Direction::Ltr => (self.start, self.end),
            Direction::Rtl => (self.end, self.start),
        };
        strip
            .pt(dimension(self.top, viewport.height))
            .pb(dimension(self.bottom, viewport.height))
            .pl(dimension(left, viewport.width))
            .pr(dimension(right, viewport.width))
    }
}

/// Elevations of an expanded and collapsed toolbar strip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToolbarElevations {
    pub expanded: Option<Elevation>,
    pub collapsed: Option<Elevation>,
}

/// Uses nonnegative flexible gaps so constrained children still flow forward.
/// Each item is at index `2 * item + 1` before any direction reversal.
pub(super) fn arrange_horizontal(
    children: Vec<AnyElement>,
    arrangement: ToolbarArrangement,
    spacing: Pixels,
) -> Vec<AnyElement> {
    if children.is_empty() {
        return children;
    }
    let (before, between, after) = match arrangement {
        ToolbarArrangement::Start => (0., 0., 1.),
        ToolbarArrangement::Center => (1., 0., 1.),
        ToolbarArrangement::End => (1., 0., 0.),
        ToolbarArrangement::SpaceBetween => (0., 1., 0.),
        ToolbarArrangement::SpaceAround => (0.5, 1., 0.5),
        ToolbarArrangement::SpaceEvenly => (1., 1., 1.),
    };
    let spacer = |weight, minimum| {
        div()
            .min_w(minimum)
            .flex_grow(weight)
            .flex_shrink(0.)
            .flex_basis(minimum)
            .into_any_element()
    };
    let mut result = Vec::with_capacity(children.len() * 2 + 1);
    result.push(spacer(before, px(0.)));
    let mut children = children.into_iter().peekable();
    while let Some(child) = children.next() {
        result.push(child);
        if children.peek().is_some() {
            result.push(spacer(between, spacing));
        }
    }
    result.push(spacer(after, px(0.)));
    result
}

pub(super) struct ToolbarSlot {
    id: ElementId,
    focus: FocusHandle,
    element: AnyElement,
}

impl ToolbarSlot {
    pub(super) fn new(
        toolbar: &ElementId,
        part: &'static str,
        element: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let id = ElementId::from((toolbar.clone(), part));
        let focus = focus_handle(&id, "scope", window, cx);
        Self { id, focus, element }
    }

    pub(super) fn content(self) -> (AnyElement, FocusHandle) {
        let host = div()
            .id(self.id)
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .min_w_0()
            .min_h_0()
            .max_w_full()
            .max_h_full()
            .overflow_hidden()
            .flex_none()
            .child(self.element);
        (host.into_any_element(), self.focus)
    }

    pub(super) fn expandable(
        self,
        expanded: bool,
        horizontal: bool,
        motion: MotionSpec,
        ink: Hsla,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<(AnyElement, FocusHandle)> {
        let shown = expanded || self.focus.contains_focused(window, cx);
        let key = self.id.clone();
        let fraction = animate(
            (key.clone(), "slot-expansion"),
            f32::from(shown),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        if fraction == 0. {
            return None;
        }
        let bounds = window.use_keyed_state((key, "slot-intrinsic"), cx, |_, _| {
            Size::<Pixels>::default()
        });
        let focus = self.focus.clone();
        let child = shown.then(|| {
            let (mut child, _) = self.content();
            let measured = window.with_text_style(
                Some(TextStyleRefinement {
                    color: Some(ink),
                    ..Default::default()
                }),
                |window| {
                    child.layout_as_root(
                        size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                        window,
                        cx,
                    )
                },
            );
            bounds.update(cx, |size, _| *size = measured);
            child
        });
        let preferred = *bounds.read(cx);
        let footprint = if horizontal {
            size(preferred.width * fraction, preferred.height)
        } else {
            size(preferred.width, preferred.height * fraction)
        };
        let host = div()
            .w(footprint.width)
            .h(footprint.height)
            .min_w_0()
            .min_h_0()
            .max_w_full()
            .max_h_full()
            .flex_none()
            .overflow_hidden()
            .children(child.map(|mut child| {
                canvas(
                    move |bounds, window, cx| {
                        child.prepaint_as_root(
                            bounds.origin,
                            bounds.size.map(AvailableSpace::Definite),
                            window,
                            cx,
                        );
                        child
                    },
                    |_, mut child, window, cx| child.paint(window, cx),
                )
                .size_full()
            }));
        Some((host.into_any_element(), focus))
    }
}

pub(super) struct StripGeometry {
    pub horizontal: bool,
    pub docked: bool,
    pub direction: Direction,
    pub footprint: Rc<Cell<Size<Pixels>>>,
    pub shape: CornerShape,
    pub surface: Hsla,
    pub shadow: Hsla,
    pub elevation: f32,
    pub slide: Point<Pixels>,
    pub hidden: bool,
}

impl StripGeometry {
    pub(super) fn frame(self, strip: impl IntoElement) -> AnyElement {
        let frame = || {
            div()
                .relative()
                .flex()
                .items_center()
                .min_w_0()
                .min_h_0()
                .max_w_full()
                .max_h_full()
                .flex_shrink(1.)
                .when(self.docked && self.horizontal, Styled::w_full)
                .when(self.docked && !self.horizontal, Styled::h_full)
        };
        let mut host = frame();
        if self.hidden {
            let previous = self.footprint.get();
            host = host
                .when(!(self.docked && self.horizontal), |host| {
                    host.w(previous.width)
                })
                .when(!(self.docked && !self.horizontal), |host| {
                    host.h(previous.height)
                });
        } else {
            let shape = self.shape;
            let direction = self.direction;
            let surface = self.surface;
            let shadow = self.shadow;
            let elevation = self.elevation;
            let paint = canvas(
                |_, _, _| (),
                move |bounds, (), window, _| {
                    let [top_left, top_right, bottom_right, bottom_left] = shape
                        .radii(
                            bounds.size.width.into(),
                            bounds.size.height.into(),
                            direction,
                        )
                        .map(px);
                    let corners = Corners {
                        top_left,
                        top_right,
                        bottom_right,
                        bottom_left,
                    };
                    if elevation > 0. {
                        window.paint_drop_shadows(
                            bounds,
                            corners,
                            &elevation_shadows(elevation, 1., shadow),
                        );
                    }
                    if surface.a > 0. {
                        window.paint_quad(quad(
                            bounds,
                            corners,
                            surface,
                            px(0.),
                            gpui::transparent_black(),
                            BorderStyle::Solid,
                        ));
                    }
                },
            )
            .absolute()
            .inset_0();
            host = host
                .when(!self.slide.is_zero(), Styled::overflow_hidden)
                .child(
                    frame()
                        .left(self.slide.x)
                        .top(self.slide.y)
                        .child(paint)
                        .child(strip),
                );
        }
        host.child(
            canvas(
                move |bounds, window, _| {
                    if self.footprint.replace(bounds.size) != bounds.size {
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .into_any_element()
    }
}
