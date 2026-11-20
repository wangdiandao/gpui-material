use std::panic::{AssertUnwindSafe, catch_unwind};

use gpui::{
    Bounds, FillOptions, Hsla, Path, PathBuilder, PathStyle, Pixels, Point, Window, point, px, size,
};
use gpui_material_theme::{CornerShape, Direction, MaterialShape, PolygonRole, PolygonSpec};
use polymorpher::{
    CornerRounding, Cubic, Morph, RoundedPolygon, geometry::Point as PolygonPoint, shapes,
};

const RIPPLE_SEGMENTS: usize = 64;

/// How a normalized polygon contour maps into layout bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShapeFit {
    /// Fits the centered unit square inside the shorter side while preserving aspect ratio.
    #[default]
    Contain,
    /// Scales independently to fill the width and height of the bounds.
    Stretch,
}

/// A reusable normalized polygon contour backed by `polymorpher`.
///
/// Third-party polygon types stay inside Foundation so components and applications
/// share one shape pipeline across Expressive presets, rounded rectangles, custom
/// polygons, and morph transitions.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapePolygon {
    kind: PolygonKind,
}

#[derive(Clone, Debug, PartialEq)]
enum PolygonKind {
    Rounded(RoundedPolygon),
    Cubics(Vec<Cubic>),
}

impl ShapePolygon {
    /// Constructs one of the 35 Material 3 Expressive shapes matching Compose `MaterialShapes`.
    pub fn material(shape: MaterialShape) -> Self {
        Self::from_rounded(material_polygon(shape))
    }

    /// Constructs a regular rounded circle polygon with at least three vertices.
    pub fn circle(vertices: usize) -> Option<Self> {
        if vertices < 3 {
            return None;
        }
        Some(Self::from_rounded(
            RoundedPolygon::circle()
                .with_vertices(vertices)
                .build()
                .normalized(),
        ))
    }

    /// Constructs a regular polygon with optional corner rounding, smoothing, and rotation.
    pub fn regular(
        vertices: usize,
        rounding: f32,
        smoothing: f32,
        rotation_degrees: f32,
    ) -> Option<Self> {
        if vertices < 3 || !valid_rounding(rounding, smoothing) || !rotation_degrees.is_finite() {
            return None;
        }
        let polygon = RoundedPolygon::from_vertices_count(
            vertices,
            1.,
            Some(CornerRounding::smoothed(rounding, smoothing)),
            &[],
        );
        let polygon = if rotation_degrees != 0. {
            rotate_polygon(polygon, rotation_degrees)
        } else {
            polygon
        };
        Some(Self::from_rounded(polygon.normalized()))
    }

    /// Constructs a star polygon with outer radius `1.0`, `inner_radius` in `(0, 1)`,
    /// outer corner rounding, and optional inner corner rounding `(radius, smoothing)`.
    pub fn star(
        vertices: usize,
        inner_radius: f32,
        rounding: f32,
        smoothing: f32,
        inner_rounding: Option<(f32, f32)>,
    ) -> Option<Self> {
        if vertices < 3
            || !inner_radius.is_finite()
            || !(0. < inner_radius && inner_radius < 1.)
            || !valid_rounding(rounding, smoothing)
        {
            return None;
        }
        let mut builder = RoundedPolygon::star(vertices)
            .with_inner_radius(inner_radius)
            .with_rounding(CornerRounding::smoothed(rounding, smoothing));
        if let Some((inner_r, inner_s)) = inner_rounding {
            if !valid_rounding(inner_r, inner_s) {
                return None;
            }
            builder = builder.with_inner_rounding(CornerRounding::smoothed(inner_r, inner_s));
        }
        Some(Self::from_rounded(builder.build().normalized()))
    }

    /// Constructs a rounded rectangle mapped to the full unit square `[0, 1] x [0, 1]`
    /// for use with [`ShapeFit::Stretch`] against matching `(width, height)` bounds.
    ///
    /// When rendered into matching bounds with [`ShapeFit::Stretch`], this reproduces
    /// [`Shape::rounded`] and allows morphing between [`CornerShape`] and [`MaterialShape`].
    pub fn rounded_rect(
        width: f32,
        height: f32,
        corners: CornerShape,
        direction: Direction,
    ) -> Option<Self> {
        let polygon = rounded_rect_polygon(width, height, corners, direction)?;
        Some(Self::from_rounded(polygon))
    }

    /// Creates a normalized rounded polygon from perimeter vertices in either winding order.
    /// The radius uses the same units as the vertices. Non-finite, degenerate, or
    /// self-intersecting geometry returns `None`.
    pub fn from_vertices(vertices: &[Point<f32>], radius: f32) -> Option<Self> {
        if vertices.len() < 3 || !radius.is_finite() || radius < 0. {
            return None;
        }
        let (mut vertices, side) = normalized_vertices(vertices)?;
        if !simple_polygon(&vertices) {
            return None;
        }
        let area: f32 = polygon_edges(&vertices)
            .map(|(a, b)| a.x * b.y - a.y * b.x)
            .sum();
        if !area.is_finite() || area.abs() < 1e-5 {
            return None;
        }
        if area < 0. {
            vertices.reverse();
        }
        let first = vertices
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.x.total_cmp(&b.x).then_with(|| b.y.total_cmp(&a.y)))?
            .0;
        vertices.rotate_left(first);
        let rounding = CornerRounding::new((f64::from(radius) / side).min(1.) as f32);
        let polygon = RoundedPolygon::from_vertices(&vertices, rounding, &[], PolygonPoint::zero());
        (polygon.cubics.len() > 1).then(|| Self::from_rounded(polygon.normalized()))
    }

    /// Returns a copy rotated clockwise by `degrees` around its center and re-normalized.
    pub fn rotated(&self, degrees: f32) -> Self {
        if !degrees.is_finite() || degrees == 0. {
            return self.clone();
        }
        match &self.kind {
            PolygonKind::Rounded(polygon) => {
                let center = polygon.center;
                let (sin, cos) = degrees.to_radians().sin_cos();
                Self::from_rounded(
                    polygon
                        .clone()
                        .transformed(|p: PolygonPoint| {
                            let dx = p.x - center.x;
                            let dy = p.y - center.y;
                            PolygonPoint::new(
                                center.x + dx * cos - dy * sin,
                                center.y + dx * sin + dy * cos,
                            )
                        })
                        .normalized(),
                )
            }
            PolygonKind::Cubics(cubics) => {
                let center = cubic_center(cubics);
                let (sin, cos) = degrees.to_radians().sin_cos();
                let rotate = |p: PolygonPoint| {
                    let dx = p.x - center.x;
                    let dy = p.y - center.y;
                    PolygonPoint::new(
                        center.x + dx * cos - dy * sin,
                        center.y + dx * sin + dy * cos,
                    )
                };
                let rotated: Vec<_> = cubics
                    .iter()
                    .map(|cubic| {
                        Cubic::new(
                            rotate(cubic.anchor0()),
                            rotate(cubic.control0()),
                            rotate(cubic.control1()),
                            rotate(cubic.anchor1()),
                        )
                    })
                    .collect();
                Self {
                    kind: PolygonKind::Cubics(normalize_cubics(rotated)),
                }
            }
        }
    }

    /// Scale factor that keeps this polygon inside its unit bounds under arbitrary rotation.
    ///
    /// Matches Compose's `LoadingIndicator` rotation-envelope calculation, comparing the
    /// polygon's bounds against the furthest sampled anchor or cubic midpoint from center.
    pub fn rotation_scale(&self) -> f32 {
        let cubics = self.cubics();
        if cubics.is_empty() {
            return 1.;
        }
        let (bounds_max, center) = match &self.kind {
            PolygonKind::Rounded(polygon) => {
                let bounds = polygon.aabb(true).size();
                (bounds.width.max(bounds.height), polygon.center)
            }
            PolygonKind::Cubics(cubics) => {
                let (min, max) = cubic_aabb(cubics);
                let size = max - min;
                (size.x.max(size.y), min + size * 0.5)
            }
        };
        let radius = cubics
            .iter()
            .flat_map(|cubic| [cubic.anchor0(), cubic.point_on_curve(0.5)])
            .map(|point| (point - center).length())
            .fold(0_f32, f32::max);
        if radius <= f32::EPSILON || bounds_max <= f32::EPSILON {
            1.
        } else {
            (bounds_max / (2. * radius)).clamp(0., 1.)
        }
    }

    /// Projects this polygon into `bounds` using [`ShapeFit::Contain`].
    pub fn to_shape(&self, bounds: Bounds<Pixels>) -> Option<Shape> {
        self.to_shape_with(bounds, ShapeFit::Contain, 0.)
    }

    /// Projects this polygon into `bounds` with explicit fit mode and clockwise rotation in radians.
    pub fn to_shape_with(
        &self,
        bounds: Bounds<Pixels>,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Shape> {
        Shape::from_cubics(self.cubics().to_vec(), bounds, fit, rotation_radians)
    }

    /// Returns the number of cubic Bézier segments in this normalized contour.
    pub fn cubic_count(&self) -> usize {
        self.cubics().len()
    }

    fn from_rounded(polygon: RoundedPolygon) -> Self {
        Self {
            kind: PolygonKind::Rounded(polygon),
        }
    }

    fn cubics(&self) -> &[Cubic] {
        match &self.kind {
            PolygonKind::Rounded(polygon) => &polygon.cubics,
            PolygonKind::Cubics(cubics) => cubics,
        }
    }

    fn control_point_radius(&self) -> f32 {
        self.cubics()
            .iter()
            .flat_map(|cubic| {
                [
                    cubic.anchor0(),
                    cubic.control0(),
                    cubic.control1(),
                    cubic.anchor1(),
                ]
            })
            .map(|point| (point - PolygonPoint::splat(0.5)).length())
            .fold(0_f32, f32::max)
    }
}

