use gpui_material_theme::{CornerRole, PolygonRole};

use super::polygon::role_polygon;
use super::*;

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
        assert_eq!(
            prepared.stroke_path(px(2.)).unwrap().vertices.len(),
            prepared.stroke_path(px(2.)).unwrap().vertices.len()
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
        ShapePolygon::rounded_rect(160., 40., CornerShape::Radii([8.; 4]), Direction::Ltr).unwrap();
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
    assert!(circle.intersects_bounds(Bounds::new(point(px(40.), px(40.)), size(px(20.), px(20.)))));
    assert!(!circle.intersects_bounds(Bounds::new(point(px(0.), px(0.)), size(px(8.), px(8.)))));
    assert!(burst.intersects_bounds(Bounds::new(point(px(45.), px(45.)), size(px(10.), px(10.)))));
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
