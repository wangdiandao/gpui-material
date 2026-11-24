use std::f32::consts::TAU;

use gpui::{
    App, Bounds, Corners, ElementId, Entity, Hsla, InteractiveElement, IntoElement, ParentElement,
    Path, Pixels, RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement, Styled, Window,
    canvas, div, fill, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    LoopClock, Shape, ShapeFit, ShapePolygon, color, current_material_theme, is_bounds_visible,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, TypeRole, TypographyEmphasis};
use web_time::Instant;

const PULSE_PERIOD_SECS: f32 = 1.4;
const DEFAULT_TEXT_LINE_HEIGHT: Pixels = px(16.);

/// Optional base and highlight pigments for [`Skeleton`].
///
/// Defaults to `SurfaceContainerHigh` and `SurfaceContainerHighest`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkeletonColors {
    /// Resting fill color of the skeleton placeholder.
    pub base: Option<Hsla>,
    /// Peak pulse highlight color interpolated during animation.
    pub highlight: Option<Hsla>,
}

enum SkeletonGeometry {
    Rect {
        width: Option<Pixels>,
        height: Pixels,
        corner: CornerShape,
    },
    Shape {
        width: Pixels,
        height: Pixels,
        polygon: ShapePolygon,
    },
}

/// An animated Material 3 loading placeholder built on [`LoopClock`] and
/// [`is_bounds_visible`].
///
/// Offscreen or clipped skeletons schedule zero animation frames, and
/// `reduced_motion` or `.animated(false)` renders the static base color with
/// zero animation frames.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    geometry: SkeletonGeometry,
    text_role: Option<TypeRole>,
    colors: SkeletonColors,
    animated: bool,
    accessible_label: Option<SharedString>,
}

impl Skeleton {
    /// Creates a rectangular placeholder with `CornerRole::Small` corners.
    pub fn rect(id: impl Into<ElementId>, width: Pixels, height: Pixels) -> Self {
        Self {
            id: id.into(),
            geometry: SkeletonGeometry::Rect {
                width: Some(sanitize_extent(width)),
                height: sanitize_extent(height),
                corner: CornerRole::Small.shape(),
            },
            text_role: None,
            colors: SkeletonColors::default(),
            animated: true,
            accessible_label: None,
        }
    }

