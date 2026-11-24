use gpui::{Edges, Pixels, px};
use gpui_material_foundation::resolve_edges;
use gpui_material_theme::{Density, Direction, LogicalEdges, TypeStyle};

use super::palette::FieldAnimation;

pub(super) struct FieldMetrics {
    pub(super) label_size: f32,
    pub(super) label_height: f32,
    pub(super) outline_width: Pixels,
    pub(super) padding: Edges<Pixels>,
    pub(super) editor_top_padding: Pixels,
    pub(super) editor_bottom_padding: Pixels,
    pub(super) minimum_height: Pixels,
    pub(super) label_top: f32,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_metrics(
    type_style: TypeStyle,
    caption_style: TypeStyle,
    animation: &FieldAnimation,
    density: Density,
    direction: Direction,
    min_height: Option<Pixels>,
    indicator_width: (Pixels, Pixels),
    content_padding: Option<LogicalEdges>,
    label_inside: bool,
    filled: bool,
) -> FieldMetrics {
    let label_size = type_style.size + (caption_style.size - type_style.size) * animation.floating;
    let label_height = type_style.line_height
        + (caption_style.line_height - type_style.line_height) * animation.floating;
    let (unfocused_indicator, focused_indicator) = indicator_width;
    let unfocused_w = f32::from(unfocused_indicator);
    let focused_w = f32::from(focused_indicator);
    let outline_width = px(unfocused_w + (focused_w - unfocused_w) * animation.focus_amount);
    let inside_vertical_pad = if density == Density::Compact { 6. } else { 8. };
    let padding = content_padding
        .map(|edges| resolve_edges(edges, direction))
        .unwrap_or(Edges {
            left: px(16.),
            right: px(16.),
            top: px(if label_inside {
                inside_vertical_pad
            } else {
                4.
            }),
            bottom: if label_inside && filled {
                (px(inside_vertical_pad) - outline_width).max(px(0.))
            } else {
                px(4.)
            },
        });
    let label_reserve = if label_inside {
        px(caption_style.line_height)
    } else {
        px(0.)
    };
    let editor_top_padding = padding.top + label_reserve;
    // The canvas covers the whole container, including the filled indicator.
    let editor_bottom_padding = padding.bottom + if filled { outline_width } else { px(0.) };
    let content_min_height =
        px(type_style.line_height) + editor_top_padding + editor_bottom_padding;
    let minimum_height = match min_height {
        Some(height) => height.max(content_min_height),
        None => px(density.visual_size(56.))
            .max(px(type_style.line_height + 2. * inside_vertical_pad) + label_reserve)
            .max(content_min_height),
    };
    let resting_top = f32::from(minimum_height - px(type_style.line_height)) * 0.5;
    let floating_top = if label_inside {
        f32::from(padding.top)
    } else {
        -caption_style.line_height * 0.5
    };
    let label_top = resting_top + (floating_top - resting_top) * animation.floating;
    FieldMetrics {
        label_size,
        label_height,
        outline_width,
        padding,
        editor_top_padding,
        editor_bottom_padding,
        minimum_height,
        label_top,
    }
}
