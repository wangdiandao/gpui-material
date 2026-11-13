use gpui::{
    AppContext, Context, IntoElement, Path, PathBuilder, Pixels, Point, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, black, canvas, div, point, prelude::*,
    px, size,
};
use gpui_base::{TestSupportExt, Theme};
use p1_host::open_probe;
use polymorpher::{CornerRounding, Cubic, Morph, RoundedPolygon};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

const RIPPLE_SEGMENTS: usize = 64;
const RIPPLE_DURATION_SECONDS: f32 = 0.32;

fn curves(progress: f32) -> Vec<Cubic> {
    let rounding = Some(CornerRounding::smoothed(0.2, 0.5));
    Morph::new(
        RoundedPolygon::from_vertices_count(3, 1., rounding, &[]),
        RoundedPolygon::from_vertices_count(4, 1., rounding, &[]),
    )
    .as_cubics(progress)
}

fn path(cubics: &[Cubic], center: Point<Pixels>, radius: Pixels) -> Path<Pixels> {
    let project = |p: polymorpher::geometry::Point| center + point(radius * p.x, radius * p.y);
    let mut builder = PathBuilder::fill();
    builder.move_to(project(cubics[0].anchor0()));
    for cubic in cubics {
        builder.cubic_bezier_to(
            project(cubic.anchor1()),
            project(cubic.control0()),
            project(cubic.control1()),
        );
    }
    builder.close();
    builder.build().expect("tessellate morph path")
}

struct ShapesProbe {
    samples: [Vec<Cubic>; 3],
    ripple: Option<(usize, Point<Pixels>)>,
    pressed_at: Instant,
    still_frame: bool,
}

impl ShapesProbe {
    fn new() -> Self {
        Self {
            samples: [0., 0.5, 1.].map(curves),
            ripple: None,
            pressed_at: Instant::now(),
            still_frame: std::env::var_os("P1_SHAPES_STILL").is_some(),
        }
    }
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

fn clipped_ripple(fill: &Path<Pixels>, origin: Point<Pixels>, radius: Pixels) -> Path<Pixels> {
    let circle: [_; RIPPLE_SEGMENTS] = std::array::from_fn(|index| {
        let angle = std::f32::consts::TAU * index as f32 / RIPPLE_SEGMENTS as f32;
        origin + point(radius * angle.cos(), radius * angle.sin())
    });
    let mut ripple = Path::new(origin);
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let mut clipped = Vec::with_capacity(RIPPLE_SEGMENTS + 3);

    // Intersect the disc with each disjoint fill triangle so paint and hit
    // testing use the same tessellated curve, including concave shapes.
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
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

fn pointer_origin(
    cubics: &[Cubic],
    center: Point<Pixels>,
    radius: Pixels,
    position: Point<Pixels>,
) -> Option<Point<Pixels>> {
    contains_fill(&path(cubics, center, radius), position).then_some(position)
}

impl Render for ShapesProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let elapsed = self.pressed_at.elapsed().as_secs_f32() / RIPPLE_DURATION_SECONDS;
        let progress = if self.still_frame || cx.reduce_motion() {
            0.45
        } else {
            elapsed.min(1.)
        };
        if self.ripple.is_some() && !self.still_frame && !cx.reduce_motion() && elapsed < 1. {
            window.request_animation_frame();
        }
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(colors.background)
            .children(self.samples.iter().enumerate().map(|(index, cubics)| {
                let label = ["0", "0.5", "1"][index];
                let paint_curves = cubics.clone();
                let hit_curves = cubics.clone();
                let ripple = self
                    .ripple
                    .filter(|(active, _)| *active == index && (self.still_frame || elapsed < 1.));
                let center = Rc::new(Cell::new(Point::default()));
                let painted_center = center.clone();
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .text_color(colors.foreground)
                    .child(
                        div()
                            .id(("shape", index))
                            .test_support()
                            .size(px(128.))
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(
                                    move |this, event: &gpui::MouseDownEvent, window, cx| {
                                        this.ripple = pointer_origin(
                                            &hit_curves,
                                            center.get(),
                                            px(48.),
                                            event.position,
                                        )
                                        .map(|origin| (index, origin));
                                        this.pressed_at = Instant::now();
                                        if this.ripple.is_some()
                                            && !this.still_frame
                                            && cx.reduce_motion()
                                        {
                                            let pressed_at = this.pressed_at;
                                            let timer = cx.background_executor().timer(
                                                Duration::from_secs_f32(RIPPLE_DURATION_SECONDS),
                                            );
                                            cx.spawn_in(window, async move |view, cx| {
                                                timer.await;
                                                let _ = view.update(cx, |this, cx| {
                                                    if this.pressed_at == pressed_at {
                                                        this.ripple = None;
                                                        cx.notify();
                                                    }
                                                });
                                            })
                                            .detach();
                                        }
                                        cx.notify();
                                    },
                                ),
                            )
                            .child(
                                canvas(
                                    move |bounds, _, _| {
                                        let center = bounds.center();
                                        painted_center.set(center);
                                        center
                                    },
                                    move |_, center, window, _| {
                                        let fill = path(&paint_curves, center, px(48.));
                                        for (offset, spread, alpha) in
                                            [(4., 3., 0.05), (3., 2., 0.07), (2., 1., 0.10)]
                                        {
                                            window.paint_path(
                                                path(
                                                    &paint_curves,
                                                    center + point(px(0.), px(offset)),
                                                    px(48. + spread),
                                                ),
                                                black().opacity(alpha),
                                            );
                                        }
                                        if let Some((_, origin)) = ripple {
                                            let disc = clipped_ripple(
                                                &fill,
                                                origin,
                                                px(2. + 94. * progress),
                                            );
                                            window.paint_path(fill, colors.primary);
                                            window.paint_path(
                                                disc,
                                                colors
                                                    .primary_foreground
                                                    .opacity(0.54 * (1. - progress)),
                                            );
                                        } else {
                                            window.paint_path(fill, colors.primary);
                                        }
                                    },
                                )
                                .size_full(),
                            ),
                    )
                    .child(label)
            }))
    }
}

