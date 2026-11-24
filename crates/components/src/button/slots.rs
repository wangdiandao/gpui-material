use std::{cell::RefCell, f32::consts::TAU, rc::Rc};

use gpui::{
    AnyElement, App, BorderStyle, Bounds, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, PathBuilder, PathStyle, Pixels, Point, SharedString, StrokeOptions, Styled,
    Window, canvas, div, point, px, quad, size,
};
use gpui_base::{TestSupportExt, motion::Easing};
use gpui_material_foundation::{
    LoopClock, current_material_theme, is_bounds_visible, resolve_edges,
};
use gpui_material_theme::{Direction, LogicalEdges, TypographyEmphasis};

use crate::action::icon_slot;
use crate::control_style::finite_length;

use super::types::Metrics;

const MIN_COMPRESSED_HORIZONTAL_PADDING: f32 = 4.;
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

struct SpinnerClock {
    clock: LoopClock,
    visible: bool,
}

impl Default for SpinnerClock {
    fn default() -> Self {
        Self {
            clock: LoopClock::default(),
            visible: true,
        }
    }
}

fn spinner_arc(elapsed_ms: f32, motionless: bool) -> (f32, f32) {
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

fn render_loading_spinner(
    id: &ElementId,
    diameter: f32,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let clock = window
        .use_keyed_state((id.clone(), "loading-spinner-clock"), cx, |_, _| {
            Rc::new(RefCell::new(SpinnerClock::default()))
        })
        .read(cx)
        .clone();
    let reduced = cx.reduce_motion();
    let animate_loop = window.is_window_active() && window.is_visible() && !reduced;
    let now = cx.background_executor().now();
    let elapsed = {
        let mut guard = clock.borrow_mut();
        let active = animate_loop && guard.visible;
        guard.clock.advance(now, active, reduced)
    };
    let motionless = reduced || (!animate_loop && elapsed.is_zero());
    let (start, sweep) = spinner_arc(elapsed.as_secs_f32() * 1000., motionless);
    let diameter = diameter.max(12.);

    canvas(
        move |bounds, window, cx| {
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            let visible = width.is_finite()
                && height.is_finite()
                && width > 0.
                && height > 0.
                && is_bounds_visible(bounds, window);
            let mut guard = clock.borrow_mut();
            if guard.visible != visible {
                guard.visible = visible;
                if !visible {
                    guard.clock.pause(cx.background_executor().now());
                }
            }
            visible
        },
        move |bounds, visible, window, _| {
            if !visible || tint.a <= 0. {
                return;
            }
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
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

pub(super) fn loading_overlay(
    id: &ElementId,
    resting_content: AnyElement,
    icon_size: f32,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .child(div().opacity(0.).child(resting_content))
        .child(
            div()
                .id((id.clone(), "loading-spinner"))
                .test_support()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .child(render_loading_spinner(id, icon_size, tint, window, cx)),
        )
        .into_any_element()
}

pub(super) fn loading_icon_spinner(
    id: &ElementId,
    icon_size: f32,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .id((id.clone(), "loading-spinner"))
        .test_support()
        .flex()
        .items_center()
        .justify_center()
        .child(render_loading_spinner(id, icon_size, tint, window, cx))
        .into_any_element()
}

fn compress_horizontal_padding(
    left: Pixels,
    right: Pixels,
    width: Pixels,
    icon_extent: Pixels,
    natural_label_width: Pixels,
) -> (Pixels, Pixels, Pixels) {
    let mut left_pad = finite_length(left);
    let mut right_pad = finite_length(right);
    let available = (width - left_pad - right_pad - icon_extent).max(px(0.));
    let deficit = (natural_label_width - available).max(px(0.));
    if deficit > px(0.) {
        let floor_left = left_pad.min(px(MIN_COMPRESSED_HORIZONTAL_PADDING));
        let floor_right = right_pad.min(px(MIN_COMPRESSED_HORIZONTAL_PADDING));
        let slack_left = (left_pad - floor_left).max(px(0.));
        let slack_right = (right_pad - floor_right).max(px(0.));
        let total_slack = slack_left + slack_right;
        let absorb = deficit.min(total_slack);
        if absorb > px(0.) && total_slack > px(0.) {
            let left_cut = absorb * (f32::from(slack_left) / f32::from(total_slack));
            let right_cut = absorb - left_cut;
            left_pad -= left_cut;
            right_pad -= right_cut;
        }
    }
    let label_width = (width - left_pad - right_pad - icon_extent).max(px(0.));
    (left_pad, right_pad, label_width)
}

#[derive(Default)]
pub(super) struct Slots {
    pub(super) leading: Option<AnyElement>,
    pub(super) trailing: Option<AnyElement>,
    pub(super) content: Option<AnyElement>,
    pub(super) padding: Option<LogicalEdges>,
    pub(super) width: Option<Pixels>,
}

impl Slots {
    pub(super) fn render(
        self,
        label: SharedString,
        metrics: Metrics,
        scale: f32,
        window: &Window,
        tint: Hsla,
        cx: &App,
    ) -> AnyElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let typography = metrics.label.style(TypographyEmphasis::Baseline);
        let max_padding = self.width.map_or_else(
            || f32::from(window.viewport_size().width) * 0.125,
            |width| (f32::from(width) - 16.).max(0.) / 2.,
        );
        let horizontal = metrics.padding.min(max_padding);
        let edges = resolve_edges(
            self.padding
                .unwrap_or_else(|| LogicalEdges::symmetric(horizontal, 4.)),
            theme.layout.direction,
        );
        let (mut left_pad, mut right_pad) = (edges.left, edges.right);
        let icons = usize::from(self.leading.is_some()) + usize::from(self.trailing.is_some());
        let icon_extent = px((metrics.icon + metrics.gap) * icons as f32);
        let mut shaped_label = None;
        let label_width = self.width.map(|width| {
            if self.content.is_none()
                && let Some(shaped) = crate::text::shape_single_line(
                    label.clone(),
                    typography,
                    scale,
                    tint,
                    window,
                    cx,
                )
            {
                let natural_width = if scale.is_finite() && scale > 0. {
                    shaped.1.line().width.ceil() / scale
                } else {
                    shaped.1.line().width.ceil()
                };
                let (next_left, next_right, next_label_width) = compress_horizontal_padding(
                    left_pad,
                    right_pad,
                    width,
                    icon_extent,
                    natural_width,
                );
                left_pad = next_left;
                right_pad = next_right;
                shaped_label = Some(shaped);
                next_label_width
            } else {
                (width - left_pad - right_pad - icon_extent).max(px(0.))
            }
        });
        let row_typography = theme.typography.scale_style(typography);
        let mut row = div()
            .flex()
            .items_center()
            .justify_center()
            .min_w_0()
            .max_w_full()
            .gap(px(metrics.gap * scale))
            .pl(left_pad * scale)
            .pr(right_pad * scale)
            .pt(edges.top * scale)
            .pb(edges.bottom * scale)
            .text_size(px(row_typography.size * scale))
            .line_height(px(row_typography.line_height * scale));
        if direction == Direction::Rtl {
            row = row.flex_row_reverse();
        }
        if let Some(width) = self.width {
            row = row.w(width).overflow_hidden();
        }
        row.children(
            self.leading
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .child(self.content.unwrap_or_else(|| {
            if let Some(shaped) = shaped_label {
                crate::text::single_line_with_shaped(
                    shaped,
                    typography,
                    scale,
                    tint,
                    (label_width, 1., 1.),
                    window,
                    cx,
                )
            } else {
                crate::text::single_line(
                    label,
                    typography,
                    scale,
                    tint,
                    (label_width, 1., 1.),
                    window,
                    cx,
                )
            }
            .into_any_element()
        }))
        .children(
            self.trailing
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .into_any_element()
    }
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

    #[test]
    fn compressed_button_shrinks_horizontal_padding_before_truncating_label() {
        let (left, right, label_width) =
            compress_horizontal_padding(px(16.), px(16.), px(54.), px(0.), px(28.));
        assert_eq!(left, px(13.));
        assert_eq!(right, px(13.));
        assert_eq!(label_width, px(28.));

        let (left, right, label_width) =
            compress_horizontal_padding(px(16.), px(16.), px(72.), px(0.), px(28.));
        assert_eq!(left, px(16.));
        assert_eq!(right, px(16.));
        assert_eq!(label_width, px(40.));

        let (left, right, label_width) =
            compress_horizontal_padding(px(16.), px(16.), px(30.), px(0.), px(28.));
        assert_eq!(left, px(4.));
        assert_eq!(right, px(4.));
        assert_eq!(label_width, px(22.));

        let (left, right, label_width) =
            compress_horizontal_padding(px(2.), px(10.), px(30.), px(0.), px(28.));
        assert_eq!(left, px(2.));
        assert_eq!(right, px(4.));
        assert_eq!(label_width, px(24.));
    }
}
