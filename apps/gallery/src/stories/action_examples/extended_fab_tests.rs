use std::time::Duration;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels,
    Render, Role, Styled, TestAppContext, Window, canvas, div, px,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Button, ButtonColors, ControlElevations, ExtendedFab, FabAlignment, FabSize,
    FabVisibilityAnimation, InteractionSource, MaterialRoot,
    foundation::{set_window_material_theme, window_material_theme},
    theme::{CornerShape, CubicBezier, Direction, Elevation, MotionSpec},
};

use super::fab_tests::{draw, pointer, space};

const CONTAINER: Hsla = gpui::hsla(0.37, 0.68, 0.42, 1.);
const CONTENT: Hsla = gpui::hsla(0.88, 0.53, 0.48, 1.);
const ICON: Hsla = gpui::hsla(0.10, 0.92, 0.37, 1.);
const DISABLED: Hsla = gpui::hsla(0.52, 0.26, 0.52, 1.);

fn painted(tint: Hsla, window: &Window) -> Option<(Bounds<Pixels>, f32)> {
    window.painted_quads().iter().find_map(|quad| {
        let paint = quad.background.as_solid()?;
        ((paint.h, paint.s, paint.l) == (tint.h, tint.s, tint.l)).then(|| {
            (
                quad.bounds.map(|side| px(side.0 / window.scale_factor())),
                paint.a,
            )
        })
    })
}

fn direction(direction: Direction, window: &mut Window, cx: &mut App) {
    let mut theme = window_material_theme(window, cx).clone();
    theme.layout.direction = direction;
    set_window_material_theme(Some(theme), window, cx);
}

#[gpui::test]
fn extended_fab_all_sizes_offer_text_icon_and_arbitrary_content(cx: &mut TestAppContext) {
    struct Examples;

    impl Render for Examples {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
            div().flex().flex_col().children(
                [FabSize::Small, FabSize::Medium, FabSize::Large]
                    .into_iter()
                    .enumerate()
                    .map(|(index, tier)| {
                        div()
                            .flex()
                            .items_start()
                            .child(ExtendedFab::text(("text-tier", index), "New form").size(tier))
                            .child(
                                ExtendedFab::new(("icon-tier", index), "New form", "+").size(tier),
                            )
                            .child(
                                ExtendedFab::text(("custom-tier", index), "New blank form")
                                    .size(tier)
                                    .expanded(false)
                                    .content(div().size(px(60.)).bg(CONTENT)),
                            )
                    }),
            )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(Default::default(), cx, |_, cx| cx.new(|_| Examples)).unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        for (index, extent) in [56., 80., 96.].into_iter().enumerate() {
            for id in ["text-tier", "icon-tier", "custom-tier"] {
                let node = find(window, &[], &(id, index).into()).unwrap();
                assert_eq!(node.role(), Some(Role::Button));
                assert_eq!(node.bounds().size.height, px(extent));
                assert!(node.bounds().size.width > px(extent));
            }
        }
    })
    .unwrap();
}