impl From<MaterialShape> for ShapePolygon {
    fn from(shape: MaterialShape) -> Self {
        Self::material(shape)
    }
}

impl From<PolygonRole> for ShapePolygon {
    fn from(role: PolygonRole) -> Self {
        Self::from_rounded(role_polygon(role))
    }
}

/// A prepared shape with a shared mesh for drawing, clipping, hit testing, and effects.
#[derive(Clone)]
pub struct Shape {
    outline: Outline,
    fill: Path<Pixels>,
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
        let cubics = rounded_rect_polygon(width, height, corners, direction)?.cubics;
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
        Some(Self { outline, fill })
    }

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
        self.outline.path(PathBuilder::stroke(width))
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

/// A reusable morph between any two [`ShapePolygon`] contours.
///
/// Uses `polymorpher`'s feature-matched morph when both endpoints are `RoundedPolygon`s
/// and falls back to perimeter-length cubic alignment when an endpoint is an arbitrary
/// cubic contour or triggers a feature-matching edge case.
#[derive(Clone)]
pub struct ShapeMorph {
    kind: MorphKind,
}

#[derive(Clone)]
enum MorphKind {
    FeatureMatched(Morph),
    PerimeterAligned(Vec<(Cubic, Cubic)>),
}

impl ShapeMorph {
    /// Creates a reusable morph between any two shapes (such as [`MaterialShape`],
    /// [`PolygonRole`], or [`ShapePolygon`]).
    pub fn new(start: impl Into<ShapePolygon>, end: impl Into<ShapePolygon>) -> Self {
        let start = start.into();
        let end = end.into();
        let kind = match (&start.kind, &end.kind) {
            (PolygonKind::Rounded(from), PolygonKind::Rounded(to)) => {
                match catch_unwind(AssertUnwindSafe(|| Morph::new(from.clone(), to.clone()))) {
                    Ok(morph) => MorphKind::FeatureMatched(morph),
                    Err(_) => {
                        MorphKind::PerimeterAligned(align_perimeter(&from.cubics, &to.cubics))
                    }
                }
            }
            _ => MorphKind::PerimeterAligned(align_perimeter(start.cubics(), end.cubics())),
        };
        Self { kind }
    }

    /// Creates a morph aligned strictly by normalized perimeter length, preserving exact
    /// endpoint curves with bounded cubic count even for arbitrary concave polygons.
    pub fn perimeter_aligned(start: impl Into<ShapePolygon>, end: impl Into<ShapePolygon>) -> Self {
        let start = start.into();
        let end = end.into();
        Self {
            kind: MorphKind::PerimeterAligned(align_perimeter(start.cubics(), end.cubics())),
        }
    }

    /// Samples the interpolated normalized contour at `progress` (clamped to `0..=1`).
    pub fn sample_polygon(&self, progress: f32) -> ShapePolygon {
        ShapePolygon {
            kind: PolygonKind::Cubics(self.as_cubics(clamp_progress(progress))),
        }
    }

    /// Fits the sampled shape inside `bounds` while preserving aspect ratio.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        self.sample_with(bounds, progress, ShapeFit::Contain, 0.)
    }

    /// Samples the morph into `bounds` with explicit fit mode and clockwise rotation in radians.
    pub fn sample_with(
        &self,
        bounds: Bounds<Pixels>,
        progress: f32,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Shape> {
        Shape::from_cubics(
            self.as_cubics(clamp_progress(progress)),
            bounds,
            fit,
            rotation_radians,
        )
    }

    fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        match &self.kind {
            MorphKind::FeatureMatched(morph) => morph.as_cubics(progress),
            MorphKind::PerimeterAligned(pairs) => pairs
                .iter()
                .map(|(a, b)| {
                    Cubic::new(
                        a.anchor0().lerp(b.anchor0(), progress),
                        a.control0().lerp(b.control0(), progress),
                        a.control1().lerp(b.control1(), progress),
                        a.anchor1().lerp(b.anchor1(), progress),
                    )
                })
                .collect(),
        }
    }
}

/// A reusable multi-step sequence of [`ShapeMorph`] transitions.
#[derive(Clone)]
pub struct ShapeMorphSequence {
    steps: Vec<ShapeMorph>,
    wrap: Option<ShapeMorph>,
    rotation_scale: f32,
    control_radius: Option<f32>,
}

impl ShapeMorphSequence {
    /// Builds a feature-matched morph sequence from at least two shapes.
    /// When `looping` is `true`, adds a wrap transition from the last shape back to the first.
    pub fn new(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
    ) -> Option<Self> {
        Self::build(shapes, looping, false)
    }

    /// Builds a perimeter-aligned morph sequence from at least two shapes.
    pub fn perimeter_aligned(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
    ) -> Option<Self> {
        Self::build(shapes, looping, true)
    }

