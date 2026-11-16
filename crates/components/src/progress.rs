use std::f32::consts::TAU;

use gpui::{
    App, Bounds, ContentMask, ElementId, Hsla, IntoElement, ParentElement, PathBuilder, PathStyle,
    Pixels, Point, RenderOnce, SharedString, StrokeOptions, Styled, Window, canvas, point, px,
    size,
};
use gpui_base::motion::Easing;
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{
    ColorRole, Direction, MotionKind, MotionSpec, MotionSpeed, TokenValue, token,
};
use web_time::Instant;

const STROKE_WIDTH: f32 = 4.;
const TRACK_GAP: f32 = 4.;
const STOP_SIZE: f32 = 4.;
const LINEAR_AMPLITUDE: f32 = 3.;
const CIRCULAR_AMPLITUDE: f32 = 1.6;
const CIRCULAR_WAVELENGTH: f32 = 15.;
const MAX_WAVES: f32 = 128.;

/// Straight or Expressive wavy active progress indicator.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressShape {
    #[default]
    Straight,
    Wavy,
}

/// Shape at the ends of a progress stroke.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressCap {
    #[default]
    Round,
    Butt,
    Square,
}

/// Per-stroke appearance. The default width is 4 dp and the color comes from
/// the window palette's active or track role.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressStroke {
    pub width: Option<Pixels>,
    pub color: Option<Hsla>,
    pub cap: ProgressCap,
}

/// Geometry and appearance passed to a linear determinate stop painter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressStop {
    pub bounds: Bounds<Pixels>,
    pub color: Hsla,
    pub cap: ProgressCap,
    pub direction: Direction,
}

type StopPainter = dyn Fn(ProgressStop, &mut Window, &mut App);

struct Progress {
    id: ElementId,
    name: SharedString,
    value: Option<f32>,
    shape: ProgressShape,
    active: ProgressStroke,
    track: ProgressStroke,
    gap: f32,
    stop_size: f32,
    stop_painter: Option<Box<StopPainter>>,
    amplitude: Option<Box<dyn Fn(f32) -> f32>>,
    wavelength: Option<f32>,
    wave_speed: Option<f32>,
}

