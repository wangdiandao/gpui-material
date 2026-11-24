use std::f32::consts::TAU;

use gpui::{
    App, BorderStyle, Bounds, ElementId, Hsla, IntoElement, PathBuilder, PathStyle, Pixels, Point,
    StrokeOptions, Styled, Window, canvas, point, px, quad, size,
};
use gpui_base::motion::Easing;
use gpui_material_foundation::{LoopClock, is_bounds_visible};

const EMPHASIZED_DECELERATE: Easing = Easing::CubicBezier {
    x1: 0.05,
    y1: 0.7,
    x2: 0.1,
    y2: 1.0,
};
const STANDARD: Easing = Easing::CubicBezier {
    x1: 0.2,
    y1: 0.0,
    x2: 0.0,
    y2: 1.0,
};

const REDUCED_MOTION_ARC: (f32, f32) = (0.0, 0.72);
const CYCLE_MS: f32 = 6000.;
const SAMPLES_PER_TURN: f32 = 64.;

struct SpinnerState {
    clock: LoopClock,
    visible: bool,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self {
            clock: LoopClock::default(),
            visible: true,
        }
    }
}

pub(super) fn spinner_arc(elapsed_ms: f32, motionless: bool) -> (f32, f32) {
    if motionless || !elapsed_ms.is_finite() {
        return REDUCED_MOTION_ARC;
    }
    let clock = elapsed_ms.rem_euclid(CYCLE_MS);
    let step = (clock / 1500.).floor();
    let turn = EMPHASIZED_DECELERATE.sample(((clock % 1500.) / 300.).clamp(0., 1.));
    let phase = clock / 3000.;
    let sweep = if phase < 1. {
        0.12 + (0.78 - 0.12) * phase
    } else {
        0.78 - (0.78 - 0.12) * STANDARD.sample((phase - 1.).clamp(0., 1.))
    };
    let start = (clock / CYCLE_MS) * 3. + (step + turn) / 4.;
    (start, sweep)
}

fn arc_point(center: Point<Pixels>, radius: f32, turn: f32) -> Point<Pixels> {
    let angle = turn * TAU - std::f32::consts::FRAC_PI_2;
    let (sin, cos) = angle.sin_cos();
    point(center.x + px(radius * cos), center.y + px(radius * sin))
}

fn paint_round_cap(center: Point<Pixels>, diameter: f32, color: Hsla, window: &mut Window) {
    let half = px(diameter * 0.5);
    let bounds = Bounds::new(
        point(center.x - half, center.y - half),
        size(px(diameter), px(diameter)),
    );
    window.paint_quad(quad(
        bounds,
        half,
        color,
        px(0.),
        gpui::transparent_black(),
        BorderStyle::Solid,
    ));
}

pub(super) fn render_loading_spinner(
    id: &ElementId,
    diameter: f32,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state((id.clone(), "loading-spinner-clock"), cx, |_, _| {
        SpinnerState::default()
    });
    let reduced = cx.reduce_motion();
    let animate_loop = window.is_window_active() && window.is_visible() && !reduced;
    let now = cx.background_executor().now();
    let elapsed = state.update(cx, |state, _| {
        state
            .clock
            .advance(now, animate_loop && state.visible, reduced)
    });
    let motionless = reduced || (!animate_loop && elapsed.is_zero());
    let (start, sweep) = spinner_arc(elapsed.as_secs_f32() * 1000., motionless);
    let diameter = diameter.max(12.);

    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            let visible = width.is_finite()
                && height.is_finite()
                && width > 0.
                && height > 0.
                && is_bounds_visible(bounds, window);
            let now = cx.background_executor().now();
            state.update(cx, |state, _| {
                if state.visible != visible {
                    state.visible = visible;
                    if !visible {
                        state.clock.pause(now);
                    }
                }
            });
            if !visible || tint.a <= 0. {
                return;
            }
            let side = width.min(height);
            let stroke_width = (side * 0.12).clamp(2., 4.).min(side * 0.45);
            let radius = (side - stroke_width) * 0.5;
            if radius <= 0. {
                return;
            }
            let center = bounds.center();
            let steps = ((sweep * SAMPLES_PER_TURN).ceil() as usize).max(4);
            let step = sweep / steps as f32;
            let first = arc_point(center, radius, start);
            let last = arc_point(center, radius, start + sweep);
            let mut path = PathBuilder::default().with_style(PathStyle::Stroke(StrokeOptions {
                line_width: stroke_width,
                miter_limit: 1.,
                ..StrokeOptions::default()
            }));
            path.move_to(first);
            for ix in 1..=steps {
                path.line_to(arc_point(center, radius, start + step * ix as f32));
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, tint);
            }
            paint_round_cap(first, stroke_width, tint, window);
            paint_round_cap(last, stroke_width, tint, window);

            if animate_loop {
                window.request_animation_frame();
            }
        },
    )
    .size(px(diameter))
    .flex_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_arc_returns_static_fallback_under_reduced_motion_and_cycles_smoothly() {
        assert_eq!(spinner_arc(1234., true), REDUCED_MOTION_ARC);
        assert_eq!(spinner_arc(f32::NAN, false), REDUCED_MOTION_ARC);

        let (s0, l0) = spinner_arc(0., false);
        let (s_mid, l_mid) = spinner_arc(3000., false);
        let (s_wrap, l_wrap) = spinner_arc(CYCLE_MS, false);
        assert!((l0 - 0.12).abs() < 1e-4);
        assert!((l_mid - 0.78).abs() < 1e-4);
        assert!(s_mid > s0);
        assert!((s_wrap - s0).abs() < 1e-4);
        assert!((l_wrap - l0).abs() < 1e-4);
    }
}
