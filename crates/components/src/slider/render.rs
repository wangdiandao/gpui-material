use std::{cell::RefCell, rc::Rc};

use gpui::{
    Along, AnyElement, App, AppContext, AvailableSpace, Bounds, DragMoveEvent, ElementId, Entity,
    FocusHandle, HitboxBehavior, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, Point, Render, RenderOnce, Role, SharedString, Size, Stateful,
    StatefulInteractiveElement, Styled, TouchDragEvent, TouchPhase, Window, canvas, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{
    AxisExt, TestSupportExt,
    slider::{Slider as BaseSlider, SliderIndicator, SliderState, SliderValue},
};
use gpui_material_foundation::{
    DragCapture, InteractionSource, InteractionState, InteractionTracker, RangeThumb, animate,
    color, current_material_theme, range_semantics, shape_text,
};
use gpui_material_theme::{
    ColorRole, MotionKind, MotionSpec, MotionSpeed, TypeRole, TypographyEmphasis,
};

use crate::ControlSize;

use super::geometry::{Geometry, choose_thumb, dimension, track_segments, value_label_origin};
use super::{
    DrawMarker, Slider, SliderColors, SliderKind, SliderMarker, SliderThumbStyle, SliderTrackStyle,
    thumb_id, thumb_index,
};

#[derive(Default)]
struct Gesture {
    thumb: Option<RangeThumb>,
    origin: Option<Point<Pixels>>,
    capture: Option<DragCapture>,
    dragged: bool,
    disabled: bool,
    feedback: [ThumbFeedback; 2],
}

#[derive(Default)]
struct ThumbFeedback {
    focus: Option<FocusHandle>,
    tracker: Option<InteractionTracker>,
    hovered: bool,
}

impl Gesture {
    fn record(&self, window: &Window, cx: &mut App) {
        for (which, feedback) in [RangeThumb::Start, RangeThumb::End]
            .into_iter()
            .zip(&self.feedback)
        {
            let Some(tracker) = &feedback.tracker else {
                continue;
            };
            let mut snapshot = InteractionState::default();
            snapshot.disabled = self.disabled;
            if !self.disabled && window.is_window_active() {
                snapshot.hovered = feedback.hovered;
                snapshot.focus_visible = window.last_input_was_keyboard()
                    && feedback
                        .focus
                        .as_ref()
                        .is_some_and(|focus| focus.is_focused(window));
                snapshot.pressed = self.thumb == Some(which);
                snapshot.dragged = snapshot.pressed && self.dragged;
            }
            tracker.record(snapshot, cx);
        }
    }
}

struct SliderDrag(Entity<SliderState>);

impl Render for SliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone, Copy)]
struct SliderPalette {
    active: Hsla,
    inactive: Hsla,
    active_tick: Hsla,
    inactive_tick: Hsla,
    thumb: Hsla,
    ring: Hsla,
    active_icon: Hsla,
    inactive_icon: Hsla,
    disabled_icon: Hsla,
    inverse_surface: Hsla,
    inverse_on_surface: Hsla,
}

