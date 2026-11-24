use std::time::Duration;

use gpui::{Bounds, point, px, size};

use super::*;

#[test]
fn loading_shapes_fit_the_active_size_through_rotation_and_morphs() {
    let shape_sets = [&*INDETERMINATE_SHAPES, &*DETERMINATE_SHAPES];
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
    for shapes in shape_sets {
        let scale = shapes.rotation_scale();
        assert!(scale > 0. && scale <= 1.);
        for step in 0..shapes.step_count() {
            for tick in 0..=20 {
                let progress = tick as f32 / 20.;
                let sampled = shapes
                    .sample_step_in_envelope(step, progress, bounds, px(ACTIVE_SIZE), 0.)
                    .unwrap();
                assert!(sampled.contains(bounds.center()));
                let outer_radius = ACTIVE_SIZE * 0.5 + 0.1;
                for sample in 0..36 {
                    let angle = std::f32::consts::TAU * sample as f32 / 36.;
                    let outside = bounds.center()
                        + point(
                            px(outer_radius * angle.cos()),
                            px(outer_radius * angle.sin()),
                        );
                    assert!(!sampled.contains(outside));
                }
            }
        }
    }
}

#[test]
fn determinate_progress_handles_numeric_boundaries() {
    for (input, expected) in [
        (f32::NAN, 0.),
        (f32::NEG_INFINITY, 0.),
        (-0.1, 0.),
        (0.5, 0.5),
        (1.1, 1.),
        (f32::INFINITY, 1.),
    ] {
        assert_eq!(
            LoadingIndicator::new("loading", "Loading")
                .value(input)
                .value,
            Some(expected)
        );
    }
    let shapes = &*DETERMINATE_SHAPES;
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
    let start = determinate_sample(shapes, 0.);
    let end = determinate_sample(shapes, 1.);
    assert_eq!((start.progress, start.angle), (0., 0.));
    assert_eq!((end.progress, end.angle), (1., -TAU * 0.5));
    let start_shape = shapes
        .sample_step_in_envelope(
            start.step,
            start.progress,
            bounds,
            px(ACTIVE_SIZE),
            start.angle,
        )
        .unwrap();
    let end_shape = shapes
        .sample_step_in_envelope(end.step, end.progress, bounds, px(ACTIVE_SIZE), end.angle)
        .unwrap();
    assert_ne!(start_shape.fill_bounds(), end_shape.fill_bounds());
}

#[test]
fn a_custom_sequence_morphs_between_endpoints_and_wraps_only_when_indeterminate() {
    let rectangle = |w, h| {
        LoadingIndicatorShape::rounded_polygon(
            &[point(0., 0.), point(w, 0.), point(w, h), point(0., h)],
            0.,
        )
        .unwrap()
    };
    let custom = LoadingIndicatorShapes::new([
        rectangle(2., 1.),
        LoadingIndicatorShape::circle(),
        rectangle(1., 2.),
    ])
    .unwrap();
    let shapes = &custom.0;
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
    let frame = |value| {
        let sample = determinate_sample(shapes, value);
        shapes
            .sample_step_in_envelope(
                sample.step,
                sample.progress,
                bounds,
                px(ACTIVE_SIZE),
                sample.angle,
            )
            .unwrap()
            .fill_bounds()
            .size
    };
    let start = frame(0.);
    let middle = frame(0.5);
    let end = frame(1.);
    assert!((start.width / start.height - 2.).abs() < 0.05);
    assert!((middle.width / middle.height - 1.).abs() < 0.05);
    assert!((end.height / end.width - 2.).abs() < 0.05);
    for value in [0.4999, 0.5001] {
        let adjacent = frame(value);
        assert!((adjacent.width - middle.width).abs() < px(0.05));
        assert!((adjacent.height - middle.height).abs() < px(0.05));
    }
    let first = indeterminate_sample(shapes, 0.);
    let wrap = indeterminate_sample(shapes, 2.);
    let repeated = indeterminate_sample(shapes, 3.);
    assert_eq!(first.step, 0);
    assert_eq!(wrap.step, 2);
    assert_eq!(repeated.step, 0);
    assert_eq!(
        (first.progress, wrap.progress, repeated.progress),
        (0., 0., 0.)
    );
}

