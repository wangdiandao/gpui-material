use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AppContext, Bounds, Context, Hsla, IntoElement, ParentElement, Render, Role, Styled,
    TestAppContext, Window, div, point, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    CircularProgress, LinearProgress, MaterialRoot, ProgressCap, ProgressShape, ProgressStop,
    ProgressStroke,
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

fn ink(hex: u32) -> Hsla {
    rgb(hex).into()
}

const ACTIVE: u32 = 0x993b32;
const TRACK: u32 = 0xa9cdb8;
const MARKER: u32 = 0x36467f;
const WAVE: u32 = 0x803986;
const CIRCLE: u32 = 0x30687a;
const TINY: u32 = 0x809841;

struct Example {
    value: Option<f32>,
    speed: f32,
    stop_size: f32,
    stop: Rc<Cell<Option<ProgressStop>>>,
    wave_input: Rc<Cell<f32>>,
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let stop = self.stop.clone();
        let wave_input = self.wave_input.clone();
        let straight = LinearProgress::new("straight-line", "Transfer")
            .active_stroke(ProgressStroke {
                width: Some(px(8.)),
                color: Some(ink(ACTIVE)),
                cap: ProgressCap::Square,
            })
            .track_stroke(ProgressStroke {
                width: Some(px(2.)),
                color: Some(ink(TRACK)),
                cap: ProgressCap::Butt,
            })
            .gap(px(6.))
            .stop_size(px(self.stop_size))
            .draw_stop_indicator(move |marker, window, _| {
                stop.set(Some(marker));
                let larger = Bounds::new(
                    marker.bounds.origin - point(px(10.), px(10.)),
                    size(px(26.), px(26.)),
                );
                window.paint_quad(gpui::fill(larger, ink(MARKER)));
            });
        let wave = LinearProgress::new("wavy-line", "Wavy transfer")
            .shape(ProgressShape::Wavy)
            .active_stroke(ProgressStroke {
                width: Some(px(6.)),
                color: Some(ink(WAVE)),
                ..Default::default()
            })
            .wave_amplitude_by(move |progress| {
                wave_input.set(progress);
                if progress == 0. { 1. } else { progress }
            })
            .wavelength(px(18.))
            .wave_speed(px(self.speed))
            .stop_size(px(0.));
        let flat_circle =
            CircularProgress::new("flat-circle", "Sync").active_stroke(ProgressStroke {
                width: Some(px(6.)),
                color: Some(ink(CIRCLE)),
                ..Default::default()
            });
        let wavy_circle = CircularProgress::new("wavy-circle", "Wavy sync")
            .shape(ProgressShape::Wavy)
            .active_stroke(ProgressStroke {
                width: Some(px(7.)),
                color: Some(ink(CIRCLE)),
                ..Default::default()
            })
            .track_stroke(ProgressStroke {
                width: Some(px(2.)),
                cap: ProgressCap::Square,
                ..Default::default()
            })
            .gap(px(3.))
            .wave_amplitude(0.8)
            .wave_speed(px(self.speed))
            .wavelength(px(23.));
        let linear = |control: LinearProgress| match self.value {
            Some(value) => control.value(value),
            None => control.value(0.5).indeterminate(),
        };
        let circular = |control: CircularProgress| match self.value {
            Some(value) => control.value(value),
            None => control.value(0.5).indeterminate(),
        };
        div()
            .w(px(120.))
            .flex()
            .flex_col()
            .items_start()
            .gap_2()
            .child(linear(LinearProgress::new(
                "default-line",
                "Default transfer",
            )))
            .child(linear(wave))
            .child(linear(straight))
            .child(circular(flat_circle))
            .child(circular(wavy_circle))
            .child(
                div().w(px(2.)).child(
                    LinearProgress::new("tiny-line", "Tiny")
                        .value(1.0)
                        .stop_size(px(50.))
                        .active_stroke(ProgressStroke {
                            width: Some(px(32.)),
                            color: Some(ink(TINY)),
                            ..Default::default()
                        }),
                ),
            )
    }
}

fn draw(window: &mut Window, cx: &mut gpui::App) -> usize {
    window.simulate_next_frame(cx);
    window.refresh();
    window.draw(cx).clear(cx);
    window.simulate_next_frame(cx)
}

fn footprints(window: &Window) {
    for (id, label, height) in [
        ("default-line", "Default transfer", 4.),
        ("straight-line", "Transfer", 8.),
        ("wavy-line", "Wavy transfer", 12.),
        ("flat-circle", "Sync", 40.),
        ("wavy-circle", "Wavy sync", 48.),
    ] {
        let node = find(window, &[], &id.into()).unwrap();
        assert_eq!(node.role(), Some(Role::ProgressIndicator));
        assert_eq!(node.label(), Some(label));
        assert_eq!(node.focused(), None);
        assert_eq!(node.bounds().size.height, px(height));
    }
}

fn quads(window: &Window, tint: u32) -> Vec<gpui::Quad> {
    window
        .painted_quads()
        .into_iter()
        .filter(|quad| quad.background.as_solid() == Some(ink(tint)))
        .collect()
}

