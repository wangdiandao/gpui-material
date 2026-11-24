use std::time::Duration;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Hsla, InputEvent, InteractiveElement,
    IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, Role, Styled, TestAppContext, Window, canvas, div, px,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Button, ButtonColors, ButtonElevations, Fab, FabAlignment, FabSize, FabVisibilityAnimation,
    Icon, IconName, InteractionSource, MaterialRoot,
    foundation::{color, set_window_material_theme, window_material_theme},
    theme::{ColorRole, CornerShape, CubicBezier, Direction, Elevation, MotionSpec},
};

pub(super) fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

pub(super) fn pointer(position: Point<Pixels>, window: &mut Window, cx: &mut App) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(
        MouseUpEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
}

pub(super) fn space(window: &mut Window, cx: &mut App) {
    let keystroke = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
}

const MEDIUM: Hsla = gpui::hsla(0.23, 0.81, 0.29, 1.);
const LARGE: Hsla = gpui::hsla(0.82, 0.61, 0.48, 1.);
const INK: Hsla = gpui::hsla(0.44, 0.85, 0.27, 1.);
const DISABLED: Hsla = gpui::hsla(0.03, 0.29, 0.57, 1.);

#[gpui::test]
fn facade_fab_keeps_the_default_size_and_paints_independent_appearance(cx: &mut TestAppContext) {
    struct Example {
        source: gpui::Entity<InteractionSource>,
        large_disabled: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let swatch = canvas(
                |_, window, _| window.text_style().color,
                |bounds, tint, window, _| window.paint_quad(gpui::fill(bounds, tint)),
            )
            .size(px(12.));
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(Fab::new("default", "New item", Icon::new(IconName::Add)))
                .child(
                    Fab::new("medium", "Attach item", swatch)
                        .size(FabSize::Medium)
                        .shape(CornerShape::Radii([3., 7., 11., 13.]))
                        .colors(ButtonColors {
                            container: Some(MEDIUM),
                            content: Some(INK),
                            ..Default::default()
                        })
                        .elevations(ButtonElevations {
                            resting: Some(Elevation::Level0),
                            pressed: Some(Elevation::Level5),
                            ..Default::default()
                        })
                        .interaction_source(&self.source)
                        .on_activate(|_, _| {}),
                )
                .child(
                    Fab::new("large", "Edit item", Icon::new(IconName::Add))
                        .size(FabSize::Large)
                        .shape(CornerShape::Radii([8.; 4]))
                        .colors(ButtonColors {
                            container: Some(LARGE),
                            disabled_container: Some(DISABLED),
                            ..Default::default()
                        })
                        .elevations(ButtonElevations {
                            resting: Some(Elevation::Level4),
                            disabled: Some(Elevation::Level1),
                            ..Default::default()
                        })
                        .interaction_source(&self.source)
                        .disabled(self.large_disabled),
                )
        }
    }

    let (handle, view, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|_| Example {
            source: source.clone(),
            large_disabled: false,
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, source)
    });
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        let node = |id| find(window, &[], &gpui::ElementId::Name(id)).unwrap();
        for (id, width, label) in [
            ("default", 56., "New item"),
            ("medium", 80., "Attach item"),
            ("large", 96., "Edit item"),
        ] {
            let button = node(id.into());
            assert_eq!(button.bounds().size, gpui::size(px(width), px(width)));
            assert_eq!(button.label(), Some(label));
            assert_eq!(button.role(), Some(Role::Button));
        }
        let scale = window.scale_factor();
        let radius = |tint| {
            crate::painted_quads(window)
                .iter()
                .find(|quad| quad.background.as_solid() == Some(tint))
                .unwrap()
                .corner_radii
                .top_left
                .0
                / scale
        };
        let primary_container = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::PrimaryContainer),
        );
        assert_eq!(radius(primary_container), 16.);
        assert_eq!(radius(MEDIUM), 3.);
        assert_eq!(radius(LARGE), 8.);
        assert_eq!(radius(INK), 0.);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        let id = window.window_handle().window_id();
        assert!(source.read(cx).state(id, &"medium".into()).is_some());
        assert!(source.read(cx).state(id, &"large".into()).is_some());
        view.update(cx, |view, cx| {
            view.large_disabled = true;
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"large".into()).unwrap().disabled(),
            Some(true)
        );
        assert!(
            crate::painted_quads(window)
                .iter()
                .any(|quad| quad.background.as_solid() == Some(DISABLED))
        );
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        let id = window.window_handle().window_id();
        assert!(
            !source
                .read(cx)
                .state(id, &"medium".into())
                .unwrap()
                .disabled
        );
        assert!(source.read(cx).state(id, &"large".into()).unwrap().disabled);
    })
    .unwrap();
}

