use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    Along, App, Axis, BorderStyle, Bounds, ContentMask, ElementId, Hsla, IntoElement, Pixels,
    Point, RenderOnce, Size, Styled, Task, Window, canvas, point, px, quad, size,
    transparent_black,
};
pub use gpui_base::ScrollbarHandle;
use gpui_base::motion::Easing;
use gpui_material_foundation::{color, current_material_theme, is_bounds_visible};
use gpui_material_theme::{ColorRole, Direction};
use web_time::Instant;

/// Display scrollbar colors and dimensions. Lengths are in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayScrollbarStyle {
    pub thumb_color: Option<Hsla>,
    pub track_color: Option<Hsla>,
    pub thickness: Pixels,
    pub thumb_min_length: Pixels,
    /// Maximum as a fraction of the track length after main-axis insets.
    pub thumb_max_length_fraction: f32,
    /// Inset at both ends of the track.
    pub main_axis_track_inset: Pixels,
    /// Inset from the viewport edge underneath the track.
    pub cross_axis_track_inset: Pixels,
}

impl Default for DisplayScrollbarStyle {
    fn default() -> Self {
        Self {
            thumb_color: None,
            track_color: None,
            thickness: px(4.),
            thumb_min_length: px(24.),
            thumb_max_length_fraction: 0.9,
            main_axis_track_inset: px(2.),
            cross_axis_track_inset: px(0.),
        }
    }
}

/// Shows the indicator after a scroll offset change, then fades it out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayScrollbarFade {
    pub enabled: bool,
    pub delay: Duration,
    pub duration: Duration,
}

impl Default for DisplayScrollbarFade {
    fn default() -> Self {
        Self {
            enabled: true,
            delay: Duration::from_millis(400),
            duration: Duration::from_millis(250),
        }
    }
}

/// A read-only scroll position indicator. Mount it after the scroll viewport
/// as a child of the same relative parent and give it a stable ID.
#[derive(IntoElement)]
pub struct DisplayScrollbar {
    id: ElementId,
    axis: Axis,
    handle: Box<dyn ScrollbarHandle>,
    style: DisplayScrollbarStyle,
    offset: Point<Pixels>,
    fade: DisplayScrollbarFade,
}

impl DisplayScrollbar {
    /// Reads the scroll handle used by a viewport, such as `ScrollHandle`,
    /// `UniformListScrollHandle`, or `ListState`.
    pub fn new(
        id: impl Into<ElementId>,
        axis: Axis,
        handle: &(impl ScrollbarHandle + Clone),
    ) -> Self {
        Self {
            id: id.into(),
            axis,
            handle: Box::new(handle.clone()),
            style: DisplayScrollbarStyle::default(),
            offset: point(px(0.), px(0.)),
            fade: DisplayScrollbarFade::default(),
        }
    }

    pub fn style(mut self, style: DisplayScrollbarStyle) -> Self {
        self.style = style;
        self
    }

    /// Translates the painted indicator in physical logical pixels.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    pub fn fade(mut self, fade: DisplayScrollbarFade) -> Self {
        self.fade = fade;
        self
    }
}

