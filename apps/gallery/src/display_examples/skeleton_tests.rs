use gpui::{
    AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render, Role, Styled,
    TestAppContext, Window, div, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, Skeleton, SkeletonColors,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, CornerRole, MaterialShape, MaterialTheme, TypeRole},
};

#[gpui::test]
fn skeleton_presets_respect_a11y_reduced_motion_and_offscreen_zero_frames(cx: &mut TestAppContext) {
    const CUSTOM_BASE: u32 = 0x334455;
    const CUSTOM_HIGHLIGHT: u32 = 0x778899;

    struct Example {
        animated: bool,
        offscreen_only: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            if self.offscreen_only {
                // Viewport clips height to 60 px while the skeleton starts at y = 400 px.
                return div()
                    .id("clipped-host")
                    .w(px(300.))
                    .h(px(60.))
                    .overflow_hidden()
                    .child(div().pt(px(400.)).child(Skeleton::rect(
                        "offscreen-skeleton",
                        px(180.),
                        px(24.),
                    )))
                    .into_any_element();
            }

            div()
                .id("skeleton-root")
                .tab_group()
                .w(px(400.))
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .child(Button::new("before-skel", "Before").on_activate(|_, _| {}))
                .child(
                    Skeleton::rect("rect-skel", px(120.), px(40.))
                        .corner_role(CornerRole::Medium)
                        .colors(SkeletonColors {
                            base: Some(rgb(CUSTOM_BASE).into()),
                            highlight: Some(rgb(CUSTOM_HIGHLIGHT).into()),
                        })
                        .animated(self.animated),
                )
                .child(
                    Skeleton::text_line("line-skel")
                        .role(TypeRole::TitleMedium)
                        .animated(self.animated),
                )
                .child(Skeleton::circle("circle-skel", px(48.)).animated(self.animated))
                .child(
                    Skeleton::shape("cookie-skel", px(56.), px(56.), MaterialShape::Cookie7Sided)
                        .accessible_label("Loading badge")
                        .animated(self.animated),
                )
                .child(Button::new("after-skel", "After").on_activate(|_, _| {}))
                .into_any_element()
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        let view = cx.new(|_| Example {
            animated: true,
            offscreen_only: false,
        });
        let handle =
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| view.clone())
                .unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(handle, size(px(480.), px(520.)));

    let draw_and_count_frames = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx)
        })
        .unwrap()
    };

    cx.update_window(handle, |_, window, cx| {
        set_window_material_theme(Some(MaterialTheme::baseline(Appearance::Light)), window, cx);
        window.activate_window();
    })
    .unwrap();
    cx.run_until_parked();

    // Active visible animated skeletons schedule frames.
    assert!(
        draw_and_count_frames(cx) >= 4,
        "Active visible animated skeletons should schedule animation frames"
    );

    // Geometry and accessibility semantics checks.
    cx.update_window(handle, |_, window, _| {
        let rect = find(window, &[], &"rect-skel".into()).unwrap();
        assert_eq!(rect.bounds().size, size(px(120.), px(40.)));
        assert_eq!(rect.role(), None);

        let line = find(window, &[], &"line-skel".into()).unwrap();
        assert!(line.bounds().size.width > px(300.));
        assert_eq!(line.bounds().size.height, px(24.)); // TitleMedium line height

        let circle = find(window, &[], &"circle-skel".into()).unwrap();
        assert_eq!(circle.bounds().size, size(px(48.), px(48.)));

        let cookie = find(window, &[], &"cookie-skel".into()).unwrap();
        assert_eq!(cookie.bounds().size, size(px(56.), px(56.)));
        assert_eq!(cookie.role(), Some(Role::Status));
        assert_eq!(cookie.label(), Some("Loading badge"));
    })
    .unwrap();

    // Skeletons never add Tab stops between neighboring buttons.
    cx.update_window(handle, |_, window, cx| {
        window.focus_next(cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, _| {
        assert_eq!(
            find(window, &[], &"before-skel".into()).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.focus_next(cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, _| {
        assert_eq!(
            find(window, &[], &"after-skel".into()).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();

    // Reduced motion renders static base color and schedules ZERO frames.
    cx.update(|cx| cx.set_reduce_motion(true));
    assert_eq!(
        draw_and_count_frames(cx),
        0,
        "Reduced motion must schedule zero animation frames"
    );
    cx.update_window(handle, |_, window, _| {
        let theme = MaterialTheme::baseline(Appearance::Light);
        let default_base = color(theme.colors.color(ColorRole::SurfaceContainerHigh));
        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|q| q.background.as_solid() == Some(rgb(CUSTOM_BASE).into())),
            "Reduced motion must paint static custom base color"
        );
        assert!(
            quads
                .iter()
                .any(|q| q.background.as_solid() == Some(default_base)),
            "Reduced motion must paint static SurfaceContainerHigh base color"
        );
    })
    .unwrap();

    // Explicit animated(false) schedules ZERO frames even with motion enabled.
    cx.update(|cx| {
        cx.set_reduce_motion(false);
        view.update(cx, |this, cx| {
            this.animated = false;
            cx.notify();
        });
    });
    assert_eq!(
        draw_and_count_frames(cx),
        0,
        "Static animated(false) skeletons must schedule zero animation frames"
    );

    // Clipped/offscreen animated skeleton schedules ZERO frames.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.animated = true;
            this.offscreen_only = true;
            cx.notify();
        });
    });
    assert_eq!(
        draw_and_count_frames(cx),
        0,
        "Offscreen/clipped skeletons must schedule zero animation frames"
    );
}