    fn build(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
        perimeter_aligned: bool,
    ) -> Option<Self> {
        let polygons: Vec<ShapePolygon> = shapes.into_iter().map(Into::into).collect();
        if polygons.len() < 2 {
            return None;
        }
        let make = |a: &ShapePolygon, b: &ShapePolygon| {
            if perimeter_aligned {
                ShapeMorph::perimeter_aligned(a.clone(), b.clone())
            } else {
                ShapeMorph::new(a.clone(), b.clone())
            }
        };
        let steps = polygons
            .windows(2)
            .map(|pair| make(&pair[0], &pair[1]))
            .collect();
        let wrap = looping.then(|| make(polygons.last().unwrap(), &polygons[0]));
        let rotation_scale = polygons
            .iter()
            .fold(1_f32, |scale, polygon| scale.min(polygon.rotation_scale()));
        let control_radius = perimeter_aligned.then(|| {
            polygons
                .iter()
                .map(ShapePolygon::control_point_radius)
                .fold(0_f32, f32::max)
        });
        Some(Self {
            steps,
            wrap,
            rotation_scale,
            control_radius,
        })
    }

    /// Number of endpoint shapes in this sequence.
    pub fn len(&self) -> usize {
        self.steps.len() + 1
    }

    /// Always `false` because valid sequences contain at least two shapes.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Number of morph transitions (`len() - 1` when open, `len()` when looping).
    pub fn step_count(&self) -> usize {
        self.steps.len() + usize::from(self.wrap.is_some())
    }

    pub fn is_looping(&self) -> bool {
        self.wrap.is_some()
    }

    /// Minimum rotation-safe scale factor across all shapes in the sequence.
    pub fn rotation_scale(&self) -> f32 {
        self.rotation_scale
    }

    /// Samples the open progression across `[0, 1]` from the first shape to the last.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        let value = clamp_progress(progress);
        let phase = value * self.steps.len() as f32;
        let index = (phase as usize).min(self.steps.len() - 1);
        self.sample_step(index, phase - index as f32, bounds, ShapeFit::Contain, 0.)
    }

    /// Samples a specific transition step at `progress` in `0..=1`.
    pub fn sample_step(
        &self,
        step: usize,
        progress: f32,
        bounds: Bounds<Pixels>,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Shape> {
        let morph = self.morph_for_step(step)?;
        morph.sample_with(bounds, progress, fit, rotation_radians)
    }

    /// Samples a specific transition step as a reusable [`ShapePolygon`].
    pub fn sample_step_polygon(&self, step: usize, progress: f32) -> Option<ShapePolygon> {
        let morph = self.morph_for_step(step)?;
        Some(morph.sample_polygon(progress))
    }

    /// Samples a transition step centered on its live cubic bounding box and scaled to
    /// fit safely inside a circular envelope of diameter `extent` under arbitrary rotation.
    ///
    /// Used by `LoadingIndicator` to keep rotating Material and custom morph sequences
    /// strictly inside their active footprint without resizing shared endpoints.
    pub fn sample_step_in_envelope(
        &self,
        step: usize,
        progress: f32,
        bounds: Bounds<Pixels>,
        extent: Pixels,
        rotation_radians: f32,
    ) -> Option<Shape> {
        if !rotation_radians.is_finite() {
            return None;
        }
        let morph = self.morph_for_step(step)?;
        let cubics = morph.as_cubics(clamp_progress(progress));
        if cubics.is_empty() {
            return None;
        }
        let pivot = cubic_center(&cubics);
        let side = bounds.size.width.min(bounds.size.height);
        let extent = side.min(extent);
        if !(extent > px(0.) && extent < px(f32::INFINITY)) {
            return None;
        }
        let mut target_side = extent * self.rotation_scale;
        if let Some(radius) = self.control_radius {
            let radius = radius + (pivot - PolygonPoint::splat(0.5)).length();
            if radius > f32::EPSILON {
                target_side = target_side.min(extent / (2. * radius));
            }
        }
        let scale = f32::from(target_side) / f32::from(side);
        Shape::new(Outline {
            cubics,
            bounds,
            fit: ShapeFit::Contain,
            rotation: rotation_radians,
            pivot,
            scale,
        })
    }

    fn morph_for_step(&self, step: usize) -> Option<&ShapeMorph> {
        if self.wrap.is_some() {
            let count = self.steps.len() + 1;
            let index = step % count;
            self.steps.get(index).or(self.wrap.as_ref())
        } else {
            self.steps.get(step)
        }
    }
}

/// A reusable morph between two pinned Material polygon roles.
#[derive(Clone)]
pub struct PolygonMorph(ShapeMorph);

impl PolygonMorph {
    pub fn new(start: PolygonRole, end: PolygonRole) -> Self {
        Self(ShapeMorph::new(start, end))
    }

    /// Fits the sampled polygon inside the bounds while preserving aspect ratio.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        self.0.sample(bounds, progress)
    }
}

fn clamp_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0., 1.)
    } else {
        0.
    }
}

fn valid_rounding(radius: f32, smoothing: f32) -> bool {
    radius.is_finite() && radius >= 0. && smoothing.is_finite() && (0.0..=1.0).contains(&smoothing)
}

fn rotate_polygon(polygon: RoundedPolygon, degrees: f32) -> RoundedPolygon {
    let (sin, cos) = degrees.to_radians().sin_cos();
    polygon.transformed(|p: PolygonPoint| {
        PolygonPoint::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
    })
}

fn rounded_rect_polygon(
    width: f32,
    height: f32,
    corners: CornerShape,
    direction: Direction,
) -> Option<RoundedPolygon> {
    if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
        return None;
    }
    let [tl, tr, br, bl] = corners.radii(width, height, direction);
    let side = width.max(height);
    let polygon = RoundedPolygon::rectangle()
        .with_width(width / side)
        .with_height(height / side)
        .with_rounding_per_vertex([br, bl, tl, tr].map(|radius| CornerRounding::new(radius / side)))
        .build();
    Some(polygon.transformed(|p: PolygonPoint| {
        PolygonPoint::new(p.x * side / width + 0.5, p.y * side / height + 0.5)
    }))
}

fn material_polygon(shape: MaterialShape) -> RoundedPolygon {
    match shape {
        MaterialShape::Circle => shapes::circle(None),
        MaterialShape::Square => shapes::square(),
        MaterialShape::Slanted => shapes::slanted(),
        MaterialShape::Arch => shapes::arch(),
        MaterialShape::Fan => shapes::fan(),
        MaterialShape::Arrow => shapes::arrow(),
        MaterialShape::SemiCircle => shapes::semi_circle(),
        MaterialShape::Oval => shapes::oval(),
        MaterialShape::Pill => shapes::pill(),
        MaterialShape::Triangle => shapes::triangle(),
        MaterialShape::Diamond => shapes::diamond(),
        MaterialShape::ClamShell => shapes::clam_shell(),
        MaterialShape::Pentagon => shapes::pentagon(),
        MaterialShape::Gem => shapes::gem(),
        MaterialShape::Sunny => shapes::sunny(),
        MaterialShape::VerySunny => shapes::very_sunny(),
        MaterialShape::Cookie4Sided => shapes::cookie4(),
        MaterialShape::Cookie6Sided => shapes::cookie6(),
        MaterialShape::Cookie7Sided => shapes::cookie7(),
        MaterialShape::Cookie9Sided => shapes::cookie9(),
        MaterialShape::Cookie12Sided => shapes::cookie12(),
        MaterialShape::Ghostish => shapes::ghostish(),
        MaterialShape::Clover4Leaf => shapes::clover4(),
        MaterialShape::Clover8Leaf => shapes::clover8(),
        MaterialShape::Burst => shapes::burst(),
        MaterialShape::SoftBurst => shapes::soft_burst(),
        MaterialShape::Boom => shapes::boom(),
        MaterialShape::SoftBoom => shapes::soft_boom(),
        MaterialShape::Flower => shapes::flower(),
        MaterialShape::Puffy => shapes::puffy(),
        MaterialShape::PuffyDiamond => shapes::puffy_diamond(),
        MaterialShape::PixelCircle => shapes::pixel_circle(),
        MaterialShape::PixelTriangle => shapes::pixel_triangle(),
        MaterialShape::Bun => shapes::bun(),
        MaterialShape::Heart => shapes::heart(),
    }
}

