use gpui::{
    AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render, Role, Styled,
    TestAppContext, Window, div, point, px, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    LoadingIndicator, LoadingIndicatorShape, LoadingIndicatorShapes, MaterialRoot,
    foundation::{set_window_material_theme, window_material_theme},
    theme::{CornerShape, Direction},
};

#[gpui::test]
fn facade_loading_indicators_report_progress_and_stop_idle_frames(cx: &mut TestAppContext) {
    struct Example {
        active: bool,
        value: Option<f32>,
        shapes: LoadingIndicatorShapes,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let default = LoadingIndicator::new("default-loading", "Load documents");
            let contained = LoadingIndicator::new("contained-loading", "Load archive")
                .contained(true)
                .container_color(gpui::rgb(0x334455))
                .container_shape(CornerShape::Radii([3., 7., 11., 13.]))
                .indicator_color(gpui::white())
                .shapes(&self.shapes);
            let indicators = [default, contained].map(|indicator| {
                let indicator = indicator.active(self.active);
                match self.value {
                    Some(value) => indicator.value(value),
                    None => indicator,
                }
            });
            div().flex().items_start().gap_2().children(
                indicators
                    .into_iter()
                    .enumerate()
                    .map(|(i, indicator)| div().id(("slot", i)).test_support().child(indicator)),
            )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example {
            active: true,
            value: None,
            shapes: LoadingIndicatorShapes::new([
                LoadingIndicatorShape::circle(),
                LoadingIndicatorShape::rounded_polygon(
                    &[point(0., -1.), point(1., 1.), point(-1., 1.)],
                    0.2,
                )
                .unwrap(),
            ])
            .unwrap(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx)
        })
        .unwrap()
    };
    let assert_active = |cx: &mut TestAppContext, top_left| {
        cx.update_window(handle, |_, window, _| {
            for (id, label) in [
                ("default-loading", "Load documents"),
                ("contained-loading", "Load archive"),
            ] {
                let node = find(window, &[], &id.into()).unwrap();
                assert_eq!(node.role(), Some(Role::ProgressIndicator));
                assert_eq!(node.label(), Some(label));
                assert_eq!(node.bounds().size, size(px(48.), px(48.)));
            }
            let container = window
                .painted_quads()
                .into_iter()
                .find(|q| q.background.as_solid() == Some(gpui::rgb(0x334455).into()))
                .unwrap();
            assert_eq!(
                container.corner_radii.top_left.0 / window.scale_factor(),
                top_left
            );
        })
        .unwrap();
    };
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    assert!(
        draw(cx) >= 2,
        "both active indeterminate indicators request frames"
    );
    assert_active(cx, 3.);

    cx.update(|cx| cx.set_reduce_motion(true));
    assert_eq!(draw(cx), 0);
    assert_active(cx, 3.);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    assert_eq!(draw(cx), 0, "reduced motion stays idle");

    cx.update(|cx| cx.set_reduce_motion(false));
    cx.update_window(handle, |_, window, cx| {
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    for value in [0., 0.5, 1.] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.value = Some(value);
                cx.notify();
            })
        });
        assert_eq!(draw(cx), 0, "determinate mode has no idle timeline");
        assert_active(cx, 7.);
    }

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.value = None;
            view.active = false;
            cx.notify();
        })
    });
    assert_eq!(draw(cx), 0, "inactive indicators leave the render loop");
    cx.update_window(handle, |_, window, _| {
        assert!(find(window, &[], &"default-loading".into()).is_none());
        assert!(find(window, &[], &"contained-loading".into()).is_none());
        for i in 0_usize..2 {
            let slot = find(window, &[], &("slot", i).into()).unwrap();
            assert_eq!(slot.bounds().size, size(px(48.), px(48.)));
        }
        assert!(
            !window
                .painted_quads()
                .into_iter()
                .any(|q| { q.background.as_solid() == Some(gpui::rgb(0x334455).into()) })
        );
    })
    .unwrap();
}