    /// Creates a full-width single-line text placeholder (default 16 dp height,
    /// `CornerRole::Small`).
    pub fn text_line(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            geometry: SkeletonGeometry::Rect {
                width: None,
                height: DEFAULT_TEXT_LINE_HEIGHT,
                corner: CornerRole::Small.shape(),
            },
            text_role: None,
            colors: SkeletonColors::default(),
            animated: true,
            accessible_label: None,
        }
    }

    /// Creates a circular placeholder (`CornerRole::Full`).
    pub fn circle(id: impl Into<ElementId>, diameter: Pixels) -> Self {
        let side = sanitize_extent(diameter);
        Self {
            id: id.into(),
            geometry: SkeletonGeometry::Rect {
                width: Some(side),
                height: side,
                corner: CornerRole::Full.shape(),
            },
            text_role: None,
            colors: SkeletonColors::default(),
            animated: true,
            accessible_label: None,
        }
    }

    /// Creates a placeholder using any Foundation [`ShapePolygon`] (including
    /// [`gpui_material_theme::MaterialShape`] presets such as `Cookie7Sided`).
    pub fn shape(
        id: impl Into<ElementId>,
        width: Pixels,
        height: Pixels,
        shape: impl Into<ShapePolygon>,
    ) -> Self {
        Self {
            id: id.into(),
            geometry: SkeletonGeometry::Shape {
                width: sanitize_extent(width),
                height: sanitize_extent(height),
                polygon: shape.into(),
            },
            text_role: None,
            colors: SkeletonColors::default(),
            animated: true,
            accessible_label: None,
        }
    }

    /// Expands a rectangular or text-line placeholder to fill its parent's width.
    pub fn full_width(mut self) -> Self {
        if let SkeletonGeometry::Rect { width, .. } = &mut self.geometry {
            *width = None;
        }
        self
    }

    /// Sets an explicit width in logical pixels.
    pub fn width(mut self, width: Pixels) -> Self {
        let width = sanitize_extent(width);
        match &mut self.geometry {
            SkeletonGeometry::Rect { width: slot, .. } => *slot = Some(width),
            SkeletonGeometry::Shape { width: slot, .. } => *slot = width,
        }
        self
    }

    /// Sets an explicit height in logical pixels, overriding any `.role(...)`.
    pub fn height(mut self, height: Pixels) -> Self {
        let height = sanitize_extent(height);
        self.text_role = None;
        match &mut self.geometry {
            SkeletonGeometry::Rect { height: slot, .. } => *slot = height,
            SkeletonGeometry::Shape { height: slot, .. } => *slot = height,
        }
        self
    }

    /// Sizes a text-line or rectangular placeholder's height to match a Material
    /// typography role's scaled line height.
    pub fn role(mut self, role: TypeRole) -> Self {
        self.text_role = Some(role);
        self
    }

    /// Sets custom logical corners on rectangular, text-line, or circular placeholders.
    pub fn corner(mut self, shape: CornerShape) -> Self {
        if let SkeletonGeometry::Rect { corner, .. } = &mut self.geometry {
            *corner = shape;
        }
        self
    }

    /// Sets a Material [`CornerRole`] on rectangular, text-line, or circular placeholders.
    pub fn corner_role(self, role: CornerRole) -> Self {
        self.corner(role.shape())
    }

    /// Overrides the resting base and pulse highlight colors.
    pub fn colors(mut self, colors: SkeletonColors) -> Self {
        self.colors = colors;
        self
    }

    /// Enables or disables the pulse animation. Defaults to `true`.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Exposes an optional `Role::Status` label for assistive technology without
    /// adding a keyboard Tab stop. Placeholders are decorative by default.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let base = self
            .colors
            .base
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerHigh)));
        let highlight = self
            .colors
            .highlight
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerHighest)));
        let role_height = self.text_role.map(|role| {
            let scaled = theme
                .typography
                .scale_style(role.style(TypographyEmphasis::Baseline));
            px(scaled.line_height)
        });

        let reduced = cx.reduce_motion();
        let running = self.animated && !reduced && window.is_window_active() && window.is_visible();
        let motion = self.animated.then(|| {
            window.use_keyed_state((self.id.clone(), "skeleton-motion"), cx, |_, _| {
                SkeletonMotion::default()
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
        let fill_color = if running {
            mix_color(base, highlight, pulse_fraction(phase))
        } else {
            base
        };

        let (width, height, paint_kind) = match self.geometry {
            SkeletonGeometry::Rect {
                width,
                height,
                corner,
            } => (
                width,
                role_height.unwrap_or(height),
                PaintKind::Rect(corner),
            ),
            SkeletonGeometry::Shape {
                width,
                height,
                polygon,
            } => {
                let cache = window.use_keyed_state(
                    (self.id.clone(), "skeleton-polygon-cache"),
                    cx,
                    |_, _| PolygonPathCache::default(),
                );
                (Some(width), height, PaintKind::Polygon(polygon, cache))
            }
        };

        let painter = canvas(
            |_, _, _| (),
            move |bounds, (), window, cx| {
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
                match &paint_kind {
                    PaintKind::Rect(corner) => {
                        let [top_left, top_right, bottom_right, bottom_left] = corner
                            .radii(
                                f32::from(bounds.size.width),
                                f32::from(bounds.size.height),
                                direction,
                            )
                            .map(px);
                        window.paint_quad(fill(bounds, fill_color).corner_radii(Corners {
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                        }));
                    }
                    PaintKind::Polygon(polygon, cache) => {
                        let path = cache.update(cx, |state, _| state.get_or_build(bounds, polygon));
                        if let Some(path) = path {
                            window.paint_path(path, fill_color);
                        }
                    }
                }
                if running {
                    window.request_animation_frame();
                }
            },
        )
        .size_full();

        div()
            .id(self.id)
            .test_support()
            .h(height)
            .when_some(width, |el, w| el.w(w).flex_none())
            .when(width.is_none(), Styled::w_full)
            .when_some(self.accessible_label, |el, label| {
                el.role(Role::Status).aria_label(label)
            })
            .child(painter)
    }
}

enum PaintKind {
    Rect(CornerShape),
    Polygon(ShapePolygon, Entity<PolygonPathCache>),
}

#[derive(Default)]
struct PolygonPathCache {
    key: Option<(Bounds<Pixels>, ShapePolygon)>,
    path: Option<Path>,
}

impl PolygonPathCache {
    fn get_or_build(&mut self, bounds: Bounds<Pixels>, polygon: &ShapePolygon) -> Option<Path> {
        let matches = self
            .key
            .as_ref()
            .is_some_and(|(cached_bounds, cached_polygon)| {
                *cached_bounds == bounds && cached_polygon == polygon
            });
        if !matches {
            self.key = Some((bounds, polygon.clone()));
            self.path = Shape::polygon(bounds, polygon, ShapeFit::Contain)
                .map(|shape| shape.fill_path().clone());
        }
        self.path.clone()
    }
}

struct SkeletonMotion {
    clock: LoopClock,
    visible: bool,
}

impl Default for SkeletonMotion {
    fn default() -> Self {
        Self {
            clock: LoopClock::default(),
            visible: true,
        }
    }
}

impl SkeletonMotion {
    fn set_visible(&mut self, visible: bool, now: Instant) {
        if self.visible != visible {
            self.visible = visible;
            if !visible {
                self.clock.pause(now);
            }
        }
    }

    fn phase(&mut self, now: Instant, running: bool, reduced: bool) -> f32 {
        self.clock.advance(now, running, reduced).as_secs_f32() / PULSE_PERIOD_SECS
    }
}

fn pulse_fraction(phase: f32) -> f32 {
    if !phase.is_finite() {
        return 0.;
    }
    (0.5 - 0.5 * (phase.fract() * TAU).cos()).clamp(0., 1.)
}

fn sanitize_extent(value: Pixels) -> Pixels {
    let raw = f32::from(value);
    if raw.is_finite() && raw >= 0. {
        value
    } else {
        px(0.)
    }
}

fn mix_color(from: Hsla, to: Hsla, fraction: f32) -> Hsla {
    let fraction = fraction.clamp(0., 1.);
    if fraction == 0. {
        return from;
    }
    if fraction == 1. {
        return to;
    }
    let from = Rgba::from(from);
    let to = Rgba::from(to);
    let lerp = |a, b| a + (b - a) * fraction;
    Rgba {
        r: lerp(from.r, to.r),
        g: lerp(from.g, to.g),
        b: lerp(from.b, to.b),
        a: lerp(from.a, to.a),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use gpui_material_theme::MaterialShape;

    #[test]
    fn pulse_fraction_starts_and_ends_at_base_and_peaks_at_half_period() {
        assert!((pulse_fraction(0.) - 0.).abs() < 1e-5);
        assert!((pulse_fraction(0.5) - 1.).abs() < 1e-5);
        assert!((pulse_fraction(1.) - 0.).abs() < 1e-5);
        assert!((pulse_fraction(1.5) - 1.).abs() < 1e-5);
        assert_eq!(pulse_fraction(f32::NAN), 0.);
    }

    #[test]
    fn skeleton_motion_pauses_offscreen_and_resets_under_reduced_motion() {
        let mut motion = SkeletonMotion::default();
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);

        assert_eq!(motion.phase(start, true, false), 0.);
        let mid = motion.phase(at(350), true, false);
        assert!(mid > 0.);
        motion.set_visible(false, at(350));
        assert!(!motion.visible);
        assert_eq!(motion.phase(at(700), false, false), mid);
        motion.set_visible(true, at(700));
        assert_eq!(motion.phase(at(700), true, false), mid);
        assert!(motion.phase(at(1050), true, false) > mid);
        assert_eq!(motion.phase(at(1100), false, true), 0.);
    }

    #[test]
    fn skeleton_constructors_sanitize_dimensions_and_accept_material_shapes() {
        assert_eq!(sanitize_extent(px(-10.)), px(0.));
        assert_eq!(sanitize_extent(px(f32::NAN)), px(0.));
        assert_eq!(sanitize_extent(px(24.)), px(24.));
        let cookie = Skeleton::shape("cookie", px(48.), px(48.), MaterialShape::Cookie7Sided);
        assert!(matches!(
            cookie.geometry,
            SkeletonGeometry::Shape {
                width,
                height,
                ..
            } if width == px(48.) && height == px(48.)
        ));
    }

    #[test]
    fn polygon_path_cache_reuses_and_invalidates_on_bounds_or_polygon_change() {
        let mut cache = PolygonPathCache::default();
        let bounds_a = Bounds::new(gpui::point(px(0.), px(0.)), gpui::size(px(48.), px(48.)));
        let bounds_b = Bounds::new(gpui::point(px(0.), px(0.)), gpui::size(px(64.), px(64.)));
        let cookie = ShapePolygon::from(MaterialShape::Cookie7Sided);
        let other = ShapePolygon::from(MaterialShape::Cookie9Sided);

        assert!(cache.get_or_build(bounds_a, &cookie).is_some());
        assert_eq!(cache.key, Some((bounds_a, cookie.clone())));
        // Same key reuses cached path.
        assert!(cache.get_or_build(bounds_a, &cookie).is_some());
        assert_eq!(cache.key, Some((bounds_a, cookie.clone())));
        // Changed bounds updates key.
        assert!(cache.get_or_build(bounds_b, &cookie).is_some());
        assert_eq!(cache.key, Some((bounds_b, cookie.clone())));
        // Changed polygon updates key.
        assert!(cache.get_or_build(bounds_b, &other).is_some());
        assert_eq!(cache.key, Some((bounds_b, other)));
    }
}
