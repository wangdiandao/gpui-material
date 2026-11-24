use gpui::{App, ElementId, FocusHandle, Pixels, Size, Window, px};
use gpui_material_foundation::{RangeThumb, animate};
use gpui_material_theme::{Density, MotionSpec};

use crate::ControlSize;

use super::geometry::dimension;
use super::{SliderThumbStyle, SliderTrackStyle, thumb_index};

#[derive(Clone, Copy)]
pub(super) struct TrackLayout {
    pub(super) thickness: Pixels,
    pub(super) track_radius: Pixels,
    pub(super) inner_radius: Pixels,
    pub(super) thumb_gap: Pixels,
    pub(super) icon_size: Option<f32>,
    pub(super) target: Pixels,
    pub(super) padding: Pixels,
    pub(super) handle_sizes: [Size<Pixels>; 2],
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
pub(super) fn resolve_track_layout(
    id: &ElementId,
    size: ControlSize,
    density: Density,
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
    let thickness = dimension(
        track_style.thickness,
        density.visual_size(thickness).max(8.),
        false,
    );
    let track_radius = dimension(track_style.outer_radius, track_radius, true).min(thickness / 2.);
    let inner_radius = dimension(track_style.inner_radius, 2., true).min(thickness / 2.);
    let thumb_gap = dimension(track_style.thumb_gap, 6., true);
    let default_cross = density.visual_size(thumb_extent).max(24.);
    let start_cross = thumb_cross_extent(thumb_styles[0], default_cross);
    let end_cross = thumb_cross_extent(thumb_styles[1], default_cross);
    let min_target = px(density.interactive_size(0.));
    let mut target = thickness.max(end_cross).max(min_target);
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

#[allow(clippy::too_many_arguments)]
pub(super) fn label_opacity(
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
