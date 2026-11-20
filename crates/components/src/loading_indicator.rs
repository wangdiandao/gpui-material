//! Loading indicators and rounded polygon constructors for custom sequences.

use std::{f32::consts::TAU, rc::Rc, sync::LazyLock, time::Duration};

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Path, Pixels,
    Point, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
};
use gpui_material_foundation::{ShapeMorphSequence, ShapePolygon, color, current_material_theme};
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
        let running = self.value.is_none() && !reduced && window.is_window_active();
        let phase = if self.value.is_none() {
            let now = cx.background_executor().now();
            window
                .use_keyed_state((self.id.clone(), "loading-motion"), cx, |_, _| {
                    IndeterminateMotion::default()
                })
                .update(cx, |state, _| state.phase(now, running, reduced))
        } else {
            0.
        };
        let icon = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
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

#[derive(Default)]
struct IndeterminateMotion {
    epoch: Option<Instant>,
    elapsed: Duration,
}

impl IndeterminateMotion {
    fn phase(&mut self, now: Instant, running: bool, reduced: bool) -> f32 {
        if reduced {
            *self = Self::default();
            return 0.;
        }
        let elapsed = if running {
            self.elapsed + now.saturating_duration_since(*self.epoch.get_or_insert(now))
        } else {
            if let Some(epoch) = self.epoch.take() {
                self.elapsed += now.saturating_duration_since(epoch);
            }
            self.elapsed
        };
        elapsed.as_secs_f32() / MORPH_INTERVAL
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
    ) -> Option<Path<Pixels>> {
        sequence
            .sample_step_in_envelope(self.step, self.progress, bounds, px(extent), self.angle)
            .map(|shape| shape.fill_path().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};

    #[test]
    fn loading_shapes_fit_the_active_size_through_rotation_and_morphs() {
        let shape_sets = [&*INDETERMINATE_SHAPES, &*DETERMINATE_SHAPES];
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        for shapes in shape_sets {
            let scale = shapes.rotation_scale();
            assert!(scale > 0. && scale <= 1.);
            for step in 0..shapes.step_count() {
                for tick in 0..=20 {
                    let progress = tick as f32 / 20.;
                    let sampled = shapes
                        .sample_step_in_envelope(step, progress, bounds, px(ACTIVE_SIZE), 0.)
                        .unwrap();
                    assert!(sampled.fill_path().vertices.iter().all(|v| {
                        let delta = v.xy_position - bounds.center();
                        f32::from(delta.x).hypot(f32::from(delta.y)) <= ACTIVE_SIZE * 0.5 + 0.05
                    }));
                }
            }
        }
    }

    #[test]
    fn determinate_progress_handles_numeric_boundaries() {
        for (input, expected) in [
            (f32::NAN, 0.),
            (f32::NEG_INFINITY, 0.),
            (-0.1, 0.),
            (0.5, 0.5),
            (1.1, 1.),
            (f32::INFINITY, 1.),
        ] {
            assert_eq!(
                LoadingIndicator::new("loading", "Loading")
                    .value(input)
                    .value,
                Some(expected)
            );
        }
        let shapes = &*DETERMINATE_SHAPES;
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        let start = determinate_sample(shapes, 0.);
        let end = determinate_sample(shapes, 1.);
        assert_eq!((start.progress, start.angle), (0., 0.));
        assert_eq!((end.progress, end.angle), (1., -TAU * 0.5));
        let start_path = start.path(bounds, ACTIVE_SIZE, shapes).unwrap();
        let end_path = end.path(bounds, ACTIVE_SIZE, shapes).unwrap();
        assert_ne!(start_path.vertices.len(), end_path.vertices.len());
    }

    #[test]
    fn a_custom_sequence_morphs_between_endpoints_and_wraps_only_when_indeterminate() {
        let rectangle = |w, h| {
            LoadingIndicatorShape::rounded_polygon(
                &[point(0., 0.), point(w, 0.), point(w, h), point(0., h)],
                0.,
            )
            .unwrap()
        };
        let custom = LoadingIndicatorShapes::new([
            rectangle(2., 1.),
            LoadingIndicatorShape::circle(),
            rectangle(1., 2.),
        ])
        .unwrap();
        let shapes = &custom.0;
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        let frame = |value| {
            determinate_sample(shapes, value)
                .path(bounds, ACTIVE_SIZE, shapes)
                .unwrap()
                .bounds
                .size
        };
        let start = frame(0.);
        let middle = frame(0.5);
        let end = frame(1.);
        assert!((start.width / start.height - 2.).abs() < 0.05);
        assert!((middle.width / middle.height - 1.).abs() < 0.05);
        assert!((end.height / end.width - 2.).abs() < 0.05);
        for value in [0.4999, 0.5001] {
            let adjacent = frame(value);
            assert!((adjacent.width - middle.width).abs() < px(0.05));
            assert!((adjacent.height - middle.height).abs() < px(0.05));
        }
        let first = indeterminate_sample(shapes, 0.);
        let wrap = indeterminate_sample(shapes, 2.);
        let repeated = indeterminate_sample(shapes, 3.);
        assert_eq!(first.step, 0);
        assert_eq!(wrap.step, 2);
        assert_eq!(repeated.step, 0);
        assert_eq!(
            (first.progress, wrap.progress, repeated.progress),
            (0., 0., 0.)
        );
    }

    #[test]
    fn custom_concave_paths_align_winding_and_fit_the_active_envelope() {
        let mut vertices = [
            point(-1., -1.),
            point(1., -1.),
            point(0., 0.),
            point(1., 1.),
            point(-1., 1.),
        ];
        let polygon = LoadingIndicatorShape::rounded_polygon(&vertices, 0.1).unwrap();
        vertices.reverse();
        let reversed = LoadingIndicatorShape::rounded_polygon(&vertices, 0.1).unwrap();
        assert_eq!(polygon, reversed);
        let circle = LoadingIndicatorShape::circle();
        let limit = polygon.0.cubic_count() + circle.0.cubic_count();
        let sequence = LoadingIndicatorShapes::new([polygon, circle]).unwrap();
        let shapes = &sequence.0;
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        for step in 0..=20 {
            let sample = determinate_sample(shapes, step as f32 / 20.);
            let outline = shapes
                .sample_step_polygon(sample.step, sample.progress)
                .unwrap();
            assert!(outline.cubic_count() < limit);
            let path = sample.path(bounds, ACTIVE_SIZE, shapes).unwrap();
            assert!(path.vertices.iter().all(|point| {
                let delta = point.xy_position - bounds.center();
                f32::from(delta.x).hypot(delta.y.into()) <= ACTIVE_SIZE * 0.5 + 0.05
            }));
        }
    }

    #[test]
    fn expressive_material_shapes_work_in_loading_indicator_sequences() {
        let sequence = LoadingIndicatorShapes::new([
            LoadingIndicatorShape::material(MaterialShape::Heart),
            LoadingIndicatorShape::from(MaterialShape::Cookie12Sided),
            LoadingIndicatorShape::from(ShapePolygon::material(MaterialShape::Clover4Leaf)),
        ])
        .unwrap();
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        let path = determinate_sample(&sequence.0, 0.5)
            .path(bounds, ACTIVE_SIZE, &sequence.0)
            .unwrap();
        assert!(!path.vertices.is_empty());
    }

    #[test]
    fn rejects_short_and_degenerate_shape_sequences() {
        assert!(LoadingIndicatorShapes::new([]).is_none());
        assert!(LoadingIndicatorShapes::new([LoadingIndicatorShape::circle()]).is_none());
        for outline in [
            &[point(0., 0.), point(0., 0.), point(0., 0.)][..],
            &[point(0., 0.), point(1., 1.), point(2., 2.)],
            &[point(0., 0.), point(1., 1.), point(1., 0.), point(0., 1.)],
            &[point(0., 0.), point(f32::INFINITY, 0.), point(2., 2.)],
        ] {
            assert!(LoadingIndicatorShape::rounded_polygon(outline, 0.).is_none());
        }
        for radius in [f32::NAN, -1.] {
            assert!(
                LoadingIndicatorShape::rounded_polygon(
                    &[point(0., 0.), point(1., 0.), point(1., 1.)],
                    radius,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn indeterminate_phase_pauses_and_reduced_motion_holds_the_start() {
        let mut motion = IndeterminateMotion::default();
        let start = Instant::now();
        let after = |millis| start + Duration::from_millis(millis);
        assert_eq!(motion.phase(start, true, false), 0.);
        let paused = motion.phase(after(200), false, false);
        assert!(paused > 0.);
        assert_eq!(motion.phase(after(400), false, false), paused);
        assert_eq!(motion.phase(after(550), true, false), paused);
        assert!(motion.phase(after(700), true, false) > paused);
        assert_eq!(motion.phase(after(800), false, true), 0.);
        assert_eq!(motion.phase(after(950), false, true), 0.);
        assert_eq!(motion.phase(after(1050), true, false), 0.);
    }
}