impl Progress {
    fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value: None,
            shape: ProgressShape::default(),
            active: ProgressStroke::default(),
            track: ProgressStroke::default(),
            gap: TRACK_GAP,
            stop_size: STOP_SIZE,
            stop_painter: None,
            amplitude: None,
            wavelength: None,
            wave_speed: None,
        }
    }

    fn render(self, circular: bool, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let active = Stroke::resolve(self.active, color(theme.colors.color(ColorRole::Primary)));
        let track = Stroke::resolve(
            self.track,
            color(theme.colors.color(ColorRole::SecondaryContainer)),
        );
        let reduced = cx.reduce_motion();
        let window_active = window.is_window_active();
        let effects = theme.motion.spec(
            MotionKind::Effects,
            MotionSpeed::Default,
            reduced || !window_active,
        );
        let spatial = theme.motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Default,
            reduced || !window_active,
        );
        let direction = theme.layout.direction;
        let value = self
            .value
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0., 1.));
        let measured = value.is_some();
        let motion = window.use_keyed_state((self.id.clone(), "progress-motion"), cx, |_, _| {
            ProgressMotion {
                last_measured: value.unwrap_or(0.4),
                ..Default::default()
            }
        });
        let target = motion.update(cx, |state, _| {
            if let Some(value) = value {
                state.last_measured = value;
            }
            state.last_measured
        });
        let mode = animate(
            (self.id.clone(), "indeterminate-mix"),
            f32::from(!measured),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let measured_value = animate(
            (self.id.clone(), "determinate-value"),
            target,
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let wavy = self.shape == ProgressShape::Wavy;
        let amplitude = if wavy {
            let endpoint = mode == 0. && (measured_value == 0. || measured_value == 1.);
            let default_target = wave_amplitude(measured_value) * (1. - mode) + mode;
            let target = self.amplitude.as_ref().map_or(default_target, |amplitude| {
                normalized_amplitude(amplitude(if measured { measured_value } else { 0. }))
            });
            animate(
                (self.id.clone(), "wave-amplitude"),
                if endpoint { 0. } else { target },
                if endpoint {
                    MotionSpec::Instant
                } else {
                    effects
                },
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let wavelength = self.wavelength.unwrap_or(if circular {
            CIRCULAR_WAVELENGTH
        } else {
            40. - 20. * mode
        });
        let hz = self
            .wave_speed
            .map_or(1., |speed| f64::from(speed) / f64::from(wavelength));
        let motion_enabled = window_active && !reduced && active.width > 0.;
        let animate_loop = motion_enabled && !measured;
        let animate_wave =
            motion_enabled && hz > 0. && amplitude > 0. && (mode > 0. || measured_value > 0.);
        let now = cx.background_executor().now();
        let (loop_time, phase) = motion.update(cx, |state, _| {
            state.sample(
                now,
                animate_loop,
                if animate_wave { hz } else { 0. },
                reduced,
                mode == 1.,
            )
        });
        let motionless = reduced || !window_active && loop_time == 0.;
        let (arc_start, arc_length) = if circular && mode > 0. && !motionless {
            circular_indeterminate(loop_time)
        } else {
            (0., 0.25)
        };
        let arc_origin = motion.update(cx, |state, _| {
            if mode == 0. || mode == 1. || motionless {
                state.arc_origin = arc_start.round();
            }
            state.arc_origin
        });
        let arc = (
            (arc_start - arc_origin) * mode,
            measured_value + (arc_length - measured_value) * mode,
        );
        let [first, second] = if circular || mode == 0. || motionless {
            [(0.25, 0.65), (0., 0.)]
        } else {
            linear_indeterminate(loop_time)
        };
        let mut lines = [
            (
                first.0 * mode,
                measured_value + (first.1 - measured_value) * mode,
            ),
            (second.0, second.0 + (second.1 - second.0) * mode),
        ];
        lines.sort_by(|left, right| left.0.total_cmp(&right.0));

        let wave_space = if wavy { 2. * LINEAR_AMPLITUDE } else { 0. };
        let side = if circular {
            if wavy { 48. } else { 40. }
        } else {
            STROKE_WIDTH
                .max(active.width)
                .max(track.width)
                .max(self.stop_size)
                + wave_space
        };
        let picture = canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let width = f32::from(bounds.size.width);
                let height = f32::from(bounds.size.height);
                if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
                    return;
                }
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    let paint = ProgressPaint {
                        active,
                        track,
                        gap: self.gap,
                        wavelength,
                        amplitude,
                        phase,
                    };
                    if circular {
                        paint.circular(bounds, arc, wavy, window);
                    } else {
                        paint.linear(bounds, lines, direction, window);
                        if measured && self.stop_size > 0. {
                            let stop = ProgressStop {
                                bounds: stop_bounds(bounds, self.stop_size, wavy, direction),
                                color: active.color,
                                cap: active.cap,
                                direction,
                            };
                            paint_stop(stop, self.stop_painter.as_deref(), window, cx);
                        }
                    }
                });
                if animate_loop || animate_wave {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();
        let mut element = gpui_base::Progress::new(self.id)
            .value(value.unwrap_or_default() * 100.)
            .indeterminate(!measured)
            .accessibility_label(self.name)
            .h(px(side))
            .child(picture);
        if circular {
            element = element.w(px(side)).flex_none();
        } else {
            element = element.w_full();
        }
        element
    }
}

#[derive(Default)]
struct ProgressMotion {
    last_measured: f32,
    arc_origin: f32,
    last_time: Option<Instant>,
    loop_running: bool,
    loop_ms: f64,
    wave_hz: f64,
    wave_phase: f64,
}

impl ProgressMotion {
    fn sample(
        &mut self,
        now: Instant,
        animate_loop: bool,
        wave_hz: f64,
        reduced: bool,
        settled_loop: bool,
    ) -> (f32, f32) {
        if reduced {
            self.last_time = None;
            self.loop_ms = 0.;
            self.wave_phase = 0.;
        }
        let elapsed = self
            .last_time
            .replace(now)
            .map_or(0., |last| now.saturating_duration_since(last).as_secs_f64());
        if animate_loop && self.loop_running {
            self.loop_ms += elapsed * 1000.;
        }
        if settled_loop {
            // 42 seconds is a multiple of both the linear and circular cycles.
            self.loop_ms %= 42_000.;
        }
        if wave_hz > 0. && self.wave_hz > 0. {
            self.wave_phase = (self.wave_phase + elapsed * self.wave_hz).fract();
        }
        self.loop_running = animate_loop;
        self.wave_hz = wave_hz;
        (self.loop_ms as f32, self.wave_phase as f32)
    }
}
// The keyframe timings match ProgressIndicator.kt in Material 3 1.5.0-rc01.
fn linear_indeterminate(elapsed_ms: f32) -> [(f32, f32); 2] {
    let clock = elapsed_ms % 1750.;
    let sample = |delay, duration| {
        easing_sample(
            "md.sys.motion.easing-emphasized-accelerate",
            (clock - delay) / duration,
        )
    };
    [
        (sample(250., 1000.), sample(0., 1000.)),
        (sample(900., 850.), sample(650., 850.)),
    ]
}

