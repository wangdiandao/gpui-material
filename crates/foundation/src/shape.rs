use gpui::{Bounds, Path, PathBuilder, Pixels, Point, Window, point, px};
use gpui_material_theme::{CornerShape, Direction, PolygonRole, PolygonSpec};
use polymorpher::{CornerRounding, Cubic, Morph, RoundedPolygon};

const RIPPLE_SEGMENTS: usize = 64;

/// A prepared shape with a shared mesh for drawing, hit testing and effects.
pub struct Shape {
    outline: Outline,
    fill: Path<Pixels>,
}

struct Outline {
    cubics: Vec<Cubic>,
    bounds: Bounds<Pixels>,
    stretch: bool,
}

impl Outline {
    fn path(&self, mut builder: PathBuilder) -> Option<Path<Pixels>> {
        let size = self.bounds.size;
        let side = size.width.min(size.height);
        if !(side > px(0.) && side < px(f32::INFINITY)) {
            return None;
        }
        let center = self.bounds.center();
        let (width, height) = if self.stretch {
            (size.width, size.height)
        } else {
            (side, side)
        };
        let project = |p: polymorpher::geometry::Point| {
            center + point(width * (p.x - 0.5), height * (p.y - 0.5))
        };
        builder.move_to(project(self.cubics.first()?.anchor0()));
        for cubic in &self.cubics {
            builder.cubic_bezier_to(
                project(cubic.anchor1()),
                project(cubic.control0()),
                project(cubic.control1()),
            );
        }
        builder.close();
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
        if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
            return None;
        }
        let [tl, tr, br, bl] = corners.radii(width, height, direction);
        let side = width.max(height);
        let polygon = RoundedPolygon::rectangle()
            .with_width(width / side)
            .with_height(height / side)
            .with_rounding_per_vertex(
                [br, bl, tl, tr].map(|radius| CornerRounding::new(radius / side)),
            )
            .build();
        let cubics = polygon
            .transformed(|p: polymorpher::geometry::Point| {
                polymorpher::geometry::Point::new(
                    p.x * side / width + 0.5,
                    p.y * side / height + 0.5,
                )
            })
            .cubics;
        Self::new(Outline {
            cubics,
            bounds,
            stretch: true,
        })
    }

    fn new(outline: Outline) -> Option<Self> {
        let fill = outline.path(PathBuilder::fill())?;
        Some(Self { outline, fill })
    }

    pub fn bounds(&self) -> Bounds<Pixels> {
        self.outline.bounds
    }

    pub fn contains(&self, position: Point<Pixels>) -> bool {
        contains_fill(&self.fill, position)
    }

    pub fn paint(&self, color: gpui::Hsla, window: &mut Window) {
        window.paint_path(self.fill.clone(), color);
    }

    pub(crate) fn paint_ring(&self, width: Pixels, color: gpui::Hsla, window: &mut Window) {
        if let Some(path) = self.outline.path(PathBuilder::stroke(width)) {
            window.paint_path(path, color);
        }
    }

    pub(crate) fn paint_ripple(
        &self,
        origin: Point<Pixels>,
        radius: Pixels,
        color: gpui::Hsla,
        window: &mut Window,
    ) {
        let path = clipped_ripple(&self.fill, origin, radius);
        if !path.vertices.is_empty() {
            window.paint_path(path, color);
        }
    }
}

/// A reusable morph between two pinned Material polygon roles.
pub struct PolygonMorph(Morph);

impl PolygonMorph {
    pub fn new(start: PolygonRole, end: PolygonRole) -> Self {
        Self(Morph::new(polygon(start), polygon(end)))
    }

    /// Fits the sampled polygon inside the bounds while preserving aspect ratio.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        let progress = if progress.is_finite() {
            progress.clamp(0., 1.)
        } else {
            0.
        };
        Shape::new(Outline {
            cubics: self.0.as_cubics(progress),
            bounds,
            stretch: false,
        })
    }
}

fn polygon(role: PolygonRole) -> RoundedPolygon {
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
        } => {
            let (sin, cos) = rotation_degrees.to_radians().sin_cos();
            RoundedPolygon::from_vertices_count(
                usize::from(vertices),
                radius,
                Some(CornerRounding::smoothed(rounding_radius, smoothing)),
                &[],
            )
            .transformed(|p: polymorpher::geometry::Point| {
                polymorpher::geometry::Point::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
            })
        }
    }
    .normalized()
}

fn cross(a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    f32::from(a.x) * f32::from(b.y) - f32::from(a.y) * f32::from(b.x)
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

pub(crate) fn clipped_ripple(
    fill: &Path<Pixels>,
    origin: Point<Pixels>,
    radius: Pixels,
) -> Path<Pixels> {
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
            let Some(&last) = polygon.last() else {
                break;
            };
            let distance = |p| cross(end - start, p - start);
            let mut previous = last;
            let mut previous_distance = distance(previous);
            clipped.clear();
            for &current in &polygon {
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
            std::mem::swap(&mut polygon, &mut clipped);
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
    use gpui::size;
    use gpui_material_theme::CornerRole;

    const TOLERANCE: f32 = 0.000_01;

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
            let expected: Vec<_> = polygon(role)
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