fn resolve_palette(
    colors: SliderColors,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> SliderPalette {
    let scheme = current_material_theme(window, cx).colors.clone();
    let resolve = |role| color(scheme.color(role));
    let muted = resolve(ColorRole::OnSurface);
    let (active, inactive, active_tick, inactive_tick, thumb) = if disabled {
        (
            colors
                .disabled_active_track
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors
                .disabled_inactive_track
                .unwrap_or_else(|| muted.opacity(0.12)),
            colors
                .disabled_active_tick
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors
                .disabled_inactive_tick
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors.disabled_thumb.unwrap_or_else(|| muted.opacity(0.38)),
        )
    } else {
        (
            colors
                .active_track
                .unwrap_or_else(|| resolve(ColorRole::Primary)),
            colors
                .inactive_track
                .unwrap_or_else(|| resolve(ColorRole::SecondaryContainer)),
            colors
                .active_tick
                .unwrap_or_else(|| resolve(ColorRole::OnPrimary)),
            colors
                .inactive_tick
                .unwrap_or_else(|| resolve(ColorRole::OnSecondaryContainer)),
            colors.thumb.unwrap_or_else(|| resolve(ColorRole::Primary)),
        )
    };
    SliderPalette {
        active,
        inactive,
        active_tick,
        inactive_tick,
        thumb,
        ring: resolve(ColorRole::Secondary),
        active_icon: resolve(ColorRole::OnPrimary),
        inactive_icon: resolve(ColorRole::OnSecondaryContainer),
        disabled_icon: muted.opacity(0.38),
        inverse_surface: resolve(ColorRole::InverseSurface),
        inverse_on_surface: resolve(ColorRole::InverseOnSurface),
    }
}

#[derive(Clone, Copy)]
struct TrackLayout {
    thickness: Pixels,
    track_radius: Pixels,
    inner_radius: Pixels,
    thumb_gap: Pixels,
    icon_size: Option<f32>,
    target: Pixels,
    padding: Pixels,
    handle_sizes: [Size<Pixels>; 2],
}

fn thumb_cross_extent(style: SliderThumbStyle, default_extent: f32) -> Pixels {
    dimension(style.size.map(|size| size.height), default_extent, false)
}

fn thumb_rest_width(style: SliderThumbStyle) -> Pixels {
    dimension(style.size.map(|size| size.width), 4., false)
}

fn thumb_pressed_width(style: SliderThumbStyle) -> Pixels {
    dimension(style.pressed_width, 2., false)
}

#[allow(clippy::too_many_arguments)]
fn resolve_track_layout(
    id: &ElementId,
    size: ControlSize,
    track_style: SliderTrackStyle,
    thumb_styles: [SliderThumbStyle; 2],
    range: bool,
    dragging: Option<RangeThumb>,
    resize: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> TrackLayout {
    // Material slider specs, expressive track and handle sizes.
    let (thickness, thumb_extent, track_radius, icon_size): (f32, f32, f32, Option<f32>) =
        match size {
            ControlSize::ExtraSmall => (16., 44., 8., None),
            ControlSize::Small => (24., 44., 8., None),
            ControlSize::Medium => (40., 52., 12., Some(24.)),
            ControlSize::Large => (56., 68., 16., Some(24.)),
            ControlSize::ExtraLarge => (96., 108., 28., Some(32.)),
        };
    let thickness = dimension(track_style.thickness, thickness, false);
    let track_radius = dimension(track_style.outer_radius, track_radius, true).min(thickness / 2.);
    let inner_radius = dimension(track_style.inner_radius, 2., true).min(thickness / 2.);
    let thumb_gap = dimension(track_style.thumb_gap, 6., true);
    let start_cross = thumb_cross_extent(thumb_styles[0], thumb_extent);
    let end_cross = thumb_cross_extent(thumb_styles[1], thumb_extent);
    let mut target = thickness.max(end_cross).max(px(48.));
    if range {
        target = target.max(start_cross);
    }

    let animate_width = |which, window: &mut Window, cx: &mut App| {
        let style = thumb_styles[thumb_index(which)];
        let rest = f32::from(thumb_rest_width(style));
        let pressed = f32::from(thumb_pressed_width(style));
        px(animate(
            (
                id.clone(),
                if which == RangeThumb::Start {
                    "start-width"
                } else {
                    "end-width"
                },
            ),
            if dragging == Some(which) {
                pressed
            } else {
                rest
            },
            resize,
            window,
            cx,
        )
        .clamp(rest.min(pressed), rest.max(pressed)))
    };
    let start_width = if range {
        animate_width(RangeThumb::Start, window, cx)
    } else {
        px(4.)
    };
    let end_width = animate_width(RangeThumb::End, window, cx);
    let handle_sizes = [
        gpui::size(start_width, start_cross),
        gpui::size(end_width, end_cross),
    ];
    let axis_padding = |style| thumb_rest_width(style).max(thumb_pressed_width(style)) / 2.;
    let mut padding = px(8.).max(axis_padding(thumb_styles[1]));
    if range {
        padding = padding.max(axis_padding(thumb_styles[0]));
    }
    TrackLayout {
        thickness,
        track_radius,
        inner_radius,
        thumb_gap,
        icon_size,
        target,
        padding,
        handle_sizes,
    }
}

fn sync_gesture_feedback(
    slider: &Slider,
    focus_handles: [&FocusHandle; 2],
    gesture: &RefCell<Gesture>,
    range: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let mut gesture = gesture.borrow_mut();
    gesture.disabled = slider.disabled;
    for which in [RangeThumb::Start, RangeThumb::End] {
        let index = thumb_index(which);
        let feedback = &mut gesture.feedback[index];
        feedback.focus = Some(focus_handles[index].clone());
        feedback.tracker = slider.thumbs[index]
            .source
            .as_ref()
            .filter(|_| range || which == RangeThumb::End)
            .map(|source| {
                InteractionSource::track(source, &thumb_id(&slider.id, which), window, cx)
            });
        if slider.disabled || !window.is_window_active() || !range && which == RangeThumb::Start {
            feedback.hovered = false;
        }
    }
    gesture.record(window, cx);
}

#[allow(clippy::too_many_arguments)]
fn label_opacity(
    id: &ElementId,
    which: RangeThumb,
    focus: &FocusHandle,
    value_indicator: bool,
    disabled: bool,
    dragging: Option<RangeThumb>,
    reveal: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    if !value_indicator || disabled {
        return 0.;
    }
    let visible = window.is_window_active()
        && (dragging == Some(which)
            || dragging.is_none() && focus.is_focused(window) && window.last_input_was_keyboard());
    animate(
        (
            id.clone(),
            if which == RangeThumb::Start {
                "start-label"
            } else {
                "end-label"
            },
        ),
        f32::from(visible),
        reveal,
        window,
        cx,
    )
    .clamp(0., 1.)
}

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
fn render_track_artwork(
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

#[allow(clippy::too_many_arguments)]
fn render_value_label(
    which: RangeThumb,
    opacity: f32,
    value: SliderValue,
    format_value: Option<&super::FormatValue>,
    geometry: Geometry,
    palette: SliderPalette,
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    if opacity <= 0. {
        return None;
    }
    let numeric_value = if which == RangeThumb::Start {
        value.start()
    } else {
        value.end()
    };
    let text = format_value.map_or_else(
        || format!("{numeric_value:.2}").into(),
        |format| format(numeric_value),
    );
    let theme = current_material_theme(window, cx);
    let foreground = palette.inverse_on_surface.opacity(opacity);
    let background = palette.inverse_surface.opacity(opacity);
    let horizontal = geometry.axis.is_horizontal();
    let line = shape_text(
        text,
        TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
        &theme.typography,
        1.,
        foreground,
        window,
        cx,
    )?;
    Some(
        canvas(
            |_, _, _| (),
            move |thumb, _, window, cx| {
                let size = gpui::size(
                    (line.line().width.ceil() + px(16.)).max(px(48.)),
                    line.line_height().max(px(44.)),
                );
                let visible = window.content_mask().bounds;
                if !visible.contains(&thumb.center()) {
                    return;
                }
                let origin =
                    value_label_origin(thumb, size, visible, horizontal, geometry.direction);
                let bounds = Bounds::new(origin, size);
                window.paint_quad(gpui::quad(
                    bounds,
                    px(6.),
                    background,
                    px(0.),
                    background,
                    gpui::BorderStyle::Solid,
                ));
                let _ = line.paint(
                    origin
                        + point(
                            (size.width - line.line().width) / 2.,
                            (size.height - line.line_height()) / 2.,
                        ),
                    gpui::TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            },
        )
        .absolute()
        .inset_0()
        .size_full()
        .into_any_element(),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_thumb(
    id: &ElementId,
    which: RangeThumb,
    position: f32,
    name: SharedString,
    focus: &FocusHandle,
    content: Option<AnyElement>,
    style: SliderThumbStyle,
    label_opacity: f32,
    value: SliderValue,
    format_value: Option<&super::FormatValue>,
    state: &Entity<SliderState>,
    gesture: &Rc<RefCell<Gesture>>,
    geometry: Geometry,
    layout: TrackLayout,
    palette: SliderPalette,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let horizontal = geometry.axis.is_horizontal();
    let handle_size = layout.handle_sizes[thumb_index(which)];
    let handle_width = handle_size.width;
    let half_handle = handle_width / 2.;
    let cross = handle_size.height;
    let (width, height) = if horizontal {
        (handle_width, cross)
    } else {
        (cross, handle_width)
    };
    let tint = if disabled {
        style.disabled_color
    } else {
        style.color
    }
    .unwrap_or(palette.thumb);
    let focus_color = style.focused_color.unwrap_or(palette.ring);
    let label = render_value_label(
        which,
        label_opacity,
        value,
        format_value,
        geometry,
        palette,
        window,
        cx,
    );
    let custom = content.is_some();
    let element = geometry
        .place(
            div()
                .absolute()
                .when(horizontal, |thumb| {
                    thumb
                        .top((layout.target - height) / 2.)
                        .ml(-half_handle)
                        .when(geometry.reversed(), |thumb| {
                            thumb.ml(px(0.)).mr(-half_handle)
                        })
                })
                .when(!horizontal, |thumb| {
                    thumb.left((layout.target - width) / 2.).when_else(
                        geometry.reversed(),
                        |thumb| thumb.mt(-half_handle),
                        |thumb| thumb.mb(-half_handle),
                    )
                }),
            position,
        )
        .id(thumb_id(id, which))
        .when(!disabled, |thumb| thumb.role(Role::Slider))
        .when(!disabled, |thumb| thumb.aria_label(name))
        .w(width)
        .h(height)
        .when_else(
            custom,
            |thumb| {
                thumb
                    .text_color(tint)
                    .focus_visible(move |style| style.border_1().border_color(focus_color))
                    .children(content.map(|art| div().size_full().child(art)))
            },
            |thumb| {
                thumb
                    .rounded_full()
                    .bg(tint)
                    .focus_visible(move |style| style.bg(focus_color))
            },
        )
        .children(label);

    let element = if disabled {
        element
    } else {
        let keyboard_state = state.clone();
        let hover = gesture.clone();
        range_semantics(element, state, which, geometry.axis, cx)
            .capture_key_down(move |event, window, cx| {
                let Some(sign) = geometry.reversed_key_delta(&event.keystroke.key) else {
                    return;
                };
                keyboard_state.update(cx, |state, cx| {
                    state.step_thumb(
                        which == RangeThumb::Start,
                        sign * state.accessibility_step(),
                        window,
                        cx,
                    );
                });
                cx.stop_propagation();
            })
            .on_hover(move |hovered, window, cx| {
                let mut hover = hover.borrow_mut();
                hover.feedback[thumb_index(which)].hovered = *hovered;
                hover.record(window, cx);
            })
    };
    element.test_support().when(!disabled, |thumb| {
        thumb.track_focus(&focus.clone().tab_stop(true))
    })
}

fn render_touch_layer(
    state: Entity<SliderState>,
    gesture: Rc<RefCell<Gesture>>,
    focus: [FocusHandle; 2],
    geometry: Geometry,
    range: bool,
    handle_sizes: [Size<Pixels>; 2],
) -> impl IntoElement {
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, _| {
            let gesture = gesture.clone();
            let state = state.clone();
            let focus = focus.clone();
            window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                if !phase.bubble() {
                    return;
                }
                if event.phase == TouchPhase::Started {
                    if window.default_prevented()
                        || !hitbox.is_hovered_at(event.start_position, window)
                    {
                        return;
                    }
                    let bounds = state.read(cx).bounds();
                    if bounds.size.along(geometry.axis) <= px(0.) {
                        return;
                    }
                    let current = state.read(cx).percentage();
                    let which = if range && current.start == current.end {
                        let fraction = geometry.fraction(bounds, event.start_position);
                        choose_thumb(
                            true,
                            fraction,
                            fraction,
                            geometry.fraction(bounds, event.position),
                        )
                    } else {
                        geometry.choose_at(
                            range,
                            current,
                            handle_sizes,
                            bounds,
                            event.start_position,
                        )
                    };
                    window.prevent_default();
                    {
                        let mut gesture = gesture.borrow_mut();
                        gesture.origin = Some(event.start_position);
                        gesture.thumb = Some(which);
                    }
                    focus[thumb_index(which)].focus(window, cx);
                }
                let which = {
                    let gesture = gesture.borrow();
                    if gesture.origin != Some(event.start_position) || gesture.capture.is_some() {
                        return;
                    }
                    let Some(which) = gesture.thumb else {
                        return;
                    };
                    which
                };
                match event.phase {
                    TouchPhase::Started | TouchPhase::Moved => {
                        {
                            let mut gesture = gesture.borrow_mut();
                            gesture.dragged = event.position != event.start_position;
                            gesture.record(window, cx);
                        }
                        update(&state, geometry, which, event.position, window, cx);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        finish(&state, &gesture, window, cx, false);
                    }
                }
                cx.stop_propagation();
            });
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

#[allow(clippy::too_many_arguments)]
fn attach_pointer_handlers(
    root: Stateful<gpui::Div>,
    state: Entity<SliderState>,
    gesture: Rc<RefCell<Gesture>>,
    focus: [FocusHandle; 2],
    geometry: Geometry,
    range: bool,
    start: f32,
    end: f32,
    handle_sizes: [Size<Pixels>; 2],
) -> AnyElement {
    let touch_layer = render_touch_layer(
        state.clone(),
        gesture.clone(),
        focus.clone(),
        geometry,
        range,
        handle_sizes,
    );
    let [start_focus, end_focus] = focus.clone();
    let down = gesture.clone();
    let begin = gesture.clone();
    let drag = gesture.clone();
    let release = gesture.clone();
    let release_out = gesture.clone();
    let scroll = gesture.clone();
    let escape = gesture;
    let down_state = state.clone();
    let begin_state = state.clone();
    let drag_state = state.clone();
    let drag_source = state.clone();
    let drag_focus = focus;
    let release_state = state.clone();
    let outside_state = state.clone();
    let escape_state = state;
    root.child(touch_layer)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let bounds = down_state.read(cx).bounds();
            if bounds.size.along(geometry.axis) <= px(0.) {
                return;
            }
            let which = geometry.choose_at(
                range,
                down_state.read(cx).percentage(),
                handle_sizes,
                bounds,
                event.position,
            );
            {
                let mut down = down.borrow_mut();
                down.thumb = Some(which);
                down.origin = Some(event.position);
            }
            if which == RangeThumb::Start {
                start_focus.focus(window, cx);
            } else {
                end_focus.focus(window, cx);
            }
            down.borrow().record(window, cx);
            update(&down_state, geometry, which, event.position, window, cx);
        })
        .on_drag(SliderDrag(drag_source), move |payload, _, window, cx| {
            let gesture = Rc::downgrade(&begin);
            let state = begin_state.clone();
            begin.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |window, cx| {
                if let Some(gesture) = gesture.upgrade() {
                    finish(&state, &gesture, window, cx, false);
                }
            }));
            {
                let mut begin = begin.borrow_mut();
                begin.dragged = true;
                begin.record(window, cx);
            }
            cx.stop_propagation();
            cx.new(|_| SliderDrag(payload.0.clone()))
        })
        .on_drag_move(move |event: &DragMoveEvent<SliderDrag>, window, cx| {
            if event.drag(cx).0.entity_id() != drag_state.entity_id() {
                return;
            }
            let mut gesture = drag.borrow_mut();
            let Some(mut which) = gesture.thumb else {
                return;
            };
            if range
                && start == end
                && let Some(origin) = gesture.origin.take()
            {
                let bounds = drag_state.read(cx).bounds();
                which = if geometry.fraction(bounds, event.event.position)
                    < geometry.fraction(bounds, origin)
                {
                    RangeThumb::Start
                } else {
                    RangeThumb::End
                };
                gesture.thumb = Some(which);
                drag_focus[thumb_index(which)].focus(window, cx);
            }
            gesture.record(window, cx);
            update(
                &drag_state,
                geometry,
                which,
                event.event.position,
                window,
                cx,
            );
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            finish(&release_state, &release, window, cx, false)
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            finish(&outside_state, &release_out, window, cx, false)
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" && escape.borrow().thumb.is_some() {
                finish(&escape_state, &escape, window, cx, true);
                cx.stop_propagation();
            }
        })
        .on_scroll(move |_, _, cx| {
            // Wheel events over the active range gesture stay with it.
            if scroll.borrow().thumb.is_some() {
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

impl RenderOnce for Slider {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ([start_focus, end_focus], gesture) = window
            .use_keyed_state(
                ElementId::from((self.id.clone(), "slider-state")),
                cx,
                |_, cx| {
                    (
                        [cx.focus_handle(), cx.focus_handle()],
                        Rc::new(RefCell::new(Gesture::default())),
                    )
                },
            )
            .read(cx)
            .clone();
        let geometry = Geometry {
            axis: self.axis,
            direction: current_material_theme(window, cx).layout.direction,
            reverse_vertical: self.reverse_vertical,
        };
        let horizontal = self.axis.is_horizontal();
        let state = self.state.read(cx);
        let value = state.value();
        let percentage = state.percentage();
        let start = percentage.start;
        let end = percentage.end;
        let range = value.is_range();
        let centered = self.kind == SliderKind::Centered && !range;
        let center = if centered { 0.5 } else { start };
        sync_gesture_feedback(
            &self,
            [&start_focus, &end_focus],
            &gesture,
            range,
            window,
            cx,
        );

        let [start_content, end_content] = self.thumbs.each_mut().map(|thumb| thumb.content.take());
        let thumb_styles = [self.thumbs[0].style, self.thumbs[1].style];
        let palette = resolve_palette(self.colors, self.disabled, window, cx);
        let dragging = gesture
            .borrow()
            .thumb
            .filter(|_| !self.disabled && window.is_window_active());
        let reduced = cx.reduce_motion();
        let scheme = current_material_theme(window, cx).motion;
        let resize = scheme.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let reveal = scheme.spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let layout = resolve_track_layout(
            &self.id,
            self.size,
            self.track_style,
            thumb_styles,
            range,
            dragging,
            resize,
            window,
            cx,
        );

        let start_label = if range {
            label_opacity(
                &self.id,
                RangeThumb::Start,
                &start_focus,
                self.value_indicator,
                self.disabled,
                dragging,
                reveal,
                window,
                cx,
            )
        } else {
            0.
        };
        let end_label = label_opacity(
            &self.id,
            RangeThumb::End,
            &end_focus,
            self.value_indicator,
            self.disabled,
            dragging,
            reveal,
            window,
            cx,
        );
        // Expressive tracks leave 6 dp between a handle's edge and each adjacent segment.
        let end_gap = layout.thumb_gap + layout.handle_sizes[1].width / 2.;
        let center_gap = if range {
            layout.thumb_gap + layout.handle_sizes[0].width / 2.
        } else if centered {
            layout.thumb_gap
        } else {
            px(0.)
        };
        let (first, last) = if center <= end {
            ((center, center_gap), (end, end_gap))
        } else {
            ((end, end_gap), (center, center_gap))
        };
        let track_artwork = render_track_artwork(
            self.track.take(),
            self.icon.take(),
            geometry,
            first,
            last,
            layout,
            palette,
            self.disabled,
            self.tick_intervals,
            self.show_stop_indicators,
            self.draw_tick.take(),
            self.draw_stop.take(),
        );
        let track = SliderIndicator::new(&self.state)
            .relative()
            .size_full()
            .child(track_artwork);
        let disabled_name = self.range_end_name.as_ref().map_or_else(
            || self.name.clone(),
            |end| format!("{} – {end}", self.name).into(),
        );
        // The indicator records these bounds for Base input. Absolute thumbs are children
        // of the same unpadded coordinate space, so percentages use those bounds as well.
        let format_value = self.format_value.as_ref();
        let track_and_handles = div()
            .relative()
            .size_full()
            .child(track)
            .when(range, |root| {
                root.child(render_thumb(
                    &self.id,
                    RangeThumb::Start,
                    start,
                    self.name.clone(),
                    &start_focus,
                    start_content,
                    thumb_styles[0],
                    start_label,
                    value,
                    format_value,
                    &self.state,
                    &gesture,
                    geometry,
                    layout,
                    palette,
                    self.disabled,
                    window,
                    cx,
                ))
            })
            .child(render_thumb(
                &self.id,
                RangeThumb::End,
                end,
                self.range_end_name.unwrap_or(self.name),
                &end_focus,
                end_content,
                thumb_styles[1],
                end_label,
                value,
                format_value,
                &self.state,
                &gesture,
                geometry,
                layout,
                palette,
                self.disabled,
                window,
                cx,
            ));
        let root = div()
            .id(self.id.clone())
            .relative()
            .when(horizontal, |root| {
                root.w_full().h(layout.target).px(layout.padding)
            })
            .when(!horizontal, |root| {
                root.w(layout.target).h(px(200.)).py(layout.padding)
            })
            .child(track_and_handles);
        let active_thumb = gesture.borrow().thumb;
        let inactive = !window.is_window_active();
        if active_thumb
            .is_some_and(|which| self.disabled || inactive || !range && which == RangeThumb::Start)
        {
            let state = self.state.clone();
            let gesture = gesture.clone();
            window.defer(cx, move |window, cx| {
                finish(&state, &gesture, window, cx, true)
            });
        }
        if self.disabled || inactive && active_thumb.is_some() {
            return if self.disabled {
                BaseSlider::new(&self.state)
                    .axis(self.axis)
                    .disabled(true)
                    .accessibility_label(disabled_name)
                    .child(root)
                    .into_any_element()
            } else {
                root.into_any_element()
            };
        }

        attach_pointer_handlers(
            root,
            self.state,
            gesture,
            [start_focus, end_focus],
            geometry,
            range,
            start,
            end,
            layout.handle_sizes,
        )
    }
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

fn update(
    state: &Entity<SliderState>,
    geometry: Geometry,
    thumb: RangeThumb,
    pointer: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        let bounds = state.bounds();
        if bounds.size.along(geometry.axis) <= px(0.) {
            return;
        }
        let position = geometry.position(bounds, pointer);
        state.update_value_by_position(
            geometry.axis,
            position,
            thumb == RangeThumb::Start,
            window,
            cx,
        );
    });
}

fn finish(
    state: &Entity<SliderState>,
    gesture: &RefCell<Gesture>,
    window: &mut Window,
    cx: &mut App,
    cancel: bool,
) {
    let capture = {
        let mut gesture = gesture.borrow_mut();
        if gesture.thumb.take().is_none() {
            return;
        }
        gesture.origin = None;
        gesture.dragged = false;
        gesture.record(window, cx);
        gesture.capture.take()
    };
    if let Some(capture) = capture
        && cancel
    {
        capture.cancel(window, cx);
    }
    state.update(cx, |state, cx| state.handle_release(cx));
    window.refresh();
}
