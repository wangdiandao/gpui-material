use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AppContext, Bounds, Element as _, InputEvent as _, KeyDownEvent, KeyUpEvent, Keystroke, Pixels,
    RenderOnce as _, Role, ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, WindowOptions,
    accesskit, point, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::MaterialRoot;

use super::*;

struct Controlled {
    state: Entity<CarouselState>,
    order: Vec<usize>,
    scroll: bool,
    fling: CarouselFling,
}

impl Render for Controlled {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let region = Carousel::new(
            "controlled-carousel",
            "Controlled places",
            self.order.iter().map(|&index| {
                let place = &PLACES[index];
                CarouselItem::new(place.id, place.title, div())
            }),
        )
        .variant(CarouselVariant::FullScreen)
        .height(px(200.))
        .content_padding(LogicalEdges {
            start: 24.,
            end: 16.,
            ..Default::default()
        })
        .state(&self.state)
        .user_scroll_enabled(self.scroll)
        .fling_behavior(self.fling.clone())
        .render(window, cx)
        .into_element();
        let mut node = accesskit::Node::new(Role::Region);
        region.write_a11y_info(&mut node);
        assert_eq!(region.a11y_role(), Some(Role::Region));
        assert_eq!(node.label(), Some("Controlled places"));
        assert_eq!(
            node.supports_action(accesskit::Action::ScrollUp),
            self.scroll
        );
        assert_eq!(
            node.supports_action(accesskit::Action::ScrollDown),
            self.scroll
        );
        region
    }
}

fn window(
    cx: &mut TestAppContext,
    initial: usize,
    scroll: bool,
) -> (
    gpui::WindowHandle<gpui_base::Root>,
    Entity<Controlled>,
    Entity<CarouselState>,
) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        let state = cx.new(|_| CarouselState::new(initial));
        let view = cx.new(|_| Controlled {
            state: state.clone(),
            order: vec![0, 1, 2, 3, 4, 5],
            scroll,
            fling: CarouselFling::Default,
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view, state)
    })
}

fn draw(cx: &mut TestAppContext, handle: gpui::WindowHandle<gpui_base::Root>) {
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn gesture(
    cx: &mut TestAppContext,
    handle: gpui::WindowHandle<gpui_base::Root>,
    delta: f32,
    phase: TouchPhase,
) {
    cx.update_window(handle.into(), |_, window, cx| {
        let carousel = find(window, &[], &"controlled-carousel".into()).unwrap();
        let event = ScrollEvent {
            position: carousel.bounds().center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
            touch_phase: phase,
            ..ScrollEvent::default()
        };
        window.dispatch_event(event.to_platform_input(), cx);
    })
    .unwrap();
}

fn item_bounds(window: &Window, name: &'static str) -> Bounds<Pixels> {
    use gpui::ElementId;
    let id = (ElementId::from("controlled-carousel"), name).into();
    let item = find(window, &[], &id).unwrap();
    assert_eq!(item.role(), Some(Role::Button));
    item.bounds()
}

#[gpui::test]
fn carousel_readable_position_survives_resize_reorder_and_disabled_user_scroll(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (handle, view, state) = window(cx, 2, false);
    draw(cx, handle);
    cx.update(|cx| {
        let value = state.read(cx);
        assert_eq!(value.current_item(), Some(2));
        assert_eq!(value.current_item_id().map(AsRef::as_ref), Some("market"));
        assert_eq!(value.current_item_offset_fraction(), 0.);
    });
    cx.simulate_window_resize(handle.into(), size(px(380.), px(350.)));
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let viewport = find(window, &[], &"controlled-carousel".into())
            .unwrap()
            .bounds();
        let bounds = item_bounds(window, "market");
        assert_eq!(bounds.left(), viewport.left() + px(24.));
        assert_eq!(bounds.right(), viewport.right() - px(16.));
        window.focus_next(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "down");
    gesture(cx, handle, -260., TouchPhase::Moved);
    gesture(cx, handle, 0., TouchPhase::Ended);
    draw(cx, handle);
    cx.update(|cx| assert_eq!(state.read(cx).current_item(), Some(2)));
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.order.insert(0, 5);
            view.order.pop();
            view.scroll = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update(|cx| {
        assert_eq!(state.read(cx).current_item(), Some(3));
        assert_eq!(
            state.read(cx).current_item_id().map(AsRef::as_ref),
            Some("market")
        );
    });
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.order.remove(3);
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update(|cx| {
        assert_eq!(
            state.read(cx).current_item_id().map(AsRef::as_ref),
            Some("ridge")
        );
        view.update(cx, |view, cx| {
            view.order.clear();
            cx.notify();
        });
    });
    draw(cx, handle);
    cx.update(|cx| {
        assert_eq!(state.read(cx).current_item(), None);
        state.update(cx, |state, cx| state.scroll_to_item(usize::MAX, cx));
        view.update(cx, |view, cx| {
            view.order.extend([0, 4, 2]);
            cx.notify();
        });
    });
    draw(cx, handle);
    cx.update(|cx| {
        assert_eq!(state.read(cx).current_item(), Some(2));
        state.update(cx, |state, cx| state.animate_scroll_to_item(0, cx));
        assert_eq!(state.read(cx).current_item(), Some(0));
    });
}

#[gpui::test]
fn carousel_settles_precise_input_with_default_and_custom_fling(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (handle, view, state) = window(cx, 0, true);
    draw(cx, handle);
    gesture(cx, handle, -455., TouchPhase::Moved);
    cx.update(|cx| {
        let value = state.read(cx);
        assert_eq!(value.current_item(), Some(2));
        assert!(value.current_item_offset_fraction() > 0.);
    });
    cx.executor().advance_clock(Duration::from_millis(140));
    cx.run_until_parked();
    cx.update(|cx| assert_eq!(state.read(cx).current_item_offset_fraction(), 0.));

    for (fling, target) in [
        (CarouselFling::SingleAdvance, 1),
        (
            CarouselFling::custom(|gesture| {
                assert_eq!(gesture.start, 0.);
                assert!(gesture.position > 2.);
                assert_eq!(gesture.item_count, 6);
                Some(usize::MAX)
            }),
            5,
        ),
    ] {
        cx.update(|cx| {
            state.update(cx, |state, cx| state.scroll_to_item(0, cx));
            view.update(cx, |view, cx| {
                view.fling = fling;
                cx.notify();
            });
        });
        draw(cx, handle);
        gesture(cx, handle, -455., TouchPhase::Moved);
        gesture(cx, handle, 0., TouchPhase::Ended);
        cx.update(|cx| assert_eq!(state.read(cx).current_item(), Some(target)));
    }

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.fling = CarouselFling::Free;
            cx.notify();
        });
        state.update(cx, |state, cx| state.scroll_to_item(0, cx));
    });
    draw(cx, handle);
    gesture(cx, handle, -455., TouchPhase::Moved);
    gesture(cx, handle, 0., TouchPhase::Ended);
    cx.update(|cx| assert!(state.read(cx).current_item_offset_fraction() > 0.));
    let before = cx.update(|cx| state.read(cx).current_item_offset_fraction());
    gesture(cx, handle, -40., TouchPhase::Moved);
    gesture(cx, handle, 0., TouchPhase::Cancelled);
    cx.update(|cx| assert_eq!(state.read(cx).current_item_offset_fraction(), before));
}

