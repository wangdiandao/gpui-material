//! Loading indicators and rounded polygon constructors for custom sequences.

use std::{f32::consts::TAU, rc::Rc, sync::LazyLock, time::Duration};

use gpui::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Path,
    PathBuilder, Pixels, Point, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, canvas, div, point, px,
};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerShape};
use polymorpher::{
    CornerRounding, Cubic, Morph, RoundedPolygon, geometry::Point as PolygonPoint, shapes,
};
use web_time::Instant;

const CONTAINER_SIZE: f32 = 48.;
const ACTIVE_SIZE: f32 = 38.;
const MORPH_INTERVAL: f32 = 0.65;

static INDETERMINATE_SHAPES: LazyLock<MorphSequence> = LazyLock::new(|| {
    MorphSequence::new(
        &[
            shapes::soft_burst(),
            shapes::cookie9(),
            shapes::pentagon(),
            shapes::pill(),
            shapes::sunny(),
            shapes::cookie4(),
            shapes::oval(),
        ],
        true,
        MorphPolicy::Material,
    )
});

static DETERMINATE_SHAPES: LazyLock<MorphSequence> = LazyLock::new(|| {
    let (sin, cos) = (TAU / 20.).sin_cos();
    let circle = RoundedPolygon::circle()
        .with_vertices(10)
        .build()
        .transformed(|p: PolygonPoint| {
            PolygonPoint::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
        });
    MorphSequence::new(
        &[circle, shapes::soft_burst()],
        false,
        MorphPolicy::Material,
    )
});

/// An endpoint shape for a loading indicator morph.
#[derive(Clone)]
pub struct LoadingIndicatorShape(RoundedPolygon);

impl LoadingIndicatorShape {
    pub fn circle() -> Self {
        Self(shapes::circle(None))
    }

    /// Creates a rounded polygon from its perimeter vertices in either winding order.
    /// The radius uses the same units as the vertices. Invalid geometry returns `None`.
    pub fn rounded_polygon(vertices: &[Point<f32>], radius: f32) -> Option<Self> {
        if vertices.len() < 3 || !radius.is_finite() || radius < 0. {
            return None;
        }
        let (mut vertices, side) = normalized_vertices(vertices)?;
        if !simple_polygon(&vertices) {
            return None;
        }
        let area: f32 = polygon_edges(&vertices)
            .map(|(a, b)| a.x * b.y - a.y * b.x)
            .sum();
        if !area.is_finite() || area.abs() < 1e-5 {
            return None;
        }
        if area < 0. {
            vertices.reverse();
        }
        let first = vertices
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.x.total_cmp(&b.x).then_with(|| b.y.total_cmp(&a.y)))?
            .0;
        vertices.rotate_left(first);
        let rounding = CornerRounding::new((f64::from(radius) / side).min(1.) as f32);
        let polygon = RoundedPolygon::from_vertices(&vertices, rounding, &[], PolygonPoint::zero());
        (polygon.cubics.len() > 1).then_some(Self(polygon))
    }
}

/// A prepared sequence of rounded polygons for a loading indicator.
#[derive(Clone)]
pub struct LoadingIndicatorShapes(Rc<MorphSequence>);

