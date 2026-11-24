use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Pixels, Render, Styled,
    Window, div, prelude::FluentBuilder as _, px, size,
};
use gpui_material::foundation::{color, window_material_theme};
use gpui_material::theme::{ColorRole, CornerRole, Direction, TypeRole};
use gpui_material::{
    DragHandleColors, DragHandleShapes, DragHandleSizes, InteractionSource, Surface, Text,
    VerticalDragHandle,
};

use crate::story::section;

const MIN_PANE_WIDTH: Pixels = px(72.);

pub(crate) struct DragHandleExamples {
    width: Pixels,
    available: Pixels,
    interaction: Entity<InteractionSource>,
}

impl DragHandleExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            width: px(164.),
            available: px(360.),
            interaction: cx.new(|_| InteractionSource::new()),
        }
    }
}

impl Render for DragHandleExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let target = px(theme.layout.density.interactive_size(0.));
        let max = (self.available - target - MIN_PANE_WIDTH).max(MIN_PANE_WIDTH + px(1.));
        let width = self.width.clamp(MIN_PANE_WIDTH, max);
        let owner = cx.weak_entity();
        let handle = VerticalDragHandle::new("start-pane-width")
            .sizes(DragHandleSizes {
                pressed: Some(size(px(12.), px(52.))),
                dragged: Some(size(px(16.), px(56.))),
                ..Default::default()
            })
            .colors(DragHandleColors {
                pressed: Some(color(theme.colors.color(ColorRole::Secondary))),
                dragged: Some(color(theme.colors.color(ColorRole::Primary))),
                ..Default::default()
            })
            .shapes(DragHandleShapes {
                dragged: Some(CornerRole::Small.shape()),
                ..Default::default()
            })
            .interaction_source(&self.interaction)
            .on_resize(
                "Start pane width",
                width,
                MIN_PANE_WIDTH..=max,
                move |width, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.width = width;
                        cx.notify();
                    });
                },
            );
        let owner = cx.weak_entity();
        section("Split panes")
            .description("Drag the handle or focus it and use the arrow keys.")
            .child(
                div()
                    .w_full()
                    .on_children_prepainted(move |bounds, _, cx| {
                        let Some(available) = bounds.first().map(|bounds| bounds.size.width) else {
                            return;
                        };
                        let _ = owner.update(cx, |this, cx| {
                            if this.available != available {
                                this.available = available;
                                cx.notify();
                            }
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .h(px(160.))
                            .items_center()
                            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                            .child(
                                Surface::new(ColorRole::SecondaryContainer)
                                    .w(width)
                                    .h_full()
                                    .flex_none()
                                    .rounded_lg()
                                    .p_3()
                                    .child(
                                        Text::new(format!(
                                            "Start pane · {:.0} px",
                                            f32::from(width)
                                        ))
                                        .id("start-pane-label")
                                        .role(TypeRole::BodyMedium)
                                        .wrap(),
                                    ),
                            )
                            .child(handle)
                            .child(
                                Surface::new(ColorRole::SurfaceContainerHigh)
                                    .flex_1()
                                    .h_full()
                                    .rounded_lg()
                                    .p_3()
                                    .child(
                                        Text::new("End pane")
                                            .id("end-pane-label")
                                            .role(TypeRole::BodyMedium)
                                            .wrap(),
                                    ),
                            ),
                    ),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        Bounds, FocusHandle, Hsla, InteractiveElement as _, Modifiers, MouseButton, Point, Role,
        TestAppContext, TouchDragEvent, TouchPhase, VisualTestContext, point, rgb,
    };
    use gpui_base::{Button, TestSupportExt as _, test_support::find};
    use gpui_material::foundation::set_window_material_theme;
    use gpui_material::{InteractionState, MaterialRoot, MaterialTheme};

    const START: Pixels = px(100.);
    const RANGE: std::ops::RangeInclusive<Pixels> = px(80.)..=px(180.);

    struct HandlePreview {
        source: Entity<InteractionSource>,
        width: Pixels,
        requests: Vec<Pixels>,
        other_focus: FocusHandle,
    }

    impl Render for HandlePreview {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let direction = window_material_theme(window, cx).layout.direction;
            let owner = cx.weak_entity();
            let handle = VerticalDragHandle::new("splitter")
                .interaction_source(&self.source)
                .sizes(DragHandleSizes {
                    resting: Some(size(px(6.), px(34.))),
                    pressed: Some(size(px(10.), px(50.))),
                    dragged: Some(size(px(16.), px(58.))),
                })
                .colors(DragHandleColors {
                    resting: Some(rgb(0x925700).into()),
                    pressed: Some(rgb(0x165bb4).into()),
                    dragged: Some(rgb(0x2c6934).into()),
                })
                .shapes(DragHandleShapes {
                    resting: Some(CornerShape::Full),
                    pressed: Some(CornerShape::Radii([1., 2., 3., 4.])),
                    dragged: Some(CornerShape::Radii([0., 0., 0., 0.])),
                })
                .keyboard_step(px(12.))
                .on_resize(
                    "Start pane width",
                    self.width,
                    RANGE,
                    move |width, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.requests.push(width);
                            if width != px(124.) {
                                this.width = width;
                            }
                            cx.notify();
                        });
                    },
                );
            div()
                .w(px(360.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .w_full()
                        .h(px(100.))
                        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                        .child(
                            div()
                                .id("start-pane")
                                .test_support()
                                .flex_none()
                                .w(self.width)
                                .h_full(),
                        )
                        .child(handle)
                        .child(div().flex_1().h_full()),
                )
                .child(
                    Button::new("other")
                        .accessibility_label("Other control")
                        .track_focus(&self.other_focus),
                )
                .child(VerticalDragHandle::new("passive").interaction_source(&self.source))
        }
    }

    use gpui_material::theme::CornerShape;

    fn open(cx: &mut TestAppContext) -> (Entity<HandlePreview>, &mut VisualTestContext) {
        cx.update(|cx| MaterialRoot::init(MaterialTheme::default(), cx));
        let (view, cx) = cx.add_window_view(|window, cx| {
            window.activate_window();
            HandlePreview {
                source: cx.new(|_| InteractionSource::new()),
                width: START,
                requests: Vec::new(),
                other_focus: cx.focus_handle(),
            }
        });
        draw(cx);
        (view, cx)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }

    fn target(cx: &mut VisualTestContext, id: &'static str) -> Bounds<Pixels> {
        cx.update(|window, _| find(window, &[], &id.into()).unwrap().bounds())
    }

    fn snapshot(
        cx: &mut VisualTestContext,
        view: &Entity<HandlePreview>,
        id: &'static str,
    ) -> InteractionState {
        cx.update(|window, cx| {
            view.read(cx)
                .source
                .read(cx)
                .state(window.window_handle().window_id(), &id.into())
                .unwrap()
        })
    }

    fn appearance(
        cx: &mut VisualTestContext,
        tint: Hsla,
        width: f32,
        height: f32,
        radii: [f32; 4],
    ) {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let size = size(px(width), px(height)).scale(scale);
            let painted = crate::painted_quads(window);
            let quad = painted
                .iter()
                .find(|quad| quad.background == tint.into() && quad.bounds.size == size)
                .expect("handle mark with the configured color and size");
            assert_eq!(
                [
                    quad.corner_radii.top_left,
                    quad.corner_radii.top_right,
                    quad.corner_radii.bottom_right,
                    quad.corner_radii.bottom_left
                ],
                radii.map(|radius| px(radius).scale(scale)),
            );
        });
    }

    fn press(cx: &mut VisualTestContext, at: Point<Pixels>) {
        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        draw(cx);
    }

    fn drag(cx: &mut VisualTestContext, from: Point<Pixels>, dx: f32) {
        cx.simulate_mouse_move(
            from + point(px(dx), px(0.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
    }

    fn release(cx: &mut VisualTestContext, at: Point<Pixels>) {
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
        draw(cx);
    }

    #[gpui::test]
    fn drag_handle_paints_each_state_with_a_stable_target_and_controlled_width(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open(cx);
        let target_size = size(px(48.), px(58.));
        let handle = target(cx, "splitter");
        assert_eq!(handle.size, target_size);
        assert_eq!(target(cx, "passive").size, size(px(48.), px(52.)));
        let outline = cx.update(|window, cx| {
            color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::Outline),
            )
        });
        appearance(cx, outline, 4., 48., [2.; 4]);
        appearance(cx, rgb(0x925700).into(), 6., 34., [3.; 4]);

        let origin = handle.center();
        cx.simulate_mouse_move(origin, None, Modifiers::default());
        draw(cx);
        let interaction = snapshot(cx, &view, "splitter");
        assert!(interaction.hovered && !interaction.pressed && !interaction.dragged);
        assert!(!snapshot(cx, &view, "passive").hovered);
        appearance(cx, rgb(0x925700).into(), 6., 34., [3.; 4]);

        press(cx, origin);
        assert_eq!(target(cx, "splitter"), handle);
        let interaction = snapshot(cx, &view, "splitter");
        assert!(interaction.pressed && !interaction.dragged);
        appearance(cx, rgb(0x165bb4).into(), 10., 50., [1., 2., 3., 4.]);

        drag(cx, origin, 12.);
        assert_eq!(target(cx, "splitter").size, target_size);
        assert_eq!(target(cx, "start-pane").size.width, px(112.));
        let interaction = snapshot(cx, &view, "splitter");
        assert!(interaction.pressed && interaction.dragged);
        appearance(cx, rgb(0x2c6934).into(), 16., 58., [0.; 4]);
        drag(cx, origin, 24.);
        drag(cx, origin, 24.);
        assert_eq!(target(cx, "start-pane").size.width, px(112.));
        drag(cx, origin, 36.);
        release(cx, origin + point(px(36.), px(0.)));
        assert_eq!(target(cx, "start-pane").size.width, px(136.));
        assert_eq!(target(cx, "splitter").size, target_size);
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).requests, [px(112.), px(124.), px(136.)]);
            assert!(window.captured_hitbox().is_none());
        });
        let interaction = snapshot(cx, &view, "splitter");
        assert!(!interaction.pressed && !interaction.dragged);
        appearance(cx, rgb(0x925700).into(), 6., 34., [3.; 4]);
    }

    #[gpui::test]
    fn drag_handle_cancels_when_focus_leaves_and_mirrors_rtl_resize(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        let origin = target(cx, "splitter").center();
        press(cx, origin);
        drag(cx, origin, 8.);
        cx.update(|window, cx| {
            let focus = view.read(cx).other_focus.clone();
            focus.focus(window, cx);
        });
        draw(cx);
        let interaction = snapshot(cx, &view, "splitter");
        assert!(!interaction.pressed && !interaction.dragged);
        cx.update(|window, _| assert!(window.captured_hitbox().is_none()));
        drag(cx, origin, 30.);
        release(cx, origin + point(px(30.), px(0.)));
        cx.update(|_, cx| assert_eq!(view.read(cx).requests, [px(108.)]));

        cx.update(|window, cx| {
            let mut theme = MaterialTheme::default();
            theme.layout.direction = Direction::Rtl;
            set_window_material_theme(Some(theme), window, cx);
        });
        draw(cx);
        let handle = target(cx, "splitter");
        assert_eq!(handle.right(), target(cx, "start-pane").left());
        let origin = handle.center();
        press(cx, origin);
        drag(cx, origin, -14.);
        assert_eq!(target(cx, "start-pane").size.width, px(122.));
        cx.simulate_keystrokes("escape");
        draw(cx);
        assert!(!snapshot(cx, &view, "splitter").pressed);
        drag(cx, origin, -28.);
        release(cx, origin - point(px(28.), px(0.)));
        assert_eq!(target(cx, "start-pane").size.width, px(122.));

        let origin = target(cx, "splitter").center();
        press(cx, origin);
        drag(cx, origin, -10.);
        cx.deactivate_window();
        draw(cx);
        let interaction = snapshot(cx, &view, "splitter");
        assert!(!interaction.pressed && !interaction.dragged && !interaction.hovered);
        cx.update(|window, _| assert!(window.captured_hitbox().is_none()));
        release(cx, origin - point(px(20.), px(0.)));
        assert_eq!(target(cx, "start-pane").size.width, px(132.));
    }

    #[gpui::test]
    fn drag_handle_exposes_a_named_splitter_with_keyboard_resize(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        cx.update(|window, cx| window.focus_next(cx));
        draw(cx);
        cx.update(|window, _| {
            let handle = find(window, &[], &"splitter".into()).unwrap();
            assert_eq!(handle.role(), Some(Role::Slider));
            assert_eq!(handle.label(), Some("Start pane width"));
            assert_eq!(handle.focused(), Some(true));
            assert_eq!(find(window, &[], &"passive".into()).unwrap().role(), None);
        });
        for (key, width) in [
            ("right", px(112.)),
            ("end", px(180.)),
            ("right", px(180.)),
            ("home", px(80.)),
        ] {
            cx.simulate_keystrokes(key);
            draw(cx);
            assert_eq!(target(cx, "start-pane").size.width, width);
        }
        assert!(snapshot(cx, &view, "splitter").focus_visible);

        cx.update(|window, cx| {
            let mut theme = MaterialTheme::default();
            theme.layout.direction = Direction::Rtl;
            set_window_material_theme(Some(theme), window, cx);
        });
        draw(cx);
        cx.simulate_keystrokes("left");
        draw(cx);
        assert_eq!(target(cx, "start-pane").size.width, px(92.));
    }

    #[gpui::test]
    fn drag_handle_resizes_on_horizontal_touch_and_stops_on_cancellation(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        let origin = target(cx, "splitter").center();
        let touch = |cx: &mut VisualTestContext, phase, dx, dy| {
            cx.simulate_event(TouchDragEvent {
                phase,
                start_position: origin,
                position: origin + point(px(dx), px(dy)),
            });
            draw(cx);
        };

        touch(cx, TouchPhase::Started, 4., 20.);
        touch(cx, TouchPhase::Ended, 8., 30.);
        cx.update(|_, cx| assert!(view.read(cx).requests.is_empty()));

        touch(cx, TouchPhase::Started, 16., 2.);
        touch(cx, TouchPhase::Moved, 30., 2.);
        assert!(snapshot(cx, &view, "splitter").dragged);
        assert_eq!(target(cx, "start-pane").size.width, px(130.));
        touch(cx, TouchPhase::Cancelled, 40., 2.);
        assert!(!snapshot(cx, &view, "splitter").pressed);
        touch(cx, TouchPhase::Ended, 40., 2.);
        cx.update(|_, cx| assert_eq!(view.read(cx).requests, [px(116.), px(130.)]));
    }
}
