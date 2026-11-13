use gpui::{
    AppContext, Context, IntoElement, Path, PathBuilder, Pixels, Point, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, canvas, div, point, prelude::*, px, size,
};
use gpui_base::Theme;
use p1_host::open_probe;
use polymorpher::{CornerRounding, Cubic, Morph, RoundedPolygon};

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

struct ShapesProbe([Vec<Cubic>; 3]);

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
            .children(self.0.iter().zip(["0", "0.5", "1"]).map(|(cubics, label)| {
                let cubics = cubics.clone();
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .text_color(colors.foreground)
                    .child(
                        canvas(
                            |bounds, _, _| bounds.center(),
                            move |_, center, window, _| {
                                window.paint_path(path(&cubics, center, px(48.)), colors.primary);
                            },
                        )
                        .size(px(128.)),
                    )
                    .child(label)
            }))
    }
}

fn main() {
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
        open_probe(cx, options, |_, cx| {
            cx.new(|_| ShapesProbe([0., 0.5, 1.].map(curves)))
        })
        .unwrap();
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