#[test]
fn custom_concave_paths_align_winding_and_fit_the_active_envelope() {
    let mut vertices = [
        point(-1., -1.),
        point(1., -1.),
        point(0., 0.),
        point(1., 1.),
        point(-1., 1.),
    ];
    let polygon = LoadingIndicatorShape::rounded_polygon(&vertices, 0.1).unwrap();
    vertices.reverse();
    let reversed = LoadingIndicatorShape::rounded_polygon(&vertices, 0.1).unwrap();
    assert_eq!(polygon, reversed);
    let circle = LoadingIndicatorShape::circle();
    let limit = polygon.0.cubic_count() + circle.0.cubic_count();
    let sequence = LoadingIndicatorShapes::new([polygon, circle]).unwrap();
    let shapes = &sequence.0;
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
    for step in 0..=20 {
        let sample = determinate_sample(shapes, step as f32 / 20.);
        let outline = shapes
            .sample_step_polygon(sample.step, sample.progress)
            .unwrap();
        assert!(outline.cubic_count() < limit);
        let shape = shapes
            .sample_step_in_envelope(
                sample.step,
                sample.progress,
                bounds,
                px(ACTIVE_SIZE),
                sample.angle,
            )
            .unwrap();
        let outer_radius = ACTIVE_SIZE * 0.5 + 0.1;
        for i in 0..36 {
            let angle = TAU * i as f32 / 36.;
            let outside = bounds.center()
                + point(
                    px(outer_radius * angle.cos()),
                    px(outer_radius * angle.sin()),
                );
            assert!(!shape.contains(outside));
        }
    }
}

#[test]
fn expressive_material_shapes_work_in_loading_indicator_sequences() {
    let sequence = LoadingIndicatorShapes::new([
        LoadingIndicatorShape::material(MaterialShape::Heart),
        LoadingIndicatorShape::from(MaterialShape::Cookie12Sided),
        LoadingIndicatorShape::from(ShapePolygon::material(MaterialShape::Clover4Leaf)),
    ])
    .unwrap();
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
    let sample = determinate_sample(&sequence.0, 0.5);
    let shape = sequence
        .0
        .sample_step_in_envelope(
            sample.step,
            sample.progress,
            bounds,
            px(ACTIVE_SIZE),
            sample.angle,
        )
        .unwrap();
    assert!(sample.path(bounds, ACTIVE_SIZE, &sequence.0).is_some());
    assert!(shape.contains(bounds.center()));
}

#[test]
fn rejects_short_and_degenerate_shape_sequences() {
    assert!(LoadingIndicatorShapes::new([]).is_none());
    assert!(LoadingIndicatorShapes::new([LoadingIndicatorShape::circle()]).is_none());
    for outline in [
        &[point(0., 0.), point(0., 0.), point(0., 0.)][..],
        &[point(0., 0.), point(1., 1.), point(2., 2.)],
        &[point(0., 0.), point(1., 1.), point(1., 0.), point(0., 1.)],
        &[point(0., 0.), point(f32::INFINITY, 0.), point(2., 2.)],
    ] {
        assert!(LoadingIndicatorShape::rounded_polygon(outline, 0.).is_none());
    }
    for radius in [f32::NAN, -1.] {
        assert!(
            LoadingIndicatorShape::rounded_polygon(
                &[point(0., 0.), point(1., 0.), point(1., 1.)],
                radius,
            )
            .is_none()
        );
    }
}

#[test]
fn indeterminate_phase_pauses_and_reduced_motion_holds_the_start() {
    let mut motion = IndeterminateMotion::default();
    let start = Instant::now();
    let after = |millis| start + Duration::from_millis(millis);
    assert_eq!(motion.phase(start, true, false), 0.);
    let paused = motion.phase(after(200), false, false);
    assert!(paused > 0.);
    assert_eq!(motion.phase(after(400), false, false), paused);
    assert_eq!(motion.phase(after(550), true, false), paused);
    assert!(motion.phase(after(700), true, false) > paused);
    assert_eq!(motion.phase(after(800), false, true), 0.);
    assert_eq!(motion.phase(after(950), false, true), 0.);
    assert_eq!(motion.phase(after(1050), true, false), 0.);
}