fn circular_indeterminate(elapsed_ms: f32) -> (f32, f32) {
    let clock = elapsed_ms % 6000.;
    let step = (clock / 1500.).floor();
    let turn = easing_sample(
        "md.sys.motion.easing-emphasized-decelerate",
        (clock % 1500.) / 300.,
    );
    let phase = clock / 3000.;
    let length = if phase < 1. {
        0.1 + (0.87 - 0.1) * phase
    } else {
        0.87 - (0.87 - 0.1) * easing_sample("md.sys.motion.easing-standard", phase - 1.)
    };
    // Keep the angle unwrapped during a mode change, then paint it periodically.
    (
        (elapsed_ms / 6000.) * 3. + (elapsed_ms / 6000.).floor() + (step + turn) / 4.,
        length,
    )
}

fn easing_sample(name: &str, progress: f32) -> f32 {
    let progress = progress.clamp(0., 1.);
    let TokenValue::Easing(curve) = token(name).expect("bundled progress easing") else {
        unreachable!("progress easing token must contain a cubic Bézier");
    };
    let easing =
        Easing::cubic_bezier(curve.x1, curve.y1, curve.x2, curve.y2).expect("valid Material curve");
    easing.sample(progress)
}

#[derive(Clone, Copy)]
struct Stroke {
    width: f32,
    color: Hsla,
    cap: ProgressCap,
}

