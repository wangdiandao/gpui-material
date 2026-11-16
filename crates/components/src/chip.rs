use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, BorderStyle, Corners, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window, canvas, div, point, prelude::FluentBuilder, px, quad,
};
use gpui_material_foundation::{
    InteractionSource, InteractionState, animate, color, elevation_shadows, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, Change, DisabledStyle, icon_slot,
    interpolated_corners,
};
use crate::button_style::{ButtonStyle, finite_length};
use crate::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes, Icon, IconName,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChipVariant {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// Per-slot pigments for one chip state. Omitted fields retain the defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipStateColors {
    pub container: Option<Hsla>,
    pub label: Option<Hsla>,
    pub leading: Option<Hsla>,
    pub trailing: Option<Hsla>,
    pub border: Option<Hsla>,
}

/// Enabled, selected, and disabled chip pigments. Disabled-selected fields fall
/// back to disabled overrides. Selection applies to Filter and Input chips.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipColors {
    pub enabled: ChipStateColors,
    pub selected: ChipStateColors,
    pub disabled: ChipStateColors,
    pub disabled_selected: ChipStateColors,
}

impl ChipColors {
    fn for_state(self, disabled: bool, selected: bool) -> ChipStateColors {
        match (disabled, selected) {
            (false, false) => self.enabled,
            (false, true) => self.selected,
            (true, false) => self.disabled,
            (true, true) => ChipStateColors {
                container: self.disabled_selected.container.or(self.disabled.container),
                label: self.disabled_selected.label.or(self.disabled.label),
                leading: self.disabled_selected.leading.or(self.disabled.leading),
                trailing: self.disabled_selected.trailing.or(self.disabled.trailing),
                border: self.disabled_selected.border.or(self.disabled.border),
            },
        }
    }
}

/// Optional shadows for each state. Disabled and dragged take precedence over
/// press, keyboard focus, hover, and rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChipElevations {
    pub resting: Option<Elevation>,
    pub pressed: Option<Elevation>,
    pub focused: Option<Elevation>,
    pub hovered: Option<Elevation>,
    pub dragged: Option<Elevation>,
    pub disabled: Option<Elevation>,
}

impl ChipElevations {
    fn button(self) -> ButtonElevations {
        ButtonElevations {
            resting: self.resting,
            pressed: self.pressed,
            focused: self.focused,
            hovered: self.hovered,
            disabled: self.disabled,
        }
    }
}

