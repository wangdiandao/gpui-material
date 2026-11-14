use std::{
    f32::consts::TAU,
    time::{Duration, Instant},
};

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, PathBuilder,
    Pixels, Point, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window,
    canvas, div, point, px,
};
use gpui_base::motion::Easing;
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpeed, TokenValue, token};
use polymorpher::{Morph, shapes};

/// Straight or Expressive wavy progress track.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressShape {
    #[default]
    Straight,
    Wavy,
}

struct Progress {
    id: ElementId,
    name: SharedString,
    value: Option<f32>,
    shape: ProgressShape,
}

impl Progress {
    fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value: None,
            shape: ProgressShape::default(),
        }
    }

    fn paint(self, circular: bool, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = cx.background_executor().now();
        let epoch = *window
            .use_keyed_state(ElementId::from((self.id.clone(), "phase")), cx, |_, _| now)
            .read(cx);
        let reduced = cx.reduce_motion();
        let theme = window_material_theme(window, cx);
        let policy = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Default, reduced);
        let active = color(theme.colors.color(ColorRole::Primary));
        let track = color(theme.colors.color(ColorRole::SecondaryContainer));
        let direction = theme.layout.direction;
        let wavy = self.shape == ProgressShape::Wavy;
        let value = self
            .value
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0., 1.));
        let animated = value.is_none() && !reduced;
        let moving_wave = wavy && !reduced;
        let mode_spec = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Default, reduced);
        let indeterminate = animate(
            (self.id.clone(), "indeterminate-mix"),
            f32::from(value.is_none()),
            mode_spec,
            window,
            cx,
        )
        .clamp(0., 1.);
        let last_value = window.use_keyed_state((self.id.clone(), "last-progress"), cx, |_, _| {
            value.unwrap_or(0.4)
        });
        let determinate = match value {
            Some(value) => {
                if value != *last_value.read(cx) {
                    last_value.update(cx, |last, _| *last = value);
                }
                value
            }
            None => *last_value.read(cx),
        };
        let determinate = animate(
            (self.id.clone(), "determinate-value"),
            determinate,
            policy,
            window,
            cx,
        )
        .clamp(0., 1.);
        let wave_phase = if moving_wave {
            now.saturating_duration_since(epoch).as_secs_f32().fract()
        } else {
            0.
        };
        let amplitude = animate(
            (self.id.clone(), "wave-amplitude"),
            wave_amplitude(determinate) + (1. - wave_amplitude(determinate)) * indeterminate,
            mode_spec,
            window,
            cx,
        )
        .clamp(0., 1.);
        let elapsed = now.saturating_duration_since(epoch).as_secs_f32() * 1000.;
        let circular_segment = if !circular || reduced {
            (0., 0.25)
        } else {
            circular_indeterminate(elapsed)
        };
        let loop_origin = window.use_keyed_state((self.id.clone(), "arc-rotation"), cx, |_, _| 0.);
        if indeterminate == 1. || indeterminate == 0. {
            let rotation = circular_segment.0.round();
            if *loop_origin.read(cx) != rotation {
                loop_origin.update(cx, |origin, _| *origin = rotation);
            }
        }
        let start = (circular_segment.0 - *loop_origin.read(cx)) * indeterminate;
        let extent = determinate + (circular_segment.1 - determinate) * indeterminate;
        let segment = (start, start + extent);
        let lines = if circular || indeterminate == 0. || reduced {
            [(0.25, 0.65), (0., 0.)]
        } else {
            linear_indeterminate(elapsed)
        };
        let side = if circular {
            if wavy { 48. } else { 40. }
        } else if wavy {
            10.
        } else {
            4.
        };
        let picture = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                if circular {
                    let center = bounds.center();
                    let radius = (bounds.size.width.min(bounds.size.height)
                        - px(if wavy { 8. } else { 4. }))
                        / 2.;
                    let sample = |fraction: f32, wave: bool| {
                        let angle = (fraction - 0.25) * TAU;
                        let radius = radius
                            + px(if wave {
                                1.6 * amplitude * ((fraction * 8. - wave_phase) * TAU).sin()
                            } else {
                                0.
                            });
                        center + point(radius * angle.cos(), radius * angle.sin())
                    };
                    // Round caps consume another stroke width around the 4 dp gap.
                    let gap = (px(8.) / (radius * TAU)).clamp(0., 1.);
                    if extent == 0. {
                        stroke(0., 1., track, |f| sample(f, false), window);
                    } else {
                        stroke(
                            segment.1 + gap,
                            segment.0 + 1. - gap,
                            track,
                            |f| sample(f, false),
                            window,
                        );
                    }
                    stroke(segment.0, segment.1, active, |f| sample(f, wavy), window);
                } else {
                    let sample = |fraction: f32, wave: bool| {
                        let from_start = if direction == Direction::Rtl {
                            1. - fraction
                        } else {
                            fraction
                        };
                        let x = px(linear_position(f32::from(bounds.size.width), from_start));
                        let wave = if wave {
                            let wavelength = 40. - 20. * indeterminate;
                            px(3.
                                * amplitude
                                * ((f32::from(x) / wavelength - wave_phase) * TAU).sin())
                        } else {
                            px(0.)
                        };
                        point(bounds.left() + x, bounds.center().y + wave)
                    };
                    let gap = (px(8.) / bounds.size.width.max(px(4.))).min(1.);
                    let [first, second] = lines;
                    let start = first.0 * indeterminate;
                    let finish = determinate + (first.1 - determinate) * indeterminate;
                    let second_end = second.0 + (second.1 - second.0) * indeterminate;
                    let mut segments = [(start, finish), (second.0, second_end)];
                    segments.sort_by(|left, right| left.0.total_cmp(&right.0));
                    let mut position = 0.;
                    for (start, finish) in segments {
                        if finish <= start {
                            continue;
                        }
                        stroke(position, start - gap, track, |f| sample(f, false), window);
                        position = position.max(finish + gap);
                    }
                    stroke(position, 1., track, |f| sample(f, false), window);
                    stroke(start, finish, active, |f| sample(f, wavy), window);
                    if indeterminate > 0. {
                        stroke(second.0, second_end, active, |f| sample(f, wavy), window);
                    }
                    if value.is_some() {
                        let stop = point(
                            bounds.left()
                                + px(stop_position(f32::from(bounds.size.width), side, direction)),
                            bounds.center().y,
                        );
                        window.paint_quad(gpui::quad(
                            Bounds::from_corners(
                                stop - point(px(2.), px(2.)),
                                stop + point(px(2.), px(2.)),
                            ),
                            px(2.),
                            active,
                            px(0.),
                            active,
                            gpui::BorderStyle::Solid,
                        ));
                    }
                }
                if (animated || moving_wave && amplitude > 0.) && window.is_window_active() {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();
        // Base owns the controlled numeric semantics; Material owns the drawing.
        let mut element = gpui_base::Progress::new(self.id)
            .value(value.unwrap_or_default() * 100.)
            .indeterminate(value.is_none())
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

fn stroke(
    start: f32,
    end: f32,
    tint: Hsla,
    sample: impl Fn(f32) -> Point<Pixels>,
    window: &mut Window,
) {
    if end <= start {
        return;
    }
    let count = ((end - start) * 100.).ceil() as usize;
    let mut path = PathBuilder::stroke(px(4.));
    path.move_to(sample(start));
    for ix in 1..=count {
        path.line_to(sample(start + (end - start) * ix as f32 / count as f32));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, tint);
    }
    // GPUI's path tessellator uses butt caps. Material uses round caps for
    // all three presentations, with the same endpoints as the sampled path.
    for center in [sample(start), sample(end)] {
        window.paint_quad(gpui::quad(
            Bounds::from_corners(
                center - point(px(2.), px(2.)),
                center + point(px(2.), px(2.)),
            ),
            px(2.),
            tint,
            px(0.),
            tint,
            gpui::BorderStyle::Solid,
        ));
    }
}