fn role_polygon(role: PolygonRole) -> RoundedPolygon {
    match role.spec() {
        PolygonSpec::Circle { vertices, radius } => RoundedPolygon::circle()
            .with_vertices(usize::from(vertices))
            .with_radius(radius)
            .build(),
        PolygonSpec::Rectangle {
            width,
            height,
            rounding_radius,
            smoothing,
        } => RoundedPolygon::rectangle()
            .with_width(width)
            .with_height(height)
            .with_rounding(CornerRounding::smoothed(rounding_radius, smoothing))
            .build(),
        PolygonSpec::Regular {
            vertices,
            radius,
            rounding_radius,
            smoothing,
            rotation_degrees,
        } => rotate_polygon(
            RoundedPolygon::from_vertices_count(
                usize::from(vertices),
                radius,
                Some(CornerRounding::smoothed(rounding_radius, smoothing)),
                &[],
            ),
            rotation_degrees,
        ),
    }
    .normalized()
}

fn align_perimeter(start: &[Cubic], end: &[Cubic]) -> Vec<(Cubic, Cubic)> {
    let mut starts = weighted_segments(start).into_iter();
    let mut ends = weighted_segments(end).into_iter();
    let (mut from, mut to) = (starts.next(), ends.next());
    let mut position = 0.;
    let mut pairs = Vec::new();
    while let (Some(mut a), Some(mut b)) = (from, to) {
        let boundary = a.0.min(b.0);
        let first = split_until(&mut a, position, boundary);
        let last = split_until(&mut b, position, boundary);
        pairs.push((first, last));
        from = if boundary == a.0 {
            starts.next()
        } else {
            Some(a)
        };
        to = if boundary == b.0 {
            ends.next()
        } else {
            Some(b)
        };
        position = boundary;
    }
    pairs
}

fn weighted_segments(cubics: &[Cubic]) -> Vec<(f64, Cubic)> {
    let mut cumulative = 0.;
    let mut segments: Vec<_> = cubics
        .iter()
        .map(|cubic| {
            let samples = [
                cubic.anchor0(),
                cubic.point_on_curve(0.25),
                cubic.point_on_curve(0.5),
                cubic.point_on_curve(0.75),
                cubic.anchor1(),
            ];
            cumulative += samples
                .windows(2)
                .map(|points| f64::from((points[1] - points[0]).length()))
                .sum::<f64>();
            (cumulative, *cubic)
        })
        .collect();
    if cumulative > 0. {
        for (end, _) in &mut segments {
            *end /= cumulative;
        }
    }
    segments
}

fn split_until(segment: &mut (f64, Cubic), from: f64, until: f64) -> Cubic {
    if until == segment.0 {
        return segment.1;
    }
    let (head, tail) = segment
        .1
        .split(((until - from) / (segment.0 - from)) as f32);
    segment.1 = tail;
    head
}

fn normalized_vertices(vertices: &[Point<f32>]) -> Option<(Vec<PolygonPoint>, f64)> {
    let (mut left, mut top, mut right, mut bottom) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for point in vertices {
        if !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let x = f64::from(point.x);
        let y = f64::from(point.y);
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
    }
    let side = (right - left).max(bottom - top);
    if side == 0. {
        return None;
    }
    let centered = vertices.iter().map(|point| {
        PolygonPoint::new(
            ((f64::from(point.x) - (left + right) * 0.5) / side) as f32,
            ((f64::from(point.y) - (top + bottom) * 0.5) / side) as f32,
        )
    });
    Some((centered.collect(), side))
}

fn polygon_edges(
    vertices: &[PolygonPoint],
) -> impl Iterator<Item = (PolygonPoint, PolygonPoint)> + Clone + '_ {
    vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
}

fn simple_polygon(vertices: &[PolygonPoint]) -> bool {
    let edges = polygon_edges(vertices);
    for (index, (a, b)) in edges.clone().enumerate() {
        if (b - a).length() < 1e-5 {
            return false;
        }
        for (other, (c, d)) in edges.clone().enumerate().skip(index + 2) {
            if index == 0 && other == vertices.len() - 1 {
                continue;
            }
            let orient = |a: PolygonPoint, b: PolygonPoint, c: PolygonPoint| {
                let u = b - a;
                let v = c - a;
                u.x * v.y - u.y * v.x
            };
            let overlaps =
                |a: f32, b: f32, c: f32, d: f32| a.min(b) <= c.max(d) && c.min(d) <= a.max(b);
            if overlaps(a.x, b.x, c.x, d.x)
                && overlaps(a.y, b.y, c.y, d.y)
                && orient(a, b, c) * orient(a, b, d) <= 0.
                && orient(c, d, a) * orient(c, d, b) <= 0.
            {
                return false;
            }
        }
    }
    true
}

fn cubic_aabb(cubics: &[Cubic]) -> (PolygonPoint, PolygonPoint) {
    let mut minimum = PolygonPoint::new(f32::INFINITY, f32::INFINITY);
    let mut maximum = PolygonPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for cubic in cubics {
        let bounds = cubic.aabb(true);
        minimum = minimum.min(bounds.min);
        maximum = maximum.max(bounds.max);
    }
    (minimum, maximum)
}

fn cubic_center(cubics: &[Cubic]) -> PolygonPoint {
    let (minimum, maximum) = cubic_aabb(cubics);
    minimum + (maximum - minimum) * 0.5
}

fn normalize_cubics(cubics: Vec<Cubic>) -> Vec<Cubic> {
    if cubics.is_empty() {
        return cubics;
    }
    let (min, max) = cubic_aabb(&cubics);
    let size = max - min;
    let max_side = size.x.max(size.y);
    if !max_side.is_finite() || max_side <= f32::EPSILON {
        return cubics;
    }
    let offset = (PolygonPoint::splat(max_side) - size) * 0.5 - min.to_vector();
    let map = |p: PolygonPoint| (p + offset.to_vector()) / max_side;
    cubics
        .into_iter()
        .map(|cubic| {
            Cubic::new(
                map(cubic.anchor0()),
                map(cubic.control0()),
                map(cubic.control1()),
                map(cubic.anchor1()),
            )
        })
        .collect()
}

fn cross(a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    f32::from(a.x) * f32::from(b.y) - f32::from(a.y) * f32::from(b.x)
}

fn segment_distance_sq(p: Point<Pixels>, a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    let ab_x = f32::from(b.x - a.x);
    let ab_y = f32::from(b.y - a.y);
    let ap_x = f32::from(p.x - a.x);
    let ap_y = f32::from(p.y - a.y);
    let len_sq = ab_x * ab_x + ab_y * ab_y;
    let t = if len_sq <= f32::EPSILON {
        0.
    } else {
        ((ap_x * ab_x + ap_y * ab_y) / len_sq).clamp(0., 1.)
    };
    let dx = ap_x - ab_x * t;
    let dy = ap_y - ab_y * t;
    dx * dx + dy * dy
}

fn polygon_area(vertices: &[Point<Pixels>]) -> f32 {
    let Some(&first) = vertices.first() else {
        return 0.;
    };
    vertices
        .windows(2)
        .skip(1)
        .map(|edge| cross(edge[0] - first, edge[1] - first))
        .sum::<f32>()
        * 0.5
}

