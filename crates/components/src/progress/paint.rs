use std::f32::consts::TAU;

use gpui::{
    App, Bounds, ContentMask, Hsla, PathBuilder, PathStyle, Pixels, Point, StrokeOptions, Window,
    point, px, size,
};
use gpui_material_theme::Direction;

use super::{
    CIRCULAR_AMPLITUDE, LINEAR_AMPLITUDE, MAX_WAVES, MIN_WAVELENGTH, ProgressCap, ProgressStop,
    ProgressStroke, STROKE_WIDTH, StopPainter, nonnegative_length,
};

#[derive(Clone, Copy)]
pub(super) struct Stroke {
    pub(super) width: f32,
    pub(super) color: Hsla,
    pub(super) cap: ProgressCap,
}

impl Stroke {
    pub(super) fn resolve(style: ProgressStroke, color: Hsla) -> Self {
        Self {
            width: style
                .width
                .and_then(nonnegative_length)
                .unwrap_or(STROKE_WIDTH),
            color: style.color.unwrap_or(color),
            cap: style.cap,
        }
    }

    fn fitted(self, max: f32) -> Self {
        Self {
            width: self.width.min(max),
            ..self
        }
    }

    fn cap_length(self) -> f32 {
        if self.cap == ProgressCap::Butt {
            0.
        } else {
            self.width / 2.
        }
    }

    fn paint_flat(
        self,
        (start, end): (f32, f32),
        sample: impl Fn(f32) -> Point<Pixels>,
        window: &mut Window,
    ) {
        if self.width == 0. || self.color.a == 0. || end <= start {
            return;
        }
        let first = sample(start);
        let last = sample(end);
        let extent = px(self.cap_length());
        let half_width = px(self.width / 2.);
        let bounds = Bounds::from_corners(
            point(first.x.min(last.x) - extent, first.y - half_width),
            point(first.x.max(last.x) + extent, first.y + half_width),
        );
        if bounds.size.width > px(0.) {
            paint_marker(bounds, self.color, self.cap, window);
        }
    }

    fn paint(
        self,
        range: (f32, f32),
        samples_per_turn: f32,
        closed: bool,
        sample: impl Fn(f32) -> Point<Pixels>,
        window: &mut Window,
    ) {
        let (start, end) = range;
        if self.width == 0. || self.color.a == 0. || end <= start {
            return;
        }
        let first = sample(start);
        if !closed && first == sample(end) {
            if self.cap != ProgressCap::Butt {
                paint_marker(
                    centered_square(first, self.width),
                    self.color,
                    self.cap,
                    window,
                );
            }
            return;
        }
        let steps = (((end - start) * samples_per_turn).ceil() as usize).max(1);
        let last = sample(end);
        let step = (end - start) / steps as f32;
        let mut path = PathBuilder::default().with_style(PathStyle::Stroke(StrokeOptions {
            line_width: self.width,
            miter_limit: 1.,
            ..StrokeOptions::default()
        }));
        let square = !closed && self.cap == ProgressCap::Square;
        path.move_to(if square {
            extend_endpoint(first, sample(start + step), self.cap_length())
        } else {
            first
        });
        let last_step = if closed { steps - 1 } else { steps };
        for index in 1..=last_step {
            let point = if index == steps && square {
                extend_endpoint(last, sample(end - step), self.cap_length())
            } else {
                sample(start + step * index as f32)
            };
            path.line_to(point);
        }
        if closed {
            path.close();
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, self.color);
        }
        if !closed && self.cap == ProgressCap::Round {
            for endpoint in [first, last] {
                paint_marker(
                    centered_square(endpoint, self.width),
                    self.color,
                    self.cap,
                    window,
                );
            }
        }
    }
}

fn extend_endpoint(endpoint: Point<Pixels>, neighbor: Point<Pixels>, radius: f32) -> Point<Pixels> {
    let difference = endpoint - neighbor;
    let distance = f32::from(difference.x).hypot(f32::from(difference.y));
    if distance > 0. {
        endpoint + difference * (radius / distance)
    } else {
        endpoint
    }
}

#[derive(Clone, Copy)]
pub(super) struct ProgressPaint {
    pub(super) active: Stroke,
    pub(super) track: Stroke,
    pub(super) gap: f32,
    pub(super) wavelength: f32,
    pub(super) amplitude: f32,
    pub(super) phase: f32,
}