fn linear_position(width: f32, fraction: f32) -> f32 {
    if width < 4. {
        width * fraction
    } else {
        (width * fraction).clamp(2., width - 2.)
    }
}

fn wave_amplitude(progress: f32) -> f32 {
    // WavyProgressIndicatorDefaults.indicatorAmplitude suppresses waves at the
    // beginning and end; only the change between these targets is animated.
    f32::from(progress > 0.1 && progress < 0.95)
}

fn stop_position(width: f32, height: f32, direction: Direction) -> f32 {
    let inset = 2. + ((height - 4.).max(0.) / 2.).min(4.);
    if direction == Direction::Rtl {
        inset.min(width / 2.)
    } else {
        (width - inset).max(width / 2.)
    }
}

/// Horizontal progress. Supply a fraction from 0 to 1, or omit it for indeterminate progress.
#[derive(IntoElement)]
pub struct LinearProgress(Progress);

/// Circular progress. Supply a fraction from 0 to 1, or omit it for indeterminate progress.
#[derive(IntoElement)]
pub struct CircularProgress(Progress);

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

            pub fn shape(mut self, shape: ProgressShape) -> Self {
                self.0.shape = shape;
                self
            }
        }

        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                self.0.paint($circular, window, cx)
            }
        }
    };
}

progress_api!(LinearProgress, false);
progress_api!(CircularProgress, true);

