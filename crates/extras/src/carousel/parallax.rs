use gpui::{
    AnyElement, App, Axis, BorderStyle, Bounds, ContentMask, Corners, Element, ElementId,
    GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId, PathBuilder, Pixels, Point,
    Window, point, px, quad, size,
};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape};

/// A stroke drawn inside the parallax mask in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselBorder {
    pub width: Pixels,
    pub color: Hsla,
}

/// Clips and translates arbitrary scrollable content at the viewport edges.
///
/// The element keeps its original layout size and clips to the current GPUI
/// content mask or an explicit viewport. Rounded corners use an opaque
/// backdrop because GPUI's content masks are rectangular.
pub struct CarouselParallax {
    child: AnyElement,
    axis: Axis,
    viewport: Option<Bounds<Pixels>>,
    shape: CornerShape,
    border: Option<CarouselBorder>,
    backdrop: Option<Hsla>,
}

impl CarouselParallax {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            child: content.into_any_element(),
            axis: Axis::Horizontal,
            viewport: None,
            shape: CornerRole::ExtraLarge.shape(),
            border: None,
            backdrop: None,
        }
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Overrides the effect viewport with bounds in window coordinates.
    pub fn viewport(mut self, viewport: Bounds<Pixels>) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn border(mut self, border: CarouselBorder) -> Self {
        self.border = Some(border);
        self
    }

    /// Matches the opaque color of the scroll viewport behind rounded corners.
    /// The window theme's Surface color is used by default.
    pub fn backdrop(mut self, backdrop: Hsla) -> Self {
        self.backdrop = Some(Hsla { a: 1., ..backdrop });
        self
    }
}

struct Mask {
    bounds: Bounds<Pixels>,
    offset: Point<Pixels>,
}

impl Mask {
    fn new(
        item: Bounds<Pixels>,
        viewport: Bounds<Pixels>,
        axis: Axis,
        reduce_motion: bool,
    ) -> Self {
        let (start, end, clip_start, clip_end, cross) = match axis {
            Axis::Horizontal => (
                item.left(),
                item.right(),
                viewport.left(),
                viewport.right(),
                item.size.height,
            ),
            Axis::Vertical => (
                item.top(),
                item.bottom(),
                viewport.top(),
                viewport.bottom(),
                item.size.width,
            ),
        };
        let extent = end - start;
        let mut crop_start = px(0.);
        let mut crop_end = px(0.);
        let mut translate = px(0.);
        if !reduce_motion && extent > px(0.) && cross > px(0.) {
            let ratio = f32::from(extent) / f32::from(cross);
            let lerp = |a: f32, b: f32, lo, hi| {
                let progress: f32 = (ratio - lo) / (hi - lo);
                a + (b - a) * progress.clamp(0., 1.)
            };
            let intensity = if ratio < 1. {
                lerp(0.25, 1. / 3., 9. / 16., 1.)
            } else {
                lerp(1. / 3., 0.5, 1., 16. / 9.)
            };
            // Recenter part of the content as either viewport edge crops it.
            if start < clip_start {
                let distance = (clip_start - start).min(extent);
                crop_start = distance * (1. - intensity);
                translate = distance * intensity;
            } else if end > clip_end {
                let distance = (end - clip_end).min(extent);
                crop_end = distance * (1. - intensity);
                translate = -distance * intensity;
            }
        }
        let (crop, cropped_size, offset) = match axis {
            Axis::Horizontal => (
                point(crop_start, px(0.)),
                size(extent - crop_start - crop_end, item.size.height),
                point(translate, px(0.)),
            ),
            Axis::Vertical => (
                point(px(0.), crop_start),
                size(item.size.width, extent - crop_start - crop_end),
                point(px(0.), translate),
            ),
        };
        Self {
            bounds: Bounds::new(item.origin + crop, cropped_size),
            offset,
        }
    }
}

/// Layout and clip resolved against the scroll viewport for one draw.
#[doc(hidden)]
pub struct CarouselParallaxPaint {
    mask: Bounds<Pixels>,
    clip: Bounds<Pixels>,
}

impl IntoElement for CarouselParallax {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for CarouselParallax {
    type RequestLayoutState = ();
    type PrepaintState = Option<CarouselParallaxPaint>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let parent_clip = window.content_mask().bounds;
        let viewport = self.viewport.unwrap_or(parent_clip);
        let mask = Mask::new(bounds, viewport, self.axis, cx.reduce_motion());
        let clip = mask.bounds.intersect(&viewport).intersect(&parent_clip);
        if clip.is_empty() {
            return None;
        }
        window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
            window.with_element_offset(mask.offset, |window| {
                self.child.prepaint(window, cx);
            });
        });
        Some(CarouselParallaxPaint {
            mask: mask.bounds,
            clip,
        })
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        paint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(paint) = paint else { return };
        let theme = current_material_theme(window, cx);
        let backdrop = self
            .backdrop
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::Surface)));
        let radii = self
            .shape
            .radii(
                paint.mask.size.width.into(),
                paint.mask.size.height.into(),
                theme.layout.direction,
            )
            .map(px);
        window.with_content_mask(Some(ContentMask { bounds: paint.clip }), |window| {
            self.child.paint(window, cx);
            paint_corners(paint.mask, radii, backdrop, window);
            if let Some(border) = self.border {
                let width = px(super::nonnegative(border.width))
                    .min(paint.mask.size.width.min(paint.mask.size.height) / 2.);
                if width > px(0.) {
                    let [top_left, top_right, bottom_right, bottom_left] = radii;
                    window.paint_quad(quad(
                        paint.mask,
                        Corners {
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                        },
                        gpui::transparent_black(),
                        width,
                        border.color,
                        BorderStyle::Solid,
                    ));
                }
            }
        });
    }
}

fn paint_corners(bounds: Bounds<Pixels>, radii: [Pixels; 4], backdrop: Hsla, window: &mut Window) {
    let mut cutouts = PathBuilder::fill();
    let corners = [
        (point(bounds.left(), bounds.top()), 1., 1.),
        (point(bounds.right(), bounds.top()), -1., 1.),
        (point(bounds.right(), bounds.bottom()), -1., -1.),
        (point(bounds.left(), bounds.bottom()), 1., -1.),
    ];
    for ((corner, x, y), radius) in corners.into_iter().zip(radii) {
        if radius <= px(0.) {
            continue;
        }
        // The two control points approximate the circular arc of a GPUI quad.
        let control = radius * (1. - 0.552_284_8);
        cutouts.move_to(corner);
        cutouts.line_to(corner + point(radius * x, px(0.)));
        cutouts.cubic_bezier_to(
            corner + point(px(0.), radius * y),
            corner + point(control * x, px(0.)),
            corner + point(px(0.), control * y),
        );
        cutouts.close();
    }
    if let Ok(path) = cutouts.build() {
        window.paint_path(path, backdrop);
    }
}
