use std::{cell::RefCell, rc::Rc};

use gpui::{
    ElementId, Modifiers, MouseButton, Pixels, Role, Subscription, TestAppContext, TouchDragEvent,
    TouchPhase, VisualTestContext, rgb,
};
use gpui_base::test_support::find;
use gpui_material::{
    InteractionState, MaterialRoot, MaterialTheme, SliderEvent, SliderMarker, SliderValue,
    foundation::set_window_material_theme, theme::Direction,
};

use super::*;

const TICK: u32 = 0x069371;
const DISABLED_TICK: u32 = 0x9c5726;
const ACTIVE: u32 = 0x114477;
const START: u32 = 0x57113c;
const END: u32 = 0x97601b;
const VERTICAL_TRACK: u32 = 0x553a88;

struct Preview {
    range: Entity<SliderState>,
    normal: Entity<SliderState>,
    reversed: Entity<SliderState>,
    sources: [Entity<InteractionSource>; 2],
    events: Events,
    log: Rc<RefCell<PaintLog>>,
    disabled: bool,
    _subscription: Subscription,
}

#[derive(Default)]
struct PaintLog {
    ticks: Vec<SliderMarker>,
    stops: Vec<SliderMarker>,
    labels: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Events {
    changes: usize,
    releases: usize,
    latest_change: Option<SliderValue>,
}

impl Preview {
    fn new(cx: &mut Context<Self>) -> Self {
        let range = cx.new(|_| SliderState::new().max(10.).step(1.).default_value(4.0..5.0));
        let subscription = cx.subscribe(&range, |this, _, event, _| match event {
            SliderEvent::Change(value) => {
                this.events.changes += 1;
                this.events.latest_change = Some(*value);
            }
            SliderEvent::Release(_) => this.events.releases += 1,
        });
        Self {
            range,
            normal: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.)),
            reversed: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.)),
            sources: std::array::from_fn(|_| cx.new(|_| InteractionSource::new())),
            events: Events::default(),
            log: Default::default(),
            disabled: false,
            _subscription: subscription,
        }
    }
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ticks = self.log.clone();
        let stops = self.log.clone();
        let labels = self.log.clone();
        div()
            .w(px(400.))
            .child(
                Slider::range("test-range", "Lowest", "Highest", &self.range)
                    .track_style(SliderTrackStyle {
                        thickness: Some(px(18.)),
                        outer_radius: Some(px(3.)),
                        inner_radius: Some(px(1.)),
                        thumb_gap: Some(px(0.)),
                    })
                    .thumb_style(
                        RangeThumb::Start,
                        SliderThumbStyle {
                            size: Some(size(px(40.), px(58.))),
                            pressed_width: Some(px(26.)),
                            ..Default::default()
                        },
                    )
                    .thumb_style(
                        RangeThumb::End,
                        SliderThumbStyle {
                            size: Some(size(px(12.), px(30.))),
                            pressed_width: Some(px(8.)),
                            ..Default::default()
                        },
                    )
                    .thumb(RangeThumb::Start, div().size_full().bg(rgb(START)))
                    .thumb(RangeThumb::End, div().size_full().bg(rgb(END)))
                    .colors(SliderColors {
                        active_track: Some(rgb(ACTIVE).into()),
                        active_tick: Some(rgb(TICK).into()),
                        inactive_tick: Some(rgb(TICK).into()),
                        disabled_active_tick: Some(rgb(DISABLED_TICK).into()),
                        disabled_inactive_tick: Some(rgb(DISABLED_TICK).into()),
                        ..Default::default()
                    })
                    .disabled(self.disabled)
                    .tick_intervals(10)
                    .draw_ticks(move |marker, _, _| ticks.borrow_mut().ticks.push(marker))
                    .show_stop_indicators(!self.disabled)
                    .draw_stop_indicators(move |marker, _, _| stops.borrow_mut().stops.push(marker))
                    .format_value(move |value| {
                        labels.borrow_mut().labels.push(value);
                        format!("{value:.0} steps").into()
                    })
                    .interaction_source(RangeThumb::Start, &self.sources[0])
                    .interaction_source(RangeThumb::End, &self.sources[1]),
            )
            .child(
                div()
                    .flex()
                    .items_start()
                    .child(Slider::new("normal", "Normal level", &self.normal).axis(Axis::Vertical))
                    .child(
                        Slider::new("reversed", "Reversed level", &self.reversed)
                            .axis(Axis::Vertical)
                            .reverse_vertical(true)
                            .track_style(SliderTrackStyle {
                                thickness: Some(px(14.)),
                                ..Default::default()
                            })
                            .track(div().size_full().bg(rgb(VERTICAL_TRACK)))
                            .value_indicator(false)
                            .format_value(|_| panic!("disabled indicator must not format a value")),
                    ),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Preview::new(cx)
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

fn id(slider: &'static str, thumb: &'static str) -> ElementId {
    (ElementId::from(slider), thumb).into()
}

fn thumb(cx: &mut VisualTestContext, slider: &'static str, name: &'static str) -> Bounds<Pixels> {
    cx.update(|window, _| find(window, &[], &id(slider, name)).unwrap().bounds())
}

fn snapshot(cx: &mut VisualTestContext, view: &Entity<Preview>, index: usize) -> InteractionState {
    cx.update(|window, cx| {
        let label = ["start", "end"][index];
        view.read(cx).sources[index]
            .read(cx)
            .state(window.window_handle().window_id(), &id("test-range", label))
            .unwrap()
    })
}

fn assert_range(view: &Entity<Preview>, end_points: (f32, f32), cx: &mut VisualTestContext) {
    cx.update(|_, cx| assert_eq!(view.read(cx).range.read(cx).value(), end_points.into()));
}

fn range_events(view: &Entity<Preview>, cx: &mut VisualTestContext) -> Events {
    cx.update(|_, cx| view.read(cx).events)
}

#[gpui::test]
fn slider_styles_separate_handles_and_clear_pointer_and_touch_sources(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    let start = thumb(cx, "test-range", "start");
    let end = thumb(cx, "test-range", "end");
    assert_eq!(start.size, size(px(40.), px(58.)));
    assert_eq!(end.size, size(px(12.), px(30.)));
    assert_eq!(start.center().y, end.center().y);
    cx.update(|window, cx| {
        for (name, label) in [("start", "Lowest"), ("end", "Highest")] {
            let option = find(window, &[], &id("test-range", name)).unwrap();
            assert_eq!(option.role(), Some(Role::Slider));
            assert_eq!(option.label(), Some(label));
        }
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        for (tint, size) in [(START, start.size), (END, end.size)] {
            assert!(
                quads.iter().any(|quad| quad.background == rgb(tint).into()
                    && quad.bounds.size == size.scale(scale))
            );
        }
        let log = view.read(cx).log.borrow();
        assert!(!log.ticks.is_empty());
        assert!(
            log.ticks
                .iter()
                .all(|tick| tick.diameter == px(4.) && tick.color == rgb(TICK).into())
        );
        assert!(
            log.stops
                .iter()
                .any(|stop| stop.color == rgb(ACTIVE).into())
        );
        assert!(log.labels.is_empty());
    });
    // The pointer lies inside only the wider start artwork, even though the
    // end value is closer. Its value clamps at the end without swapping focus.
    let edge = start.center() + point(px(19.), px(0.));
    cx.simulate_mouse_down(edge, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert!(snapshot(cx, &view, 0).pressed);
    assert!(!snapshot(cx, &view, 1).pressed);
    assert_range(&view, (5., 5.), cx);
    assert_eq!(thumb(cx, "test-range", "start").size.width, px(26.));
    cx.simulate_mouse_move(
        edge - point(px(70.), px(0.)),
        MouseButton::Left,
        Modifiers::default(),
    );
    draw(cx);
    assert!(snapshot(cx, &view, 0).dragged);
    assert!(!snapshot(cx, &view, 1).dragged);
    cx.simulate_keystrokes("escape");
    draw(cx);
    assert!(!snapshot(cx, &view, 0).pressed);
    assert!(!snapshot(cx, &view, 0).dragged);
    assert_eq!(range_events(&view, cx).releases, 1);
    cx.simulate_mouse_up(edge, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_eq!(range_events(&view, cx).releases, 1);

    let origin = thumb(cx, "test-range", "end").center();
    let touch = |cx: &mut VisualTestContext, phase, offset: f32| {
        cx.simulate_event(TouchDragEvent {
            phase,
            start_position: origin,
            position: origin + point(px(offset), px(0.)),
        });
        draw(cx);
    };
    touch(cx, TouchPhase::Started, 10.);
    touch(cx, TouchPhase::Moved, 55.);
    assert!(snapshot(cx, &view, 1).dragged);
    assert!(!snapshot(cx, &view, 0).pressed);
    touch(cx, TouchPhase::Cancelled, 55.);
    assert!(!snapshot(cx, &view, 1).pressed);
    assert!(!snapshot(cx, &view, 1).dragged);
    let events = range_events(&view, cx);
    assert_eq!(events.releases, 2);
    touch(cx, TouchPhase::Ended, 90.);
    assert_eq!(range_events(&view, cx), events);
    cx.update(|_, cx| {
        view.update(cx, |preview, cx| {
            assert!(preview.log.borrow().labels.contains(&5.));
            *preview.log.borrow_mut() = PaintLog::default();
            preview.disabled = true;
            cx.notify();
        });
    });
    draw(cx);
    assert!(snapshot(cx, &view, 0).disabled && snapshot(cx, &view, 1).disabled);
    cx.update(|_, cx| {
        let log = view.read(cx).log.borrow();
        assert!(!log.ticks.is_empty());
        assert!(
            log.ticks
                .iter()
                .all(|tick| tick.color == rgb(DISABLED_TICK).into())
        );
        assert!(log.stops.is_empty() && log.labels.is_empty());
    });
}

#[gpui::test]
fn slider_reversal_mirrors_geometry_and_physical_keys(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    cx.update(|window, cx| {
        let reversed = view.read(cx).reversed.read(cx).bounds();
        let normal = view.read(cx).normal.read(cx).bounds();
        let increasing_down = find(window, &[], &id("reversed", "end")).unwrap();
        assert_eq!(increasing_down.role(), Some(Role::Slider));
        assert_eq!(increasing_down.label(), Some("Reversed level"));
        assert!(
            (increasing_down.bounds().center().y - (reversed.top() + reversed.size.height * 0.3))
                .abs()
                < px(1.)
        );
        let increasing_up = find(window, &[], &id("normal", "end")).unwrap();
        assert!(
            (increasing_up.bounds().center().y - (normal.bottom() - normal.size.height * 0.3))
                .abs()
                < px(1.)
        );
        let scale = window.scale_factor();
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == rgb(VERTICAL_TRACK).into()
                    && quad.bounds.size == size(px(14.), reversed.size.height).scale(scale))
        );
    });
    let at = cx.update(|_, cx| {
        let bounds = view.read(cx).reversed.read(cx).bounds();
        point(bounds.center().x, bounds.top() + bounds.size.height * 0.64)
    });
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    draw(cx);
    let value =
        |cx: &mut VisualTestContext| cx.update(|_, cx| view.read(cx).reversed.read(cx).value());
    assert_eq!(value(cx), SliderValue::Single(6.));
    for (key, expected) in [("down", 7.), ("up", 6.)] {
        cx.simulate_keystrokes(key);
        draw(cx);
        assert_eq!(value(cx), SliderValue::Single(expected));
    }

    cx.update(|window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    let start = thumb(cx, "test-range", "start").center();
    assert!(start.x > thumb(cx, "test-range", "end").center().x);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(start, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_range(&view, (4., 5.), cx);
    let before = range_events(&view, cx);
    cx.simulate_keystrokes("left");
    draw(cx);
    assert_range(&view, (5., 5.), cx);
    let movement = range_events(&view, cx);
    assert_eq!(movement.changes, before.changes + 1);
    assert_eq!(movement.latest_change, Some((5., 5.).into()));
    assert_eq!(movement.releases, before.releases + 1);
    cx.simulate_keystrokes("left");
    draw(cx);
    assert_eq!(range_events(&view, cx), movement);
    assert!(snapshot(cx, &view, 0).focus_visible);
    assert!(!snapshot(cx, &view, 1).focus_visible);
}