fn rect_is_valid(rect: Bounds<Pixels>) -> bool {
    let x = f32::from(rect.origin.x);
    let y = f32::from(rect.origin.y);
    let w = f32::from(rect.size.width);
    let h = f32::from(rect.size.height);
    x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite() && w > 0. && h > 0.
}

fn bounds_overlap(a: Bounds<Pixels>, b: Bounds<Pixels>) -> bool {
    a.left() < b.right() && a.right() > b.left() && a.top() < b.bottom() && a.bottom() > b.top()
}

fn triangle_disjoint_from_rect(vertices: [Point<Pixels>; 3], rect: Bounds<Pixels>) -> bool {
    let min_x = rect.left();
    let max_x = rect.right();
    let min_y = rect.top();
    let max_y = rect.bottom();
    vertices.iter().all(|v| v.x <= min_x)
        || vertices.iter().all(|v| v.x >= max_x)
        || vertices.iter().all(|v| v.y <= min_y)
        || vertices.iter().all(|v| v.y >= max_y)
}

fn contains_fill(path: &Path<Pixels>, position: Point<Pixels>) -> bool {
    path.bounds.contains(&position)
        && path.vertices.as_chunks::<3>().0.iter().any(|triangle| {
            let a = triangle[0].xy_position;
            let b = triangle[1].xy_position;
            let c = triangle[2].xy_position;
            let area = cross(b - a, c - a);
            if area.abs() <= f32::EPSILON {
                return false;
            }
            let signs = [
                cross(b - a, position - a),
                cross(c - b, position - b),
                cross(a - c, position - c),
            ];
            signs.into_iter().all(|sign| sign * area >= 0.)
        })
}

fn clip_half_plane(
    polygon: &mut Vec<Point<Pixels>>,
    clipped: &mut Vec<Point<Pixels>>,
    distance: impl Fn(Point<Pixels>) -> f32,
) {
    let Some(&last) = polygon.last() else {
        return;
    };
    let mut previous = last;
    let mut previous_distance = distance(previous);
    clipped.clear();
    for &current in polygon.iter() {
        let current_distance = distance(current);
        if (previous_distance >= 0.) != (current_distance >= 0.) {
            let t = previous_distance / (previous_distance - current_distance);
            clipped.push(previous + (current - previous) * t);
        }
        if current_distance >= 0. {
            clipped.push(current);
        }
        previous = current;
        previous_distance = current_distance;
    }
    std::mem::swap(polygon, clipped);
}

fn clip_polygon_to_rect(
    polygon: &mut Vec<Point<Pixels>>,
    scratch: &mut Vec<Point<Pixels>>,
    rect: Bounds<Pixels>,
) {
    let left = f32::from(rect.left());
    let right = f32::from(rect.right());
    let top = f32::from(rect.top());
    let bottom = f32::from(rect.bottom());
    clip_half_plane(polygon, scratch, |p| f32::from(p.x) - left);
    clip_half_plane(polygon, scratch, |p| right - f32::from(p.x));
    clip_half_plane(polygon, scratch, |p| f32::from(p.y) - top);
    clip_half_plane(polygon, scratch, |p| bottom - f32::from(p.y));
}

fn clip_rect_fill(fill: &Path<Pixels>, rect: Bounds<Pixels>) -> Path<Pixels> {
    let mut result = Path::new(rect.origin);
    if !rect_is_valid(rect) || fill.vertices.is_empty() || !bounds_overlap(fill.bounds, rect) {
        return result;
    }
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
        if triangle_disjoint_from_rect([a, b, c], rect) || cross(b - a, c - a).abs() <= f32::EPSILON
        {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        clip_polygon_to_rect(&mut polygon, &mut scratch, rect);
        push_fan(&mut result, &polygon, stencil);
    }
    result
}

fn clip_convex_polygon_fill(fill: &Path<Pixels>, vertices: &[Point<Pixels>]) -> Path<Pixels> {
    let origin = vertices.first().copied().unwrap_or(point(px(0.), px(0.)));
    let mut result = Path::new(origin);
    if vertices.len() < 3
        || fill.vertices.is_empty()
        || !vertices
            .iter()
            .all(|v| f32::from(v.x).is_finite() && f32::from(v.y).is_finite())
    {
        return result;
    }
    let area = polygon_area(vertices);
    if !area.is_finite() || area.abs() <= f32::EPSILON {
        return result;
    }
    let orientation = if area > 0. { 1. } else { -1. };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    for &v in vertices {
        let x = f32::from(v.x);
        let y = f32::from(v.y);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let clip_bounds = Bounds::new(
        point(px(min_x), px(min_y)),
        size(px(max_x - min_x), px(max_y - min_y)),
    );
    if !bounds_overlap(fill.bounds, clip_bounds) {
        return result;
    }
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(vertices.len() + 3);
    let mut scratch = Vec::with_capacity(vertices.len() + 3);
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
        if triangle_disjoint_from_rect([a, b, c], clip_bounds)
            || cross(b - a, c - a).abs() <= f32::EPSILON
        {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        for (&start, &end) in vertices.iter().zip(vertices.iter().cycle().skip(1)) {
            if polygon.is_empty() {
                break;
            }
            clip_half_plane(&mut polygon, &mut scratch, |p| {
                orientation * cross(end - start, p - start)
            });
        }
        push_fan(&mut result, &polygon, stencil);
    }
    result
}

fn clip_path_fill(clip: &Path<Pixels>, source: &Path<Pixels>) -> Path<Pixels> {
    let mut result = Path::new(source.bounds.origin);
    if clip.vertices.is_empty()
        || source.vertices.is_empty()
        || !bounds_overlap(clip.bounds, source.bounds)
    {
        return result;
    }
    let clip_triangles: Vec<_> = clip
        .vertices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|triangle| {
            let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
            let winding = cross(b - a, c - a);
            if winding.abs() <= f32::EPSILON
                || triangle_disjoint_from_rect([a, b, c], source.bounds)
            {
                return None;
            }
            let min_x = a.x.min(b.x).min(c.x);
            let max_x = a.x.max(b.x).max(c.x);
            let min_y = a.y.min(b.y).min(c.y);
            let max_y = a.y.max(b.y).max(c.y);
            let sign = if winding > 0. { 1. } else { -1. };
            Some(([a, b, c], min_x, max_x, min_y, max_y, sign))
        })
        .collect();
    if clip_triangles.is_empty() {
        return result;
    }

    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for triangle in source.vertices.as_chunks::<3>().0 {
        let [sa, sb, sc] = triangle.each_ref().map(|vertex| vertex.xy_position);
        if cross(sb - sa, sc - sa).abs() <= f32::EPSILON
            || triangle_disjoint_from_rect([sa, sb, sc], clip.bounds)
        {
            continue;
        }
        let s_min_x = sa.x.min(sb.x).min(sc.x);
        let s_max_x = sa.x.max(sb.x).max(sc.x);
        let s_min_y = sa.y.min(sb.y).min(sc.y);
        let s_max_y = sa.y.max(sb.y).max(sc.y);
        for &([ca, cb, cc], c_min_x, c_max_x, c_min_y, c_max_y, sign) in &clip_triangles {
            if s_max_x <= c_min_x || s_min_x >= c_max_x || s_max_y <= c_min_y || s_min_y >= c_max_y
            {
                continue;
            }
            polygon.clear();
            polygon.extend([sa, sb, sc]);
            for (start, end) in [(ca, cb), (cb, cc), (cc, ca)] {
                if polygon.is_empty() {
                    break;
                }
                clip_half_plane(&mut polygon, &mut scratch, |p| {
                    sign * cross(end - start, p - start)
                });
            }
            push_fan(&mut result, &polygon, stencil);
        }
    }
    result
}

