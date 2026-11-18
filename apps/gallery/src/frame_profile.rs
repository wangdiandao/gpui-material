use std::time::Duration;

use gpui::{
    App,
    profiler::{FrameEvent, FrameTimingCollector, set_trace_enabled},
};

pub(crate) fn capture(cx: &App) {
    let executor = cx.background_executor().clone();
    cx.spawn(async move |_| {
        executor.timer(Duration::from_secs(2)).await;
        let mut collector = FrameTimingCollector::new();
        let enabled_here = set_trace_enabled(true);
        println!("Frame profile started (10 seconds)");
        executor.timer(Duration::from_secs(10)).await;
        let events = collector.collect_unseen();
        if enabled_here {
            set_trace_enabled(false);
        }

        let mut draws = Vec::new();
        let mut submissions = Vec::new();
        for event in events {
            match event {
                FrameEvent::Draw(frame) => draws.push(frame.draw_end - frame.draw_start),
                FrameEvent::Present(frame) => submissions.extend(frame.animation_interval),
            }
        }
        report("CPU draw", &mut draws);
        report("Active frame submission interval", &mut submissions);
    })
    .detach();
}

fn report(label: &str, samples: &mut [Duration]) {
    let count = samples.len();
    if count == 0 {
        println!("{label}: 0 samples");
        return;
    }
    samples.sort_unstable();
    let milliseconds =
        |percentile: usize| samples[(count * percentile).div_ceil(100) - 1].as_secs_f64() * 1_000.;
    println!(
        "{label}: {count} samples, p50 {:.3} ms, p95 {:.3} ms, p99 {:.3} ms",
        milliseconds(50),
        milliseconds(95),
        milliseconds(99)
    );
}