impl ProgressPaint {
    pub(super) fn linear(
        self,
        bounds: Bounds<Pixels>,
        segments: [(f32, f32); 2],
        direction: Direction,
        window: &mut Window,
    ) {
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let active = self.active.fitted(height.min(width));
        let track = self.track.fitted(height.min(width));
        let amplitude = self.amplitude * ((height - active.width) / 2.).min(LINEAR_AMPLITUDE);
        let wavelength = self.wavelength.max(MIN_WAVELENGTH).max(width / MAX_WAVES);
        let axis = |x| logical_x(x, width, direction);
        let mid = bounds.center().y;
        let track_sample = |f| {
            let offset = axis(linear_position(width, track.cap_length(), f));
            point(bounds.left() + px(offset), mid)
        };
        let active_inset = if amplitude > 0. {
            active.width / 2.
        } else {
            active.cap_length()
        };
        let active_sample = |f| {
            let offset = linear_position(width, active_inset, f);
            let wave = wave_offset(amplitude, offset / wavelength, self.phase);
            point(bounds.left() + px(axis(offset)), mid + px(wave))
        };
        let gap = ((self.gap + active.cap_length() + track.cap_length()) / width).min(1.);
        let samples = if amplitude > 0. {
            (width / wavelength * 16.).max(100.)
        } else {
            1.
        };
        let mut cursor = 0.;
        for (start, end) in segments {
            if active.width > 0. && end > start {
                track.paint_flat((cursor, start - gap), track_sample, window);
                cursor = cursor.max(end + gap);
            }
        }
        track.paint_flat((cursor, 1.), track_sample, window);
        for segment in segments {
            if amplitude > 0. {
                active.paint(segment, samples, false, active_sample, window);
            } else {
                active.paint_flat(segment, active_sample, window);
            }
        }
    }

    pub(super) fn circular(
        self,
        bounds: Bounds<Pixels>,
        (start, extent): (f32, f32),
        wavy: bool,
        window: &mut Window,
    ) {
        let side = f32::from(bounds.size.width.min(bounds.size.height));
        let margin = if wavy { 4.0_f32.min(side / 2.) } else { 0. };
        let active = self.active.fitted((side - margin) / 2.);
        let track = self.track.fitted((side - margin) / 2.);
        let diameter = side
            - margin
            - active
                .width
                .max(track.width)
                .max(STROKE_WIDTH.min(side / 2.));
        let radius = diameter / 2.;
        if radius <= 0. {
            return;
        }
        let center = bounds.center();
        let amplitude = self.amplitude * CIRCULAR_AMPLITUDE.min(margin / 2.);
        let waves = circular_wave_count(radius, self.wavelength);
        let sample = |fraction: f32, amplitude: f32| {
            let arc = (fraction - 0.25) * TAU;
            let (sin, cos) = arc.sin_cos();
            let wave = wave_offset(amplitude, fraction * waves, self.phase);
            let radius = px(radius + wave);
            center + point(radius * cos, radius * sin)
        };
        let active_sample = |fraction| sample(fraction, amplitude);
        let track_sample = |fraction| sample(fraction, 0.);
        let samples = if amplitude > 0. {
            (waves * 16.).max(100.)
        } else {
            100.
        };
        if active.width == 0. || extent <= 0. {
            track.paint((0., 1.), 100., true, track_sample, window);
        } else if extent >= 1. {
            active.paint((0., 1.), samples, true, active_sample, window);
        } else {
            let gap =
                ((self.gap + active.cap_length() + track.cap_length()) / (radius * TAU)).min(1.);
            let end = start + extent;
            track.paint(
                (end + gap, start + 1. - gap),
                100.,
                false,
                track_sample,
                window,
            );
            active.paint((start, end), samples, false, active_sample, window);
        }
    }
}

fn linear_position(width: f32, inset: f32, fraction: f32) -> f32 {
    let inset = inset.min(width / 2.);
    (width * fraction).clamp(inset, width - inset)
}

fn logical_x(offset: f32, width: f32, direction: Direction) -> f32 {
    if direction == Direction::Rtl {
        width - offset
    } else {
        offset
    }
}

fn circular_wave_count(radius: f32, wavelength: f32) -> f32 {
    if !wavelength.is_finite() || wavelength <= 0. {
        return MAX_WAVES;
    }
    ((radius * TAU) / wavelength).round().clamp(1., MAX_WAVES)
}

fn wave_offset(amplitude: f32, cycles: f32, phase: f32) -> f32 {
    if !amplitude.is_finite() || !cycles.is_finite() || !phase.is_finite() {
        return 0.;
    }
    amplitude * ((cycles.rem_euclid(1.) - phase) * TAU).sin()
}

