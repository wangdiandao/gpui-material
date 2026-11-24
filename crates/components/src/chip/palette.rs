use gpui::{App, ElementId, Hsla, Pixels, Window, px};
use gpui_material_foundation::{animate, color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, Elevation, MotionKind, MotionSpeed};

use crate::SemanticTone;
use crate::action::{ActionLayout, ActionStyle, DisabledStyle};
use crate::control_style::{
    ButtonColors, ButtonStyle, ControlBorder, ControlElevations, ControlShapes, finite_length,
    mix_color,
};

use super::{ChipColors, ChipVariant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ChipToneRoles {
    foreground: ColorRole,
    leading: ColorRole,
    trailing: ColorRole,
    selected_container: ColorRole,
    border: ColorRole,
}

fn chip_tone_roles(
    variant: ChipVariant,
    selected: bool,
    tonal: bool,
    tone: SemanticTone,
) -> ChipToneRoles {
    use ColorRole::*;
    if let Some((role, _, container, on_container)) = tone.roles() {
        let fg = if selected { on_container } else { role };
        ChipToneRoles {
            foreground: fg,
            leading: fg,
            trailing: fg,
            selected_container: container,
            border: role,
        }
    } else {
        ChipToneRoles {
            foreground: if selected {
                OnSecondaryContainer
            } else if variant == ChipVariant::Assist {
                OnSurface
            } else {
                OnSurfaceVariant
            },
            leading: match (variant, selected, tonal) {
                (ChipVariant::Input, false, _) | (ChipVariant::Filter, false, true) => {
                    OnSurfaceVariant
                }
                (ChipVariant::Filter, true, _) => OnSecondaryContainer,
                _ => Primary,
            },
            trailing: if selected {
                OnSecondaryContainer
            } else {
                OnSurfaceVariant
            },
            selected_container: SecondaryContainer,
            border: OutlineVariant,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ChipPalette {
    pub(super) foreground: ColorRole,
    pub(super) label: Hsla,
    pub(super) leading: Hsla,
    pub(super) trailing: Hsla,
    pub(super) container: Hsla,
    pub(super) border_color: Hsla,
    pub(super) border_width: Pixels,
    pub(super) selection: f32,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_palette(
    id: &ElementId,
    variant: ChipVariant,
    selectable: bool,
    selected: bool,
    tonal: bool,
    tone: SemanticTone,
    elevated: bool,
    disabled: bool,
    chip_colors: ChipColors,
    border: Option<ControlBorder>,
    window: &mut Window,
    cx: &mut App,
) -> ChipPalette {
    use ColorRole::*;
    let colors = chip_colors.for_state(disabled, selected);
    let roles = chip_tone_roles(variant, selected, tonal, tone);
    let foreground = roles.foreground;
    let (on_surface, label, leading, trailing, idle, checked, border_color, effects) = {
        let theme = current_material_theme(window, cx);
        let theme_color = |role| color(theme.colors.color(role));
        let on_surface = theme_color(OnSurface);
        let label = colors.label.unwrap_or_else(|| {
            if disabled {
                on_surface.opacity(0.38)
            } else {
                theme_color(foreground)
            }
        });
        let leading = colors.leading.unwrap_or_else(|| {
            if disabled {
                on_surface.opacity(0.38)
            } else {
                theme_color(roles.leading)
            }
        });
        let trailing = colors.trailing.unwrap_or_else(|| {
            if disabled {
                on_surface.opacity(0.38)
            } else {
                theme_color(roles.trailing)
            }
        });
        let idle = chip_colors.enabled.container.unwrap_or_else(|| {
            if elevated {
                theme_color(SurfaceContainerLow)
            } else {
                theme_color(roles.selected_container).opacity(0.)
            }
        });
        let checked = chip_colors
            .selected
            .container
            .unwrap_or_else(|| theme_color(roles.selected_container));
        let border_color = colors
            .border
            .or(border.map(|border| border.color))
            .unwrap_or_else(|| {
                if disabled {
                    on_surface.opacity(0.12)
                } else {
                    theme_color(roles.border)
                }
            });
        let effects = theme.motion.spec(
            MotionKind::Effects,
            MotionSpeed::Fast,
            cx.reduce_motion() || disabled,
        );
        (
            on_surface,
            label,
            leading,
            trailing,
            idle,
            checked,
            border_color,
            effects,
        )
    };
    let selection = if selectable {
        animate(
            (id.clone(), "chip-tint"),
            f32::from(selected),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.)
    } else {
        0.
    };
    let container = if disabled {
        colors
            .container
            .unwrap_or_else(|| on_surface.opacity(if selected || elevated { 0.12 } else { 0. }))
    } else if selection == 0. {
        idle
    } else if selection == 1. {
        checked
    } else {
        mix_color(idle, checked, selection)
    };
    let border_width = border.map_or(px(f32::from(!elevated && !selected)), |border| {
        finite_length(border.width)
    });
    ChipPalette {
        foreground,
        label,
        leading,
        trailing,
        container,
        border_color,
        border_width,
        selection,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_chip_style(
    variant: ChipVariant,
    selectable: bool,
    selected: bool,
    tonal: bool,
    elevated: bool,
    shapes: ControlShapes,
    foreground: ColorRole,
) -> ActionStyle {
    ActionStyle {
        background: None,
        background_color: None,
        foreground,
        disabled: DisabledStyle {
            foreground: ColorRole::OnSurface,
            background: None,
            outline: (ColorRole::OnSurface, 0.12),
        },
        height: 32.,
        min_width: 0.,
        corner: if tonal {
            CornerRole::Medium
        } else {
            CornerRole::Small
        },
        pressed_corner: CornerRole::Small,
        selected_corner: if tonal {
            Some(CornerRole::Full)
        } else if selectable && shapes.checked.is_some() {
            Some(CornerRole::Small)
        } else {
            None
        },
        corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
        outline: 0.,
        outline_role: ColorRole::OutlineVariant,
        elevation: if elevated {
            Elevation::Level1
        } else {
            Elevation::Level0
        },
        hover_elevation: if elevated {
            Elevation::Level2
        } else if selected && variant == ChipVariant::Filter {
            Elevation::Level1
        } else {
            Elevation::Level0
        },
    }
}

pub(super) fn build_chip_layout(
    split: bool,
    tab_stop: bool,
    dragged: bool,
    shapes: ControlShapes,
    elevations: ControlElevations,
    palette: ChipPalette,
) -> ActionLayout {
    let resolved_elevations = ControlElevations {
        dragged: elevations.dragged.or(Some(Elevation::Level4)),
        ..elevations
    };
    ActionLayout {
        tab_stop,
        dragged,
        appearance: ButtonStyle {
            shapes,
            colors: action_colors(
                palette.label,
                if split {
                    gpui::transparent_black()
                } else {
                    palette.container
                },
            ),
            border: (!split).then_some(ControlBorder {
                width: palette.border_width,
                color: palette.border_color,
            }),
            elevations: if split {
                ControlElevations::default()
            } else {
                resolved_elevations
            },
        },
        ..Default::default()
    }
}

pub(super) fn action_colors(label: Hsla, container: Hsla) -> ButtonColors {
    ButtonColors {
        container: Some(container),
        content: Some(label),
        disabled_container: Some(container),
        disabled_content: Some(label),
        checked_container: Some(container),
        checked_content: Some(label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_tone_roles_map_semantic_tones_and_preserve_defaults() {
        use ColorRole::*;
        let default_assist =
            chip_tone_roles(ChipVariant::Assist, false, false, SemanticTone::Default);
        assert_eq!(default_assist.foreground, OnSurface);
        assert_eq!(default_assist.border, OutlineVariant);
        assert_eq!(default_assist.selected_container, SecondaryContainer);

        let default_selected =
            chip_tone_roles(ChipVariant::Filter, true, false, SemanticTone::Default);
        assert_eq!(default_selected.foreground, OnSecondaryContainer);
        assert_eq!(default_selected.leading, OnSecondaryContainer);
        assert_eq!(default_selected.trailing, OnSecondaryContainer);

        for (tone, role, container, on_container) in [
            (
                SemanticTone::Danger,
                Error,
                ErrorContainer,
                OnErrorContainer,
            ),
            (
                SemanticTone::Success,
                Success,
                SuccessContainer,
                OnSuccessContainer,
            ),
            (
                SemanticTone::Warning,
                Warning,
                WarningContainer,
                OnWarningContainer,
            ),
            (SemanticTone::Info, Info, InfoContainer, OnInfoContainer),
        ] {
            let unselected = chip_tone_roles(ChipVariant::Filter, false, false, tone);
            assert_eq!(unselected.foreground, role);
            assert_eq!(unselected.leading, role);
            assert_eq!(unselected.trailing, role);
            assert_eq!(unselected.border, role);
            assert_eq!(unselected.selected_container, container);

            let selected = chip_tone_roles(ChipVariant::Filter, true, false, tone);
            assert_eq!(selected.foreground, on_container);
            assert_eq!(selected.leading, on_container);
            assert_eq!(selected.trailing, on_container);
            assert_eq!(selected.selected_container, container);
        }
    }
}
