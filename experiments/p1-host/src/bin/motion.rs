use std::time::Duration;

use gpui::{
    AppContext, Context, IntoElement, Render, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_base::{
    Button, Theme,
    motion::{Spring, Transition, spring, transition},
};
use p1_host::open_probe;

#[derive(Default)]
struct MotionProbe {
    expanded: bool,
    geometry: f32,
    opacity: f32,
    trace: Option<MotionTrace>,
}

#[derive(Default)]
struct MotionTrace {
    started: Option<std::time::Instant>,
    last_frame: Option<std::time::Instant>,
    interval_total_ms: f64,
    interval_count: u32,
}

impl Render for MotionProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_started = std::time::Instant::now();
        let colors = Theme::global(cx).tokens.colors;
        let width = if self.expanded { 280. } else { 80. };
        let alpha = if self.expanded { 1. } else { 0.35 };
        self.geometry = spring(
            ("motion", "width"),
            width,
            Spring::new(Duration::from_millis(320)).with_epsilon(0.1),
            window,
            cx,
        );
        self.opacity = transition(
            ("motion", "opacity"),
            alpha,
            Transition::new(Duration::from_millis(180)),
            window,
            cx,
        );

        let element = div()
            .size_full()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                Button::new("retarget")
                    .border_1()
                    .border_color(colors.border)
                    .px_3()
                    .py_2()
                    .focus_visible(|style| style.border_color(colors.ring))
                    .accessibility_label("切换宽度")
                    .child(if self.expanded { "收起" } else { "展开" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.expanded = !this.expanded;
                        if let Some(trace) = &mut this.trace {
                            *trace = MotionTrace {
                                started: Some(std::time::Instant::now()),
                                ..Default::default()
                            };
                        }
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .w(px(self.geometry))
                    .h_8()
                    .bg(colors.primary.opacity(self.opacity)),
            )
            .child(if cx.reduce_motion() {
                "减少动态效果已开启"
            } else {
                "减少动态效果已关闭"
            });

        if let Some(trace) = &mut self.trace
            && let Some(started) = trace.started
        {
            if let Some(previous) = trace.last_frame.replace(render_started) {
                trace.interval_total_ms +=
                    render_started.duration_since(previous).as_secs_f64() * 1000.;
                trace.interval_count += 1;
            }
            let settled = self.geometry == width && self.opacity == alpha;
            println!(
                "P1-06 t={:.2}ms render={:.3}ms width={:.2} opacity={:.3} settled={settled}",
                render_started.duration_since(started).as_secs_f64() * 1000.,
                render_started.elapsed().as_secs_f64() * 1000.,
                self.geometry,
                self.opacity
            );
            if settled {
                let mean = trace.interval_total_ms / f64::from(trace.interval_count.max(1));
                println!(
                    "P1-06 frames={} mean_interval={mean:.2}ms",
                    trace.interval_count + 1
                );
                trace.started = None;
            }
        }
        element
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        println!("P1-06 system reduce_motion={}", cx.reduce_motion());
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(480.), px(280.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-06 动效探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| {
            cx.new(|_| MotionProbe {
                trace: std::env::var_os("P1_MOTION_TRACE").map(|_| MotionTrace::default()),
                ..Default::default()
            })
        })
        .unwrap();
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn motion_retargets_and_stops_requesting_frames(cx: &mut gpui::TestAppContext) {
    let window = cx.open_window(size(px(480.), px(280.)), |_, _| MotionProbe::default());
    cx.run_until_parked();

    let sample = |cx: &mut gpui::TestAppContext, millis, target| {
        cx.executor().advance_clock(Duration::from_millis(millis));
        window
            .update(cx, |view, window, cx| {
                view.expanded = target;
                window.refresh();
                cx.notify();
            })
            .unwrap();
        cx.run_until_parked();
        window
            .update(cx, |view, _, _| (view.geometry, view.opacity))
            .unwrap()
    };

    assert_eq!(sample(cx, 0, true), (80., 0.35));
    let outbound = sample(cx, 60, true);
    let reversal = sample(cx, 0, false);
    assert_eq!(outbound, reversal);

    let reversed = sample(cx, 16, false);
    println!("outbound={outbound:?}, after reversal={reversed:?}");
    assert!(reversed.0 > outbound.0);
    assert!(reversed.1 < outbound.1);
    assert_eq!(sample(cx, 0, true), reversed);

    assert_eq!(sample(cx, 1000, true), (280., 1.));
    // Drain callbacks requested by earlier active frames before measuring idle.
    window
        .update(cx, |_, window, cx| window.simulate_next_frame(cx))
        .unwrap();
    cx.run_until_parked();
    let pending = window
        .update(cx, |_, window, cx| window.simulate_next_frame(cx))
        .unwrap();
    assert_eq!(pending, 0);

    cx.update(|cx| cx.set_reduce_motion(true));
    assert_eq!(sample(cx, 0, false), (80., 0.35));
}