fn centered_square(center: Point<Pixels>, side: f32) -> Bounds<Pixels> {
    let side = px(side);
    Bounds::new(center - point(side / 2., side / 2.), size(side, side))
}

pub(super) fn stop_bounds(
    bounds: Bounds<Pixels>,
    requested: f32,
    wavy: bool,
    direction: Direction,
) -> Bounds<Pixels> {
    let width = f32::from(bounds.size.width);
    let side = requested.min(width).min(f32::from(bounds.size.height));
    let inset = (side / 2. + if wavy { LINEAR_AMPLITUDE } else { 0. }).min(width / 2.);
    let x = bounds.left() + px(logical_x(width - inset, width, direction));
    centered_square(point(x, bounds.center().y), side)
}

fn paint_marker(bounds: Bounds<Pixels>, color: Hsla, cap: ProgressCap, window: &mut Window) {
    let radius = if cap == ProgressCap::Round {
        bounds.size.height / 2.
    } else {
        px(0.)
    };
    window.paint_quad(gpui::quad(
        bounds,
        radius,
        color,
        px(0.),
        gpui::transparent_black(),
        gpui::BorderStyle::Solid,
    ));
}

pub(super) fn paint_stop(
    stop: ProgressStop,
    painter: Option<&StopPainter>,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(painter) = painter {
        window.with_content_mask(
            Some(ContentMask {
                bounds: stop.bounds,
            }),
            |window| {
                painter(stop, window, cx);
            },
        );
    } else {
        paint_marker(stop.bounds, stop.color, stop.cap, window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::{normalized_amplitude, wave_amplitude};

    #[test]
    fn rounded_progress_endpoints_and_stop_stay_inside_the_track() {
        assert_eq!(linear_position(100., 2., 0.), 2.);
        assert_eq!(linear_position(100., 2., 1.), 98.);
        assert_eq!(linear_position(100., 2., 0.5), 50.);
        assert_eq!(linear_position(1., 40., 0.), 0.5);
        assert_eq!(linear_position(1., 40., 1.), 0.5);
        for (width, height, wavy) in [(100., 4., false), (100., 10., true), (1., 2., false)] {
            let track = Bounds::new(point(px(0.), px(0.)), size(px(width), px(height)));
            let ltr = stop_bounds(track, 4., wavy, Direction::Ltr);
            let rtl = stop_bounds(track, 4., wavy, Direction::Rtl);
            assert_eq!(ltr.center().x + rtl.center().x, px(width));
            assert!(ltr.right() <= px(width) && rtl.left() >= px(0.));
            let oversized = stop_bounds(track, f32::MAX, wavy, Direction::Ltr);
            assert!(oversized.right() <= px(width) && oversized.size.height <= px(height));
        }
    }

    #[test]
    fn determinate_wave_is_flat_at_both_ends_of_the_range() {
        for value in [0., 0.05, 0.1, 0.95, 0.99, 1.] {
            assert_eq!(wave_amplitude(value), 0.);
        }
        for value in [0.11, 0.5, 0.94] {
            assert_eq!(wave_amplitude(value), 1.);
        }
    }

    #[test]
    fn circular_wavelengths_close_at_the_seam_and_bound_tessellation() {
        assert_eq!(circular_wave_count(20., 15.), 8.);
        assert_eq!(circular_wave_count(20., 23.), 5.);
        assert_eq!(circular_wave_count(20., f32::MAX), 1.);
        assert_eq!(circular_wave_count(20., f32::MIN_POSITIVE), MAX_WAVES);
        for wavelength in [1., 6.4, 15., 23., 200.] {
            let cycles = circular_wave_count(20., wavelength);
            let start = wave_offset(1.6, 0., 0.75);
            let end = wave_offset(1.6, cycles, 0.75);
            assert_eq!(start, end);
        }
    }

    #[test]
    fn invalid_stroke_settings_keep_defaults_while_zero_disables_stroke() {
        let default_color = gpui::white();
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.] {
            let style = ProgressStroke {
                width: Some(px(invalid)),
                ..Default::default()
            };
            let stroke = Stroke::resolve(style, default_color);
            assert_eq!(stroke.width, STROKE_WIDTH);
            assert_eq!(stroke.color, default_color);
            assert!(nonnegative_length(px(invalid)).is_none());
        }
        assert_eq!(nonnegative_length(px(0.)), Some(0.));
        assert_eq!(normalized_amplitude(f32::NAN), 0.);
        assert_eq!(normalized_amplitude(f32::NEG_INFINITY), 0.);
        assert_eq!(normalized_amplitude(f32::INFINITY), 1.);
    }
}