impl LoadingIndicatorShapes {
    /// Returns `None` when the sequence contains fewer than two shapes.
    pub fn new(shapes: impl IntoIterator<Item = LoadingIndicatorShape>) -> Option<Self> {
        let polygons: Vec<_> = shapes.into_iter().map(|shape| shape.0).collect();
        if polygons.len() < 2 {
            return None;
        }
        Some(Self(Rc::new(MorphSequence::new(
            &polygons,
            true,
            MorphPolicy::Custom,
        ))))
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
                    Some(value) => shapes.determinate(value),
                    None => shapes.indeterminate(phase),
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

struct MorphSequence {
    steps: Vec<MorphStep>,
    wrap: Option<MorphStep>,
    scale: f32,
    custom_radius: Option<f32>,
}

enum MorphPolicy {
    Material,
    Custom,
}

impl MorphSequence {
    fn new(shapes: &[RoundedPolygon], looping: bool, policy: MorphPolicy) -> Self {
        let normalized: Vec<_> = shapes
            .iter()
            .cloned()
            .map(RoundedPolygon::normalized)
            .collect();
        let step = match policy {
            MorphPolicy::Material => MorphStep::material,
            MorphPolicy::Custom => MorphStep::custom,
        };
        // Every control point of a subdivision or interpolation lies within
        // the control-point envelope of the source shapes. Use one envelope
        // for the full sequence so subdivision cannot resize a shared endpoint.
        let custom_radius = matches!(policy, MorphPolicy::Custom).then(|| {
            normalized
                .iter()
                .flat_map(|polygon| &polygon.cubics)
                .flat_map(|cubic| {
                    [
                        cubic.anchor0(),
                        cubic.control0(),
                        cubic.control1(),
                        cubic.anchor1(),
                    ]
                })
                .map(|point| (point - PolygonPoint::splat(0.5)).length())
                .fold(0_f32, f32::max)
        });
        Self {
            steps: normalized
                .windows(2)
                .map(|pair| step(&pair[0], &pair[1]))
                .collect(),
            wrap: looping.then(|| step(&normalized[normalized.len() - 1], &normalized[0])),
            scale: loading_scale(shapes),
            custom_radius,
        }
    }

    fn determinate(&self, value: f32) -> MorphSample<'_> {
        let phase = value * self.steps.len() as f32;
        let index = (phase as usize).min(self.steps.len() - 1);
        MorphSample {
            morph: &self.steps[index],
            progress: phase - index as f32,
            angle: -value * TAU * 0.5,
        }
    }

    fn indeterminate(&self, phase: f32) -> MorphSample<'_> {
        let count = self.steps.len() + 1;
        let step = phase.floor();
        let index = step as usize % count;
        let time = phase.fract() * MORPH_INTERVAL;
        let frequency = 200_f32.sqrt();
        let damped = frequency * 0.8 * time;
        let progress = (1.
            - (-0.6 * frequency * time).exp() * (damped.cos() + 0.75 * damped.sin()))
        .clamp(0., 1.);
        MorphSample {
            morph: self.steps.get(index).unwrap_or_else(|| {
                self.wrap
                    .as_ref()
                    .expect("indeterminate morph sequences wrap")
            }),
            progress,
            angle: (phase * MORPH_INTERVAL / 4.666 + (1. + step + progress) / 4.) * TAU,
        }
    }
}

struct MorphSample<'a> {
    morph: &'a MorphStep,
    progress: f32,
    angle: f32,
}

impl MorphSample<'_> {
    fn path(
        self,
        bounds: Bounds<Pixels>,
        extent: f32,
        sequence: &MorphSequence,
    ) -> Option<Path<Pixels>> {
        let cubics = self.morph.as_cubics(self.progress);
        let first = cubics.first()?;
        let center = cubic_center(&cubics);
        let extent = bounds.size.width.min(bounds.size.height).min(px(extent));
        let scaled = extent * sequence.scale;
        let side = if let Some(radius) = sequence.custom_radius {
            let radius = radius + (center - PolygonPoint::splat(0.5)).length();
            scaled.min(extent / (2. * radius))
        } else {
            scaled
        };
        let (sin, cos) = self.angle.sin_cos();
        let project = |p: PolygonPoint| {
            let x = p.x - center.x;
            let y = p.y - center.y;
            bounds.center() + point(side * (cos * x - sin * y), side * (sin * x + cos * y))
        };
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
        path.build().ok()
    }
}

enum MorphStep {
    Material(Morph),
    Custom(Vec<(Cubic, Cubic)>),
}

impl MorphStep {
    fn material(start: &RoundedPolygon, end: &RoundedPolygon) -> Self {
        Self::Material(Morph::new(start.clone(), end.clone()))
    }