impl RenderOnce for DisplayScrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let axis = self.axis;
        let axis_id = match axis {
            Axis::Horizontal => "display-scrollbar-horizontal",
            Axis::Vertical => "display-scrollbar-vertical",
        };
        let state = window
            .use_keyed_state((self.id, axis_id), cx, |_, _| {
                Rc::new(RefCell::new(FadeState::default()))
            })
            .read(cx)
            .clone();
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let style = self.style;
        let thumb_color = style
            .thumb_color
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::Outline)).opacity(0.7));
        let track_color = style.track_color.unwrap_or_else(transparent_black);
        let (handle, offset, fade) = (self.handle, self.offset, self.fade);

        canvas(
            move |bounds, window, cx| {
                // The earlier sibling has laid out the current viewport and content.
                let viewport = handle.viewport_bounds();
                let clip = bounds.intersect(&viewport);
                let geometry = geometry(
                    axis,
                    direction,
                    viewport,
                    handle.content_size(),
                    handle.offset().along(axis),
                    style,
                    offset,
                );
                let mut state = state.borrow_mut();
                let Some(geometry) = geometry.filter(|geometry| {
                    clip.intersects(&geometry.track) && is_bounds_visible(clip, window)
                }) else {
                    *state = FadeState::default();
                    return None;
                };
                let opacity = state.opacity(geometry.scroll_offset, fade, window, cx);
                (opacity > 0.).then_some((clip, geometry, opacity))
            },
            move |_, paint, window, _| {
                if let Some((clip, geometry, opacity)) = paint {
                    window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
                        for (bounds, tone) in
                            [(geometry.track, track_color), (geometry.thumb, thumb_color)]
                        {
                            if tone.a > 0. {
                                window.paint_quad(quad(
                                    bounds,
                                    geometry.radius,
                                    tone.opacity(opacity),
                                    px(0.),
                                    transparent_black(),
                                    BorderStyle::Solid,
                                ));
                            }
                        }
                    });
                }
            },
        )
        .absolute()
        .inset_0()
    }
}

#[derive(Debug, PartialEq)]
struct Geometry {
    track: Bounds<Pixels>,
    thumb: Bounds<Pixels>,
    radius: Pixels,
    scroll_offset: Pixels,
}

fn geometry(
    axis: Axis,
    direction: Direction,
    viewport: Bounds<Pixels>,
    content_size: Size<Pixels>,
    scroll_offset: Pixels,
    style: DisplayScrollbarStyle,
    offset: Point<Pixels>,
) -> Option<Geometry> {
    let viewport_length = viewport.size.along(axis);
    let content_length = content_size.along(axis);
    if !finite(viewport_length)
        || !finite(content_length)
        || viewport_length <= px(0.)
        || content_length <= viewport_length
    {
        return None;
    }
    let defaults = DisplayScrollbarStyle::default();
    let resolve = |value, fallback| {
        if finite(value) && value >= px(0.) {
            value
        } else {
            fallback
        }
    };
    let main_inset = resolve(style.main_axis_track_inset, defaults.main_axis_track_inset);
    let length = viewport_length - main_inset * 2.;
    let minimum = resolve(style.thumb_min_length, defaults.thumb_min_length);
    let maximum = length
        * if style.thumb_max_length_fraction.is_finite() {
            style.thumb_max_length_fraction.clamp(0., 1.)
        } else {
            defaults.thumb_max_length_fraction
        };
    if length <= px(0.) || maximum <= px(0.) || maximum < minimum {
        return None;
    }
    let cross_inset = resolve(
        style.cross_axis_track_inset,
        defaults.cross_axis_track_inset,
    );
    let cross_extent = match axis {
        Axis::Horizontal => viewport.size.height,
        Axis::Vertical => viewport.size.width,
    };
    let thickness = resolve(style.thickness, defaults.thickness).min(cross_extent - cross_inset);
    if !finite(cross_extent) || thickness <= px(0.) {
        return None;
    }
    let extent = content_length - viewport_length;
    // GPUI/Base represents displacement from the scroll origin as a negative offset.
    let scroll_offset = if finite(scroll_offset) {
        (-scroll_offset).clamp(px(0.), extent)
    } else {
        px(0.)
    };
    let progress = scroll_offset / extent;
    let thumb_length = (length * (viewport_length / content_length)).clamp(minimum, maximum);
    let rtl = direction == Direction::Rtl;
    let (origin, track_size, thumb_origin, thumb_size) = match axis {
        Axis::Horizontal => {
            let origin = point(
                viewport.left() + main_inset,
                viewport.bottom() - cross_inset - thickness,
            );
            let travel = (length - thumb_length) * if rtl { 1. - progress } else { progress };
            (
                origin,
                size(length, thickness),
                origin + point(travel, px(0.)),
                size(thumb_length, thickness),
            )
        }
        Axis::Vertical => {
            let left = if rtl {
                viewport.left() + cross_inset
            } else {
                viewport.right() - cross_inset - thickness
            };
            let origin = point(left, viewport.top() + main_inset);
            let travel = (length - thumb_length) * progress;
            (
                origin,
                size(thickness, length),
                origin + point(px(0.), travel),
                size(thickness, thumb_length),
            )
        }
    };
    let shift = |value| if finite(value) { value } else { px(0.) };
    let offset = point(shift(offset.x), shift(offset.y));
    Some(Geometry {
        track: Bounds::new(origin + offset, track_size),
        thumb: Bounds::new(thumb_origin + offset, thumb_size),
        radius: thickness / 2.,
        scroll_offset,
    })
}

