use std::panic::{AssertUnwindSafe, catch_unwind};

use gpui::{Bounds, Pixels, px};
use gpui_material_theme::PolygonRole;
use polymorpher::{Cubic, Morph, geometry::Point as PolygonPoint};

use super::polygon::{ShapePolygon, cubic_center};
use super::{Outline, Shape, ShapeFit};

/// A reusable morph between any two [`ShapePolygon`] contours.
///
/// Uses `polymorpher`'s feature-matched morph when both endpoints are `RoundedPolygon`s
/// and falls back to perimeter-length cubic alignment when an endpoint is an arbitrary
/// cubic contour or triggers a feature-matching edge case.
#[derive(Clone)]
pub struct ShapeMorph {
    kind: MorphKind,
}

#[derive(Clone)]
enum MorphKind {
    FeatureMatched(Morph),
    PerimeterAligned(Vec<(Cubic, Cubic)>),
}

impl ShapeMorph {
    /// Creates a reusable morph between any two shapes (such as
    /// [`MaterialShape`](gpui_material_theme::MaterialShape), [`PolygonRole`], or
    /// [`ShapePolygon`]).
    pub fn new(start: impl Into<ShapePolygon>, end: impl Into<ShapePolygon>) -> Self {
        let start = start.into();
        let end = end.into();
        let kind = match (start.as_rounded(), end.as_rounded()) {
            (Some(from), Some(to)) => {
                match catch_unwind(AssertUnwindSafe(|| Morph::new(from.clone(), to.clone()))) {
                    Ok(morph) => MorphKind::FeatureMatched(morph),
                    Err(_) => {
                        MorphKind::PerimeterAligned(align_perimeter(&from.cubics, &to.cubics))
                    }
                }
            }
            _ => MorphKind::PerimeterAligned(align_perimeter(start.cubics(), end.cubics())),
        };
        Self { kind }
    }

    /// Creates a morph aligned strictly by normalized perimeter length, preserving exact
    /// endpoint curves with bounded cubic count even for arbitrary concave polygons.
    pub fn perimeter_aligned(start: impl Into<ShapePolygon>, end: impl Into<ShapePolygon>) -> Self {
        let start = start.into();
        let end = end.into();
        Self {
            kind: MorphKind::PerimeterAligned(align_perimeter(start.cubics(), end.cubics())),
        }
    }

    /// Samples the interpolated normalized contour at `progress` (clamped to `0..=1`).
    pub fn sample_polygon(&self, progress: f32) -> ShapePolygon {
        ShapePolygon::from_cubics(self.as_cubics(clamp_progress(progress)))
    }

