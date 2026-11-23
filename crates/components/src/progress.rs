mod motion;
mod paint;

use gpui::{
    App, Bounds, ContentMask, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, canvas, px,
};
use gpui_material_foundation::{animate, color, current_material_theme, is_bounds_visible};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpec, MotionSpeed};

use crate::SemanticTone;
use motion::{ProgressMotion, circular_indeterminate, linear_indeterminate};
use paint::{ProgressPaint, Stroke, paint_stop, stop_bounds};

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
    /// Standard flat bar or circular arc.
    #[default]
    Straight,
    /// Expressive animated sine-wave active stroke.
    Wavy,
}

/// Shape at the ends of a progress stroke.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressCap {
    /// Semicircular stroke end caps (default).
    #[default]
    Round,
    /// Flat stroke ends flush with the segment endpoints.
    Butt,
    /// Square stroke ends extending half the stroke width past the endpoints.
    Square,
}

/// Per-stroke appearance. The default width is 4 dp and the color comes from
/// the window palette's active or track role.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressStroke {
    /// Stroke thickness in logical pixels. Defaults to `4 px`.
    pub width: Option<Pixels>,
    /// Explicit stroke color overriding the theme or semantic tone role.
    pub color: Option<Hsla>,
    /// End-cap geometry for this stroke.
    pub cap: ProgressCap,
}

/// Geometry and appearance passed to a linear determinate stop painter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressStop {
    /// Clipped bounding box of the trailing stop indicator.
    pub bounds: Bounds<Pixels>,
    /// Resolved active indicator color.
    pub color: Hsla,
    /// Active stroke cap style.
    pub cap: ProgressCap,
    /// Active layout direction (`Ltr` or `Rtl`).
    pub direction: Direction,
}

type StopPainter = dyn Fn(ProgressStop, &mut Window, &mut App);

fn progress_tone_roles(tone: SemanticTone) -> (ColorRole, ColorRole) {
    tone.roles().map_or(
        (ColorRole::Primary, ColorRole::SecondaryContainer),
        |(role, _, container, _)| (role, container),
    )
}

struct Progress {
    id: ElementId,
    name: SharedString,
    value: Option<f32>,
    shape: ProgressShape,
    tone: SemanticTone,
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
            tone: SemanticTone::Default,
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
        let theme = current_material_theme(window, cx);
        let (active_role, track_role) = progress_tone_roles(self.tone);
        let active = Stroke::resolve(self.active, color(theme.colors.color(active_role)));
        let track = Stroke::resolve(self.track, color(theme.colors.color(track_role)));
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
            ProgressMotion::new(value.unwrap_or(0.4))
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
        let motion_enabled = window_active && window.is_visible() && !reduced && active.width > 0.;
        let animate_loop = motion_enabled && !measured;
        let animate_wave =
            motion_enabled && hz > 0. && amplitude > 0. && (mode > 0. || measured_value > 0.);
        let now = cx.background_executor().now();
        let (loop_time, phase) = motion.update(cx, |state, _| {
            let visible = state.visible;
            state.sample(
                now,
                animate_loop && visible,
                if animate_wave && visible { hz } else { 0. },
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
                let visible = width.is_finite()
                    && height.is_finite()
                    && width > 0.
                    && height > 0.
                    && is_bounds_visible(bounds, window);
                let now = cx.background_executor().now();
                motion.update(cx, |state, _| state.set_visible(visible, now));
                if !visible {
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
            /// Creates an indeterminate progress indicator with the given element ID and accessible name.
            pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
                Self(Progress::new(id, name))
            }

            /// Sets determinate progress as a fraction in `0.0..=1.0` (clamped during render).
            pub fn value(mut self, value: f32) -> Self {
                self.0.value = Some(value);
                self
            }

            /// Clears the measured value and shows indeterminate progress.
            pub fn indeterminate(mut self) -> Self {
                self.0.value = None;
                self
            }

            /// Selects straight or Expressive wavy active stroke geometry.
            pub fn shape(mut self, shape: ProgressShape) -> Self {
                self.0.shape = shape;
                self
            }

            /// Applies a Material 3 semantic tone shortcut to the active and track strokes.
            /// Explicit `.active_stroke` and `.track_stroke` colors still take precedence.
            pub fn tone(mut self, tone: SemanticTone) -> Self {
                self.0.tone = tone;
                self
            }

            /// Overrides the active indicator stroke width, color, and cap.
            pub fn active_stroke(mut self, stroke: ProgressStroke) -> Self {
                self.0.active = stroke;
                self
            }

            /// Overrides the background track stroke width, color, and cap.
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
    fn progress_tone_roles_map_semantic_tones() {
        use ColorRole::*;
        assert_eq!(
            progress_tone_roles(SemanticTone::Default),
            (Primary, SecondaryContainer)
        );
        assert_eq!(
            progress_tone_roles(SemanticTone::Danger),
            (Error, ErrorContainer)
        );
        assert_eq!(
            progress_tone_roles(SemanticTone::Success),
            (Success, SuccessContainer)
        );
        assert_eq!(
            progress_tone_roles(SemanticTone::Warning),
            (Warning, WarningContainer)
        );
        assert_eq!(
            progress_tone_roles(SemanticTone::Info),
            (Info, InfoContainer)
        );
    }
}
