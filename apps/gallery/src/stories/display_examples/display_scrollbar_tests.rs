use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    Axis, Bounds, Context, Entity, FocusHandle, Hsla, InteractiveElement, IntoElement, Modifiers,
    MouseButton, ParentElement, Pixels, Point, Quad, Render, Role, ScrollDelta, ScrollEvent,
    ScrollHandle, Size, StatefulInteractiveElement, Styled, TestAppContext, TouchPhase,
    VisualTestContext, Window, div, point, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, MaterialRoot, MaterialTheme,
    foundation::set_window_material_theme, theme::Direction,
};

const VERTICAL_THUMB: u32 = 0x28675e;
const VERTICAL_TRACK: u32 = 0x9fccb3;
const HORIZONTAL_THUMB: u32 = 0x7a285e;
const HORIZONTAL_TRACK: u32 = 0xe9cdda;

struct Preview {
    scroll: ScrollHandle,
    outside: FocusHandle,
    target: FocusHandle,
    viewport: Size<Pixels>,
    extent: Size<Pixels>,
    fade: DisplayScrollbarFade,
    show_vertical: bool,
    show_horizontal: bool,
    clicks: usize,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let viewport = div()
            .id("demo-viewport")
            .size_full()
            .overflow_y_scroll()
            .overflow_x_scroll()
            .track_scroll(&self.scroll)
            .child(
                gpui_base::Button::new("viewport-content")
                    .accessibility_label("Viewport content")
                    .track_focus(&self.target)
                    .w(self.extent.width)
                    .h(self.extent.height)
                    .on_click(move |_, _, cx| {
                        let _ = owner.update(cx, |owner, _| owner.clicks += 1);
                    })
                    .child("Inside"),
            );
        div()
            .child(
                gpui_base::Button::new("outside")
                    .track_focus(&self.outside)
                    .accessibility_label("Outside")
                    .child("Outside"),
            )
            .child(
                div()
                    .w(self.viewport.width)
                    .h(self.viewport.height)
                    .relative()
                    .child(viewport)
                    .children(self.show_vertical.then(|| {
                        DisplayScrollbar::new("vertical-mark", Axis::Vertical, &self.scroll)
                            .style(DisplayScrollbarStyle {
                                thumb_color: Some(rgb(VERTICAL_THUMB).into()),
                                track_color: Some(rgb(VERTICAL_TRACK).into()),
                                thickness: px(8.),
                                thumb_min_length: px(30.),
                                thumb_max_length_fraction: 0.7,
                                main_axis_track_inset: px(6.),
                                cross_axis_track_inset: px(5.),
                            })
                            .offset(point(px(2.), px(3.)))
                            .fade(self.fade)
                    }))
                    .children(self.show_horizontal.then(|| {
                        DisplayScrollbar::new("horizontal-mark", Axis::Horizontal, &self.scroll)
                            .style(DisplayScrollbarStyle {
                                thumb_color: Some(rgb(HORIZONTAL_THUMB).into()),
                                track_color: Some(rgb(HORIZONTAL_TRACK).into()),
                                thickness: px(6.),
                                thumb_min_length: px(10.),
                                thumb_max_length_fraction: 0.35,
                                main_axis_track_inset: px(8.),
                                cross_axis_track_inset: px(4.),
                            })
                            .fade(self.fade)
                    })),
            )
    }
}

