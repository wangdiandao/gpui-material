mod clip;
mod morph;
mod polygon;
#[cfg(test)]
mod tests;

use gpui::{
    Bounds, FillOptions, Hsla, Path, PathBuilder, PathStyle, Pixels, Point, Window, point, px, size,
};
use gpui_material_theme::{CornerShape, Direction, MaterialShape};
use polymorpher::{Cubic, geometry::Point as PolygonPoint};

use clip::{
    bounds_overlap, clip_convex_polygon_fill, clip_path_fill, clip_polygon_to_rect, clip_rect_fill,
    contains_fill, cross, polygon_area, rect_is_valid, segment_distance_sq,
    triangle_disjoint_from_rect,
};
use polygon::rounded_rect_cubics;

pub(crate) use clip::clipped_ripple;
pub use morph::{PolygonMorph, ShapeMorph, ShapeMorphSequence};
pub use polygon::ShapePolygon;

/// How a normalized polygon contour maps into layout bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShapeFit {
    /// Fits the centered unit square inside the shorter side while preserving aspect ratio.
    #[default]
    Contain,
    /// Scales independently to fill the width and height of the bounds.
    Stretch,
}

/// A prepared shape with a shared mesh for drawing, clipping, hit testing, and effects.
#[derive(Clone)]
pub struct Shape {
    outline: Outline,
    fill: Path<Pixels>,
    stroke_cache: std::sync::OnceLock<(u32, Path<Pixels>)>,
}

#[derive(Clone)]
struct Outline {
    cubics: Vec<Cubic>,
    bounds: Bounds<Pixels>,
    fit: ShapeFit,
    rotation: f32,
    pivot: PolygonPoint,
    scale: f32,
}

impl Outline {
    fn add_to_builder(&self, builder: &mut PathBuilder) -> bool {
        let size = self.bounds.size;
        let side = size.width.min(size.height);
        if !(side > px(0.) && side < px(f32::INFINITY))
            || !self.rotation.is_finite()
            || !self.scale.is_finite()
            || self.scale <= 0.
        {
            return false;
        }
        let Some(first) = self.cubics.first() else {
            return false;
        };
        let center = self.bounds.center();
        let (width, height) = match self.fit {
            ShapeFit::Contain => (side, side),
            ShapeFit::Stretch => (size.width, size.height),
        };
        let (sin, cos) = self.rotation.sin_cos();
        let project = |p: PolygonPoint| {
            let dx = p.x - self.pivot.x;
            let dy = p.y - self.pivot.y;
            let (rx, ry) = if self.rotation == 0. {
                (dx, dy)
            } else {
                (cos * dx - sin * dy, sin * dx + cos * dy)
            };
            center + point(width * (self.scale * rx), height * (self.scale * ry))
        };
        builder.move_to(project(first.anchor0()));
        for cubic in &self.cubics {
            builder.cubic_bezier_to(
                project(cubic.anchor1()),
                project(cubic.control0()),
                project(cubic.control1()),
            );
        }
        builder.close();
        true
    }

    fn path(&self, mut builder: PathBuilder) -> Option<Path<Pixels>> {
        if !self.add_to_builder(&mut builder) {
            return None;
        }
        builder.build().ok()
    }
}

impl Shape {
    /// Resolves Material corner radii against the actual bounds and direction.
    pub fn rounded(
        bounds: Bounds<Pixels>,
        corners: CornerShape,
        direction: Direction,
    ) -> Option<Self> {
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let cubics = rounded_rect_cubics(width, height, corners, direction)?;
        Self::new(Outline {
            cubics,
            bounds,
            fit: ShapeFit::Stretch,
            rotation: 0.,
            pivot: PolygonPoint::splat(0.5),
            scale: 1.,
        })
    }

    /// Prepares one of the 35 Material 3 Expressive shapes fitted inside `bounds`.
    pub fn material(bounds: Bounds<Pixels>, shape: MaterialShape) -> Option<Self> {
        ShapePolygon::material(shape).to_shape(bounds)
    }

    /// Prepares any reusable [`ShapePolygon`] inside `bounds`.
    pub fn polygon(bounds: Bounds<Pixels>, polygon: &ShapePolygon, fit: ShapeFit) -> Option<Self> {
        polygon.to_shape_with(bounds, fit, 0.)
    }

    fn from_cubics(
        cubics: Vec<Cubic>,
        bounds: Bounds<Pixels>,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Self> {
        if !rotation_radians.is_finite() {
            return None;
        }
        Self::new(Outline {
            cubics,
            bounds,
            fit,
            rotation: rotation_radians,
            pivot: PolygonPoint::splat(0.5),
            scale: 1.,
        })
    }

    fn new(outline: Outline) -> Option<Self> {
        let fill = outline.path(PathBuilder::fill())?;
        Some(Self {
            outline,
            fill,
            stroke_cache: std::sync::OnceLock::new(),
        })
    }

    /// Returns the laid-out bounds that this shape was fitted into.
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.outline.bounds
    }