#[gpui::test]
fn carousel_animated_command_advances_from_the_sampled_position(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(false));
    let (handle, _, state) = window(cx, 0, true);
    draw(cx, handle);
    cx.update(|cx| {
        state.update(cx, |state, cx| state.animate_scroll_to_item(2, cx));
        assert_eq!(state.read(cx).current_item(), Some(0));
    });
    draw(cx, handle);
    cx.executor().advance_clock(Duration::from_millis(20));
    draw(cx, handle);
    cx.update(|cx| {
        let state = state.read(cx);
        assert_ne!(state.current_item_offset_fraction(), 0.);
    });
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx, handle);
    cx.update(|cx| {
        assert_eq!(state.read(cx).current_item(), Some(2));
        assert_eq!(state.read(cx).current_item_offset_fraction(), 0.);
    });
}

struct Parallax {
    activated: Rc<Cell<bool>>,
}

impl Render for Parallax {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use gpui_base::TestSupportExt;
        let activated = self.activated.clone();
        div().pl(px(80.)).pt(px(12.)).child(
            CarouselParallax::new(
                div()
                    .id("parallax-child")
                    .test_support()
                    .w(px(120.))
                    .h(px(88.))
                    .bg(rgb(0x7650ab))
                    .child(
                        Button::new("parallax-button", "Parallax action").on_activate(
                            move |_, _| {
                                activated.set(true);
                            },
                        ),
                    ),
            )
            .viewport(Bounds::new(
                point(px(110.), px(0.)),
                size(px(230.), px(140.)),
            ))
            .shape(CornerShape::Radii([12., 18., 6., 8.]))
            .border(SurfaceBorder {
                width: px(2.),
                color: rgb(0x382d4e).into(),
            })
            .backdrop(rgb(0xe9dff0).into()),
        )
    }
}

#[gpui::test]
fn carousel_parallax_masks_arbitrary_content_and_preserves_child_activation(
    cx: &mut TestAppContext,
) {
    let activated = Rc::new(Cell::new(false));
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Parallax {
                activated: activated.clone(),
            })
        })
        .unwrap()
    });
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let child = find(window, &[], &"parallax-child".into()).unwrap();
        assert!(child.bounds().left() > px(80.));
        assert_eq!(child.bounds().size.width, px(120.));
        let button = find(window, &[], &"parallax-button".into()).unwrap();
        assert_eq!(button.role(), Some(Role::Button));
        assert!(button.visible());
        let quads = window.painted_quads();
        let stroke = gpui::Hsla::from(rgb(0x382d4e));
        let border = quads
            .iter()
            .find(|quad| quad.border_color == stroke)
            .unwrap();
        assert_eq!(
            border.content_mask.bounds.left().0,
            110. * window.scale_factor()
        );
        assert!(border.corner_radii.top_right.0 > border.corner_radii.top_left.0);
        window.focus_next(cx);
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let button = find(window, &[], &"parallax-button".into()).unwrap();
        assert_eq!(button.focused(), Some(true));
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert!(activated.get());
    cx.update(|cx| cx.set_reduce_motion(true));
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(
            find(window, &[], &"parallax-child".into())
                .unwrap()
                .bounds()
                .left(),
            px(80.)
        );
    })
    .unwrap();
}