fn open(
    cx: &mut TestAppContext,
    fade: DisplayScrollbarFade,
    show_horizontal: bool,
) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| MaterialRoot::init(MaterialTheme::default(), cx));
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Preview {
            scroll: ScrollHandle::new(),
            outside: cx.focus_handle(),
            target: cx.focus_handle(),
            viewport: size(px(160.), px(120.)),
            extent: size(px(400.), px(480.)),
            fade,
            show_vertical: true,
            show_horizontal,
            clicks: 0,
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

fn frame_callbacks(cx: &mut VisualTestContext) -> usize {
    cx.update(|window, cx| window.simulate_next_frame(cx))
}

fn quad(window: &Window, tint: u32) -> Option<Quad> {
    let tone = Hsla::from(rgb(tint));
    crate::painted_quads(window).into_iter().find(|quad| {
        quad.background
            .as_solid()
            .is_some_and(|paint| Hsla { a: tone.a, ..paint } == tone)
    })
}

fn wheel(cx: &mut VisualTestContext, at: Point<Pixels>, delta: Point<Pixels>) {
    cx.simulate_event(ScrollEvent {
        position: at,
        delta: ScrollDelta::Pixels(delta),
        touch_phase: TouchPhase::Moved,
        ..Default::default()
    });
    draw(cx);
}

#[gpui::test]
fn display_scrollbar_reads_first_frame_and_leaves_input_with_viewport(cx: &mut TestAppContext) {
    let (view, cx) = open(
        cx,
        DisplayScrollbarFade {
            enabled: false,
            ..Default::default()
        },
        true,
    );
    let (vertical_at, horizontal_at) = cx.update(|window, cx| {
        let viewport = view.read(cx).scroll.bounds();
        let scale = window.scale_factor();
        let vertical_track = quad(window, VERTICAL_TRACK).unwrap();
        let vertical_thumb = quad(window, VERTICAL_THUMB).unwrap();
        let horizontal_track = quad(window, HORIZONTAL_TRACK).unwrap();
        let horizontal_thumb = quad(window, HORIZONTAL_THUMB).unwrap();
        assert_eq!(
            vertical_track.bounds,
            Bounds::new(
                point(viewport.right() - px(11.), viewport.top() + px(9.)),
                size(px(8.), px(108.))
            )
            .scale(scale)
        );
        assert_eq!(
            vertical_thumb.bounds.size,
            size(px(8.), px(30.)).scale(scale)
        );
        assert_eq!(
            horizontal_track.bounds,
            Bounds::new(
                point(viewport.left() + px(8.), viewport.bottom() - px(10.)),
                size(px(144.), px(6.))
            )
            .scale(scale)
        );
        let maximum = (px(144.) * 0.35).scale(scale).as_f32();
        assert!((horizontal_thumb.bounds.size.width.as_f32() - maximum).abs() <= 1.);
        assert_eq!(horizontal_thumb.bounds.size.height, px(6.).scale(scale));
        assert_eq!(
            vertical_track.content_mask.bounds,
            vertical_track.bounds.intersect(&viewport.scale(scale))
        );
        assert_eq!(vertical_thumb.corner_radii.top_left, px(4.).scale(scale));
        view.read(cx).outside.clone().focus(window, cx);
        (
            point(viewport.right() - px(7.), viewport.top() + px(16.)),
            point(viewport.left() + px(16.), viewport.bottom() - px(7.)),
        )
    });
    draw(cx);
    cx.update(|window, cx| {
        let content = find(window, &[], &"viewport-content".into()).unwrap();
        assert_eq!(content.role(), Some(Role::Button));
        assert_eq!(content.label(), Some("Viewport content"));
        assert!(view.read(cx).outside.is_focused(window));
        window.focus_next(cx);
        assert!(view.read(cx).target.is_focused(window));
        window.focus_next(cx);
        assert!(view.read(cx).outside.is_focused(window));
    });
    cx.simulate_click(vertical_at, Modifiers::default());
    wheel(cx, vertical_at, point(px(0.), px(-40.)));
    cx.simulate_click(horizontal_at, Modifiers::default());
    wheel(cx, horizontal_at, point(px(-50.), px(0.)));
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert_eq!(view.clicks, 2);
        assert!(view.target.is_focused(window));
        assert_eq!(view.scroll.offset(), point(px(-50.), px(-40.)));
        assert!(
            quad(window, VERTICAL_THUMB).unwrap().bounds.top()
                > quad(window, VERTICAL_TRACK).unwrap().bounds.top()
        );
    });
    let drag_end = vertical_at + point(px(0.), px(55.));
    cx.simulate_mouse_down(vertical_at, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(drag_end, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(drag_end, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).scroll.offset(), point(px(-50.), px(-40.))));

    cx.update(|window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        view.update(cx, |view, cx| {
            view.viewport = size(px(190.), px(150.));
            view.extent = size(px(400.), px(450.));
            cx.notify();
        });
    });
    draw(cx);
    let (rtl_vertical, rtl_horizontal, clicks) = cx.update(|window, cx| {
        let viewport = view.read(cx).scroll.bounds();
        let scale = window.scale_factor();
        assert_eq!(viewport.size, size(px(190.), px(150.)));
        let vertical = quad(window, VERTICAL_TRACK).unwrap();
        assert_eq!(
            vertical.bounds.left(),
            (viewport.left() + px(7.)).scale(scale)
        );
        assert_eq!(
            quad(window, VERTICAL_THUMB).unwrap().bounds.size.height,
            px(46.).scale(scale)
        );
        let horizontal = quad(window, HORIZONTAL_THUMB).unwrap();
        let expected_left =
            viewport.left() + px(8.) + (px(174.) - px(174.) * 0.35) * (1. - 50. / 210.);
        assert!(
            (horizontal.bounds.left().as_f32() - expected_left.scale(scale).as_f32()).abs() <= 1.
        );
        (
            point(viewport.left() + px(11.), viewport.top() + px(25.)),
            point(viewport.right() - px(15.), viewport.bottom() - px(7.)),
            view.read(cx).clicks,
        )
    });
    cx.simulate_click(rtl_vertical, Modifiers::default());
    wheel(cx, rtl_vertical, point(px(0.), px(-10.)));
    cx.simulate_click(rtl_horizontal, Modifiers::default());
    wheel(cx, rtl_horizontal, point(px(-10.), px(0.)));
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).clicks, clicks + 2);
        assert_eq!(view.read(cx).scroll.offset(), point(px(-60.), px(-50.)));
    });
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.extent = size(px(190.), px(150.));
            cx.notify();
        })
    });
    draw(cx);
    cx.update(|window, _| {
        assert!(quad(window, VERTICAL_TRACK).is_none());
        assert!(quad(window, HORIZONTAL_TRACK).is_none());
    });
}