#[gpui::test]
fn extended_fab_content_modes_fit_and_keep_one_accessible_action(cx: &mut TestAppContext) {
    struct Example {
        expanded: bool,
        disabled: bool,
        activations: usize,
        layout_width: Pixels,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
            let owner = cx.entity();
            let icon = canvas(
                |_, _, _| (),
                |bounds, (), window, _| window.paint_quad(gpui::fill(bounds, ICON)),
            )
            .size(px(14.));
            let swatch = canvas(
                |_, window, _| window.text_style().color,
                |bounds, ink, window, _| window.paint_quad(gpui::fill(bounds, ink)),
            )
            .w(px(360.))
            .h(px(16.));
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    ExtendedFab::text("extended-text", "Create document")
                        .expanded(self.expanded)
                        .on_activate(move |_, cx| {
                            owner.update(cx, |view, _| view.activations += 1);
                        }),
                )
                .child(
                    div().w(self.layout_width).child(
                        ExtendedFab::text("extended-long", "Long document name ‒ ".repeat(25))
                            .expanded(false),
                    ),
                )
                .child(
                    div().w(self.layout_width).child(
                        ExtendedFab::new("extended-custom", "Import drawing", icon)
                            .content(swatch)
                            .size(FabSize::Medium)
                            .expanded(self.expanded)
                            .disabled(self.disabled)
                            .shape(CornerShape::Radii([3., 8., 12., 16.]))
                            .colors(ButtonColors {
                                container: Some(CONTAINER),
                                content: Some(CONTENT),
                                disabled_container: Some(DISABLED),
                                ..Default::default()
                            })
                            .elevations(ControlElevations {
                                resting: Some(Elevation::Level0),
                                ..Default::default()
                            }),
                    ),
                )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Example {
            expanded: true,
            disabled: false,
            activations: 0,
            layout_width: px(170.),
        });
        let window =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), view)
    });
    cx.update_window(window, |_, window, cx| {
        window.activate_window();
        draw(window, cx);
        let text = find(window, &[], &"extended-text".into()).unwrap();
        let text_bounds = text.bounds();
        assert_eq!(text.role(), Some(Role::Button));
        assert!(text_bounds.size.width > px(56.));
        assert_eq!(text_bounds.size.height, px(56.));
        pointer(text_bounds.center(), window, cx);
        assert_eq!(view.read(cx).activations, 1);
        let long = find(window, &[], &"extended-long".into()).unwrap().bounds();
        assert_eq!(long.size.width, px(170.));
        let custom = find(window, &[], &"extended-custom".into()).unwrap();
        assert_eq!(custom.role(), Some(Role::Button));
        let bounds = custom.bounds();
        assert_eq!(bounds.size.width, px(170.));
        assert_eq!(bounds.size.height, px(80.));
        let icon = painted(ICON, window).unwrap().0;
        let swatch = window
            .painted_quads()
            .into_iter()
            .find(|quad| quad.background.as_solid() == Some(CONTENT))
            .unwrap();
        assert!(icon.right() <= px(swatch.content_mask.bounds.left().0 / window.scale_factor()));
        assert!(swatch.content_mask.bounds.size.width.0 / window.scale_factor() < 100.);
        let corners = window
            .painted_quads()
            .into_iter()
            .find(|quad| quad.background.as_solid() == Some(CONTAINER))
            .unwrap()
            .corner_radii;
        assert_eq!(corners.top_left.0 / window.scale_factor(), 3.);

        direction(Direction::Rtl, window, cx);
        view.update(cx, |view, cx| {
            view.layout_width = px(140.);
            cx.notify();
        });
        draw(window, cx);
        let rtl_icon = painted(ICON, window).unwrap().0;
        let rtl_swatch = window
            .painted_quads()
            .into_iter()
            .find(|quad| quad.background.as_solid() == Some(CONTENT))
            .unwrap();
        assert!(
            px(rtl_swatch.content_mask.bounds.right().0 / window.scale_factor()) <= rtl_icon.left()
        );
        assert_eq!(
            find(window, &[], &"extended-long".into())
                .unwrap()
                .bounds()
                .size
                .width,
            px(140.)
        );

        view.update(cx, |view, cx| {
            view.expanded = false;
            view.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"extended-text".into()).unwrap().bounds(),
            text_bounds
        );
        assert_eq!(painted(DISABLED, window).unwrap().0.size.width, px(80.));
        assert!(painted(CONTENT, window).is_none());
        assert!(painted(ICON, window).is_some());
    })
    .unwrap();
}