/// A Material chip. Filter and Input chips request controlled selection changes.
/// Filter and Input chips can expose a separate removal action with its own focus.
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    variant: ChipVariant,
    selected: bool,
    elevated: bool,
    disabled: bool,
    tonal: bool,
    dragged: bool,
    content: Option<AnyElement>,
    leading: Option<AnyElement>,
    avatar: Option<AnyElement>,
    trailing: Option<AnyElement>,
    shapes: ButtonShapes,
    colors: ChipColors,
    border: Option<ButtonBorder>,
    elevations: ChipElevations,
    spacing: Pixels,
    padding: Option<ButtonPadding>,
    interaction_source: Option<Entity<InteractionSource>>,
    on_activate: Option<Activate>,
    on_change: Option<Change>,
    remove: Option<(SharedString, Activate)>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ChipVariant::default(),
            selected: false,
            elevated: false,
            disabled: false,
            tonal: false,
            dragged: false,
            content: None,
            leading: None,
            avatar: None,
            trailing: None,
            shapes: ButtonShapes::default(),
            colors: ChipColors::default(),
            border: None,
            elevations: ChipElevations::default(),
            spacing: px(8.),
            padding: None,
            interaction_source: None,
            on_activate: None,
            on_change: None,
            remove: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Selects the expressive Tonal defaults for Filter, elevated Filter, and Input.
    pub fn tonal(mut self, tonal: bool) -> Self {
        self.tonal = tonal;
        self
    }

    /// Supplies visual label content. The constructor's label names the action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }

    /// Sets the Input chip's circular avatar, taking precedence over its leading icon.
    pub fn avatar(mut self, avatar: impl IntoElement) -> Self {
        self.avatar = Some(avatar.into_any_element());
        self
    }

    /// Sets visual trailing content for Assist, Filter, or Input. With an
    /// independent removal action this is that button's visual content.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn shapes(mut self, shapes: ButtonShapes) -> Self {
        self.shapes = shapes;
        self
    }

    pub fn colors(mut self, colors: ChipColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn border(mut self, border: ButtonBorder) -> Self {
        self.border = Some(border);
        self
    }

    pub fn elevations(mut self, elevations: ChipElevations) -> Self {
        self.elevations = elevations;
        self
    }

    /// Reflects caller-owned drag state and cancels pending pointer activation.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = finite_length(spacing);
        self
    }

    pub fn content_padding(mut self, padding: ButtonPadding) -> Self {
        self.padding = Some(ButtonPadding {
            top: finite_length(padding.top),
            end: finite_length(padding.end),
            bottom: finite_length(padding.bottom),
            start: finite_length(padding.start),
        });
        self
    }

    /// Observes the primary chip action under the constructor's chip ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Adds an independent removal action to a Filter or Input chip.
    /// The optional restoration target is focused before calling the owner.
    /// The owner can pass the next chip or a stable control outside the group.
    pub fn on_remove(
        mut self,
        name: impl Into<SharedString>,
        restore: Option<FocusHandle>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.remove = Some((
            name.into(),
            Rc::new(move |window, cx| {
                if let Some(restore) = &restore {
                    restore.focus(window, cx);
                }
                handler(window, cx);
            }),
        ));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use ColorRole::*;
        let selectable = matches!(self.variant, ChipVariant::Filter | ChipVariant::Input);
        let selected = selectable && self.selected;
        let tonal = selectable && self.tonal;
        let elevated = self.elevated && self.variant != ChipVariant::Input;
        let split = selectable && self.remove.is_some();
        let theme = window_material_theme(window, cx).clone();
        let direction = theme.layout.direction;
        let theme_color = |role| color(theme.colors.color(role));
        let on_surface = theme_color(OnSurface);
        let colors = self.colors.for_state(self.disabled, selected);
        let foreground = if selected {
            OnSecondaryContainer
        } else if self.variant == ChipVariant::Assist {
            OnSurface
        } else {
            OnSurfaceVariant
        };
        let label_color = colors.label.unwrap_or_else(|| {
            if self.disabled {
                on_surface.opacity(0.38)
            } else {
                theme_color(foreground)
            }
        });
        let leading_color = colors.leading.unwrap_or_else(|| {
            if self.disabled {
                on_surface.opacity(0.38)
            } else {
                theme_color(match (self.variant, selected, tonal) {
                    (ChipVariant::Input, false, _) | (ChipVariant::Filter, false, true) => {
                        OnSurfaceVariant
                    }
                    (ChipVariant::Filter, true, _) => OnSecondaryContainer,
                    _ => Primary,
                })
            }
        });
        let trailing_color = colors.trailing.unwrap_or_else(|| {
            if self.disabled {
                on_surface.opacity(0.38)
            } else if selected {
                theme_color(OnSecondaryContainer)
            } else {
                theme_color(OnSurfaceVariant)
            }
        });
        let effects = theme.motion.spec(
            MotionKind::Effects,
            MotionSpeed::Fast,
            cx.reduce_motion() || self.disabled,
        );
        let selection = if selectable {
            animate(
                (self.id.clone(), "chip-tint"),
                f32::from(selected),
                effects,
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let idle = self.colors.enabled.container.unwrap_or_else(|| {
            if elevated {
                theme_color(SurfaceContainerLow)
            } else {
                theme_color(SecondaryContainer).opacity(0.)
            }
        });
        let checked = self
            .colors
            .selected
            .container
            .unwrap_or_else(|| theme_color(SecondaryContainer));
        let container = if self.disabled {
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
        let border_color = colors
            .border
            .or(self.border.map(|border| border.color))
            .unwrap_or_else(|| {
                if self.disabled {
                    on_surface.opacity(0.12)
                } else {
                    theme_color(OutlineVariant)
                }
            });
        let border_width = self
            .border
            .map_or(px(f32::from(!elevated && !selected)), |border| {
                finite_length(border.width)
            });
        let style = ActionStyle {
            background: None,
            background_color: None,
            foreground,
            disabled: DisabledStyle {
                foreground: OnSurface,
                background: None,
                outline: (OnSurface, 0.12),
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
            } else if selectable && self.shapes.checked.is_some() {
                Some(CornerRole::Small)
            } else {
                None
            },
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: 0.,
            elevation: if elevated {
                Elevation::Level1
            } else {
                Elevation::Level0
            },
            hover_elevation: if elevated {
                Elevation::Level2
            } else if selected && self.variant == ChipVariant::Filter {
                Elevation::Level1
            } else {
                Elevation::Level0
            },
        };
        let dragged_elevation = self.elevations.dragged.or(Some(Elevation::Level4));
        let layout = ActionLayout {
            tab_stop: true,
            dragged: self.dragged,
            dragged_elevation: (!split).then_some(dragged_elevation).flatten(),
            appearance: ButtonStyle {
                shapes: self.shapes,
                colors: action_colors(
                    label_color,
                    if split {
                        gpui::transparent_black()
                    } else {
                        container
                    },
                ),
                border: (!split).then_some(ButtonBorder {
                    width: border_width,
                    color: border_color,
                }),
                elevations: if split {
                    ButtonElevations::default()
                } else {
                    self.elevations.button()
                },
            },
            ..Default::default()
        };

        let avatar = (self.variant == ChipVariant::Input)
            .then_some(self.avatar)
            .flatten();
        let has_avatar = avatar.is_some();
        let filter_mark = if self.variant == ChipVariant::Filter && self.leading.is_none() {
            selection
        } else {
            0.
        };
        let leading = avatar
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
                self.leading.map(|leading| {
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
                        .w((px(18.) + self.spacing) * filter_mark)
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
            });
        let trailing = self
            .trailing
            .filter(|_| self.variant != ChipVariant::Suggestion)
            .map(|content| {
                div()
                    .flex_none()
                    .text_color(trailing_color)
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
                label_color,
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
            if split { self.spacing } else { padding.end },
            direction,
        )
        .children(leading)
        .when(filter_mark == 0., |row| row.gap(self.spacing))
        .child(label)
        .children(trailing.map(|trailing| {
            if filter_mark > 0. {
                logical_edges(div(), self.spacing, px(0.), direction)
                    .child(trailing)
                    .into_any_element()
            } else {
                trailing
            }
        }));
        let visual_height = Rc::new(Cell::new(px(theme
            .layout
            .density
            .visual_size(style.height))));
        if split {
            content = content.child(measure_height(visual_height.clone()));
        }
        let primary = ActionControl {
            id: ElementId::from((self.id.clone(), "activate")),
            label: self.label,
            focus: None,
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
            content: content.into_any_element(),
        };
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        if let Some((label, on_remove)) = self.remove.filter(|_| selectable) {
            let remove = ActionControl {
                id: ElementId::from((self.id.clone(), "remove")),
                label,
                focus: None,
                disabled: self.disabled,
                action: Action::Invoke(Some(on_remove)),
                style: split_action_style(style, 32.),
                content: logical_edges(
                    div().relative().pt(padding.top).pb(padding.bottom),
                    px(0.),
                    self.padding.map_or(px(0.), |padding| padding.end),
                    direction,
                )
                .child(remove_content.unwrap_or_else(|| {
                    icon_slot(Icon::new(IconName::Close).into_any_element(), 18.).into_any_element()
                }))
                .child(measure_height(visual_height.clone()))
                .into_any_element(),
            };
            let primary_state = primary.interaction_snapshot(self.dragged, window, cx);
            let remove_state = remove.interaction_snapshot(self.dragged, window, cx);
            let mut interaction = primary_state;
            interaction.hovered |= remove_state.hovered;
            interaction.pressed |= remove_state.pressed;
            interaction.focus_visible |= remove_state.focus_visible;
            let surface = SplitSurface {
                id: self.id,
                style,
                layout,
                direction,
                selected,
                container,
                border: ButtonBorder {
                    width: border_width,
                    color: border_color,
                },
                elevations: self.elevations.button(),
                dragged_elevation,
                visual_height,
            }
            .render(interaction, window, cx);
            row.child(surface)
                .child(primary.render_with_tracker(1., layout, tracker, window, cx))
                .child(remove.render_with_tracker(
                    1.,
                    ActionLayout {
                        appearance: ButtonStyle {
                            colors: action_colors(trailing_color, gpui::transparent_black()),
                            ..Default::default()
                        },
                        ..layout
                    },
                    None,
                    window,
                    cx,
                ))
        } else {
            row.child(primary.render_with_tracker(1., layout, tracker, window, cx))
        }
    }
}

fn logical_edges(
    element: gpui::Div,
    start: Pixels,
    end: Pixels,
    direction: Direction,
) -> gpui::Div {
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
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
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
                if elevation > 0. {
                    window.paint_drop_shadows(
                        bounds,
                        radii,
                        &elevation_shadows(elevation, 1., shadow),
                    );
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
