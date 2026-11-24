//! Loading indicators and rounded polygon constructors for custom sequences.

use std::{f32::consts::TAU, rc::Rc, sync::LazyLock};

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Path, Pixels,
    Point, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
};
use gpui_material_foundation::{
    LoopClock, ShapeMorphSequence, ShapePolygon, color, current_material_theme, is_bounds_visible,
};
use gpui_material_theme::{ColorRole, CornerShape, MaterialShape};
use web_time::Instant;

const CONTAINER_SIZE: f32 = 48.;
const ACTIVE_SIZE: f32 = 38.;
const MORPH_INTERVAL: f32 = 0.65;

static INDETERMINATE_SHAPES: LazyLock<ShapeMorphSequence> = LazyLock::new(|| {
    ShapeMorphSequence::new(
        [
            MaterialShape::SoftBurst,
            MaterialShape::Cookie9Sided,
            MaterialShape::Pentagon,
            MaterialShape::Pill,
            MaterialShape::Sunny,
            MaterialShape::Cookie4Sided,
            MaterialShape::Oval,
        ],
        true,
    )
    .expect("default indeterminate loading sequence has seven shapes")
});

static DETERMINATE_SHAPES: LazyLock<ShapeMorphSequence> = LazyLock::new(|| {
    let circle = ShapePolygon::circle(10)
        .expect("10-vertex circle is valid")
        .rotated(18.);
    ShapeMorphSequence::new(
        [circle, ShapePolygon::material(MaterialShape::SoftBurst)],
        false,
    )
    .expect("default determinate loading sequence has two shapes")
});

/// An endpoint shape for a loading indicator morph.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadingIndicatorShape(ShapePolygon);

impl LoadingIndicatorShape {
    /// Creates a circular morph endpoint (`MaterialShape::Circle`).
    pub fn circle() -> Self {
        Self(ShapePolygon::material(MaterialShape::Circle))
    }

    /// Uses one of the 35 Material 3 Expressive shapes as a morph endpoint.
    pub fn material(shape: MaterialShape) -> Self {
        Self(ShapePolygon::material(shape))
    }

    /// Creates a rounded polygon from its perimeter vertices in either winding order.
    /// The radius uses the same units as the vertices. Invalid geometry returns `None`.
    pub fn rounded_polygon(vertices: &[Point<f32>], radius: f32) -> Option<Self> {
        ShapePolygon::from_vertices(vertices, radius).map(Self)
    }
}

impl From<MaterialShape> for LoadingIndicatorShape {
    fn from(shape: MaterialShape) -> Self {
        Self::material(shape)
    }
}

impl From<ShapePolygon> for LoadingIndicatorShape {
    fn from(polygon: ShapePolygon) -> Self {
        Self(polygon)
    }
}

/// A prepared sequence of rounded polygons for a loading indicator.
#[derive(Clone)]
pub struct LoadingIndicatorShapes(Rc<ShapeMorphSequence>);

impl LoadingIndicatorShapes {
    /// Returns `None` when the sequence contains fewer than two shapes.
    pub fn new(shapes: impl IntoIterator<Item = LoadingIndicatorShape>) -> Option<Self> {
        let polygons = shapes.into_iter().map(|shape| shape.0);
        ShapeMorphSequence::perimeter_aligned(polygons, true).map(|seq| Self(Rc::new(seq)))
    }
}

/// Progress for an operation. Pass a value to select determinate mode, or omit
/// the value for an indeterminate animation.
#[derive(IntoElement)]
pub struct LoadingIndicator {
    id: ElementId,
    label: SharedString,
    contained: bool,
    active: bool,
    value: Option<f32>,
    shapes: Option<LoadingIndicatorShapes>,
    indicator_color: Option<Hsla>,
    container_color: Option<Hsla>,
    container_shape: CornerShape,
}