    fn custom(start: &RoundedPolygon, end: &RoundedPolygon) -> Self {
        // Align by perimeter length without the feature-matching assertions
        // used for Material's fixed shapes. Splitting retains the exact curves
        // at both ends even if the outlines have different segment counts.
        let mut starts = weighted_segments(&start.cubics).into_iter();
        let mut ends = weighted_segments(&end.cubics).into_iter();
        let (mut from, mut to) = (starts.next(), ends.next());
        let mut position = 0.;
        let mut pairs = Vec::new();
        while let (Some(mut a), Some(mut b)) = (from, to) {
            let boundary = a.0.min(b.0);
            let first = split_until(&mut a, position, boundary);
            let last = split_until(&mut b, position, boundary);
            pairs.push((first, last));
            from = if boundary == a.0 {
                starts.next()
            } else {
                Some(a)
            };
            to = if boundary == b.0 {
                ends.next()
            } else {
                Some(b)
            };
            position = boundary;
        }
        Self::Custom(pairs)
    }

    fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        match self {
            Self::Material(morph) => morph.as_cubics(progress),
            Self::Custom(pairs) => pairs
                .iter()
                .map(|(a, b)| {
                    Cubic::new(
                        a.anchor0().lerp(b.anchor0(), progress),
                        a.control0().lerp(b.control0(), progress),
                        a.control1().lerp(b.control1(), progress),
                        a.anchor1().lerp(b.anchor1(), progress),
                    )
                })
                .collect(),
        }
    }
}

fn weighted_segments(cubics: &[Cubic]) -> Vec<(f64, Cubic)> {
    let mut cumulative = 0.;
    let mut segments: Vec<_> = cubics
        .iter()
        .map(|cubic| {
            let samples = [
                cubic.anchor0(),
                cubic.point_on_curve(0.25),
                cubic.point_on_curve(0.5),
                cubic.point_on_curve(0.75),
                cubic.anchor1(),
            ];
            cumulative += samples
                .windows(2)
                .map(|points| f64::from((points[1] - points[0]).length()))
                .sum::<f64>();
            (cumulative, *cubic)
        })
        .collect();
    for (end, _) in &mut segments {
        *end /= cumulative;
    }
    segments
}

fn split_until(segment: &mut (f64, Cubic), from: f64, until: f64) -> Cubic {
    if until == segment.0 {
        return segment.1;
    }
    let (head, tail) = segment
        .1
        .split(((until - from) / (segment.0 - from)) as f32);
    segment.1 = tail;
    head
}

fn normalized_vertices(vertices: &[Point<f32>]) -> Option<(Vec<PolygonPoint>, f64)> {
    let (mut left, mut top, mut right, mut bottom) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for point in vertices {
        if !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let x = f64::from(point.x);
        let y = f64::from(point.y);
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
    }
    let side = (right - left).max(bottom - top);
    if side == 0. {
        return None;
    }
    let centered = vertices.iter().map(|point| {
        PolygonPoint::new(
            ((f64::from(point.x) - (left + right) * 0.5) / side) as f32,
            ((f64::from(point.y) - (top + bottom) * 0.5) / side) as f32,
        )
    });
    Some((centered.collect(), side))
}

fn polygon_edges(
    vertices: &[PolygonPoint],
) -> impl Iterator<Item = (PolygonPoint, PolygonPoint)> + Clone + '_ {
    vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
}