#[gpui::test]
fn facade_fab_retargets_visibility_and_releases_focus_when_hiding(cx: &mut TestAppContext) {
    struct Example {
        visible: bool,
        target_scale: f32,
        activations: usize,
        focus: FocusHandle,
        source: gpui::Entity<InteractionSource>,
    }

    fn tween(duration_ms: u16) -> MotionSpec {
        MotionSpec::Tween {
            duration_ms,
            easing: CubicBezier {
                x1: 0.,
                y1: 0.,
                x2: 1.,
                y2: 1.,
            },
        }
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(
                    div().id("fab-slot").test_support().child(
                        Fab::new("animated-fab", "New item", Icon::new(IconName::Add))
                            .track_focus(&self.focus)
                            .interaction_source(&self.source)
                            .colors(ButtonColors {
                                container: Some(MEDIUM),
                                ..Default::default()
                            })
                            .visible(self.visible)
                            .visibility_animation(FabVisibilityAnimation {
                                alignment: FabAlignment::BottomEnd,
                                target_scale: self.target_scale,
                                scale_motion: Some(tween(200)),
                                alpha_motion: Some(tween(400)),
                            })
                            .on_activate(move |_, cx| {
                                let _ = owner.update(cx, |view, _| view.activations += 1);
                            }),
                    ),
                )
                .child(Button::new("next-control", "Next action"))
        }
    }

    let (handle, view, focus, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        let focus = cx.focus_handle();
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|_| Example {
            visible: false,
            target_scale: 0.25,
            activations: 0,
            focus: focus.clone(),
            source: source.clone(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, focus, source)
    });
    let set_visible = |visible, cx: &mut TestAppContext| {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.visible = visible;
                cx.notify();
            });
        });
    };
    let paint = |window: &Window| {
        let scale = window.scale_factor();
        crate::painted_quads(window).iter().find_map(|quad| {
            let tint = quad.background.as_solid()?;
            ((tint.h, tint.s, tint.l) == (MEDIUM.h, MEDIUM.s, MEDIUM.l))
                .then(|| (quad.bounds.map(|pixels| px(pixels.0 / scale)), tint.a))
        })
    };
    let footprint = |window: &Window| find(window, &[], &"fab-slot".into()).unwrap().bounds();
    let sample = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            draw(window, cx);
            (footprint(window), paint(window))
        })
        .unwrap()
    };
    let assert_hidden = |footprint: Bounds<Pixels>, paint| {
        assert_eq!(footprint.size.width, px(0.));
        assert_eq!(paint, None);
    };
    let (initial, initial_paint) = sample(cx);
    assert_hidden(initial, initial_paint);

    set_visible(true, cx);
    sample(cx);
    cx.executor().advance_clock(Duration::from_millis(500));
    let (open, open_paint) = sample(cx);
    assert_eq!(open.size.width, px(56.));
    assert_eq!(open_paint, Some((open, 1.)));
    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        window.focus(&focus, cx);
        draw(window, cx);
        space(window, cx);
        assert_eq!(view.read(cx).activations, 1);
        assert_eq!(
            find(window, &[], &"animated-fab".into()).unwrap().role(),
            Some(Role::Button)
        );
    })
    .unwrap();

    set_visible(false, cx);
    let (leave, leave_paint) = sample(cx);
    assert_eq!((leave, leave_paint), (open, open_paint));
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        assert!(!focus.is_focused(window));
        assert!(find(window, &[], &"animated-fab".into()).is_none());
        assert!(
            source
                .read(cx)
                .state(window.window_handle().window_id(), &"animated-fab".into())
                .is_none()
        );
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"next-control".into()).unwrap().focused(),
            Some(true)
        );
        space(window, cx);
        pointer(open.center(), window, cx);
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(100));
    let (mid_slot, mid_paint) = sample(cx);
    let (ltr, alpha) = mid_paint.unwrap();
    assert_eq!(mid_slot, open);
    assert_eq!(ltr.size.width, px(35.));
    assert_eq!(ltr.right(), open.right());
    assert_eq!(ltr.bottom(), open.bottom());
    assert!((alpha - 0.75).abs() < 0.01);

    cx.update_window(handle, |_, window, cx| {
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    let (rtl_slot, rtl_paint) = sample(cx);
    let (rtl, rtl_alpha) = rtl_paint.unwrap();
    assert_eq!(rtl.size, ltr.size);
    assert_eq!(rtl.left(), rtl_slot.left());
    assert_eq!(rtl.bottom(), rtl_slot.bottom());
    assert_eq!(rtl_alpha, alpha);

    set_visible(true, cx);
    assert_eq!(sample(cx), (rtl_slot, rtl_paint));
    cx.executor().advance_clock(Duration::from_millis(100));
    let (_, resumed_paint) = sample(cx);
    let (resumed, resumed_alpha) = resumed_paint.unwrap();
    assert!(resumed.size.width > rtl.size.width);
    assert!(resumed_alpha > rtl_alpha);
    cx.update_window(handle, |_, window, _| assert!(!focus.is_focused(window)))
        .unwrap();

    cx.update(|cx| cx.set_reduce_motion(true));
    set_visible(false, cx);
    let (hidden, hidden_paint) = sample(cx);
    assert_hidden(hidden, hidden_paint);
    set_visible(true, cx);
    let (shown, shown_paint) = sample(cx);
    assert_eq!(shown_paint, Some((shown, 1.)));

    cx.update(|cx| cx.set_reduce_motion(false));
    set_visible(false, cx);
    sample(cx);
    cx.executor().advance_clock(Duration::from_millis(250));
    let (fading, fading_paint) = sample(cx);
    let (scaled, faded) = fading_paint.unwrap();
    assert_eq!(fading.size.width, px(56.));
    assert_eq!(scaled.size.width, px(14.));
    assert!(faded > 0. && faded < 0.5);
    cx.executor().advance_clock(Duration::from_millis(200));
    let (closed, closed_paint) = sample(cx);
    assert_hidden(closed, closed_paint);

    for (target_scale, painted_width) in [
        (0., None),
        (f32::NAN, Some(px(11.))),
        (-1., Some(px(11.))),
        (f32::MAX, Some(px(11.))),
        (1.25, Some(px(70.))),
    ] {
        cx.update(|cx| cx.set_reduce_motion(true));
        set_visible(true, cx);
        sample(cx);
        cx.update(|cx| {
            cx.set_reduce_motion(false);
            view.update(cx, |view, cx| {
                view.visible = false;
                view.target_scale = target_scale;
                cx.notify();
            });
        });
        sample(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        let (slot, paint) = sample(cx);
        assert_eq!(slot.size.width, px(56.));
        assert_eq!(
            paint.map(|(bounds, _)| bounds.size.width),
            painted_width,
            "target scale {target_scale}"
        );
    }
}
