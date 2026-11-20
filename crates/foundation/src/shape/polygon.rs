use gpui::{Bounds, Pixels, Point};
use gpui_material_theme::{CornerShape, Direction, MaterialShape, PolygonRole, PolygonSpec};
use polymorpher::{CornerRounding, Cubic, RoundedPolygon, geometry::Point as PolygonPoint, shapes};

use super::{Shape, ShapeFit};

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
        let (sin, cos) = degrees.to_radians().sin_cos();
        match &self.kind {
            PolygonKind::Rounded(polygon) => {
                let center = polygon.center;
                Self::from_rounded(
                    polygon
                        .clone()
                        .transformed(|p: PolygonPoint| rotate_point_around(p, center, sin, cos))
                        .normalized(),
                )
            }
            PolygonKind::Cubics(cubics) => {
                let center = cubic_center(cubics);
                let rotate = |p: PolygonPoint| rotate_point_around(p, center, sin, cos);
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
                Self::from_cubics(normalize_cubics(rotated))
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

    pub(super) fn from_cubics(cubics: Vec<Cubic>) -> Self {
        Self {
            kind: PolygonKind::Cubics(cubics),
        }
    }

    pub(super) fn as_rounded(&self) -> Option<&RoundedPolygon> {
        match &self.kind {
            PolygonKind::Rounded(polygon) => Some(polygon),
            PolygonKind::Cubics(_) => None,
        }
    }

    pub(super) fn cubics(&self) -> &[Cubic] {
        match &self.kind {
            PolygonKind::Rounded(polygon) => &polygon.cubics,
            PolygonKind::Cubics(cubics) => cubics,
        }
    }

    pub(super) fn control_point_radius(&self) -> f32 {
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

fn valid_rounding(radius: f32, smoothing: f32) -> bool {
    radius.is_finite() && radius >= 0. && smoothing.is_finite() && (0.0..=1.0).contains(&smoothing)
}

fn rotate_point_around(
    point: PolygonPoint,
    center: PolygonPoint,
    sin: f32,
    cos: f32,
) -> PolygonPoint {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    PolygonPoint::new(
        center.x + dx * cos - dy * sin,
        center.y + dx * sin + dy * cos,
    )
}

fn rotate_polygon(polygon: RoundedPolygon, degrees: f32) -> RoundedPolygon {
    let (sin, cos) = degrees.to_radians().sin_cos();
    polygon.transformed(|p: PolygonPoint| {
        PolygonPoint::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
    })
}

pub(super) fn rounded_rect_polygon(
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

pub(super) fn role_polygon(role: PolygonRole) -> RoundedPolygon {
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

pub(super) fn cubic_center(cubics: &[Cubic]) -> PolygonPoint {
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
