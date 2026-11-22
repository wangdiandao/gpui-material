use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, BorderStyle, Corners, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Stateful, Styled, Window, canvas, div, point,
    prelude::FluentBuilder, px, quad,
};
use gpui_material_foundation::{
    InteractionSource, InteractionState, InteractionTracker, ThemeScope, animate, color,
    current_material_theme, elevation_shadow_pair,
};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, DisabledStyle, SegmentShape,
    icon_slot, interpolated_corners,
};
use crate::button_style::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes, ButtonStyle,
    finite_length,
};
use crate::{Icon, IconName, SemanticTone};

use super::{Chip, ChipColors, ChipElevations, ChipVariant};

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
struct ChipPalette {
    foreground: ColorRole,
    label: Hsla,
    leading: Hsla,
    trailing: Hsla,
    container: Hsla,
    border_color: Hsla,
    border_width: Pixels,
    selection: f32,
}

#[allow(clippy::too_many_arguments)]
fn resolve_palette(
    id: &ElementId,
    variant: ChipVariant,
    selectable: bool,
    selected: bool,
    tonal: bool,
    tone: SemanticTone,
    elevated: bool,
    disabled: bool,
    chip_colors: ChipColors,
    border: Option<ButtonBorder>,
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
        crate::motion::mix(idle, checked, selection)
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
fn build_chip_style(
    variant: ChipVariant,
    selectable: bool,
    selected: bool,
    tonal: bool,
    elevated: bool,
    shapes: ButtonShapes,
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

fn build_chip_layout(
    split: bool,
    tab_stop: bool,
    dragged: bool,
    shapes: ButtonShapes,
    elevations: ChipElevations,
    dragged_elevation: Option<Elevation>,
    palette: ChipPalette,
) -> ActionLayout {
    ActionLayout {
        tab_stop,
        dragged,
        dragged_elevation: (!split).then_some(dragged_elevation).flatten(),
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
            border: (!split).then_some(ButtonBorder {
                width: palette.border_width,
                color: palette.border_color,
            }),
            elevations: if split {
                ButtonElevations::default()
            } else {
                elevations.button()
            },
        },
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
fn build_leading_slot(
    avatar: Option<AnyElement>,
    leading: Option<AnyElement>,
    filter_mark: f32,
    spacing: Pixels,
    leading_color: Hsla,
    direction: Direction,
) -> Option<AnyElement> {
    avatar
        .map(|avatar| {
            div()
                .size(px(24.))
                .flex_none()
                .rounded_full()
                .overflow_hidden()
                .text_color(leading_color)
                .child(icon_slot(avatar, 24.))
                .into_any_element()
        })
        .or_else(|| {
            leading.map(|leading| {
                div()
                    .flex_none()
                    .text_color(leading_color)
                    .child(icon_slot(leading, 18.))
                    .into_any_element()
            })
        })
        .or_else(|| {
            (filter_mark > 0.).then(|| {
                div()
                    .w((px(18.) + spacing) * filter_mark)
                    .flex_none()
                    .overflow_hidden()
                    .opacity(filter_mark)
                    .text_color(leading_color)
                    .when(direction == Direction::Rtl, |slot| {
                        slot.flex().justify_end()
                    })
                    .child(icon_slot(
                        Icon::new(IconName::Check).into_any_element(),
                        18.,
                    ))
                    .into_any_element()
            })
        })
}

#[allow(clippy::too_many_arguments)]
fn build_primary_content(
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    label: AnyElement,
    padding: ButtonPadding,
    spacing: Pixels,
    filter_mark: f32,
    split: bool,
    direction: Direction,
    visual_height: &Rc<Cell<Pixels>>,
) -> AnyElement {
    let content = div()
        .relative()
        .flex()
        .items_center()
        .min_w_0()
        .text_size(px(14.))
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
        .pt(padding.top)
        .pb(padding.bottom);
    let mut content = logical_edges(
        content,
        padding.start,
        if split { spacing } else { padding.end },
        direction,
    )
    .children(leading)
    .when(filter_mark == 0., |row| row.gap(spacing))
    .child(label)
    .children(trailing.map(|trailing| {
        if filter_mark > 0. {
            logical_edges(div(), spacing, px(0.), direction)
                .child(trailing)
                .into_any_element()
        } else {
            trailing
        }
    }));
    if split {
        content = content.child(measure_height(visual_height.clone()));
    }
    content.into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_split_chip(
    id: ElementId,
    row: Stateful<Div>,
    primary: ActionControl,
    remove_label: SharedString,
    on_remove: Activate,
    remove_content: Option<AnyElement>,
    disabled: bool,
    dragged: bool,
    selected: bool,
    direction: Direction,
    padding: ButtonPadding,
    explicit_padding: Option<ButtonPadding>,
    style: ActionStyle,
    layout: ActionLayout,
    palette: ChipPalette,
    elevations: ButtonElevations,
    dragged_elevation: Option<Elevation>,
    visual_height: Rc<Cell<Pixels>>,
    tracker: Option<InteractionTracker>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let primary_focus = primary.focus_handle(window, cx);
    let key_remove = on_remove.clone();
    let remove = ActionControl {
        id: ElementId::from((id.clone(), "remove")),
        label: remove_label,
        focus: None,
        disabled,
        action: Action::Invoke(Some(on_remove)),
        style: split_action_style(style, 32.),
        content: logical_edges(
            div().relative().pt(padding.top).pb(padding.bottom),
            px(0.),
            explicit_padding.map_or(px(0.), |padding| padding.end),
            direction,
        )
        .child(remove_content.unwrap_or_else(|| {
            icon_slot(Icon::new(IconName::Close).into_any_element(), 18.).into_any_element()
        }))
        .child(measure_height(visual_height.clone()))
        .into_any_element(),
    };
    let primary_state = primary.interaction_snapshot(dragged, window, cx);
    let remove_state = remove.interaction_snapshot(dragged, window, cx);
    let mut interaction = primary_state;
    interaction.hovered |= remove_state.hovered;
    interaction.pressed |= remove_state.pressed;
    interaction.focus_visible |= remove_state.focus_visible;
    let surface = SplitSurface {
        id,
        style,
        layout,
        direction,
        selected,
        container: palette.container,
        border: ButtonBorder {
            width: palette.border_width,
            color: palette.border_color,
        },
        elevations,
        dragged_elevation,
        visual_height,
    }
    .render(interaction, window, cx);
    let primary_layout = ActionLayout {
        segment: Some(SegmentShape {
            leading: true,
            trailing: false,
            outer: style.corner,
            inner: CornerRole::None,
            pressed: CornerRole::None,
        }),
        ..layout
    };
    let remove_layout = ActionLayout {
        segment: Some(SegmentShape {
            leading: false,
            trailing: true,
            outer: style.corner,
            inner: CornerRole::None,
            pressed: CornerRole::None,
        }),
        presentation_selected: Some(selected),
        appearance: ButtonStyle {
            shapes: layout.appearance.shapes,
            colors: action_colors(palette.trailing, gpui::transparent_black()),
            ..Default::default()
        },
        ..layout
    };
    row.when(!disabled, |row| {
        let on_remove =
            ThemeScope::bind_action(move |window, cx| key_remove(window, cx), window, cx);
        row.on_key_down(move |event, window, cx| {
            if !event.keystroke.modifiers.modified()
                && matches!(event.keystroke.key.as_str(), "backspace" | "delete")
                && primary_focus.is_focused(window)
            {
                cx.stop_propagation();
                on_remove(window, cx);
            }
        })
    })
    .child(surface)
    .child(primary.render_with_tracker(1., primary_layout, tracker, window, cx))
    .child(remove.render_with_tracker(1., remove_layout, None, window, cx))
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selectable = matches!(self.variant, ChipVariant::Filter | ChipVariant::Input);
        let selected = selectable && self.selected;
        let tonal = selectable && self.tonal;
        let elevated = self.elevated && self.variant != ChipVariant::Input;
        let split = selectable && self.remove.is_some();
        let (direction, density) = {
            let theme = current_material_theme(window, cx);
            (theme.layout.direction, theme.layout.density)
        };
        let palette = resolve_palette(
            &self.id,
            self.variant,
            selectable,
            selected,
            tonal,
            self.tone,
            elevated,
            self.disabled,
            self.colors,
            self.border,
            window,
            cx,
        );
        let style = build_chip_style(
            self.variant,
            selectable,
            selected,
            tonal,
            elevated,
            self.shapes,
            palette.foreground,
        );
        let dragged_elevation = self.elevations.dragged.or(Some(Elevation::Level4));
        let layout = build_chip_layout(
            split,
            self.tab_stop,
            self.dragged,
            self.shapes,
            self.elevations,
            dragged_elevation,
            palette,
        );

        let avatar = (self.variant == ChipVariant::Input)
            .then_some(self.avatar)
            .flatten();
        let has_avatar = avatar.is_some();
        let filter_mark = if self.variant == ChipVariant::Filter && self.leading.is_none() {
            palette.selection
        } else {
            0.
        };
        let leading = build_leading_slot(
            avatar,
            self.leading,
            filter_mark,
            self.spacing,
            palette.leading,
            direction,
        );
        let trailing = self
            .trailing
            .filter(|_| self.variant != ChipVariant::Suggestion)
            .map(|content| {
                div()
                    .flex_none()
                    .text_color(palette.trailing)
                    .text_size(px(18.))
                    .child(content)
                    .into_any_element()
            });
        let padding = self.padding.unwrap_or_else(|| ButtonPadding {
            start: px(if has_avatar {
                4.
            } else if filter_mark > 0. {
                16. - 8. * filter_mark
            } else if leading.is_some() {
                8.
            } else {
                16.
            }),
            end: if split {
                self.spacing
            } else if trailing.is_some() {
                px(8.)
            } else {
                px(16.)
            },
            ..Default::default()
        });
        let label = self.content.unwrap_or_else(|| {
            crate::text::single_line(
                self.label.clone(),
                TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                1.,
                palette.label,
                (None, 1., 1.),
                window,
                cx,
            )
            .into_any_element()
        });
        let row = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .min_w_0()
            .items_center()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse);
        let (trailing, remove_content) = if split {
            (None, trailing)
        } else {
            (trailing, None)
        };
        let visual_height = Rc::new(Cell::new(px(density.visual_size(style.height))));
        let content = build_primary_content(
            leading,
            trailing,
            label,
            padding,
            self.spacing,
            filter_mark,
            split,
            direction,
            &visual_height,
        );
        let primary = ActionControl {
            id: ElementId::from((self.id.clone(), "activate")),
            label: self.label,
            focus: self.focus,
            disabled: self.disabled,
            action: if selectable {
                Action::Toggle {
                    selected,
                    on_change: self.on_change,
                }
            } else {
                Action::Invoke(self.on_activate)
            },
            style: if split {
                split_action_style(style, 0.)
            } else {
                style
            },
            content,
        };
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        if let Some((label, on_remove)) = self.remove.filter(|_| selectable) {
            render_split_chip(
                self.id,
                row,
                primary,
                label,
                on_remove,
                remove_content,
                self.disabled,
                self.dragged,
                selected,
                direction,
                padding,
                self.padding,
                style,
                layout,
                palette,
                self.elevations.button(),
                dragged_elevation,
                visual_height,
                tracker,
                window,
                cx,
            )
        } else {
            row.child(primary.render_with_tracker(1., layout, tracker, window, cx))
        }
    }
}

fn logical_edges(element: Div, start: Pixels, end: Pixels, direction: Direction) -> Div {
    if direction == Direction::Rtl {
        element.pl(end).pr(start)
    } else {
        element.pl(start).pr(end)
    }
}

fn action_colors(label: Hsla, container: Hsla) -> ButtonColors {
    ButtonColors {
        container: Some(container),
        content: Some(label),
        disabled_container: Some(container),
        disabled_content: Some(label),
        checked_container: Some(container),
        checked_content: Some(label),
    }
}

fn split_action_style(style: ActionStyle, min_width: f32) -> ActionStyle {
    ActionStyle {
        min_width,
        elevation: Elevation::Level0,
        hover_elevation: Elevation::Level0,
        ..style
    }
}

fn measure_height(height: Rc<Cell<Pixels>>) -> gpui::Canvas<()> {
    canvas(
        move |bounds, _, _| height.set(height.get().max(bounds.size.height)),
        |_, (), _, _| {},
    )
    .absolute()
    .inset_0()
}

/// A split chip shares one container around its two independent actions.
struct SplitSurface {
    id: ElementId,
    style: ActionStyle,
    layout: ActionLayout,
    direction: Direction,
    selected: bool,
    container: Hsla,
    border: ButtonBorder,
    elevations: ButtonElevations,
    dragged_elevation: Option<Elevation>,
    visual_height: Rc<Cell<Pixels>>,
}

impl SplitSurface {
    fn render(
        self,
        state: InteractionState,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let theme = current_material_theme(window, cx);
        let shadow = color(theme.colors.color(ColorRole::Shadow));
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion());
        let spatial = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
        let pressed = animate(
            (self.id.clone(), "chip-press"),
            f32::from(state.pressed),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let selected = if self.style.selected_corner.is_some() {
            animate(
                (self.id.clone(), "chip-selection"),
                f32::from(self.selected),
                spatial,
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let default_elevation = if state.disabled {
            Elevation::Level0
        } else if state.hovered && !state.pressed {
            self.style.hover_elevation
        } else {
            self.style.elevation
        };
        let elevation = if state.dragged {
            self.dragged_elevation
        } else {
            self.elevations.for_state(state)
        }
        .unwrap_or(default_elevation);
        let elevation = animate(
            (self.id.clone(), "chip-elevation"),
            elevation.dp(),
            effects,
            window,
            cx,
        )
        .max(0.);
        let outline = animate(
            (self.id.clone(), "chip-outline"),
            f32::from(self.border.width),
            effects,
            window,
            cx,
        )
        .max(0.);
        canvas(
            |_, _, _| (),
            move |mut bounds, (), window, _| {
                let height = self.visual_height.get().min(bounds.size.height);
                bounds.origin += point(px(0.), (bounds.size.height - height) / 2.);
                bounds.size.height = height;
                let shape =
                    interpolated_corners(bounds, self.style, selected, pressed, 1., self.layout);
                let [top_left, top_right, bottom_right, bottom_left] = shape
                    .radii(
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                        self.direction,
                    )
                    .map(px);
                let radii = Corners {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                };
                if let Some(shadows) = elevation_shadow_pair(elevation, 1., shadow) {
                    window.paint_drop_shadows(bounds, radii, &shadows);
                }
                if self.container.a > 0. || outline > 0. {
                    window.paint_quad(quad(
                        bounds,
                        radii,
                        self.container,
                        px(outline),
                        if outline > 0. {
                            self.border.color
                        } else {
                            gpui::transparent_black()
                        },
                        BorderStyle::Solid,
                    ));
                }
            },
        )
        .absolute()
        .inset_0()
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