fn finite(value: Pixels) -> bool {
    f32::from(value).is_finite()
}

#[derive(Default)]
struct FadeState {
    last_offset: Option<Pixels>,
    scrolled_at: Option<Instant>,
    wake: Option<(Instant, Duration, Task<()>)>,
}

impl FadeState {
    fn opacity(
        &mut self,
        offset: Pixels,
        fade: DisplayScrollbarFade,
        window: &Window,
        cx: &App,
    ) -> f32 {
        let previous = self.last_offset.replace(offset);
        if !fade.enabled {
            self.scrolled_at = None;
            self.wake = None;
            return 1.;
        }
        let now = cx.background_executor().now();
        if previous.is_some_and(|previous| previous != offset) {
            self.scrolled_at = Some(now);
        }
        let Some(scrolled_at) = self.scrolled_at else {
            return 0.;
        };
        let elapsed = now.saturating_duration_since(scrolled_at);
        if elapsed < fade.delay {
            let wait = fade.delay - elapsed;
            if now.checked_add(wait).is_none() {
                self.wake = None;
                return 1.;
            }
            if self.wake.as_ref().is_none_or(|(last_scroll, delay, _)| {
                *last_scroll != scrolled_at || *delay != fade.delay
            }) {
                let current_view = window.current_view();
                let deadline = cx.background_executor().timer(wait);
                let wake = window.spawn(cx, async move |cx| {
                    deadline.await;
                    cx.update(|_, cx| cx.notify(current_view)).ok();
                });
                self.wake = Some((scrolled_at, fade.delay, wake));
            }
            return 1.;
        }

        self.wake = None;
        let fading_for = elapsed - fade.delay;
        if cx.reduce_motion() || fading_for >= fade.duration {
            self.scrolled_at = None;
            return 0.;
        }
        window.request_animation_frame();
        let easing = Easing::CubicBezier {
            x1: 0.4,
            y1: 0.,
            x2: 0.2,
            y2: 1.,
        };
        1. - easing.sample(fading_for.as_secs_f32() / fade.duration.as_secs_f32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_rejects_empty_fitting_and_too_short_tracks() {
        let origin = point(px(0.), px(0.));
        let viewport = Bounds::new(origin, size(px(100.), px(100.)));
        let sample = |viewport, width, style| {
            geometry(
                Axis::Horizontal,
                Direction::Ltr,
                viewport,
                size(px(width), px(100.)),
                px(0.),
                style,
                origin,
            )
        };
        let defaults = DisplayScrollbarStyle::default();
        for width in [0., 100., f32::NAN, f32::INFINITY] {
            assert!(sample(viewport, width, defaults).is_none());
        }
        assert!(sample(Bounds::new(origin, size(px(0.), px(100.))), 120., defaults).is_none());
        assert!(
            sample(
                viewport,
                120.,
                DisplayScrollbarStyle {
                    thumb_min_length: px(90.),
                    ..defaults
                }
            )
            .is_none()
        );
        let malformed = DisplayScrollbarStyle {
            thickness: px(f32::NAN),
            thumb_min_length: px(-1.),
            thumb_max_length_fraction: f32::INFINITY,
            main_axis_track_inset: px(-2.),
            cross_axis_track_inset: px(f32::NEG_INFINITY),
            ..defaults
        };
        assert_eq!(
            sample(viewport, 120., malformed),
            sample(viewport, 120., defaults)
        );
    }
}