#[gpui::test]
fn extended_fab_visibility_retargets_and_detaches_the_hidden_action(cx: &mut TestAppContext) {
    struct Example {
        visible: bool,
        expanded: bool,
        focus: FocusHandle,
        source: gpui::Entity<InteractionSource>,
        activations: usize,
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
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
            let owner = cx.entity();
            div()
                .flex()
                .items_start()
                .child(
                    div()
                        .id("extended-slot")
                        .test_support()
                        .max_w(px(210.))
                        .child(
                            ExtendedFab::new("extended-motion", "Create extended document", "✦")
                                .track_focus(&self.focus)
                                .expanded(self.expanded)
                                .interaction_source(&self.source)
                                .colors(ButtonColors {
                                    container: Some(CONTAINER),
                                    ..Default::default()
                                })
                                .visible(self.visible)
                                .visibility_animation(FabVisibilityAnimation {
                                    alignment: FabAlignment::BottomEnd,
                                    target_scale: 0.25,
                                    scale_motion: Some(tween(200)),
                                    alpha_motion: Some(tween(400)),
                                })
                                .on_activate(move |_, cx| {
                                    owner.update(cx, |view, _| view.activations += 1);
                                }),
                        ),
                )
                .child(Button::new("following-control", "Following action"))
        }
    }

    let (window, view, focus, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        let focus = cx.focus_handle();
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|_| Example {
            visible: true,
            expanded: true,
            focus: focus.clone(),
            source: source.clone(),
            activations: 0,
        });
        let window =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), view, focus, source)
    });
    let measure = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            draw(window, cx);
            let slot = find(window, &[], &"extended-slot".into()).unwrap().bounds();
            (slot, painted(CONTAINER, window))
        })
        .unwrap()
    };
    let update = |visible, expanded, cx: &mut TestAppContext| {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.visible = visible;
                view.expanded = expanded;
                cx.notify();
            })
        });
    };
    let (slot, shown) = measure(cx);
    assert!(slot.size.width > px(56.));
    assert_eq!(shown, Some((slot, 1.)));
    cx.update_window(window, |_, window, cx| {
        window.activate_window();
        window.focus(&focus, cx);
        draw(window, cx);
        space(window, cx);
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();

    update(false, true, cx);
    assert_eq!(measure(cx), (slot, shown));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        assert!(!focus.is_focused(window));
        assert!(find(window, &[], &"extended-motion".into()).is_none());
        assert!(
            source
                .read(cx)
                .state(
                    window.window_handle().window_id(),
                    &"extended-motion".into()
                )
                .is_none()
        );
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"following-control".into())
                .unwrap()
                .focused(),
            Some(true)
        );
        space(window, cx);
        pointer(slot.center(), window, cx);
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();

    cx.executor().advance_clock(Duration::from_millis(100));
    let (same_slot, midway) = measure(cx);
    let (shrinking, alpha) = midway.unwrap();
    assert_eq!(slot, same_slot);
    assert!((shrinking.right() - slot.right()).abs() <= px(1.));
    assert!(shrinking.size.width < slot.size.width);
    assert!((alpha - 0.75).abs() < 0.01);
    cx.update_window(window, |_, window, cx| {
        direction(Direction::Rtl, window, cx)
    })
    .unwrap();
    let (rtl_slot, rtl_visual) = measure(cx);
    let (rtl_bounds, _) = rtl_visual.unwrap();
    assert_eq!(rtl_bounds.left(), rtl_slot.left());
    update(true, false, cx);
    assert_eq!(measure(cx), (rtl_slot, rtl_visual));
    cx.executor().advance_clock(Duration::from_secs(1));
    let (collapsed, collapsed_paint) = measure(cx);
    assert_eq!(collapsed.size.width, px(56.));
    assert_eq!(collapsed_paint, Some((collapsed, 1.)));

    cx.update(|cx| cx.set_reduce_motion(true));
    update(false, true, cx);
    let (hidden, hidden_paint) = measure(cx);
    assert_eq!(hidden.size.width, px(0.));
    assert!(hidden_paint.is_none());
    update(true, true, cx);
    let (expanded, expanded_paint) = measure(cx);
    assert_eq!(expanded.size, slot.size);
    assert_eq!(expanded_paint, Some((expanded, 1.)));
}