fn simple_polygon(vertices: &[PolygonPoint]) -> bool {
    let edges = polygon_edges(vertices);
    for (index, (a, b)) in edges.clone().enumerate() {
        if (b - a).length() < 1e-5 {
            return false;
        }
        for (other, (c, d)) in edges.clone().enumerate().skip(index + 2) {
            if index == 0 && other == vertices.len() - 1 {
                continue;
            }
            let orient = |a: PolygonPoint, b: PolygonPoint, c: PolygonPoint| {
                let u = b - a;
                let v = c - a;
                u.x * v.y - u.y * v.x
            };
            let overlaps =
                |a: f32, b: f32, c: f32, d: f32| a.min(b) <= c.max(d) && c.min(d) <= a.max(b);
            if overlaps(a.x, b.x, c.x, d.x)
                && overlaps(a.y, b.y, c.y, d.y)
                && orient(a, b, c) * orient(a, b, d) <= 0.
                && orient(c, d, a) * orient(c, d, b) <= 0.
            {
                return false;
            }
        }
    }
    true
}

// The pinned Compose implementation compares the ordinary bounds with the
// shape's rotation envelope, sampled at each cubic's anchor and midpoint.
fn loading_scale(shapes: &[RoundedPolygon]) -> f32 {
    shapes.iter().fold(1., |scale, shape| {
        let bounds = shape.aabb(true).size();
        let radius = shape
            .cubics
            .iter()
            .flat_map(|cubic| [cubic.anchor0(), cubic.point_on_curve(0.5)])
            .map(|point| (point - shape.center).length())
            .fold(0_f32, f32::max);
        scale.min(bounds.width.max(bounds.height) / (2. * radius))
    })
}

fn cubic_center(cubics: &[Cubic]) -> PolygonPoint {
    let mut minimum = PolygonPoint::new(f32::INFINITY, f32::INFINITY);
    let mut maximum = PolygonPoint::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for cubic in cubics {
        let bounds = cubic.aabb(true);
        minimum = minimum.min(bounds.min);
        maximum = maximum.max(bounds.max);
    }
    minimum + (maximum - minimum) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn loading_shapes_fit_the_active_size_through_rotation_and_morphs() {
        let shape_sets = [&*INDETERMINATE_SHAPES, &*DETERMINATE_SHAPES];
        for shapes in shape_sets {
            assert!(shapes.scale > 0. && shapes.scale <= 1.);
            for morph in shapes.steps.iter().chain(shapes.wrap.iter()) {
                for step in 0..=20 {
                    let cubics = morph.as_cubics(step as f32 / 20.);
                    let center = cubic_center(&cubics);
                    for cubic in cubics {
                        for sample in 0..=20 {
                            let p = cubic.point_on_curve(sample as f32 / 20.);
                            let radius = (p - center).length() * shapes.scale;
                            assert!(
                                radius <= 0.501,
                                "shape escapes its active envelope: {radius}"
                            );
                        }
                    }
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
        let start = shapes.determinate(0.);
        let end = shapes.determinate(1.);
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
            shapes
                .determinate(value)
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
        let first = shapes.indeterminate(0.);
        let wrap = shapes.indeterminate(2.);
        let repeated = shapes.indeterminate(3.);
        assert!(std::ptr::eq(first.morph, repeated.morph));
        assert!(std::ptr::eq(wrap.morph, shapes.wrap.as_ref().unwrap()));
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
        assert_eq!(polygon.0.cubics, reversed.0.cubics);
        let circle = LoadingIndicatorShape::circle();
        let limit = polygon.0.cubics.len() + circle.0.cubics.len();
        let sequence = LoadingIndicatorShapes::new([polygon, circle]).unwrap();
        let shapes = &sequence.0;
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(48.), px(48.)));
        for step in 0..=20 {
            let sample = shapes.determinate(step as f32 / 20.);
            let outline = sample.morph.as_cubics(sample.progress);
            assert!(outline.len() < limit);
            for (a, b) in outline.iter().zip(outline.iter().cycle().skip(1)) {
                assert!((a.anchor1() - b.anchor0()).length() < 1e-5);
            }
            let path = sample.path(bounds, ACTIVE_SIZE, shapes).unwrap();
            assert!(path.vertices.iter().all(|point| {
                let delta = point.xy_position - bounds.center();
                f32::from(delta.x).hypot(delta.y.into()) <= ACTIVE_SIZE * 0.5 + 0.05
            }));
        }
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