    /// Fits the sampled shape inside `bounds` while preserving aspect ratio.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        self.sample_with(bounds, progress, ShapeFit::Contain, 0.)
    }

    /// Samples the morph into `bounds` with explicit fit mode and clockwise rotation in radians.
    pub fn sample_with(
        &self,
        bounds: Bounds<Pixels>,
        progress: f32,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Shape> {
        Shape::from_cubics(
            self.as_cubics(clamp_progress(progress)),
            bounds,
            fit,
            rotation_radians,
        )
    }

    fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        match &self.kind {
            MorphKind::FeatureMatched(morph) => morph.as_cubics(progress),
            MorphKind::PerimeterAligned(pairs) => pairs
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

/// A reusable multi-step sequence of [`ShapeMorph`] transitions.
#[derive(Clone)]
pub struct ShapeMorphSequence {
    steps: Vec<ShapeMorph>,
    wrap: Option<ShapeMorph>,
    rotation_scale: f32,
    control_radius: Option<f32>,
}

impl ShapeMorphSequence {
    /// Builds a feature-matched morph sequence from at least two shapes.
    /// When `looping` is `true`, adds a wrap transition from the last shape back to the first.
    pub fn new(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
    ) -> Option<Self> {
        Self::build(shapes, looping, false)
    }

    /// Builds a perimeter-aligned morph sequence from at least two shapes.
    pub fn perimeter_aligned(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
    ) -> Option<Self> {
        Self::build(shapes, looping, true)
    }

    fn build(
        shapes: impl IntoIterator<Item = impl Into<ShapePolygon>>,
        looping: bool,
        perimeter_aligned: bool,
    ) -> Option<Self> {
        let polygons: Vec<ShapePolygon> = shapes.into_iter().map(Into::into).collect();
        if polygons.len() < 2 {
            return None;
        }
        let make = |a: &ShapePolygon, b: &ShapePolygon| {
            if perimeter_aligned {
                ShapeMorph::perimeter_aligned(a.clone(), b.clone())
            } else {
                ShapeMorph::new(a.clone(), b.clone())
            }
        };
        let steps = polygons
            .windows(2)
            .map(|pair| make(&pair[0], &pair[1]))
            .collect();
        let wrap = looping.then(|| make(polygons.last().unwrap(), &polygons[0]));
        let rotation_scale = polygons
            .iter()
            .fold(1_f32, |scale, polygon| scale.min(polygon.rotation_scale()));
        let control_radius = perimeter_aligned.then(|| {
            polygons
                .iter()
                .map(ShapePolygon::control_point_radius)
                .fold(0_f32, f32::max)
        });
        Some(Self {
            steps,
            wrap,
            rotation_scale,
            control_radius,
        })
    }

    /// Number of endpoint shapes in this sequence.
    pub fn len(&self) -> usize {
        self.steps.len() + 1
    }

    /// Always `false` because valid sequences contain at least two shapes.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Number of morph transitions (`len() - 1` when open, `len()` when looping).
    pub fn step_count(&self) -> usize {
        self.steps.len() + usize::from(self.wrap.is_some())
    }

    pub fn is_looping(&self) -> bool {
        self.wrap.is_some()
    }

    /// Minimum rotation-safe scale factor across all shapes in the sequence.
    pub fn rotation_scale(&self) -> f32 {
        self.rotation_scale
    }

    /// Samples the open progression across `[0, 1]` from the first shape to the last.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        let value = clamp_progress(progress);
        let phase = value * self.steps.len() as f32;
        let index = (phase as usize).min(self.steps.len() - 1);
        self.sample_step(index, phase - index as f32, bounds, ShapeFit::Contain, 0.)
    }

    /// Samples a specific transition step at `progress` in `0..=1`.
    pub fn sample_step(
        &self,
        step: usize,
        progress: f32,
        bounds: Bounds<Pixels>,
        fit: ShapeFit,
        rotation_radians: f32,
    ) -> Option<Shape> {
        let morph = self.morph_for_step(step)?;
        morph.sample_with(bounds, progress, fit, rotation_radians)
    }

    /// Samples a specific transition step as a reusable [`ShapePolygon`].
    pub fn sample_step_polygon(&self, step: usize, progress: f32) -> Option<ShapePolygon> {
        let morph = self.morph_for_step(step)?;
        Some(morph.sample_polygon(progress))
    }

    /// Samples a transition step centered on its live cubic bounding box and scaled to
    /// fit safely inside a circular envelope of diameter `extent` under arbitrary rotation.
    ///
    /// Used by `LoadingIndicator` to keep rotating Material and custom morph sequences
    /// strictly inside their active footprint without resizing shared endpoints.
    pub fn sample_step_in_envelope(
        &self,
        step: usize,
        progress: f32,
        bounds: Bounds<Pixels>,
        extent: Pixels,
        rotation_radians: f32,
    ) -> Option<Shape> {
        if !rotation_radians.is_finite() {
            return None;
        }
        let morph = self.morph_for_step(step)?;
        let cubics = morph.as_cubics(clamp_progress(progress));
        if cubics.is_empty() {
            return None;
        }
        let pivot = cubic_center(&cubics);
        let side = bounds.size.width.min(bounds.size.height);
        let extent = side.min(extent);
        if !(extent > px(0.) && extent < px(f32::INFINITY)) {
            return None;
        }
        let mut target_side = extent * self.rotation_scale;
        if let Some(radius) = self.control_radius {
            let radius = radius + (pivot - PolygonPoint::splat(0.5)).length();
            if radius > f32::EPSILON {
                target_side = target_side.min(extent / (2. * radius));
            }
        }
        let scale = f32::from(target_side) / f32::from(side);
        Shape::new(Outline {
            cubics,
            bounds,
            fit: ShapeFit::Contain,
            rotation: rotation_radians,
            pivot,
            scale,
        })
    }

    fn morph_for_step(&self, step: usize) -> Option<&ShapeMorph> {
        if self.wrap.is_some() {
            let count = self.steps.len() + 1;
            let index = step % count;
            self.steps.get(index).or(self.wrap.as_ref())
        } else {
            self.steps.get(step)
        }
    }
}

/// A reusable morph between two pinned Material polygon roles.
#[derive(Clone)]
pub struct PolygonMorph(ShapeMorph);

impl PolygonMorph {
    pub fn new(start: PolygonRole, end: PolygonRole) -> Self {
        Self(ShapeMorph::new(start, end))
    }

    /// Fits the sampled polygon inside the bounds while preserving aspect ratio.
    pub fn sample(&self, bounds: Bounds<Pixels>, progress: f32) -> Option<Shape> {
        self.0.sample(bounds, progress)
    }
}

fn clamp_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0., 1.)
    } else {
        0.
    }
}

fn align_perimeter(start: &[Cubic], end: &[Cubic]) -> Vec<(Cubic, Cubic)> {
    let mut starts = weighted_segments(start).into_iter();
    let mut ends = weighted_segments(end).into_iter();
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
    pairs
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
    if cumulative > 0. {
        for (end, _) in &mut segments {
            *end /= cumulative;
        }
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
