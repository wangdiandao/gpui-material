use std::{f32::consts::TAU, time::Instant};

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, PathBuilder,
    Pixels, Point, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window,
    canvas, div, point, px,
};
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpeed};
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
        let phase = if animated {
            (now.saturating_duration_since(epoch).as_secs_f32() / 1.6).fract()
        } else {
            0.
        };
        let target = match value {
            Some(value) => (0., value),
            None if circular => (phase, phase + 0.25 + 0.12 * (phase * TAU).sin()),
            None if reduced => (0.25, 0.65),
            None => {
                let t = phase * TAU;
                let start = 0.35 - 0.35 * t.cos();
                (start, (start + 0.24 + 0.08 * t.sin()).min(1.))
            }
        };
        let transition = if animated {
            gpui_material_theme::MotionSpec::Instant
        } else {
            policy
        };
        let start = animate(
            ElementId::from((self.id.clone(), "start")),
            target.0.rem_euclid(1.),
            transition,
            window,
            cx,
        );
        let extent = animate(
            ElementId::from((self.id.clone(), "value")),
            target.1 - target.0,
            transition,
            window,
            cx,
        )
        .clamp(0., 1.);
        let segment = (start, start + extent);
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
                                1.6 * ((fraction * 8. - phase) * TAU).sin()
                            } else {
                                0.
                            });
                        center + point(radius * angle.cos(), radius * angle.sin())
                    };
                    let gap = (px(4.) / (radius * TAU)).clamp(0., 0.12);
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
                        let x = (bounds.size.width - px(4.)).max(px(0.)) * from_start + px(2.);
                        let wave = if wave {
                            px(3. * ((f32::from(x) / 40. - phase) * TAU).sin())
                        } else {
                            px(0.)
                        };
                        point(bounds.left() + x, bounds.center().y + wave)
                    };
                    let gap = (px(4.) / bounds.size.width.max(px(4.))).clamp(0., 0.12);
                    let start = segment.0.clamp(0., 1.);
                    let end = segment.1.clamp(0., 1.);
                    if end <= start {
                        stroke(0., 1., track, |f| sample(f, false), window);
                    } else {
                        stroke(0., start - gap, track, |f| sample(f, false), window);
                        stroke(end + gap, 1., track, |f| sample(f, false), window);
                        stroke(start, end, active, |f| sample(f, wavy), window);
                    }
                    if end < 1. - gap {
                        let stop = sample(1., false);
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
                if animated && window.is_window_active() {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();
        let mut element = div()
            .id(self.id)
            .role(Role::ProgressIndicator)
            .aria_label(self.name)
            .h(px(side))
            .child(picture);
        if circular {
            element = element.w(px(side)).flex_none();
        } else {
            element = element.w_full();
        }
        if let Some(value) = value {
            element = element
                .aria_numeric_value(f64::from(value) * 100.)
                .aria_min_numeric_value(0.)
                .aria_max_numeric_value(100.);
        }
        element
    }
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
        let animate = self.active && !cx.reduce_motion() && window.is_window_active();
        let now = cx.background_executor().now();
        let motion = window.use_keyed_state(
            ElementId::from((self.id.clone(), "loading-motion")),
            cx,
            |_, _| LoadingMotion::default(),
        );
        let progress = motion.update(cx, |state, _| {
            if animate {
                (now.saturating_duration_since(*state.epoch.get_or_insert(now))
                    .as_secs_f32()
                    / 0.6)
                    % state.shapes.len() as f32
            } else {
                state.epoch = None;
                0.
            }
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
                let eased = phase * phase * (3. - 2. * phase);
                let cubics = state.shapes[progress.floor() as usize].as_cubics(eased);
                let project = |p: polymorpher::geometry::Point| {
                    bounds.origin + point(bounds.size.width * p.x, bounds.size.height * p.y)
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
