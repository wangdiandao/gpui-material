use gpui::{
    AppContext, Bounds, Context, IntoElement, Path, PathBuilder, Pixels, Point, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, canvas, div, fill, point, prelude::*, px,
    size,
};
use gpui_base::{TestSupportExt, Theme};
use p1_host::open_probe;
use polymorpher::{CornerRounding, Cubic, Morph, RoundedPolygon};
use std::{cell::Cell, rc::Rc};

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
}

impl ShapesProbe {
    fn new() -> Self {
        Self {
            samples: [0., 0.5, 1.].map(curves),
            ripple: None,
        }
    }
}

fn contains_fill(path: &Path<Pixels>, position: Point<Pixels>) -> bool {
    fn cross(a: Point<Pixels>, b: Point<Pixels>) -> f32 {
        f32::from(a.x) * f32::from(b.y) - f32::from(a.y) * f32::from(b.x)
    }

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

fn pointer_origin(
    cubics: &[Cubic],
    center: Point<Pixels>,
    radius: Pixels,
    position: Point<Pixels>,
) -> Option<Point<Pixels>> {
    contains_fill(&path(cubics, center, radius), position).then_some(position)
}

impl Render for ShapesProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
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
                let ripple = self.ripple.filter(|(active, _)| *active == index);
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
                                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.ripple = pointer_origin(
                                        &hit_curves,
                                        center.get(),
                                        px(48.),
                                        event.position,
                                    )
                                    .map(|origin| (index, origin));
                                    cx.notify();
                                }),
                            )
                            .child(
                                canvas(
                                    move |bounds, _, _| {
                                        let center = bounds.center();
                                        painted_center.set(center);
                                        center
                                    },
                                    move |_, center, window, _| {
                                        window.paint_path(
                                            path(&paint_curves, center, px(48.)),
                                            colors.primary,
                                        );
                                        if let Some((_, origin)) = ripple {
                                            let mark =
                                                Bounds::centered_at(origin, size(px(3.), px(3.)));
                                            window
                                                .paint_quad(fill(mark, colors.primary_foreground));
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
fn gpui_tessellates_all_reference_progress_samples() {
    for progress in [0., 0.5, 1.] {
        let path = path(&curves(progress), point(px(64.), px(64.)), px(48.));
        assert!(path.bounds.size.width > px(40.));
        assert!(path.bounds.size.height > px(40.));
    }
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