impl Stroke {
    fn resolve(style: ProgressStroke, color: Hsla) -> Self {
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
        let mut path = PathBuilder::default().with_style(PathStyle::Stroke(
            StrokeOptions::default()
                .with_line_width(self.width)
                .with_miter_limit(1.),
        ));
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
struct ProgressPaint {
    active: Stroke,
    track: Stroke,
    gap: f32,
    wavelength: f32,
    amplitude: f32,
    phase: f32,
}

impl ProgressPaint {
    fn linear(
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
        let wavelength = self.wavelength.max(width / MAX_WAVES);
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

    fn circular(
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
            let wave = wave_offset(amplitude, fraction * waves, self.phase);
            let radius = px(radius + wave);
            center + point(radius * arc.cos(), radius * arc.sin())
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
    ((radius * TAU) / wavelength).round().clamp(1., MAX_WAVES)
}

fn wave_offset(amplitude: f32, cycles: f32, phase: f32) -> f32 {
    amplitude * ((cycles.rem_euclid(1.) - phase) * TAU).sin()
}

fn nonnegative_length(value: Pixels) -> Option<f32> {
    let length = f32::from(value);
    (length.is_finite() && length >= 0.).then_some(length)
}

fn normalized_amplitude(amplitude: f32) -> f32 {
    if amplitude.is_nan() {
        0.
    } else {
        amplitude.clamp(0., 1.)
    }
}

fn wave_amplitude(progress: f32) -> f32 {
    // WavyProgressIndicatorDefaults.indicatorAmplitude suppresses waves at the
    // beginning and end; only the change between these targets is animated.
    f32::from(progress > 0.1 && progress < 0.95)
}

fn centered_square(center: Point<Pixels>, side: f32) -> Bounds<Pixels> {
    let side = px(side);
    Bounds::new(center - point(side / 2., side / 2.), size(side, side))
}

fn stop_bounds(
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

fn paint_stop(
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
/// Horizontal progress. Supply a fraction from 0 to 1, or omit it for indeterminate progress.
#[derive(IntoElement)]
pub struct LinearProgress(Progress);

/// Circular progress. Supply a fraction from 0 to 1, or omit it for indeterminate progress.
#[derive(IntoElement)]
pub struct CircularProgress(Progress);

impl LinearProgress {
    /// Sets the determinate stop marker's size. Zero hides the marker.
    pub fn stop_size(mut self, size: Pixels) -> Self {
        self.0.stop_size = nonnegative_length(size).unwrap_or(STOP_SIZE);
        self
    }

    /// Replaces the determinate stop marker's paint within its clipped bounds.
    pub fn draw_stop_indicator(
        mut self,
        draw: impl Fn(ProgressStop, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.stop_painter = Some(Box::new(draw));
        self
    }
}

macro_rules! progress_api {
    ($name:ident, $circular:literal) => {
        impl $name {
            pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
                Self(Progress::new(id, name))
            }

            pub fn value(mut self, value: f32) -> Self {
                self.0.value = Some(value);
                self
            }

            /// Clears the measured value and shows indeterminate progress.
            pub fn indeterminate(mut self) -> Self {
                self.0.value = None;
                self
            }

            pub fn shape(mut self, shape: ProgressShape) -> Self {
                self.0.shape = shape;
                self
            }

            pub fn active_stroke(mut self, stroke: ProgressStroke) -> Self {
                self.0.active = stroke;
                self
            }

            pub fn track_stroke(mut self, stroke: ProgressStroke) -> Self {
                self.0.track = stroke;
                self
            }

            /// Sets the painted gap between the active stroke and the track.
            pub fn gap(mut self, gap: Pixels) -> Self {
                self.0.gap = nonnegative_length(gap).unwrap_or(TRACK_GAP);
                self
            }

            /// Sets a fraction of the available height or radius for the wave.
            pub fn wave_amplitude(self, fraction: f32) -> Self {
                self.wave_amplitude_by(move |_| fraction)
            }

            /// Computes the wave amplitude from displayed progress. In
            /// indeterminate mode the function receives zero.
            pub fn wave_amplitude_by(mut self, amplitude: impl Fn(f32) -> f32 + 'static) -> Self {
                self.0.amplitude = Some(Box::new(amplitude));
                self
            }

            /// Sets crest spacing in logical pixels.
            pub fn wavelength(mut self, length: Pixels) -> Self {
                self.0.wavelength = nonnegative_length(length).filter(|length| *length > 0.);
                self
            }

            /// Sets travel speed in logical pixels per second. Zero holds the phase.
            pub fn wave_speed(mut self, length_per_second: Pixels) -> Self {
                self.0.wave_speed = nonnegative_length(length_per_second);
                self
            }
        }

        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                self.0.render($circular, window, cx)
            }
        }
    };
}

progress_api!(LinearProgress, false);
progress_api!(CircularProgress, true);

#[cfg(test)]
mod tests {
    use super::*;

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
    fn indeterminate_keyframes_use_both_linear_segments_and_periodic_rotation() {
        assert_eq!(linear_indeterminate(0.), [(0., 0.); 2]);
        let [(start, end), (second_start, second_end)] = linear_indeterminate(800.);
        assert!(0. < start && start < end && end < 1.);
        assert_eq!(second_start, 0.);
        assert!(second_end > 0.);
        assert_eq!(linear_indeterminate(2550.), linear_indeterminate(800.));
        assert_eq!(circular_indeterminate(0.), (0., 0.1));
        assert_eq!(circular_indeterminate(6000.), (4., 0.1));
        assert_eq!(circular_indeterminate(3000.).1, 0.87);
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
    fn progress_motion_holds_phase_while_paused_and_resumes_without_elapsed_idle_time() {
        use std::time::Duration;
        let now = Instant::now();
        let at = |millis| now + Duration::from_millis(millis);
        let mut motion = ProgressMotion::default();
        assert_eq!(motion.sample(at(0), true, 2., false, true), (0., 0.));
        assert_eq!(motion.sample(at(125), true, 2., false, true), (125., 0.25));
        assert_eq!(
            motion.sample(at(125), false, 0., false, false),
            (125., 0.25)
        );
        assert_eq!(motion.sample(at(1000), true, 1., false, true), (125., 0.25));
        assert_eq!(motion.sample(at(1250), true, 1., false, true), (375., 0.5));
        assert_eq!(motion.sample(at(1300), false, 0., true, false), (0., 0.));
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