fn example(
    cx: &mut TestAppContext,
    reduced: bool,
) -> (gpui::AnyWindowHandle, gpui::Entity<Example>) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(reduced);
        let view = cx.new(|_| Example {
            value: Some(0.5),
            speed: 0.,
            stop_size: 6.,
            stop: Rc::new(Cell::new(None)),
            wave_input: Rc::new(Cell::new(0.)),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    (handle, view)
}

#[gpui::test]
fn facade_progress_paints_independent_strokes_and_clips_the_stop(cx: &mut TestAppContext) {
    let (handle, view) = example(cx, true);
    for direction in [Direction::Ltr, Direction::Rtl] {
        cx.update_window(handle, |_, window, cx| {
            let mut theme = window_material_theme(window, cx).clone();
            theme.layout.direction = direction;
            set_window_material_theme(Some(theme), window, cx);
            assert_eq!(draw(window, cx), 0);
            footprints(window);
            let node = find(window, &[], &"straight-line".into()).unwrap();
            let scale = window.scale_factor();
            let [active] = quads(window, ACTIVE).try_into().unwrap();
            let [track] = quads(window, TRACK).try_into().unwrap();
            let [marker] = quads(window, MARKER).try_into().unwrap();
            assert_eq!(
                active.bounds.size,
                size(px(64.).scale(scale), px(8.).scale(scale))
            );
            assert_eq!(track.bounds.size.height, px(2.).scale(scale));
            assert_eq!(active.corner_radii.top_left, px(0.).scale(scale));
            assert_eq!(track.corner_radii.top_left, px(0.).scale(scale));
            let gap = if direction == Direction::Ltr {
                track.bounds.left() - active.bounds.right()
            } else {
                active.bounds.left() - track.bounds.right()
            };
            assert_eq!(gap, px(6.).scale(scale));
            let stop = view.read(cx).stop.get().unwrap();
            assert_eq!(stop.direction, direction);
            assert_eq!(stop.cap, ProgressCap::Square);
            assert_eq!(stop.color, ink(ACTIVE));
            assert_eq!(stop.bounds.size, size(px(6.), px(6.)));
            assert_eq!(marker.content_mask.bounds, stop.bounds.scale(scale));
            if direction == Direction::Ltr {
                assert_eq!(stop.bounds.right(), node.bounds().right());
            } else {
                assert_eq!(stop.bounds.left(), node.bounds().left());
            }
            assert_eq!(
                quads(window, WAVE)[0].bounds.size.height,
                px(6.).scale(scale)
            );
            assert!(
                quads(window, CIRCLE)
                    .iter()
                    .any(|cap| cap.bounds.size.height == px(7.).scale(scale))
            );
            assert!(
                quads(window, TINY)
                    .iter()
                    .all(|cap| cap.bounds.size.width <= px(2.).scale(scale))
            );
        })
        .unwrap();
    }

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.value = Some(1.5);
            view.stop_size = 0.;
            view.stop.set(None);
            cx.notify();
        });
        assert_eq!(draw(window, cx), 0);
        assert!(view.read(cx).stop.get().is_none());
        assert!(quads(window, MARKER).is_empty());
        assert!(quads(window, TRACK).is_empty());
        assert!(
            quads(window, CIRCLE).is_empty(),
            "full circles close without painting end caps"
        );
        assert_eq!(
            quads(window, ACTIVE)[0].bounds.size.width,
            px(120.).scale(window.scale_factor())
        );

        view.update(cx, |view, cx| {
            view.value = Some(-0.5);
            cx.notify();
        });
        assert_eq!(draw(window, cx), 0);
        assert!(quads(window, ACTIVE).is_empty());
        assert!(quads(window, WAVE).is_empty());
        assert_eq!(
            quads(window, TRACK)[0].bounds.size.width,
            px(120.).scale(window.scale_factor())
        );
    })
    .unwrap();
}

#[gpui::test]
fn facade_progress_retargets_and_releases_animation_when_steady(cx: &mut TestAppContext) {
    let (handle, view) = example(cx, false);
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| draw(window, cx))
            .unwrap()
    };
    let edit = |cx: &mut TestAppContext, value, speed| {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.value = value;
                view.speed = speed;
                view.stop.set(None);
                cx.notify();
            })
        });
    };
    let sample = |cx: &mut TestAppContext| cx.update(|cx| view.read(cx).wave_input.get());
    assert_eq!(frame(cx), 0, "flat and paused wavy progress are idle");
    edit(cx, Some(0.5), 13.);
    assert_eq!(frame(cx), 2, "only the two measured waves request frames");
    cx.executor().advance_clock(Duration::from_millis(145));
    assert_eq!(frame(cx), 2);
    edit(cx, Some(0.5), 0.);
    assert_eq!(frame(cx), 0, "zero speed halts wave frames");

    edit(cx, Some(0.9), 0.);
    assert!(frame(cx) > 0);
    cx.executor().advance_clock(Duration::from_millis(40));
    frame(cx);
    let before = sample(cx);
    assert!(before > 0.5 && before < 0.9);
    edit(cx, Some(0.2), 0.);
    frame(cx);
    assert!((sample(cx) - before).abs() < 0.001);
    cx.executor().advance_clock(Duration::from_secs(2));
    frame(cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    assert_eq!(frame(cx), 0);
    assert_eq!(sample(cx), 0.2);

    for mode in [None, Some(f32::NAN), Some(f32::INFINITY)] {
        edit(cx, mode, 12.);
        assert!(frame(cx) >= 4);
        assert_eq!(sample(cx), 0.);
        assert!(cx.update(|cx| view.read(cx).stop.get().is_none()));
    }
    cx.update(|cx| cx.set_reduce_motion(true));
    assert_eq!(
        frame(cx),
        0,
        "indeterminate progress stays idle under reduced motion"
    );
    cx.executor().advance_clock(Duration::from_secs(1));
    assert_eq!(frame(cx), 0);
    edit(cx, Some(0.5), 12.);
    assert_eq!(
        frame(cx),
        0,
        "measured values remain responsive while reduced"
    );
    assert_eq!(sample(cx), 0.5);
    assert!(cx.update(|cx| view.read(cx).stop.get().is_some()));
}
