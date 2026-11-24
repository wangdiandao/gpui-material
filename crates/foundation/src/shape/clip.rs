use gpui::{
    Bounds, FillOptions, FillRule, Path, PathBuilder, PathStyle, Pixels, Point, point, px, size,
};

const RIPPLE_SEGMENTS: usize = 64;
const FLATTEN_TOLERANCE: f32 = 0.25;
const MAX_SUBDIVISION_DEPTH: u8 = 10;

/// Disjoint triangle mesh used for exact geometric containment, hit testing, and clipping.
#[derive(Clone, Debug)]
pub(super) struct FillMesh {
    pub(super) bounds: Bounds<Pixels>,
    pub(super) triangles: Vec<[Point<Pixels>; 3]>,
}

impl FillMesh {
    pub(super) fn empty(origin: Point<Pixels>) -> Self {
        Self {
            bounds: Bounds::new(origin, size(px(0.), px(0.))),
            triangles: Vec::new(),
        }
    }

    pub(super) fn from_triangles(triangles: Vec<[Point<Pixels>; 3]>) -> Self {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for tri in &triangles {
            for &v in tri {
                let x = f32::from(v.x);
                let y = f32::from(v.y);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        let bounds = if min_x.is_finite() && min_y.is_finite() {
            Bounds::new(
                point(px(min_x), px(min_y)),
                size(px((max_x - min_x).max(0.)), px((max_y - min_y).max(0.))),
            )
        } else {
            Bounds::default()
        };
        Self { bounds, triangles }
    }

    /// Triangulates a convex closed polygon contour (such as a rounded rectangle)
    /// in O(n) time using a centroid triangle fan.
    pub(super) fn from_convex_contour(contour: &[Point<Pixels>]) -> Self {
        let mut pts = clean_contour(contour);
        if pts.len() < 3 {
            return Self::empty(pts.first().copied().unwrap_or(point(px(0.), px(0.))));
        }
        let area = polygon_area(&pts);
        if !area.is_finite() || area.abs() <= f32::EPSILON {
            return Self::empty(pts[0]);
        }
        if area < 0. {
            pts.reverse();
        }
        let inv_n = 1. / pts.len() as f32;
        let (mut cx, mut cy) = (0_f32, 0_f32);
        for &p in &pts {
            cx += f32::from(p.x);
            cy += f32::from(p.y);
        }
        let centroid = point(px(cx * inv_n), px(cy * inv_n));
        let mut triangles = Vec::with_capacity(pts.len());
        for i in 0..pts.len() {
            let a = pts[i];
            let b = pts[(i + 1) % pts.len()];
            if cross(a - centroid, b - centroid).abs() > f32::EPSILON {
                triangles.push([centroid, a, b]);
            }
        }
        Self::from_triangles(triangles)
    }

    /// Triangulates a simple closed polygon contour (convex or concave, either winding)
    /// into disjoint triangles via ear clipping.
    pub(super) fn from_contour(contour: &[Point<Pixels>]) -> Self {
        let mut pts = clean_contour(contour);
        if pts.len() < 3 {
            return Self::empty(pts.first().copied().unwrap_or(point(px(0.), px(0.))));
        }

        let area = polygon_area(&pts);
        if !area.is_finite() || area.abs() <= f32::EPSILON {
            return Self::empty(pts[0]);
        }
        // Normalize to positive signed area (CCW in standard coordinates).
        if area < 0. {
            pts.reverse();
        }

        let mut indices: Vec<usize> = (0..pts.len()).collect();
        let mut triangles = Vec::with_capacity(pts.len().saturating_sub(2));

        let mut guard = indices.len() * indices.len() + 16;
        while indices.len() > 3 && guard > 0 {
            guard -= 1;
            let len = indices.len();
            let mut clipped = false;
            for i in 0..len {
                let prev = indices[(i + len - 1) % len];
                let curr = indices[i];
                let next = indices[(i + 1) % len];
                let a = pts[prev];
                let b = pts[curr];
                let c = pts[next];
                let winding = cross(b - a, c - a);
                if winding <= f32::EPSILON {
                    continue;
                }
                let has_point_inside = indices.iter().any(|&idx| {
                    if idx == prev || idx == curr || idx == next {
                        return false;
                    }
                    point_in_triangle_strict(pts[idx], a, b, c)
                });
                if !has_point_inside {
                    triangles.push([a, b, c]);
                    indices.remove(i);
                    clipped = true;
                    break;
                }
            }
            if !clipped {
                // Fallback for nearly collinear vertices: clip the ear with smallest non-negative area.
                let mut best: Option<(usize, f32)> = None;
                for i in 0..len {
                    let prev = indices[(i + len - 1) % len];
                    let curr = indices[i];
                    let next = indices[(i + 1) % len];
                    let winding = cross(pts[curr] - pts[prev], pts[next] - pts[prev]);
                    if winding >= -1e-4 && best.is_none_or(|(_, w)| winding > w) {
                        best = Some((i, winding));
                    }
                }
                let idx = best.map_or(0, |(i, _)| i);
                let prev = indices[(idx + len - 1) % len];
                let curr = indices[idx];
                let next = indices[(idx + 1) % len];
                if cross(pts[curr] - pts[prev], pts[next] - pts[prev]).abs() > f32::EPSILON {
                    triangles.push([pts[prev], pts[curr], pts[next]]);
                }
                indices.remove(idx);
            }
        }
        if indices.len() == 3 {
            let [a, b, c] = [pts[indices[0]], pts[indices[1]], pts[indices[2]]];
            if cross(b - a, c - a).abs() > f32::EPSILON {
                triangles.push([a, b, c]);
            }
        }

        Self::from_triangles(triangles)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub(super) fn push_triangle(&mut self, a: Point<Pixels>, b: Point<Pixels>, c: Point<Pixels>) {
        self.triangles.push([a, b, c]);
    }

    pub(super) fn finalize_bounds(&mut self) {
        if self.triangles.is_empty() {
            return;
        }
        let rebuilt = Self::from_triangles(std::mem::take(&mut self.triangles));
        *self = rebuilt;
    }

    /// Converts the disjoint triangle mesh into a paintable GPUI [`Path`].
    pub(super) fn to_path(&self) -> Option<Path> {
        if self.triangles.is_empty() {
            return None;
        }
        let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions {
            fill_rule: FillRule::NonZero,
        }));
        for &[a, b, c] in &self.triangles {
            let winding = cross(b - a, c - a);
            if winding.abs() <= f32::EPSILON {
                continue;
            }
            builder.move_to(a);
            if winding > 0. {
                builder.line_to(b);
                builder.line_to(c);
            } else {
                builder.line_to(c);
                builder.line_to(b);
            }
            builder.close();
        }
        builder.build().ok()
    }
}

fn clean_contour(contour: &[Point<Pixels>]) -> Vec<Point<Pixels>> {
    let mut pts: Vec<Point<Pixels>> = Vec::with_capacity(contour.len());
    for &p in contour {
        if !f32::from(p.x).is_finite() || !f32::from(p.y).is_finite() {
            continue;
        }
        if let Some(&prev) = pts.last() {
            let dx = f32::from(p.x - prev.x);
            let dy = f32::from(p.y - prev.y);
            if dx * dx + dy * dy <= 1e-6 {
                continue;
            }
        }
        pts.push(p);
    }
    while pts.len() >= 2 {
        let first = pts[0];
        let Some(&last) = pts.last() else {
            break;
        };
        let dx = f32::from(first.x - last.x);
        let dy = f32::from(first.y - last.y);
        if dx * dx + dy * dy <= 1e-6 {
            pts.pop();
        } else {
            break;
        }
    }
    pts
}

fn point_in_triangle_strict(
    p: Point<Pixels>,
    a: Point<Pixels>,
    b: Point<Pixels>,
    c: Point<Pixels>,
) -> bool {
    let c1 = cross(b - a, p - a);
    let c2 = cross(c - b, p - b);
    let c3 = cross(a - c, p - c);
    c1 >= -1e-5 && c2 >= -1e-5 && c3 >= -1e-5
}

pub(super) fn flatten_cubic(
    p0: Point<Pixels>,
    c0: Point<Pixels>,
    c1: Point<Pixels>,
    p1: Point<Pixels>,
    out: &mut Vec<Point<Pixels>>,
) {
    flatten_cubic_recursive(p0, c0, c1, p1, 0, out);
}

fn flatten_cubic_recursive(
    p0: Point<Pixels>,
    c0: Point<Pixels>,
    c1: Point<Pixels>,
    p1: Point<Pixels>,
    depth: u8,
    out: &mut Vec<Point<Pixels>>,
) {
    // Wang's / de Casteljau flatness test against chord p0 -> p1.
    let ux = 3. * f32::from(c0.x) - 2. * f32::from(p0.x) - f32::from(p1.x);
    let uy = 3. * f32::from(c0.y) - 2. * f32::from(p0.y) - f32::from(p1.y);
    let vx = 3. * f32::from(c1.x) - 2. * f32::from(p1.x) - f32::from(p0.x);
    let vy = 3. * f32::from(c1.y) - 2. * f32::from(p1.y) - f32::from(p0.y);
    let err_sq = (ux * ux).max(vx * vx) + (uy * uy).max(vy * vy);
    if depth >= MAX_SUBDIVISION_DEPTH || err_sq <= 16. * FLATTEN_TOLERANCE * FLATTEN_TOLERANCE {
        out.push(p1);
        return;
    }
    let mid = |a: Point<Pixels>, b: Point<Pixels>| point((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let p01 = mid(p0, c0);
    let p12 = mid(c0, c1);
    let p23 = mid(c1, p1);
    let p012 = mid(p01, p12);
    let p123 = mid(p12, p23);
    let p0123 = mid(p012, p123);
    flatten_cubic_recursive(p0, p01, p012, p0123, depth + 1, out);
    flatten_cubic_recursive(p0123, p123, p23, p1, depth + 1, out);
}

pub(super) fn cross(a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    f32::from(a.x) * f32::from(b.y) - f32::from(a.y) * f32::from(b.x)
}

pub(super) fn segment_distance_sq(p: Point<Pixels>, a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    let ab_x = f32::from(b.x - a.x);
    let ab_y = f32::from(b.y - a.y);
    let ap_x = f32::from(p.x - a.x);
    let ap_y = f32::from(p.y - a.y);
    let len_sq = ab_x * ab_x + ab_y * ab_y;
    let t = if len_sq <= f32::EPSILON {
        0.
    } else {
        ((ap_x * ab_x + ap_y * ab_y) / len_sq).clamp(0., 1.)
    };
    let dx = ap_x - ab_x * t;
    let dy = ap_y - ab_y * t;
    dx * dx + dy * dy
}

pub(super) fn polygon_area(vertices: &[Point<Pixels>]) -> f32 {
    let Some(&first) = vertices.first() else {
        return 0.;
    };
    vertices
        .windows(2)
        .skip(1)
        .map(|edge| cross(edge[0] - first, edge[1] - first))
        .sum::<f32>()
        * 0.5
}

pub(super) fn rect_is_valid(rect: Bounds<Pixels>) -> bool {
    let x = f32::from(rect.origin.x);
    let y = f32::from(rect.origin.y);
    let w = f32::from(rect.size.width);
    let h = f32::from(rect.size.height);
    x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite() && w > 0. && h > 0.
}

pub(super) fn bounds_overlap(a: Bounds<Pixels>, b: Bounds<Pixels>) -> bool {
    a.left() < b.right() && a.right() > b.left() && a.top() < b.bottom() && a.bottom() > b.top()
}

pub(super) fn triangle_disjoint_from_rect(
    vertices: [Point<Pixels>; 3],
    rect: Bounds<Pixels>,
) -> bool {
    let min_x = rect.left();
    let max_x = rect.right();
    let min_y = rect.top();
    let max_y = rect.bottom();
    vertices.iter().all(|v| v.x <= min_x)
        || vertices.iter().all(|v| v.x >= max_x)
        || vertices.iter().all(|v| v.y <= min_y)
        || vertices.iter().all(|v| v.y >= max_y)
}

pub(super) fn contains_fill(mesh: &FillMesh, position: Point<Pixels>) -> bool {
    mesh.bounds.contains(&position)
        && mesh.triangles.iter().any(|&[a, b, c]| {
            let area = cross(b - a, c - a);
            if area.abs() <= f32::EPSILON {
                return false;
            }
            let signs = [
                cross(b - a, position - a),
                cross(c - b, position - b),
                cross(a - c, position - c),
            ];
            signs.into_iter().all(|sign| sign * area >= 0.)
        })
}

fn clip_half_plane(
    polygon: &mut Vec<Point<Pixels>>,
    clipped: &mut Vec<Point<Pixels>>,
    distance: impl Fn(Point<Pixels>) -> f32,
) {
    let Some(&last) = polygon.last() else {
        return;
    };
    let mut previous = last;
    let mut previous_distance = distance(previous);
    clipped.clear();
    for &current in polygon.iter() {
        let current_distance = distance(current);
        if (previous_distance >= 0.) != (current_distance >= 0.) {
            let t = previous_distance / (previous_distance - current_distance);
            clipped.push(previous + (current - previous) * t);
        }
        if current_distance >= 0. {
            clipped.push(current);
        }
        previous = current;
        previous_distance = current_distance;
    }
    std::mem::swap(polygon, clipped);
}

pub(super) fn clip_polygon_to_rect(
    polygon: &mut Vec<Point<Pixels>>,
    scratch: &mut Vec<Point<Pixels>>,
    rect: Bounds<Pixels>,
) {
    let left = f32::from(rect.left());
    let right = f32::from(rect.right());
    let top = f32::from(rect.top());
    let bottom = f32::from(rect.bottom());
    clip_half_plane(polygon, scratch, |p| f32::from(p.x) - left);
    clip_half_plane(polygon, scratch, |p| right - f32::from(p.x));
    clip_half_plane(polygon, scratch, |p| f32::from(p.y) - top);
    clip_half_plane(polygon, scratch, |p| bottom - f32::from(p.y));
}

pub(super) fn clip_rect_fill(fill: &FillMesh, rect: Bounds<Pixels>) -> FillMesh {
    let mut result = FillMesh::empty(rect.origin);
    if !rect_is_valid(rect) || fill.is_empty() || !bounds_overlap(fill.bounds, rect) {
        return result;
    }
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for &[a, b, c] in &fill.triangles {
        if triangle_disjoint_from_rect([a, b, c], rect) || cross(b - a, c - a).abs() <= f32::EPSILON
        {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        clip_polygon_to_rect(&mut polygon, &mut scratch, rect);
        push_fan(&mut result, &polygon);
    }
    result.finalize_bounds();
    result
}

pub(super) fn clip_convex_polygon_fill(fill: &FillMesh, vertices: &[Point<Pixels>]) -> FillMesh {
    let origin = vertices.first().copied().unwrap_or(point(px(0.), px(0.)));
    let mut result = FillMesh::empty(origin);
    if vertices.len() < 3
        || fill.is_empty()
        || !vertices
            .iter()
            .all(|v| f32::from(v.x).is_finite() && f32::from(v.y).is_finite())
    {
        return result;
    }
    let area = polygon_area(vertices);
    if !area.is_finite() || area.abs() <= f32::EPSILON {
        return result;
    }
    let orientation = if area > 0. { 1. } else { -1. };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    for &v in vertices {
        let x = f32::from(v.x);
        let y = f32::from(v.y);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let clip_bounds = Bounds::new(
        point(px(min_x), px(min_y)),
        size(px(max_x - min_x), px(max_y - min_y)),
    );
    if !bounds_overlap(fill.bounds, clip_bounds) {
        return result;
    }
    let mut polygon = Vec::with_capacity(vertices.len() + 3);
    let mut scratch = Vec::with_capacity(vertices.len() + 3);
    for &[a, b, c] in &fill.triangles {
        if triangle_disjoint_from_rect([a, b, c], clip_bounds)
            || cross(b - a, c - a).abs() <= f32::EPSILON
        {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        for (&start, &end) in vertices.iter().zip(vertices.iter().cycle().skip(1)) {
            if polygon.is_empty() {
                break;
            }
            clip_half_plane(&mut polygon, &mut scratch, |p| {
                orientation * cross(end - start, p - start)
            });
        }
        push_fan(&mut result, &polygon);
    }
    result.finalize_bounds();
    result
}

pub(super) fn clip_mesh_fill(clip: &FillMesh, source: &FillMesh) -> FillMesh {
    let mut result = FillMesh::empty(source.bounds.origin);
    if clip.is_empty() || source.is_empty() || !bounds_overlap(clip.bounds, source.bounds) {
        return result;
    }
    let clip_triangles: Vec<_> = clip
        .triangles
        .iter()
        .filter_map(|&[a, b, c]| {
            let winding = cross(b - a, c - a);
            if winding.abs() <= f32::EPSILON
                || triangle_disjoint_from_rect([a, b, c], source.bounds)
            {
                return None;
            }
            let min_x = a.x.min(b.x).min(c.x);
            let max_x = a.x.max(b.x).max(c.x);
            let min_y = a.y.min(b.y).min(c.y);
            let max_y = a.y.max(b.y).max(c.y);
            let sign = if winding > 0. { 1. } else { -1. };
            Some(([a, b, c], min_x, max_x, min_y, max_y, sign))
        })
        .collect();
    if clip_triangles.is_empty() {
        return result;
    }

    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for &[sa, sb, sc] in &source.triangles {
        if cross(sb - sa, sc - sa).abs() <= f32::EPSILON
            || triangle_disjoint_from_rect([sa, sb, sc], clip.bounds)
        {
            continue;
        }
        let s_min_x = sa.x.min(sb.x).min(sc.x);
        let s_max_x = sa.x.max(sb.x).max(sc.x);
        let s_min_y = sa.y.min(sb.y).min(sc.y);
        let s_max_y = sa.y.max(sb.y).max(sc.y);
        for &([ca, cb, cc], c_min_x, c_max_x, c_min_y, c_max_y, sign) in &clip_triangles {
            if s_max_x <= c_min_x || s_min_x >= c_max_x || s_max_y <= c_min_y || s_min_y >= c_max_y
            {
                continue;
            }
            polygon.clear();
            polygon.extend([sa, sb, sc]);
            for (start, end) in [(ca, cb), (cb, cc), (cc, ca)] {
                if polygon.is_empty() {
                    break;
                }
                clip_half_plane(&mut polygon, &mut scratch, |p| {
                    sign * cross(end - start, p - start)
                });
            }
            push_fan(&mut result, &polygon);
        }
    }
    result.finalize_bounds();
    result
}

fn push_fan(mesh: &mut FillMesh, polygon: &[Point<Pixels>]) {
    if let Some((&first, rest)) = polygon.split_first() {
        for edge in rest.windows(2) {
            if cross(edge[0] - first, edge[1] - first).abs() > f32::EPSILON {
                mesh.push_triangle(first, edge[0], edge[1]);
            }
        }
    }
}

static UNIT_CIRCLE: std::sync::LazyLock<[(f32, f32); RIPPLE_SEGMENTS]> =
    std::sync::LazyLock::new(|| {
        std::array::from_fn(|index| {
            let angle = std::f32::consts::TAU * index as f32 / RIPPLE_SEGMENTS as f32;
            (angle.cos(), angle.sin())
        })
    });

pub(super) fn clipped_ripple(fill: &FillMesh, origin: Point<Pixels>, radius: Pixels) -> FillMesh {
    let raw_radius = f32::from(radius);
    let ox = f32::from(origin.x);
    let oy = f32::from(origin.y);
    if !raw_radius.is_finite() || raw_radius <= 0. || !ox.is_finite() || !oy.is_finite() {
        return FillMesh::empty(origin);
    }
    let mut circle: Option<[Point<Pixels>; RIPPLE_SEGMENTS]> = None;
    let mut ripple = FillMesh::empty(origin);
    let mut polygon = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let mut clipped = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let extent = point(radius, radius);
    let min = origin - extent;
    let max = origin + extent;
    let has_area = min.x < max.x && min.y < max.y;
    let apothem = raw_radius * (std::f32::consts::PI / RIPPLE_SEGMENTS as f32).cos();
    let inscribed_sq = apothem * apothem;
    let radius_sq = raw_radius * raw_radius;
    let dist_sq = |p: Point<Pixels>| {
        let dx = f32::from(p.x) - ox;
        let dy = f32::from(p.y) - oy;
        dx * dx + dy * dy
    };

    if fill.is_empty() {
        return FillMesh::empty(origin);
    }
    // Fast path: if the entire shape bounding box is outside the disc, return empty.
    let nearest_x = ox.clamp(
        f32::from(fill.bounds.left()),
        f32::from(fill.bounds.right()),
    );
    let nearest_y = oy.clamp(
        f32::from(fill.bounds.top()),
        f32::from(fill.bounds.bottom()),
    );
    if (ox - nearest_x) * (ox - nearest_x) + (oy - nearest_y) * (oy - nearest_y) >= radius_sq {
        return FillMesh::empty(origin);
    }
    // Fast path: if all four corners of the shape bounding box lie inside the disc's
    // inscribed polygon, the ripple completely covers the shape with zero clipping work.
    if dist_sq(fill.bounds.origin) <= inscribed_sq
        && dist_sq(fill.bounds.top_right()) <= inscribed_sq
        && dist_sq(fill.bounds.bottom_right()) <= inscribed_sq
        && dist_sq(fill.bounds.bottom_left()) <= inscribed_sq
    {
        return fill.clone();
    }

    // Intersect the disc with each disjoint fill triangle so paint and hit
    // testing use the same tessellated curve, including concave shapes.
    for &[a, b, c] in &fill.triangles {
        let vertices = [a, b, c];
        if has_area
            && (vertices.iter().all(|vertex| vertex.x < min.x)
                || vertices.iter().all(|vertex| vertex.x > max.x)
                || vertices.iter().all(|vertex| vertex.y < min.y)
                || vertices.iter().all(|vertex| vertex.y > max.y))
        {
            continue;
        }
        let winding = cross(b - a, c - a);
        if winding.abs() <= f32::EPSILON {
            continue;
        }
        if dist_sq(a) <= inscribed_sq && dist_sq(b) <= inscribed_sq && dist_sq(c) <= inscribed_sq {
            ripple.push_triangle(a, b, c);
            continue;
        }
        let t_min_x = f32::from(a.x.min(b.x).min(c.x));
        let t_max_x = f32::from(a.x.max(b.x).max(c.x));
        let t_min_y = f32::from(a.y.min(b.y).min(c.y));
        let t_max_y = f32::from(a.y.max(b.y).max(c.y));
        let cx = ox.clamp(t_min_x, t_max_x);
        let cy = oy.clamp(t_min_y, t_max_y);
        if (ox - cx) * (ox - cx) + (oy - cy) * (oy - cy) >= radius_sq {
            continue;
        }
        let circle = circle.get_or_insert_with(|| {
            UNIT_CIRCLE.map(|(cos, sin)| origin + point(radius * cos, radius * sin))
        });
        polygon.clear();
        polygon.extend([a, b, c]);
        for (&start, &end) in circle.iter().zip(circle.iter().cycle().skip(1)) {
            if polygon.is_empty() {
                break;
            }
            clip_half_plane(&mut polygon, &mut clipped, |p| {
                cross(end - start, p - start)
            });
        }
        if let Some((&first, rest)) = polygon.split_first() {
            for edge in rest.windows(2) {
                ripple.push_triangle(first, edge[0], edge[1]);
            }
        }
    }
    ripple.finalize_bounds();
    ripple
}
