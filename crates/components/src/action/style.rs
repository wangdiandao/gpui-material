use gpui::{App, Bounds, ElementId, Hsla, Pixels, Window};
use gpui_material_foundation::{InteractionState, animate, color, current_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

use crate::control_style::{ButtonStyle, ControlShapes, finite_length};

#[derive(Clone, Copy)]
pub(crate) struct ActionStyle {
    pub(crate) background: Option<ColorRole>,
    pub(crate) background_color: Option<Hsla>,
    pub(crate) foreground: ColorRole,
    pub(crate) disabled: DisabledStyle,
    pub(crate) height: f32,
    pub(crate) min_width: f32,
    pub(crate) corner: CornerRole,
    pub(crate) pressed_corner: CornerRole,
    pub(crate) selected_corner: Option<CornerRole>,
    pub(crate) corner_motion: (MotionKind, MotionSpeed),
    pub(crate) outline: f32,
    pub(crate) outline_role: ColorRole,
    pub(crate) elevation: Elevation,
    pub(crate) hover_elevation: Elevation,
}

/// Disabled pigments are component tokens, not a universal opacity applied to
/// the enabled colors. In particular buttons and chips use different fills.
#[derive(Clone, Copy)]
pub(crate) struct DisabledStyle {
    pub(crate) foreground: ColorRole,
    pub(crate) background: Option<(ColorRole, f32)>,
    pub(crate) outline: (ColorRole, f32),
}

impl ActionStyle {
    pub(crate) fn foreground(self, disabled: bool, window: &Window, cx: &App) -> Hsla {
        let colors = &current_material_theme(window, cx).colors;
        if disabled {
            color(colors.color(self.disabled.foreground)).opacity(0.38)
        } else {
            color(colors.color(self.foreground))
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ActionLayout {
    pub(crate) segment: Option<SegmentShape>,
    pub(crate) width: Option<Pixels>,
    pub(crate) tab_stop: bool,
    pub(crate) decorative: bool,
    pub(crate) loading: bool,
    pub(crate) appearance: ButtonStyle,
    pub(crate) presentation_selected: Option<bool>,
    pub(crate) dragged: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct SegmentShape {
    pub(crate) leading: bool,
    pub(crate) trailing: bool,
    pub(crate) outer: CornerRole,
    pub(crate) inner: CornerRole,
    pub(crate) pressed: CornerRole,
}

#[derive(Clone, Copy)]
pub(super) struct ActionPalette {
    pub(super) foreground: Hsla,
    pub(super) background: Hsla,
    pub(super) outline: Hsla,
    pub(super) ring: Hsla,
    pub(super) shadow: Hsla,
}

#[derive(Clone, Copy)]
pub(super) struct ActionVisuals {
    pub(super) progress: f32,
    pub(super) selection: f32,
    pub(super) layer: f32,
    pub(super) outline_width: f32,
    pub(super) elevation: f32,
}

pub(super) fn resolve_palette(
    style: ActionStyle,
    layout: ActionLayout,
    disabled: bool,
    selected: bool,
    window: &Window,
    cx: &App,
) -> ActionPalette {
    let theme = current_material_theme(window, cx);
    let ring = color(theme.colors.color(ColorRole::Secondary));
    let shadow = color(theme.colors.color(ColorRole::Shadow));
    let foreground = layout
        .appearance
        .colors
        .content(disabled, selected)
        .unwrap_or_else(|| style.foreground(disabled, window, cx));
    let background = layout
        .appearance
        .colors
        .container(disabled, selected)
        .unwrap_or_else(|| {
            if disabled {
                style
                    .disabled
                    .background
                    .map_or_else(Hsla::default, |(role, opacity)| {
                        color(theme.colors.color(role)).opacity(opacity)
                    })
            } else {
                style
                    .background_color
                    .or_else(|| style.background.map(|role| color(theme.colors.color(role))))
                    .unwrap_or_default()
            }
        });
    let outline = layout.appearance.border.map_or_else(
        || {
            if disabled {
                color(theme.colors.color(style.disabled.outline.0))
                    .opacity(style.disabled.outline.1)
            } else {
                color(theme.colors.color(style.outline_role))
            }
        },
        |border| border.color,
    );
    ActionPalette {
        foreground,
        background,
        outline,
        ring,
        shadow,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn animate_visuals(
    id: &ElementId,
    style: ActionStyle,
    layout: ActionLayout,
    interaction: InteractionState,
    selected: bool,
    enabled: bool,
    scale: f32,
    motion: MotionSpec,
    effects: MotionSpec,
    reduced: bool,
    window: &mut Window,
    cx: &mut App,
) -> ActionVisuals {
    let hovered = interaction.hovered;
    let pressed = interaction.pressed;
    let progress = animate(
        (id.clone(), "press-shape"),
        f32::from(pressed),
        motion,
        window,
        cx,
    )
    .clamp(0., 1.);
    let selection = if style.selected_corner.is_some() {
        animate(
            (id.clone(), "selected-shape"),
            f32::from(selected),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.)
    } else {
        0.
    };
    let layer = animate(
        (id.clone(), "state-layer"),
        interaction
            .layer_with_ripple(reduced)
            .map_or(0., |layer| layer.opacity()),
        if enabled {
            effects
        } else {
            MotionSpec::Instant
        },
        window,
        cx,
    )
    .clamp(0., 1.);
    let outline_width = animate(
        (id.clone(), "outline-width"),
        layout.appearance.border.map_or(style.outline, |border| {
            f32::from(finite_length(border.width))
        }),
        effects,
        window,
        cx,
    )
    .max(0.);
    let default_elevation = if !enabled {
        0.
    } else if hovered && !pressed {
        style.hover_elevation.dp()
    } else {
        style.elevation.dp()
    };
    let elevation = animate(
        (id.clone(), "elevation"),
        layout
            .appearance
            .elevations
            .for_state(interaction)
            .map_or(default_elevation, Elevation::dp),
        effects,
        window,
        cx,
    )
    .max(0.)
        * scale;
    ActionVisuals {
        progress,
        selection,
        layer,
        outline_width,
        elevation,
    }
}

pub(crate) fn interpolated_corners(
    bounds: Bounds<Pixels>,
    style: ActionStyle,
    selection: f32,
    press: f32,
    scale: f32,
    layout: ActionLayout,
) -> CornerShape {
    let width = f32::from(bounds.size.width) / scale;
    let height = f32::from(bounds.size.height) / scale;
    let radii = |shape: CornerShape| shape.radii(width, height, Direction::Ltr);
    let ControlShapes {
        resting,
        pressed: pressed_shape,
        checked,
    } = layout.appearance.shapes;
    let mut rest = radii(resting.unwrap_or_else(|| style.corner.shape()));
    let mut selected =
        radii(checked.unwrap_or_else(|| style.selected_corner.unwrap_or(style.corner).shape()));
    let mut pressed = radii(pressed_shape.unwrap_or_else(|| style.pressed_corner.shape()));
    if let Some(segment) = layout.segment {
        let inner = radii(segment.inner.shape())[0];
        let inner_pressed = radii(segment.pressed.shape())[0];
        let outer = radii(segment.outer.shape())[0];
        let outside = [
            segment.leading,
            segment.trailing,
            segment.trailing,
            segment.leading,
        ];
        if segment.inner == CornerRole::None {
            rest = std::array::from_fn(|ix| if outside[ix] { rest[ix] } else { inner });
            selected = std::array::from_fn(|ix| if outside[ix] { selected[ix] } else { inner });
            pressed = std::array::from_fn(|ix| {
                if outside[ix] {
                    pressed[ix]
                } else {
                    inner_pressed
                }
            });
        } else {
            if resting.is_none() {
                rest = outside.map(|outer_edge| if outer_edge { outer } else { inner });
            }
            if checked.is_none() {
                selected = radii(CornerRole::Full.shape());
            }
            if pressed_shape.is_none() {
                pressed = outside.map(|outer_edge| if outer_edge { outer } else { inner_pressed });
            }
        }
    }
    CornerShape::Radii(std::array::from_fn(|ix| {
        let resting = rest[ix] + (selected[ix] - rest[ix]) * selection;
        (resting + (pressed[ix] - resting) * press) * scale
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    #[test]
    fn split_segment_zero_inner_clips_interior_seam_in_all_states() {
        let style = ActionStyle {
            background: None,
            background_color: None,
            foreground: ColorRole::OnSurface,
            disabled: DisabledStyle {
                foreground: ColorRole::OnSurface,
                background: None,
                outline: (ColorRole::OnSurface, 0.12),
            },
            height: 32.,
            min_width: 32.,
            corner: CornerRole::Small,
            pressed_corner: CornerRole::Small,
            selected_corner: Some(CornerRole::Medium),
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: 0.,
            outline_role: ColorRole::Outline,
            elevation: Elevation::Level0,
            hover_elevation: Elevation::Level0,
        };
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(80.), px(32.)));
        let custom_shapes = ControlShapes {
            resting: Some(CornerShape::Radii([10., 12., 14., 16.])),
            pressed: Some(CornerShape::Radii([6., 7., 8., 9.])),
            checked: Some(CornerRole::Full.shape()),
        };

        for (leading, trailing) in [(true, false), (false, true)] {
            let layout = ActionLayout {
                segment: Some(SegmentShape {
                    leading,
                    trailing,
                    outer: style.corner,
                    inner: CornerRole::None,
                    pressed: CornerRole::None,
                }),
                appearance: ButtonStyle {
                    shapes: custom_shapes,
                    ..Default::default()
                },
                ..ActionLayout::default()
            };
            for (selection, press) in [(0., 0.), (1., 0.), (0., 1.), (1., 1.)] {
                let CornerShape::Radii(radii) =
                    interpolated_corners(bounds, style, selection, press, 1., layout)
                else {
                    panic!("interpolated_corners must return explicit radii");
                };
                if leading {
                    assert!(radii[0] > 0. && radii[3] > 0.);
                    assert_eq!(radii[1], 0.);
                    assert_eq!(radii[2], 0.);
                } else {
                    assert_eq!(radii[0], 0.);
                    assert_eq!(radii[3], 0.);
                    assert!(radii[1] > 0. && radii[2] > 0.);
                }
            }
        }
    }
}