fn push_fan(path: &mut Path<Pixels>, polygon: &[Point<Pixels>], stencil: Point<f32>) {
    if let Some((&first, rest)) = polygon.split_first() {
        for edge in rest.windows(2) {
            if cross(edge[0] - first, edge[1] - first).abs() > f32::EPSILON {
                path.push_triangle((first, edge[0], edge[1]), (stencil, stencil, stencil));
            }
        }
    }
}

pub(crate) fn clipped_ripple(
    fill: &Path<Pixels>,
    origin: Point<Pixels>,
    radius: Pixels,
) -> Path<Pixels> {
    let raw_radius = f32::from(radius);
    if !raw_radius.is_finite()
        || raw_radius <= 0.
        || !f32::from(origin.x).is_finite()
        || !f32::from(origin.y).is_finite()
    {
        return Path::new(origin);
    }
    let circle: [_; RIPPLE_SEGMENTS] = std::array::from_fn(|index| {
        let angle = std::f32::consts::TAU * index as f32 / RIPPLE_SEGMENTS as f32;
        origin + point(radius * angle.cos(), radius * angle.sin())
    });
    let mut ripple = Path::new(origin);
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let mut clipped = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let extent = point(radius, radius);
    let min = origin - extent;
    let max = origin + extent;
    let has_area = min.x < max.x && min.y < max.y;

    // Intersect the disc with each disjoint fill triangle so paint and hit
    // testing use the same tessellated curve, including concave shapes.
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let vertices = triangle.each_ref().map(|vertex| vertex.xy_position);
        if has_area
            && (vertices.iter().all(|vertex| vertex.x < min.x)
                || vertices.iter().all(|vertex| vertex.x > max.x)
                || vertices.iter().all(|vertex| vertex.y < min.y)
                || vertices.iter().all(|vertex| vertex.y > max.y))
        {
            continue;
        }
        let [a, b, c] = vertices;
        let winding = cross(b - a, c - a);
        if winding.abs() <= f32::EPSILON {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        for (&start, &end) in circle.iter().zip(circle.iter().cycle().skip(1)) {
            if polygon.is_empty() {
                break;
            }
            clip_half_plane(&mut polygon, &mut clipped, |p| {
                cross(end - start, p - start)
            });
        }
        if let Some((&first, rest)) = polygon.split_first() {
            for edge in rest.windows(2) {
                ripple.push_triangle((first, edge[0], edge[1]), (stencil, stencil, stencil));
            }
        }
    }
    ripple
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_material_theme::CornerRole;

    const TOLERANCE: f32 = 0.000_01;

    #[test]
    fn all_35_expressive_material_shapes_stay_closed_in_bounds_and_tessellate() {
        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(120.), px(80.)));
        let extent = -TOLERANCE..=1. + TOLERANCE;
        let outside = bounds.origin + point(px(1.), px(1.));

        for shape in MaterialShape::ALL {
            let polygon = ShapePolygon::material(shape);
            let cubics = polygon.cubics();
            assert!(!cubics.is_empty(), "{shape:?} produced no cubics");
            for (cubic, next) in cubics.iter().zip(cubics.iter().cycle().skip(1)) {
                assert!(
                    (cubic.anchor1() - next.anchor0()).length() <= 1e-4,
                    "{shape:?} contour is open",
                );
                for step in 0..=8 {
                    let pt = cubic.point_on_curve(step as f32 / 8.);
                    assert!(
                        extent.contains(&pt.x) && extent.contains(&pt.y),
                        "{shape:?} escaped normalized bounds: {pt:?}",
                    );
                }
            }
            let prepared = Shape::material(bounds, shape).unwrap();
            assert!(
                !prepared.fill_path().vertices.is_empty(),
                "{shape:?} tessellated empty fill"
            );
            assert!(
                prepared.stroke_path(px(2.)).is_some(),
                "{shape:?} failed stroke path"
            );
            assert!(
                prepared.contains(bounds.center()),
                "{shape:?} should contain its center"
            );
            assert!(
                !prepared.contains(outside),
                "{shape:?} should not contain the outer non-square corner"
            );
            assert!(
                (0.0..=1.0).contains(&polygon.rotation_scale()) && polygon.rotation_scale() > 0.,
                "{shape:?} invalid rotation scale"
            );
        }
    }

    #[test]
    fn general_morph_handles_expressive_catalog_corners_and_custom_polygons() {
        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(100.), px(100.)));
        let extent = -1e-3..=1. + 1e-3;

        // Verify consecutive transitions across all 35 MaterialShape presets.
        let sequence = ShapeMorphSequence::new(MaterialShape::ALL, true).unwrap();
        assert_eq!(sequence.len(), 35);
        assert_eq!(sequence.step_count(), 35);
        assert!(sequence.is_looping());
        for step in 0..sequence.step_count() {
            for progress in [0., 0.5, 1.] {
                let sample = sequence
                    .sample_step(step, progress, bounds, ShapeFit::Contain, 0.)
                    .unwrap();
                let cubics = &sample.outline.cubics;
                assert!(!cubics.is_empty());
                for (cubic, next) in cubics.iter().zip(cubics.iter().cycle().skip(1)) {
                    assert!(
                        (cubic.anchor1() - next.anchor0()).length() <= 1e-3,
                        "step {step} open at {progress}"
                    );
                    for k in 0..=4 {
                        let p = cubic.point_on_curve(k as f32 / 4.);
                        assert!(
                            extent.contains(&p.x) && extent.contains(&p.y),
                            "step {step} left bounds at {progress}: {p:?}"
                        );
                    }
                }
                assert!(sample.contains(bounds.center()));
            }
        }

        // Morph between a CornerShape rounded rectangle and an Expressive MaterialShape.
        let rect_bounds = Bounds::new(point(px(0.), px(0.)), size(px(160.), px(40.)));
        let rect_poly =
            ShapePolygon::rounded_rect(160., 40., CornerShape::Radii([8.; 4]), Direction::Ltr)
                .unwrap();
        let pill_poly =
            ShapePolygon::rounded_rect(160., 40., CornerShape::Full, Direction::Ltr).unwrap();
        let rect_morph = ShapeMorph::new(rect_poly.clone(), pill_poly);
        let at_start = rect_morph
            .sample_with(rect_bounds, 0., ShapeFit::Stretch, 0.)
            .unwrap();
        let at_end = rect_morph
            .sample_with(rect_bounds, 1., ShapeFit::Stretch, 0.)
            .unwrap();
        let corner_sample = rect_bounds.origin + point(px(3.), px(3.));
        assert!(at_start.contains(corner_sample));
        assert!(!at_end.contains(corner_sample));

        // Morph from a rounded rect into Heart and sample a reusable intermediate polygon.
        let to_heart = ShapeMorph::new(rect_poly, MaterialShape::Heart);
        let mid_poly = to_heart.sample_polygon(0.5);
        let mid_shape = mid_poly.to_shape(bounds).unwrap();
        assert!(mid_shape.contains(bounds.center()));

        // Custom star, regular, and concave vertex polygons morph cleanly without panicking.
        let star = ShapePolygon::star(5, 0.5, 0.1, 0.2, Some((0.05, 0.))).unwrap();
        let hex = ShapePolygon::regular(6, 0.15, 0.5, 30.).unwrap();
        let concave = ShapePolygon::from_vertices(
            &[
                point(-1., -1.),
                point(1., -1.),
                point(0., 0.),
                point(1., 1.),
                point(-1., 1.),
            ],
            0.1,
        )
        .unwrap();
        for morph in [
            ShapeMorph::new(star, hex.clone()),
            ShapeMorph::new(concave.clone(), MaterialShape::SoftBurst),
            ShapeMorph::new(mid_poly, hex),
        ] {
            for progress in [0., 0.5, 1.] {
                let sampled = morph.sample(bounds, progress).unwrap();
                assert!(!sampled.fill_path().vertices.is_empty());
            }
        }
        assert!(ShapePolygon::circle(2).is_none());
        assert!(ShapePolygon::star(4, 1.2, 0.1, 0., None).is_none());
        assert!(ShapePolygon::regular(2, 0.1, 0., 0.).is_none());
        assert!(ShapePolygon::rounded_rect(0., 10., CornerShape::Full, Direction::Ltr).is_none());
    }

    #[test]
    fn every_material_shape_pair_morphs_cleanly_at_midpoint() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(64.), px(64.)));
        for start in MaterialShape::ALL {
            for end in MaterialShape::ALL {
                let morph = ShapeMorph::new(start, end);
                let mid = morph.sample(bounds, 0.5).unwrap();
                assert!(
                    !mid.fill_path().vertices.is_empty(),
                    "empty morph fill for {start:?} -> {end:?}"
                );
            }
        }
    }

    #[test]
    fn shape_clipping_covers_rect_convex_polygon_arbitrary_path_and_exterior_mask() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));
        let heart = Shape::material(bounds, MaterialShape::Heart).unwrap();

        // Top-center cleft and bottom-left corner lie outside Heart.
        let outside_corner = point(px(5.), px(95.));
        let inside_center = bounds.center();
        assert!(heart.contains(inside_center));
        assert!(!heart.contains(outside_corner));

        // 1. clip_rect keeps only the right half of Heart.
        let right_half = Bounds::new(point(px(50.), px(0.)), size(px(50.), px(100.)));
        let clipped_rect = heart.clip_rect(right_half);
        assert!(contains_fill(&clipped_rect, point(px(62.), px(45.))));
        assert!(!contains_fill(&clipped_rect, point(px(38.), px(45.))));
        assert!(!contains_fill(&clipped_rect, point(px(95.), px(95.))));

        // 2. clip_convex_polygon works in both CW and CCW winding orders.
        let triangle_ccw = [
            point(px(50.), px(10.)),
            point(px(95.), px(90.)),
            point(px(5.), px(90.)),
        ];
        let mut triangle_cw = triangle_ccw;
        triangle_cw.reverse();
        let clipped_ccw = heart.clip_convex_polygon(&triangle_ccw);
        let clipped_cw = heart.clip_convex_polygon(&triangle_cw);
        assert!(contains_fill(&clipped_ccw, inside_center));
        assert!(contains_fill(&clipped_cw, inside_center));
        assert!(!contains_fill(&clipped_ccw, outside_corner));
        assert!(!contains_fill(&clipped_cw, outside_corner));

        // 3. clip_path intersects two non-rectangular shapes (Heart ∩ Clover4Leaf).
        let clover = Shape::material(bounds, MaterialShape::Clover4Leaf).unwrap();
        let intersection = heart.clip_path(clover.fill_path());
        assert!(contains_fill(&intersection, inside_center));
        assert!(!contains_fill(&intersection, outside_corner));
        // Every triangle centroid of the intersection must lie inside both shapes.
        for tri in intersection.vertices.as_chunks::<3>().0 {
            let centroid = (tri[0].xy_position + tri[1].xy_position + tri[2].xy_position) / 3.;
            assert!(heart.contains(centroid), "escaped heart at {centroid:?}");
            assert!(clover.contains(centroid), "escaped clover at {centroid:?}");
        }

        // 4. exterior_mask covers outer \ shape and leaves interior unmasked.
        let mask = heart.exterior_mask(bounds).unwrap();
        assert!(contains_fill(&mask, outside_corner));
        assert!(!contains_fill(&mask, inside_center));

        // Even when outer is smaller than the shape, exterior_mask stays strictly inside outer.
        let inner_box = Bounds::new(point(px(15.), px(15.)), size(px(70.), px(70.)));
        let cropped_mask = heart.exterior_mask(inner_box).unwrap();
        assert!(!contains_fill(&cropped_mask, inside_center));
        for tri in cropped_mask.vertices.as_chunks::<3>().0 {
            for v in tri {
                assert!(v.xy_position.x >= px(14.99) && v.xy_position.x <= px(85.01));
                assert!(v.xy_position.y >= px(14.99) && v.xy_position.y <= px(85.01));
            }
        }
    }

    #[test]
    fn hit_test_slop_and_bounds_intersection_respect_concave_contours() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));
        let burst = Shape::material(bounds, MaterialShape::Burst).unwrap();
        let circle = Shape::material(bounds, MaterialShape::Circle).unwrap();

        // Point just outside the right edge of the circle (center 50,50, radius ~50).
        let just_outside = point(px(102.), px(50.));
        assert!(!circle.contains(just_outside));
        assert!(!circle.hit_test(just_outside, px(1.)));
        assert!(circle.hit_test(just_outside, px(3.)));
        assert!(!circle.hit_test(point(px(f32::NAN), px(50.)), px(10.)));

        // Bounds intersection detects overlapping vs disjoint corner boxes.
        assert!(
            circle.intersects_bounds(Bounds::new(point(px(40.), px(40.)), size(px(20.), px(20.))))
        );
        assert!(
            !circle.intersects_bounds(Bounds::new(point(px(0.), px(0.)), size(px(8.), px(8.))))
        );
        assert!(
            burst.intersects_bounds(Bounds::new(point(px(45.), px(45.)), size(px(10.), px(10.))))
        );
        assert!(!burst.intersects_bounds(Bounds::new(point(px(0.), px(0.)), size(px(5.), px(5.)))));
    }

    #[test]
    fn ripple_keeps_crossing_edges_even_when_no_vertex_is_inside() {
        let origin = point(px(17.), px(31.));
        let stencil = point(0., 1.);
        let mut fill = Path::new(origin);
        fill.push_triangle(
            (
                origin + point(px(-20.), px(0.)),
                origin + point(px(20.), px(0.)),
                origin + point(px(0.), px(30.)),
            ),
            (stencil, stencil, stencil),
        );

        let radius = px(5.);
        let expected = clipped_ripple(&fill, origin, radius);
        assert!(contains_fill(&expected, origin + point(px(0.), px(4.))));
        assert!(!contains_fill(&expected, origin + point(px(0.), px(-1.))));
        assert!(!contains_fill(&expected, origin + point(px(0.), px(6.))));

        for offset in [(-12., 0.), (12., 0.), (0., -12.), (0., 40.)] {
            let start = origin + point(px(offset.0), px(offset.1));
            fill.push_triangle(
                (
                    start,
                    start + point(px(1.), px(0.)),
                    start + point(px(0.), px(1.)),
                ),
                (stencil, stencil, stencil),
            );
        }

        let actual = clipped_ripple(&fill, origin, radius);
        assert!(
            actual
                .vertices
                .iter()
                .map(|vertex| vertex.xy_position)
                .eq(expected.vertices.iter().map(|vertex| vertex.xy_position))
        );

        for (edge, visible) in [(4.9, true), (5., false), (5.1, false)] {
            let mut fill = Path::new(origin);
            fill.push_triangle(
                (
                    origin + point(px(edge), px(-6.)),
                    origin + point(px(edge), px(6.)),
                    origin + point(px(20.), px(0.)),
                ),
                (stencil, stencil, stencil),
            );
            let ripple = clipped_ripple(&fill, origin, radius);
            assert_eq!(
                contains_fill(&ripple, origin + point(px(4.95), px(0.))),
                visible
            );
        }
    }

    #[test]
    fn ripple_preserves_concave_fill() {
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(0.), px(0.)));
        for (x, y) in [(30., 0.), (30., 10.), (10., 10.), (10., 30.), (0., 30.)] {
            builder.line_to(point(px(x), px(y)));
        }
        builder.close();
        let fill = builder.build().unwrap();
        let origin = point(px(8.), px(8.));
        for (radius, far_arm_is_visible) in [(8., false), (40., true)] {
            let ripple = clipped_ripple(&fill, origin, px(radius));
            assert!(contains_fill(&ripple, point(px(12.), px(8.))));
            assert!(contains_fill(&ripple, point(px(8.), px(12.))));
            assert!(!contains_fill(&ripple, point(px(12.), px(12.))));
            assert_eq!(
                contains_fill(&ripple, point(px(25.), px(8.))),
                far_arm_is_visible
            );
        }
    }

    #[test]
    #[ignore = "Release performance benchmark"]
    fn ripple_clip_benchmark() {
        use std::{hint::black_box, time::Instant};

        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(160.), px(40.)));
        let capsule = Shape::rounded(bounds, CornerShape::Full, Direction::Ltr).unwrap();
        let morph = PolygonMorph::new(PolygonRole::Circle, PolygonRole::Triangle)
            .sample(bounds, 0.5)
            .unwrap();
        let center = bounds.center();

        for (name, shape, origins, radii) in [
            (
                "capsule expanding",
                &capsule,
                [center, center - point(px(66.), px(7.))],
                [4., 12., 28., 64.],
            ),
            ("capsule covered", &capsule, [center; 2], [180.; 4]),
            ("morph expanding", &morph, [center; 2], [2., 5., 10., 20.]),
            ("morph covered", &morph, [center; 2], [40.; 4]),
        ] {
            let run = |repetitions| {
                for _ in 0..repetitions {
                    for origin in origins {
                        for radius in radii {
                            black_box(clipped_ripple(
                                black_box(&shape.fill),
                                black_box(origin),
                                black_box(px(radius)),
                            ));
                        }
                    }
                }
            };
            run(100);
            let start = Instant::now();
            run(1_000);
            println!(
                "{name}: {:.1} ns/clip ({} fill triangles)",
                start.elapsed().as_secs_f64() * 1e9 / 8_000.,
                shape.fill.vertices.len() / 3
            );
        }
    }

    #[test]
    fn adjacent_rounded_corners_share_the_available_edge() {
        let bounds = Bounds::new(point(px(5.), px(10.)), size(px(40.), px(60.)));
        let corners = CornerShape::Radii([40., 40., 0., 0.]);
        let shape = Shape::rounded(bounds, corners, Direction::Ltr).unwrap();

        // Oversized adjacent radii meet at the midpoint of the normalized top edge.
        let mut top_anchors = shape
            .outline
            .cubics
            .iter()
            .flat_map(|cubic| [cubic.anchor0(), cubic.anchor1()])
            .filter(|point| point.y.abs() <= TOLERANCE)
            .peekable();
        assert!(top_anchors.peek().is_some());
        assert!(top_anchors.all(|point| (point.x - 0.5).abs() <= TOLERANCE));
    }

    #[test]
    fn identity_morph_preserves_cubic_control_points() {
        let bounds = Bounds::new(point(px(5.), px(10.)), size(px(80.), px(40.)));
        let points = |cubic: &Cubic| {
            [
                cubic.anchor0(),
                cubic.control0(),
                cubic.control1(),
                cubic.anchor1(),
            ]
        };

        for role in [
            PolygonRole::Triangle,
            PolygonRole::Square,
            PolygonRole::Circle,
        ] {
            // Use the feature curves before the drawing outline splits its first corner.
            let expected: Vec<_> = role_polygon(role)
                .features
                .into_iter()
                .flat_map(|feature| feature.cubics)
                .filter(|cubic| !cubic.zero_length())
                .collect();
            let morph = PolygonMorph::new(role, role);
            for progress in [0., 0.5, 1.] {
                let sample = morph.sample(bounds, progress).unwrap();
                let actual: Vec<_> = sample
                    .outline
                    .cubics
                    .iter()
                    .filter(|cubic| !cubic.zero_length())
                    .collect();
                assert_eq!(actual.len(), expected.len(), "{role:?} at {progress}");
                assert!(
                    (0..expected.len()).any(|start| {
                        actual.iter().zip(expected.iter().cycle().skip(start)).all(
                            |(actual, expected)| {
                                points(actual)
                                    .into_iter()
                                    .zip(points(expected))
                                    .all(|(a, b)| (a - b).length() <= TOLERANCE)
                            },
                        )
                    }),
                    "{role:?} changed cubic geometry at {progress}",
                );
            }
        }
    }

    #[test]
    fn distinct_morph_endpoints_preserve_their_hit_regions() {
        let bounds = Bounds::new(point(px(5.), px(10.)), size(px(80.), px(40.)));
        let near_corner = bounds.center() + point(px(17.), px(13.));
        let morph = PolygonMorph::new(PolygonRole::Square, PolygonRole::Circle);

        assert!(morph.sample(bounds, 0.).unwrap().contains(near_corner));
        assert!(!morph.sample(bounds, 1.).unwrap().contains(near_corner));
    }

    #[test]
    fn logical_corners_and_polygon_samples_have_shape_aware_hit_tests() {
        let bounds = Bounds::new(point(px(5.), px(10.)), size(px(80.), px(40.)));
        let at_start = bounds.origin + point(px(1.), px(1.));
        let shape =
            |direction| Shape::rounded(bounds, CornerRole::LargeStart.shape(), direction).unwrap();
        assert!(!shape(Direction::Ltr).contains(at_start));
        assert!(shape(Direction::Rtl).contains(at_start));
        assert!(shape(Direction::Ltr).contains(bounds.center()));

        for (from, to) in [
            (PolygonRole::Triangle, PolygonRole::Square),
            (PolygonRole::Square, PolygonRole::Circle),
            (PolygonRole::Circle, PolygonRole::Triangle),
        ] {
            let morph = PolygonMorph::new(from, to);
            for progress in [0., 0.5, 1.] {
                let sample = morph.sample(bounds, progress).unwrap();
                let cubics = &sample.outline.cubics;
                assert!(!cubics.is_empty());
                let extent = -TOLERANCE..=1. + TOLERANCE;
                for (cubic, next) in cubics.iter().zip(cubics.iter().cycle().skip(1)) {
                    assert!(
                        (cubic.anchor1() - next.anchor0()).length() <= TOLERANCE,
                        "{from:?} to {to:?} is open at {progress}",
                    );
                    for step in 0..=16 {
                        let point = cubic.point_on_curve(step as f32 / 16.);
                        assert!(
                            extent.contains(&point.x) && extent.contains(&point.y),
                            "{from:?} to {to:?} leaves the normalized bounds at {progress}: {point:?}",
                        );
                    }
                }
                assert!(sample.contains(bounds.center()));
                assert!(!sample.contains(at_start));
                let ripple = clipped_ripple(&sample.fill, bounds.center(), px(25.));
                assert!(!ripple.vertices.is_empty());
                assert!(ripple.vertices.iter().all(|vertex| {
                    let delta = vertex.xy_position - bounds.center();
                    f32::from(delta.x).hypot(f32::from(delta.y)) <= 25.001
                }));
            }
        }
    }
}