/// Status for an operation with unknown duration. The parent removes the
/// indicator when the operation settles or sets it inactive to retain its space.
#[derive(IntoElement)]
pub struct LoadingIndicator {
    id: ElementId,
    label: SharedString,
    contained: bool,
    active: bool,
}

impl LoadingIndicator {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            contained: false,
            active: true,
        }
    }

    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

struct LoadingMotion {
    epoch: Option<Instant>,
    elapsed: Duration,
    shapes: [Morph; 7],
}

impl Default for LoadingMotion {
    fn default() -> Self {
        let shapes = [
            shapes::soft_burst(),
            shapes::cookie9(),
            shapes::pentagon(),
            shapes::pill(),
            shapes::sunny(),
            shapes::cookie4(),
            shapes::oval(),
        ];
        Self {
            epoch: None,
            elapsed: Duration::ZERO,
            shapes: std::array::from_fn(|ix| {
                Morph::new(shapes[ix].clone(), shapes[(ix + 1) % shapes.len()].clone())
            }),
        }
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let fill = color(theme.colors.color(if self.contained {
            ColorRole::OnPrimaryContainer
        } else {
            ColorRole::Primary
        }));
        let background = color(theme.colors.color(ColorRole::PrimaryContainer));
        let reduced = cx.reduce_motion();
        let animate = self.active && !reduced && window.is_window_active();
        let now = cx.background_executor().now();
        let motion = window.use_keyed_state(
            ElementId::from((self.id.clone(), "loading-motion")),
            cx,
            |_, _| LoadingMotion::default(),
        );
        let progress = motion.update(cx, |state, _| {
            if !self.active || reduced {
                state.epoch = None;
                state.elapsed = Duration::ZERO;
                return 0.;
            }
            let elapsed = if animate {
                state.elapsed + now.saturating_duration_since(*state.epoch.get_or_insert(now))
            } else {
                if let Some(epoch) = state.epoch.take() {
                    state.elapsed += now.saturating_duration_since(epoch);
                }
                state.elapsed
            };
            elapsed.as_secs_f32() / 0.65
        });
        let icon = canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let state = motion.read(cx);
                let bounds = Bounds::from_corners(
                    bounds.origin + point(px(5.), px(5.)),
                    bounds.bottom_right() - point(px(5.), px(5.)),
                );
                let phase = progress.fract();
                let spring_time = phase * 0.65;
                let frequency = 200_f32.sqrt();
                let decay = 0.6 * frequency;
                let damped_frequency = frequency * 0.8;
                let t = damped_frequency * spring_time;
                let eased =
                    (1. - (-decay * spring_time).exp() * (t.cos() + 0.75 * t.sin())).clamp(0., 1.);
                let index = progress.floor() as usize % state.shapes.len();
                let cubics = state.shapes[index].as_cubics(eased);
                let angle = (progress * 0.65 / 4.666 + (1. + progress.floor() + eased) / 4.) * TAU;
                let (sin, cos) = angle.sin_cos();
                let project = |p: polymorpher::geometry::Point| {
                    let x = p.x - 0.5;
                    let y = p.y - 0.5;
                    bounds.center()
                        + point(
                            bounds.size.width * (cos * x - sin * y),
                            bounds.size.height * (sin * x + cos * y),
                        )
                };
                if let Some(first) = cubics.first() {
                    let mut path = PathBuilder::fill();
                    path.move_to(project(first.anchor0()));
                    for cubic in cubics {
                        path.cubic_bezier_to(
                            project(cubic.anchor1()),
                            project(cubic.control0()),
                            project(cubic.control1()),
                        );
                    }
                    path.close();
                    if let Ok(path) = path.build() {
                        window.paint_path(path, fill);
                    }
                }
                if animate {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();
        let mut element = div().id(self.id).size(px(48.)).flex_none();
        if self.active {
            element = element
                .role(Role::Status)
                .aria_label(self.label)
                .a11y_synthetic_children(|node| {
                    node.parent_node().set_live(gpui::accesskit::Live::Polite);
                    node.parent_node().set_live_atomic();
                })
                .child(icon);
            if self.contained {
                element = element.rounded_full().bg(background);
            }
        }
        element
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_progress_endpoints_and_stop_stay_inside_the_track() {
        assert_eq!(linear_position(100., 0.), 2.);
        assert_eq!(linear_position(100., 1.), 98.);
        assert_eq!(linear_position(100., 0.5), 50.);
        for height in [4., 10.] {
            let ltr = stop_position(100., height, Direction::Ltr);
            let rtl = stop_position(100., height, Direction::Rtl);
            assert_eq!(ltr + rtl, 100.);
            assert!(ltr + 2. <= 100. && rtl - 2. >= 0.);
        }
        assert_eq!(stop_position(100., 4., Direction::Ltr), 98.);
        assert_eq!(stop_position(100., 10., Direction::Ltr), 95.);
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
}
