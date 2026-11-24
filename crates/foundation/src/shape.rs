mod clip;
mod morph;
mod polygon;
#[cfg(test)]
mod tests;

use gpui::{
    Bounds, FillOptions, FillRule, Hsla, Path, PathBuilder, PathStyle, Pixels, Point, Window,
    point, px, size,
};
use gpui_material_theme::{CornerShape, Direction, MaterialShape};
use polymorpher::{Cubic, geometry::Point as PolygonPoint};

use clip::{
    FillMesh, bounds_overlap, clip_convex_polygon_fill, clip_mesh_fill, clip_polygon_to_rect,
    clip_rect_fill, clipped_ripple, contains_fill, cross, flatten_cubic, polygon_area,
    rect_is_valid, segment_distance_sq, triangle_disjoint_from_rect,
};
use polygon::rounded_rect_cubics;

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

/// A prepared shape with a curve path for painting and a lazily built mesh for clipping and hit testing.
#[derive(Clone)]
pub struct Shape {
    outline: Outline,
    fill: Path,
    mesh: std::sync::OnceLock<FillMesh>,
    stroke_cache: std::sync::OnceLock<(u32, Path)>,
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
    fn projection_params(&self) -> Option<(Point<Pixels>, Pixels, Pixels, f32, f32)> {
        let size = self.bounds.size;
        let side = size.width.min(size.height);
        if !(side > px(0.) && side < px(f32::INFINITY))
            || !self.rotation.is_finite()
            || !self.scale.is_finite()
            || self.scale <= 0.
            || self.cubics.is_empty()
        {
            return None;
        }
        let center = self.bounds.center();
        let (width, height) = match self.fit {
            ShapeFit::Contain => (side, side),
            ShapeFit::Stretch => (size.width, size.height),
        };
        let (sin, cos) = self.rotation.sin_cos();
        Some((center, width, height, sin, cos))
    }

    fn project_point(
        &self,
        p: PolygonPoint,
        center: Point<Pixels>,
        width: Pixels,
        height: Pixels,
        sin: f32,
        cos: f32,
    ) -> Point<Pixels> {
        let dx = p.x - self.pivot.x;
        let dy = p.y - self.pivot.y;
        let (rx, ry) = if self.rotation == 0. {
            (dx, dy)
        } else {
            (cos * dx - sin * dy, sin * dx + cos * dy)
        };
        center + point(width * (self.scale * rx), height * (self.scale * ry))
    }

