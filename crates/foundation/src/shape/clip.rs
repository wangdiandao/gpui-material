use gpui::{Bounds, Path, Pixels, Point, point, px, size};

const RIPPLE_SEGMENTS: usize = 64;

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

pub(super) fn contains_fill(path: &Path<Pixels>, position: Point<Pixels>) -> bool {
    path.bounds.contains(&position)
        && path.vertices.as_chunks::<3>().0.iter().any(|triangle| {
            let a = triangle[0].xy_position;
            let b = triangle[1].xy_position;
            let c = triangle[2].xy_position;
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

pub(super) fn clip_rect_fill(fill: &Path<Pixels>, rect: Bounds<Pixels>) -> Path<Pixels> {
    let mut result = Path::new(rect.origin);
    if !rect_is_valid(rect) || fill.vertices.is_empty() || !bounds_overlap(fill.bounds, rect) {
        return result;
    }
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
        if triangle_disjoint_from_rect([a, b, c], rect) || cross(b - a, c - a).abs() <= f32::EPSILON
        {
            continue;
        }
        polygon.clear();
        polygon.extend([a, b, c]);
        clip_polygon_to_rect(&mut polygon, &mut scratch, rect);
        push_fan(&mut result, &polygon, stencil);
    }
    result
}

pub(super) fn clip_convex_polygon_fill(
    fill: &Path<Pixels>,
    vertices: &[Point<Pixels>],
) -> Path<Pixels> {
    let origin = vertices.first().copied().unwrap_or(point(px(0.), px(0.)));
    let mut result = Path::new(origin);
    if vertices.len() < 3
        || fill.vertices.is_empty()
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
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(vertices.len() + 3);
    let mut scratch = Vec::with_capacity(vertices.len() + 3);
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
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
        push_fan(&mut result, &polygon, stencil);
    }
    result
}

pub(super) fn clip_path_fill(clip: &Path<Pixels>, source: &Path<Pixels>) -> Path<Pixels> {
    let mut result = Path::new(source.bounds.origin);
    if clip.vertices.is_empty()
        || source.vertices.is_empty()
        || !bounds_overlap(clip.bounds, source.bounds)
    {
        return result;
    }
    let clip_triangles: Vec<_> = clip
        .vertices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|triangle| {
            let [a, b, c] = triangle.each_ref().map(|vertex| vertex.xy_position);
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

    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for triangle in source.vertices.as_chunks::<3>().0 {
        let [sa, sb, sc] = triangle.each_ref().map(|vertex| vertex.xy_position);
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
            push_fan(&mut result, &polygon, stencil);
        }
    }
    result
}

fn push_fan(path: &mut Path<Pixels>, polygon: &[Point<Pixels>], stencil: Point<f32>) {
    if let Some((&first, rest)) = polygon.split_first() {
        for edge in rest.windows(2) {
            if cross(edge[0] - first, edge[1] - first).abs() > f32::EPSILON {
                path.push_triangle((first, edge[0], edge[1]), (stencil, stencil, stencil));
            }
        }
    }
}

pub(crate) fn clipped_ripple(
    fill: &Path<Pixels>,
    origin: Point<Pixels>,
    radius: Pixels,
) -> Path<Pixels> {
    let raw_radius = f32::from(radius);
    if !raw_radius.is_finite()
        || raw_radius <= 0.
        || !f32::from(origin.x).is_finite()
        || !f32::from(origin.y).is_finite()
    {
        return Path::new(origin);
    }
    let circle: [_; RIPPLE_SEGMENTS] = std::array::from_fn(|index| {
        let angle = std::f32::consts::TAU * index as f32 / RIPPLE_SEGMENTS as f32;
        origin + point(radius * angle.cos(), radius * angle.sin())
    });
    let mut ripple = Path::new(origin);
    let stencil = point(0., 1.);
    let mut polygon = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let mut clipped = Vec::with_capacity(RIPPLE_SEGMENTS + 3);
    let extent = point(radius, radius);
    let min = origin - extent;
    let max = origin + extent;
    let has_area = min.x < max.x && min.y < max.y;

    // Intersect the disc with each disjoint fill triangle so paint and hit
    // testing use the same tessellated curve, including concave shapes.
    for triangle in fill.vertices.as_chunks::<3>().0 {
        let vertices = triangle.each_ref().map(|vertex| vertex.xy_position);
        if has_area
            && (vertices.iter().all(|vertex| vertex.x < min.x)
                || vertices.iter().all(|vertex| vertex.x > max.x)
                || vertices.iter().all(|vertex| vertex.y < min.y)
                || vertices.iter().all(|vertex| vertex.y > max.y))
        {
            continue;
        }
        let [a, b, c] = vertices;
        let winding = cross(b - a, c - a);
        if winding.abs() <= f32::EPSILON {
            continue;
        }
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
                ripple.push_triangle((first, edge[0], edge[1]), (stencil, stencil, stencil));
            }
        }
    }
    ripple
}
