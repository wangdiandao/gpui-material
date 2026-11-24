use gpui::{
    Along, AnyElement, App, AvailableSpace, Bounds, IntoElement, ParentElement, Pixels, Styled,
    Window, canvas, div, point, prelude::FluentBuilder, px,
};
use gpui_base::AxisExt;

use super::geometry::{Geometry, track_segments};
use super::layout::TrackLayout;
use super::palette::SliderPalette;
use super::{DrawMarker, SliderMarker};

#[allow(clippy::too_many_arguments)]
fn render_track_icon(
    icon: AnyElement,
    side: f32,
    geometry: Geometry,
    first: (f32, Pixels),
    last: (f32, Pixels),
    layout: TrackLayout,
    palette: SliderPalette,
    disabled: bool,
) -> impl IntoElement {
    canvas(
        move |bounds, window, cx| {
            if !geometry.has_positive_extent(bounds) {
                return None;
            }
            let side = px(side);
            let length = bounds.size.along(geometry.axis);
            let (offset, active) =
                track_segments(length, first, last)
                    .into_iter()
                    .find_map(|segment| {
                        segment.icon_position(side, layout.track_radius, layout.inner_radius)
                    })?;
            let foreground = if disabled {
                palette.disabled_icon
            } else if active {
                palette.active_icon
            } else {
                palette.inactive_icon
            };
            let origin = geometry.point_at(bounds, offset) - point(side / 2., side / 2.);
            let mut icon = div()
                .size(side)
                .flex()
                .items_center()
                .justify_center()
                .text_size(side)
                .line_height(side)
                .text_color(foreground)
                .child(icon)
                .into_any_element();
            let space = gpui::size(
                AvailableSpace::Definite(side),
                AvailableSpace::Definite(side),
            );
            icon.prepaint_as_root(origin, space, window, cx);
            Some(icon)
        },
        |_, icon, window, cx| {
            if let Some(mut icon) = icon {
                icon.paint(window, cx);
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

#[allow(clippy::too_many_arguments)]
fn paint_track_canvas(
    bounds: Bounds<Pixels>,
    geometry: Geometry,
    first: (f32, Pixels),
    last: (f32, Pixels),
    layout: TrackLayout,
    palette: SliderPalette,
    tick_count: u16,
    show_stops: bool,
    draw_tick: Option<&DrawMarker>,
    draw_stop: Option<&DrawMarker>,
    window: &mut Window,
    cx: &mut App,
) {
    if !geometry.has_positive_extent(bounds) {
        return;
    }
    let horizontal = geometry.axis.is_horizontal();
    let length = bounds.size.along(geometry.axis);
    let segments = track_segments(length, first, last);
    let stop_offset = layout.track_radius.max(px(2.));
    let marker = |offset, color| SliderMarker {
        center: geometry.point_at(bounds, offset),
        diameter: px(4.),
        color,
    };
    for segment in segments {
        if segment.end <= segment.start {
            continue;
        }
        let a = geometry.point_at(bounds, segment.start);
        let b = geometry.point_at(bounds, segment.end);
        let half = layout.thickness / 2.;
        let rect = if horizontal {
            Bounds::from_corners(
                point(a.x.min(b.x), a.y - half),
                point(a.x.max(b.x), b.y + half),
            )
        } else {
            Bounds::from_corners(
                point(a.x - half, a.y.min(b.y)),
                point(a.x + half, a.y.max(b.y)),
            )
        };
        let tint = if segment.active {
            palette.active
        } else {
            palette.inactive
        };
        let (start_corner, end_corner) = segment.corners(layout.track_radius, layout.inner_radius);
        window.paint_quad(gpui::quad(
            rect,
            geometry.track_corners(start_corner, end_corner),
            tint,
            px(0.),
            gpui::transparent_black(),
            gpui::BorderStyle::Solid,
        ));
        // The stop indicator marks the outer edge of each inactive segment.
        if show_stops && !segment.active {
            if segment.outer_start && segment.start + stop_offset + px(2.) <= segment.end {
                draw_marker(
                    draw_stop,
                    marker(segment.start + stop_offset, palette.active),
                    window,
                    cx,
                );
            }
            if segment.outer_end && segment.end - stop_offset - px(2.) >= segment.start {
                draw_marker(
                    draw_stop,
                    marker(segment.end - stop_offset, palette.active),
                    window,
                    cx,
                );
            }
        }
    }
    if tick_count >= 2 {
        // Skip ticks when they would visually merge instead of iterating every
        // possible value on a fine-grained scale.
        let spacing = f32::from(length) / f32::from(tick_count);
        if spacing >= 6. {
            for index in 1..tick_count {
                let offset = length * (f32::from(index) / f32::from(tick_count));
                if let Some(segment) = segments.iter().find(|segment| {
                    segment.start + px(2.) <= offset && offset <= segment.end - px(2.)
                }) {
                    let tint = if segment.active {
                        palette.active_tick
                    } else {
                        palette.inactive_tick
                    };
                    let on_endpoint = show_stops
                        && !segment.active
                        && (segment.outer_start
                            && (offset - segment.start - stop_offset).abs() < px(4.)
                            || segment.outer_end
                                && (segment.end - stop_offset - offset).abs() < px(4.));
                    if !on_endpoint {
                        draw_marker(draw_tick, marker(offset, tint), window, cx);
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_track_artwork(
    custom: Option<AnyElement>,
    icon: Option<AnyElement>,
    geometry: Geometry,
    first: (f32, Pixels),
    last: (f32, Pixels),
    layout: TrackLayout,
    palette: SliderPalette,
    disabled: bool,
    tick_count: u16,
    show_stops: bool,
    draw_tick: Option<DrawMarker>,
    draw_stop: Option<DrawMarker>,
) -> AnyElement {
    let horizontal = geometry.axis.is_horizontal();
    if let Some(custom) = custom {
        return div()
            .absolute()
            .when(horizontal, |track| {
                track
                    .w_full()
                    .h(layout.thickness)
                    .top((layout.target - layout.thickness) / 2.)
            })
            .when(!horizontal, |track| {
                track
                    .h_full()
                    .w(layout.thickness)
                    .left((layout.target - layout.thickness) / 2.)
            })
            .child(custom)
            .into_any_element();
    }
    let track_icon = icon.zip(layout.icon_size).map(|(icon, side)| {
        render_track_icon(icon, side, geometry, first, last, layout, palette, disabled)
    });
    div()
        .absolute()
        .inset_0()
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, cx| {
                    paint_track_canvas(
                        bounds,
                        geometry,
                        first,
                        last,
                        layout,
                        palette,
                        tick_count,
                        show_stops,
                        draw_tick.as_ref(),
                        draw_stop.as_ref(),
                        window,
                        cx,
                    );
                },
            )
            .absolute()
            .inset_0()
            .size_full(),
        )
        .children(track_icon)
        .into_any_element()
}

fn draw_marker(draw: Option<&DrawMarker>, marker: SliderMarker, window: &mut Window, cx: &mut App) {
    if let Some(draw) = draw {
        draw(marker, window, cx);
    } else {
        let radius = marker.diameter / 2.;
        let offset = point(radius, radius);
        window.paint_quad(gpui::quad(
            Bounds::from_corners(marker.center - offset, marker.center + offset),
            radius,
            marker.color,
            px(0.),
            gpui::transparent_black(),
            gpui::BorderStyle::Solid,
        ));
    }
}
