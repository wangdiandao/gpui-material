use polymorpher::{CornerRounding, Cubic, FeatureType, Morph, RoundedPolygon, geometry::Point};

const TOLERANCE: f32 = 0.000_01;

fn point_error(a: Point, b: Point) -> f32 {
    (a - b).length()
}

fn cubic_error(a: &Cubic, b: &Cubic) -> f32 {
    [
        point_error(a.anchor0(), b.anchor0()),
        point_error(a.control0(), b.control0()),
        point_error(a.control1(), b.control1()),
        point_error(a.anchor1(), b.anchor1()),
    ]
    .into_iter()
    .fold(0., f32::max)
}

fn check_loop(cubics: &[Cubic]) -> f32 {
    assert!(!cubics.is_empty());
    let mut max_gap: f32 = 0.;
    for (cubic, next) in cubics.iter().zip(cubics.iter().cycle().skip(1)) {
        let gap = point_error(cubic.anchor1(), next.anchor0());
        assert!(gap <= TOLERANCE);
        max_gap = max_gap.max(gap);
        for sample in 0..=16 {
            let point = cubic.point_on_curve(sample as f32 / 16.);
            assert!(point.x.is_finite() && point.y.is_finite());
            assert!((-1.001..=1.001).contains(&point.x));
            assert!((-1.001..=1.001).contains(&point.y));
        }
    }
    max_gap
}

#[test]
fn rounded_corner_parameters_and_space_match_androidx() {
    assert_eq!(CornerRounding::UNROUNDED.radius, 0.);
    assert_eq!(CornerRounding::UNROUNDED.smoothing, 0.);
    assert_eq!(CornerRounding::new(5.).smoothing, 0.);
    assert_eq!(CornerRounding::smoothed(5., 0.5).radius, 5.);
    assert_eq!(CornerRounding::smoothed(5., 0.5).smoothing, 0.5);

    let polygon = RoundedPolygon::from_vertices(
        &[Point::new(0., 0.), Point::new(1., 0.), Point::new(0.5, 1.)],
        CornerRounding::UNROUNDED,
        &[
            CornerRounding::new(1.),
            CornerRounding::smoothed(1., 1.),
            CornerRounding::UNROUNDED,
        ],
        Point::new(0.5, 1. / 3.),
    );
    let lower_edge = polygon
        .features
        .iter()
        .find(|feature| feature.ty == FeatureType::Edge)
        .unwrap();
    assert_eq!(lower_edge.cubics.len(), 1);
    let edge = &lower_edge.cubics[0];
    let midpoint = Point::new(0.5, 0.);
    let error = point_error(edge.anchor0(), midpoint).max(point_error(edge.anchor1(), midpoint));
    println!("rounding space anchor error={error}");
    assert!(error <= TOLERANCE);
}

#[test]
fn identity_morph_recovers_cubic_controls() {
    let triangle = RoundedPolygon::from_vertices_count(3, 1., None, &[]);
    let morph = Morph::new(triangle.clone(), triangle.clone());

    for progress in [0., 0.5, 1.] {
        let cubics = morph.as_cubics(progress);
        check_loop(&cubics);
        let error = cubics
            .iter()
            .map(|cubic| {
                triangle
                    .cubics
                    .iter()
                    .map(|original| cubic_error(cubic, original))
                    .fold(f32::INFINITY, f32::min)
            })
            .fold(0., f32::max);
        println!("identity at {progress}: control error={error}");
        assert!(error <= TOLERANCE);
    }
}

#[test]
fn rounded_polygon_morphs_stay_closed_and_finite() {
    let rounding = Some(CornerRounding::smoothed(0.2, 0.5));
    let triangle = RoundedPolygon::from_vertices_count(3, 1., rounding, &[]);
    let square = RoundedPolygon::from_vertices_count(4, 1., rounding, &[]);
    let circle = RoundedPolygon::from_vertices_count(8, 1., Some(CornerRounding::new(1.)), &[]);
    let square_plain = RoundedPolygon::from_vertices_count(4, 1., None, &[]);

    for (name, start, end) in [
        ("triangle-square", triangle, square.clone()),
        ("square-circle", square.clone(), circle),
        ("square-rounded-square", square_plain, square),
    ] {
        let morph = Morph::new(start, end);
        for progress in [0., 0.5, 1.] {
            let cubics = morph.as_cubics(progress);
            let gap = check_loop(&cubics);
            println!(
                "{name} at {progress}: {} cubics, max gap={gap}",
                cubics.len()
            );
        }
    }
}