    /// Returns the tessellated fill mesh shared by painting, clipping, and hit testing.
    pub fn fill_path(&self) -> &Path<Pixels> {
        &self.fill
    }

    /// Builds a stroked outline path for this shape at the requested logical pixel width.
    pub fn stroke_path(&self, width: Pixels) -> Option<Path<Pixels>> {
        let raw = f32::from(width);
        if !raw.is_finite() || raw <= 0. {
            return None;
        }
        let bits = raw.to_bits();
        if let Some((cached_bits, cached_path)) = self.stroke_cache.get()
            && *cached_bits == bits
        {
            return Some(cached_path.clone());
        }
        let path = self.outline.path(PathBuilder::stroke(width))?;
        let _ = self.stroke_cache.set((bits, path.clone()));
        Some(path)
    }

    /// Returns whether `position` lies inside this shape's tessellated fill mesh.
    pub fn contains(&self, position: Point<Pixels>) -> bool {
        contains_fill(&self.fill, position)
    }

    /// Shape-aware hit test against the tessellated mesh with an optional outward `slop`
    /// tolerance in logical pixels. Non-finite coordinates return `false`.
    pub fn hit_test(&self, position: Point<Pixels>, slop: Pixels) -> bool {
        if !f32::from(position.x).is_finite() || !f32::from(position.y).is_finite() {
            return false;
        }
        if self.contains(position) {
            return true;
        }
        let slop = f32::from(slop);
        if !slop.is_finite() || slop <= 0. {
            return false;
        }
        let min_x = f32::from(self.fill.bounds.left()) - slop;
        let max_x = f32::from(self.fill.bounds.right()) + slop;
        let min_y = f32::from(self.fill.bounds.top()) - slop;
        let max_y = f32::from(self.fill.bounds.bottom()) + slop;
        let px_x = f32::from(position.x);
        let px_y = f32::from(position.y);
        if px_x < min_x || px_x > max_x || px_y < min_y || px_y > max_y {
            return false;
        }
        let limit_sq = slop * slop;
        self.fill
            .vertices
            .as_chunks::<3>()
            .0
            .iter()
            .any(|triangle| {
                let a = triangle[0].xy_position;
                let b = triangle[1].xy_position;
                let c = triangle[2].xy_position;
                if cross(b - a, c - a).abs() <= f32::EPSILON {
                    return false;
                }
                segment_distance_sq(position, a, b) <= limit_sq
                    || segment_distance_sq(position, b, c) <= limit_sq
                    || segment_distance_sq(position, c, a) <= limit_sq
            })
    }

    /// Returns whether this shape's fill mesh intersects the axis-aligned rectangle `rect`.
    pub fn intersects_bounds(&self, rect: Bounds<Pixels>) -> bool {
        if !rect_is_valid(rect) || !bounds_overlap(self.fill.bounds, rect) {
            return false;
        }
        let mut polygon = Vec::with_capacity(8);
        let mut scratch = Vec::with_capacity(8);
        for triangle in self.fill.vertices.as_chunks::<3>().0 {
            let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
            if triangle_disjoint_from_rect([a, b, c], rect)
                || cross(b - a, c - a).abs() <= f32::EPSILON
            {
                continue;
            }
            polygon.clear();
            polygon.extend([a, b, c]);
            clip_polygon_to_rect(&mut polygon, &mut scratch, rect);
            if polygon.len() >= 3 && polygon_area(&polygon).abs() > f32::EPSILON {
                return true;
            }
        }
        false
    }

    /// Clips an axis-aligned rectangle against this shape's fill mesh.
    pub fn clip_rect(&self, rect: Bounds<Pixels>) -> Path<Pixels> {
        clip_rect_fill(&self.fill, rect)
    }

    /// Clips a disc centered at `origin` with logical `radius` against this shape's fill mesh.
    pub fn clip_disc(&self, origin: Point<Pixels>, radius: Pixels) -> Path<Pixels> {
        clipped_ripple(&self.fill, origin, radius)
    }

    /// Clips a convex polygon (in either winding order) against this shape's fill mesh.
    pub fn clip_convex_polygon(&self, vertices: &[Point<Pixels>]) -> Path<Pixels> {
        clip_convex_polygon_fill(&self.fill, vertices)
    }

    /// Clips an arbitrary tessellated GPUI [`Path`] (convex or concave) against this shape's
    /// fill mesh, returning the exact geometric intersection.
    pub fn clip_path(&self, source: &Path<Pixels>) -> Path<Pixels> {
        clip_path_fill(&self.fill, source)
    }