impl LoadingIndicator {
    /// Creates an active, indeterminate loading indicator with the given element ID and accessible label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            contained: false,
            active: true,
            value: None,
            shapes: None,
            indicator_color: None,
            container_color: None,
            container_shape: CornerShape::Full,
        }
    }

    /// Wraps the morphing shape in a filled `PrimaryContainer` background when `true`. Defaults to `false`.
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }

    /// Retains the footprint when inactive and suppresses paint and progress semantics.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets caller-owned progress. The fraction is clamped to `0..=1`. NaN means zero.
    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(if value.is_nan() {
            0.
        } else {
            value.clamp(0., 1.)
        });
        self
    }

    /// Uses a prepared shape sequence for either progress mode.
    pub fn shapes(mut self, shapes: &LoadingIndicatorShapes) -> Self {
        self.shapes = Some(shapes.clone());
        self
    }

    /// Overrides the shape fill in either variant.
    pub fn indicator_color(mut self, color: impl Into<Hsla>) -> Self {
        self.indicator_color = Some(color.into());
        self
    }

    /// Overrides the background fill of a contained indicator.
    pub fn container_color(mut self, color: impl Into<Hsla>) -> Self {
        self.container_color = Some(color.into());
        self
    }

    /// Sets logical corner radii for the contained indicator's background.
    pub fn container_shape(mut self, shape: CornerShape) -> Self {
        self.container_shape = shape;
        self
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let container_size = px(CONTAINER_SIZE);
        if !self.active {
            return div()
                .id(self.id)
                .size(container_size)
                .flex_none()
                .into_any_element();
        }

        let theme = current_material_theme(window, cx);
        let fill = self.indicator_color.unwrap_or_else(|| {
            color(theme.colors.color(if self.contained {
                ColorRole::OnPrimaryContainer
            } else {
                ColorRole::Primary
            }))
        });
        let background = self
            .container_color
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::PrimaryContainer)));
        let radii =
            self.container_shape
                .radii(CONTAINER_SIZE, CONTAINER_SIZE, theme.layout.direction);
        let active_size = if self.contained {
            // A radius greater than half the side can arc into the centered
            // indicator. Fit the active shape into the largest centered disk.
            let radius = radii.into_iter().fold(CONTAINER_SIZE * 0.5, f32::max);
            ACTIVE_SIZE.min(2. * (radius - (radius - CONTAINER_SIZE * 0.5) * 2_f32.sqrt()))
        } else {
            ACTIVE_SIZE
        };
        let reduced = cx.reduce_motion();
        let running =
            self.value.is_none() && !reduced && window.is_window_active() && window.is_visible();
        let motion = self.value.is_none().then(|| {
            window.use_keyed_state((self.id.clone(), "loading-motion"), cx, |_, _| {
                IndeterminateMotion::default()
            })
        });
        let phase = if let Some(motion) = &motion {
            let now = cx.background_executor().now();
            motion.update(cx, |state, _| {
                state.phase(now, running && state.visible, reduced)
            })
        } else {
            0.
        };
        let icon = canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let visible = is_bounds_visible(bounds, window);
                if let Some(motion) = &motion {
                    let now = cx.background_executor().now();
                    motion.update(cx, |state, _| {
                        state.set_visible(visible, now);
                    });
                }
                if !visible {
                    return;
                }
                let shapes = self.shapes.as_ref().map_or_else(
                    || {
                        if self.value.is_some() {
                            &*DETERMINATE_SHAPES
                        } else {
                            &*INDETERMINATE_SHAPES
                        }
                    },
                    |shapes| &*shapes.0,
                );
                let sample = match self.value {
                    Some(value) => determinate_sample(shapes, value),
                    None => indeterminate_sample(shapes, phase),
                };
                if let Some(path) = sample.path(bounds, active_size, shapes) {
                    window.paint_path(path, fill);
                }
                if running {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();
        let mut control = progress_node(self.id, self.label, self.value)
            .size(container_size)
            .flex_none()
            .child(icon);
        if self.contained {
            let [tl, tr, br, bl] = radii.map(px);
            control = control
                .rounded_tl(tl)
                .rounded_tr(tr)
                .rounded_br(br)
                .rounded_bl(bl)
                .bg(background);
        }
        control.into_any_element()
    }
}

fn progress_node(id: ElementId, label: SharedString, value: Option<f32>) -> gpui_base::Progress {
    let control = gpui_base::Progress::new(id).accessibility_label(label);
    match value {
        Some(value) => control.value(value * 100.),
        None => control.indeterminate(true).a11y_synthetic_children(|node| {
            node.parent_node().set_live(gpui::accesskit::Live::Polite);
            node.parent_node().set_live_atomic();
        }),
    }
}

struct IndeterminateMotion {
    clock: LoopClock,
    visible: bool,
}

impl Default for IndeterminateMotion {
    fn default() -> Self {
        Self {
            clock: LoopClock::default(),
            visible: true,
        }
    }
}

impl IndeterminateMotion {
    fn set_visible(&mut self, visible: bool, now: Instant) {
        if self.visible != visible {
            self.visible = visible;
            if !visible {
                self.clock.pause(now);
            }
        }
    }

    fn phase(&mut self, now: Instant, running: bool, reduced: bool) -> f32 {
        self.clock.advance(now, running, reduced).as_secs_f32() / MORPH_INTERVAL
    }
}

struct MorphSample {
    step: usize,
    progress: f32,
    angle: f32,
}

fn determinate_sample(sequence: &ShapeMorphSequence, value: f32) -> MorphSample {
    let open_steps = sequence.len() - 1;
    let phase = value * open_steps as f32;
    let step = (phase as usize).min(open_steps - 1);
    MorphSample {
        step,
        progress: phase - step as f32,
        angle: -value * TAU * 0.5,
    }
}

fn indeterminate_sample(sequence: &ShapeMorphSequence, phase: f32) -> MorphSample {
    let count = sequence.step_count();
    let step_floor = phase.floor();
    let step = step_floor as usize % count;
    let time = phase.fract() * MORPH_INTERVAL;
    let frequency = 200_f32.sqrt();
    let damped = frequency * 0.8 * time;
    let progress =
        (1. - (-0.6 * frequency * time).exp() * (damped.cos() + 0.75 * damped.sin())).clamp(0., 1.);
    MorphSample {
        step,
        progress,
        angle: (phase * MORPH_INTERVAL / 4.666 + (1. + step_floor + progress) / 4.) * TAU,
    }
}

impl MorphSample {
    fn path(
        self,
        bounds: Bounds<Pixels>,
        extent: f32,
        sequence: &ShapeMorphSequence,
    ) -> Option<Path> {
        sequence
            .sample_step_in_envelope(self.step, self.progress, bounds, px(extent), self.angle)
            .map(|shape| shape.fill_path().clone())
    }
}

#[cfg(test)]
mod tests;