#[gpui::test]
fn display_scrollbar_restarts_delay_and_settles_without_idle_frames(cx: &mut TestAppContext) {
    let (view, cx) = open(
        cx,
        DisplayScrollbarFade {
            enabled: true,
            delay: Duration::from_millis(120),
            duration: Duration::from_millis(240),
        },
        false,
    );
    let opacity = |cx: &mut VisualTestContext| {
        cx.update(|window, _| {
            quad(window, VERTICAL_THUMB)
                .and_then(|quad| quad.background.as_solid())
                .map_or(0., |tone| tone.a)
        })
    };
    let offset = |cx: &mut VisualTestContext, y| {
        cx.update(|_, cx| view.read(cx).scroll.set_offset(point(px(0.), px(y))));
        draw(cx);
    };
    assert_eq!(opacity(cx), 0.);
    assert_eq!(frame_callbacks(cx), 0);
    offset(cx, -30.);
    assert_eq!(opacity(cx), 1.);
    assert_eq!(frame_callbacks(cx), 0);
    cx.executor().advance_clock(Duration::from_millis(100));
    offset(cx, -60.);
    cx.executor().advance_clock(Duration::from_millis(119));
    draw(cx);
    assert_eq!(opacity(cx), 1.);
    assert_eq!(frame_callbacks(cx), 0);
    cx.executor().advance_clock(Duration::from_millis(1));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(frame_callbacks(cx) > 0);
    cx.executor().advance_clock(Duration::from_millis(110));
    draw(cx);
    assert!((0. ..1.).contains(&opacity(cx)));
    assert!(frame_callbacks(cx) > 0);
    offset(cx, -90.);
    assert_eq!(opacity(cx), 1.);
    cx.executor().advance_clock(Duration::from_millis(360));
    draw(cx);
    assert_eq!(opacity(cx), 0.);
    frame_callbacks(cx);
    draw(cx);
    assert_eq!(frame_callbacks(cx), 0);

    cx.update(|_, cx| cx.set_reduce_motion(true));
    offset(cx, -120.);
    assert_eq!(opacity(cx), 1.);
    cx.executor().advance_clock(Duration::from_millis(120));
    draw(cx);
    assert_eq!(opacity(cx), 0.);
    assert_eq!(frame_callbacks(cx), 0);

    offset(cx, -150.);
    assert_eq!(opacity(cx), 1.);
    let notifications = Rc::new(Cell::new(0));
    let watch = notifications.clone();
    let _observation = cx.update(|_, cx| cx.observe(&view, move |_, _| watch.set(watch.get() + 1)));
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.show_vertical = false;
            cx.notify();
        })
    });
    draw(cx);
    let at_unmount = notifications.get();
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
    assert_eq!(notifications.get(), at_unmount);
    assert_eq!(opacity(cx), 0.);
    assert_eq!(frame_callbacks(cx), 0);
}