    /// Builds an EvenOdd exterior mask representing `outer \ self` within `outer`.
    ///
    /// Because GPUI's native `ContentMask` is rectangular, painting this exterior mask
    /// in a uniform parent backdrop color visually clips arbitrary child elements to the
    /// exact non-rectangular shape contour.
    pub fn exterior_mask(&self, outer: Bounds<Pixels>) -> Option<Path<Pixels>> {
        if !rect_is_valid(outer) {
            return None;
        }
        let contained = self.fill.bounds.left() >= outer.left()
            && self.fill.bounds.right() <= outer.right()
            && self.fill.bounds.top() >= outer.top()
            && self.fill.bounds.bottom() <= outer.bottom();
        let mask_outer = if contained {
            outer
        } else {
            outer.union(&self.fill.bounds)
        };
        let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions::even_odd()));
        builder.move_to(mask_outer.origin);
        builder.line_to(mask_outer.top_right());
        builder.line_to(mask_outer.bottom_right());
        builder.line_to(mask_outer.bottom_left());
        builder.close();
        if !self.outline.add_to_builder(&mut builder) {
            return None;
        }
        let mask = builder.build().ok()?;
        if contained {
            Some(mask)
        } else {
            Some(clip_rect_fill(&mask, outer))
        }
    }

    /// Paints the tessellated fill mesh with `color`.
    pub fn paint(&self, color: Hsla, window: &mut Window) {
        window.paint_path(self.fill.clone(), color);
    }

    /// Paints a stroked outline around this shape.
    pub fn paint_stroke(&self, width: Pixels, color: Hsla, window: &mut Window) {
        if let Some(path) = self.stroke_path(width) {
            window.paint_path(path, color);
        }
    }

    /// Paints `path` clipped to this shape's fill mesh.
    pub fn paint_clipped(&self, path: &Path<Pixels>, color: Hsla, window: &mut Window) {
        let clipped = self.clip_path(path);
        if !clipped.vertices.is_empty() {
            window.paint_path(clipped, color);
        }
    }

    /// Paints `rect` clipped to this shape's fill mesh.
    pub fn paint_clipped_rect(&self, rect: Bounds<Pixels>, color: Hsla, window: &mut Window) {
        let clipped = self.clip_rect(rect);
        if !clipped.vertices.is_empty() {
            window.paint_path(clipped, color);
        }
    }

    /// Paints a disc clipped to this shape's fill mesh.
    pub fn paint_clipped_disc(
        &self,
        origin: Point<Pixels>,
        radius: Pixels,
        color: Hsla,
        window: &mut Window,
    ) {
        self.paint_ripple(origin, radius, color, window);
    }

    /// Paints the EvenOdd exterior mask `outer \ self` in `backdrop` to visually clip
    /// overflowing children to this shape on a uniform background.
    pub fn paint_backdrop_mask(&self, outer: Bounds<Pixels>, backdrop: Hsla, window: &mut Window) {
        if let Some(mask) = self.exterior_mask(outer)
            && !mask.vertices.is_empty()
        {
            window.paint_path(mask, backdrop);
        }
    }

    /// Approximates a shape-matched elevation shadow with three translucent fills.
    pub fn paint_shadow(&self, elevation: f32, scale: f32, shadow: Hsla, window: &mut Window) {
        let dp = elevation * scale;
        if !dp.is_finite() || dp <= 0. || !scale.is_finite() || scale <= 0. {
            return;
        }
        for (offset_y, spread, alpha) in [
            (dp * 0.45, dp * 0.35, 0.08),
            (dp * 0.25, dp * 0.18, 0.10),
            (scale.min(dp * 0.5), dp * 0.06, 0.12),
        ] {
            let expanded = Bounds::new(
                self.outline.bounds.origin + point(px(-spread), px(offset_y - spread)),
                size(
                    self.outline.bounds.size.width + px(spread * 2.),
                    self.outline.bounds.size.height + px(spread * 2.),
                ),
            );
            let layer = Outline {
                bounds: expanded,
                ..self.outline.clone()
            };
            if let Some(path) = layer.path(PathBuilder::fill()) {
                window.paint_path(path, shadow.opacity(alpha));
            }
        }
    }

    pub(crate) fn paint_ring(&self, width: Pixels, color: Hsla, window: &mut Window) {
        self.paint_stroke(width, color, window);
    }

    pub(crate) fn paint_ripple(
        &self,
        origin: Point<Pixels>,
        radius: Pixels,
        color: Hsla,
        window: &mut Window,
    ) {
        let path = self.clip_disc(origin, radius);
        if !path.vertices.is_empty() {
            window.paint_path(path, color);
        }
    }
}