fn main() {
    #[cfg(all(
        feature = "test-support",
        any(target_os = "macos", target_os = "linux")
    ))]
    if let Some(output) = std::env::var_os("P1_SHAPES_SCREENSHOT") {
        native_renderer_draws_morph_samples(&output);
        return;
    }

    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(480.), px(280.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-05 形状探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| cx.new(|_| ShapesProbe::new())).unwrap();
    });
}

#[test]
fn morph_hit_uses_fill_and_preserves_the_pointer_origin() {
    let center = point(px(96.), px(64.));
    let radius = px(48.);
    for progress in [0., 0.5, 1.] {
        let cubics = curves(progress);
        let fill = path(&cubics, center, radius);
        let inside = center + point(px(4.), px(2.));
        let empty_corner = fill.bounds.origin + point(px(1.), px(1.));
        assert!(fill.bounds.contains(&empty_corner));
        assert_eq!(
            pointer_origin(&cubics, center, radius, inside),
            Some(inside)
        );
        assert_eq!(pointer_origin(&cubics, center, radius, empty_corner), None);
    }
}

#[test]
fn ripple_covers_the_origin_and_clips_against_all_three_fills() {
    let center = point(px(96.), px(64.));
    let origin = center + point(px(4.), px(2.));
    for progress in [0., 0.5, 1.] {
        let shape = path(&curves(progress), center, px(48.));
        let start = clipped_ripple(&shape, origin, px(2.));
        assert!(contains_fill(&start, origin));
        assert!(!contains_fill(&start, origin + point(px(3.), px(0.))));
        let expanded = clipped_ripple(&shape, origin, px(96.));
        let empty_corner = shape.bounds.origin + point(px(1.), px(1.));
        assert!(contains_fill(&expanded, origin));
        assert!(!contains_fill(&expanded, empty_corner));
        let width_error = (expanded.bounds.size.width - shape.bounds.size.width).abs();
        let height_error = (expanded.bounds.size.height - shape.bounds.size.height).abs();
        assert!(
            width_error < px(0.001) && height_error < px(0.001),
            "progress {progress}: {width_error:?} × {height_error:?}, {:?} vs {:?}",
            expanded.bounds,
            shape.bounds,
        );
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn rendered_shape_accepts_the_press_at_fractional_scale(cx: &mut gpui::TestAppContext) {
    use gpui::{InputEvent, MouseButton, MouseDownEvent};
    use gpui_base::test_support;

    let (window, probe) = cx.update(|cx| {
        gpui_base::init(cx);
        let probe = cx.new(|_| ShapesProbe::new());
        let window = open_probe(cx, WindowOptions::default(), |_, _| probe.clone()).unwrap();
        (window, probe)
    });

    cx.simulate_window_resize(window.into(), size(px(480.), px(280.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        let shape = test_support::find(window, &[], &("shape", 1_usize).into()).unwrap();
        let bounds = shape.bounds();
        let inside = bounds.center() + point(px(3.), px(4.));
        let outside = bounds.origin + point(px(2.), px(2.));
        assert_eq!(window.scale_factor(), 1.5);
        for (position, expected) in [(inside, Some((1, inside))), (outside, None)] {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    modifiers: window.modifiers(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(probe.read(cx).ripple, expected);
        }
    })
    .unwrap();
}

#[cfg(all(
    feature = "test-support",
    any(target_os = "macos", target_os = "linux")
))]
fn native_renderer_draws_morph_samples(output: &std::ffi::OsStr) {
    use gpui::HeadlessAppContext;
    use std::sync::Arc;

    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(()),
        gpui_platform::current_headless_renderer,
    );
    cx.update(gpui_base::init);
    let handle = cx
        .open_window(size(px(480.), px(280.)), |window, cx| {
            let view = cx.new(|_| ShapesProbe::new());
            cx.new(|cx| gpui_base::Root::new(view, window, cx))
        })
        .expect("open GPU-backed test window");

    let scale = cx
        .update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.scale_factor()
        })
        .unwrap();
    let image = cx
        .capture_screenshot(handle.into())
        .expect("capture native GPU pixels");
    let device_pixel = |logical: u32| (logical as f32 * scale).round() as u32;
    assert_eq!(image.dimensions(), (device_pixel(480), device_pixel(280)));

    let background = image.get_pixel(0, 0);
    for x in [96, 240, 384].map(device_pixel) {
        assert_ne!(
            image.get_pixel(x, device_pixel(128)),
            background,
            "fill at x={x}"
        );
        assert_eq!(
            image.get_pixel(x, device_pixel(52)),
            background,
            "background above x={x}"
        );
    }
    image.save(output).expect("save native GPU screenshot");
    println!(
        "{} native renderer: {:?}, scale={scale}, three interior and three exterior samples",
        std::env::consts::OS,
        image.dimensions()
    );
}