    fn add_to_builder(&self, builder: &mut PathBuilder) -> bool {
        let Some((center, width, height, sin, cos)) = self.projection_params() else {
            return false;
        };
        let project = |p| self.project_point(p, center, width, height, sin, cos);
        builder.move_to(project(self.cubics[0].anchor0()));
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

    fn flatten_mesh(&self) -> FillMesh {
        let Some((center, width, height, sin, cos)) = self.projection_params() else {
            return FillMesh::empty(self.bounds.origin);
        };
        let project = |p| self.project_point(p, center, width, height, sin, cos);
        let mut contour = Vec::with_capacity(self.cubics.len() * 4);
        let start = project(self.cubics[0].anchor0());
        contour.push(start);
        for cubic in &self.cubics {
            flatten_cubic(
                project(cubic.anchor0()),
                project(cubic.control0()),
                project(cubic.control1()),
                project(cubic.anchor1()),
                &mut contour,
            );
        }
        FillMesh::from_contour(&contour)
    }

    fn path(&self, mut builder: PathBuilder) -> Option<Path> {
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
            mesh: std::sync::OnceLock::new(),
            stroke_cache: std::sync::OnceLock::new(),
        })
    }

    fn mesh(&self) -> &FillMesh {
        self.mesh.get_or_init(|| self.outline.flatten_mesh())
    }

    /// Returns the laid-out bounds that this shape was fitted into.
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.outline.bounds
    }

    /// Returns the tight bounding box of this shape's flattened fill contour.
    pub fn fill_bounds(&self) -> Bounds<Pixels> {
        self.mesh().bounds
    }

    /// Returns the prepared GPUI fill [`Path`] for this shape.
    pub fn fill_path(&self) -> &Path {
        &self.fill
    }

    /// Builds a stroked outline path for this shape at the requested logical pixel width.
    pub fn stroke_path(&self, width: Pixels) -> Option<Path> {
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

    /// Returns whether `position` lies inside this shape's fill mesh.
    pub fn contains(&self, position: Point<Pixels>) -> bool {
        contains_fill(self.mesh(), position)
    }

    /// Shape-aware hit test against the fill mesh with an optional outward `slop`
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
        let mesh = self.mesh();
        let min_x = f32::from(mesh.bounds.left()) - slop;
        let max_x = f32::from(mesh.bounds.right()) + slop;
        let min_y = f32::from(mesh.bounds.top()) - slop;
        let max_y = f32::from(mesh.bounds.bottom()) + slop;
        let px_x = f32::from(position.x);
        let px_y = f32::from(position.y);
        if px_x < min_x || px_x > max_x || px_y < min_y || px_y > max_y {
            return false;
        }
        let limit_sq = slop * slop;
        mesh.triangles.iter().any(|&[a, b, c]| {
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
        let mesh = self.mesh();
        if !rect_is_valid(rect) || !bounds_overlap(mesh.bounds, rect) {
            return false;
        }
        let mut polygon = Vec::with_capacity(8);
        let mut scratch = Vec::with_capacity(8);
        for &[a, b, c] in &mesh.triangles {
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
    pub fn clip_rect(&self, rect: Bounds<Pixels>) -> Option<Path> {
        clip_rect_fill(self.mesh(), rect).to_path()
    }

    /// Clips a disc centered at `origin` with logical `radius` against this shape's fill mesh.
    pub fn clip_disc(&self, origin: Point<Pixels>, radius: Pixels) -> Option<Path> {
        clipped_ripple(self.mesh(), origin, radius).to_path()
    }

    /// Clips a convex polygon (in either winding order) against this shape's fill mesh.
    pub fn clip_convex_polygon(&self, vertices: &[Point<Pixels>]) -> Option<Path> {
        clip_convex_polygon_fill(self.mesh(), vertices).to_path()
    }

    /// Clips another [`Shape`] (convex or concave) against this shape's fill mesh,
    /// returning the exact geometric intersection.
    pub fn clip_shape(&self, source: &Shape) -> Option<Path> {
        clip_mesh_fill(self.mesh(), source.mesh()).to_path()
    }

    /// Builds an EvenOdd exterior mask representing `outer \ self` within `outer`.
    ///
    /// Because GPUI's native `ContentMask` is rectangular, painting this exterior mask
    /// in a uniform parent backdrop color visually clips arbitrary child elements to the
    /// exact non-rectangular shape contour.
    pub fn exterior_mask(&self, outer: Bounds<Pixels>) -> Option<Path> {
        if !rect_is_valid(outer) {
            return None;
        }
        let mesh = self.mesh();
        let contained = mesh.bounds.left() >= outer.left()
            && mesh.bounds.right() <= outer.right()
            && mesh.bounds.top() >= outer.top()
            && mesh.bounds.bottom() <= outer.bottom();
        let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions {
            fill_rule: FillRule::EvenOdd,
        }));
        builder.move_to(outer.origin);
        builder.line_to(outer.top_right());
        builder.line_to(outer.bottom_right());
        builder.line_to(outer.bottom_left());
        builder.close();
        if contained {
            if !self.outline.add_to_builder(&mut builder) {
                return None;
            }
        } else {
            let interior = clip_rect_fill(mesh, outer);
            for &[a, b, c] in &interior.triangles {
                if cross(b - a, c - a).abs() <= f32::EPSILON {
                    continue;
                }
                builder.move_to(a);
                builder.line_to(b);
                builder.line_to(c);
                builder.close();
            }
        }
        builder.build().ok()
    }

    /// Paints the fill path with `color`.
    pub fn paint(&self, color: Hsla, window: &mut Window) {
        window.paint_path(self.fill.clone(), color);
    }

    /// Paints a stroked outline around this shape.
    pub fn paint_stroke(&self, width: Pixels, color: Hsla, window: &mut Window) {
        if let Some(path) = self.stroke_path(width) {
            window.paint_path(path, color);
        }
    }

    /// Paints `source` clipped to this shape's fill mesh.
    pub fn paint_clipped_shape(&self, source: &Shape, color: Hsla, window: &mut Window) {
        if let Some(clipped) = self.clip_shape(source) {
            window.paint_path(clipped, color);
        }
    }

    /// Paints `rect` clipped to this shape's fill mesh.
    pub fn paint_clipped_rect(&self, rect: Bounds<Pixels>, color: Hsla, window: &mut Window) {
        if let Some(clipped) = self.clip_rect(rect) {
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
        if let Some(mask) = self.exterior_mask(outer) {
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
        if let Some(path) = self.clip_disc(origin, radius) {
            window.paint_path(path, color);
        }
    }
}
