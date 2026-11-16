use gpui::{Hsla, Pixels, px};
use gpui_material_foundation::InteractionState;
use gpui_material_theme::{ColorRole, CornerShape, Elevation};

/// Alignment of the slots within one row.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListItemAlignment {
    #[default]
    Auto,
    Start,
    Center,
    End,
}

/// Logical padding in pixels. Using the builder replaces the adaptive defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListItemPadding {
    pub start: Pixels,
    pub end: Pixels,
    pub top: Pixels,
    pub bottom: Pixels,
}

impl Default for ListItemPadding {
    fn default() -> Self {
        Self {
            start: px(16.),
            end: px(16.),
            top: px(10.),
            bottom: px(10.),
        }
    }
}

/// Logical shapes for one row. An omitted shape uses the resting or selected
/// shape, including the list's grouped corners.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListItemShapes {
    pub resting: Option<CornerShape>,
    pub selected: Option<CornerShape>,
    pub pressed: Option<CornerShape>,
    pub focused: Option<CornerShape>,
    pub hovered: Option<CornerShape>,
    pub dragged: Option<CornerShape>,
}

impl ListItemShapes {
    pub(super) fn resolve(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        selected: bool,
        state: InteractionState,
    ) -> CornerShape {
        let base = if selected {
            self.selected.unwrap_or(selected_shape)
        } else {
            self.resting.unwrap_or(resting)
        };
        let active = if state.disabled {
            None
        } else if state.dragged {
            self.dragged
        } else if state.pressed {
            self.pressed
        } else if state.focus_visible {
            self.focused
        } else if state.hovered {
            self.hovered
        } else {
            None
        };
        active.unwrap_or(base)
    }
}

/// Independent colors inherited by the container and each slot in one state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListItemStateColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub leading: Option<Hsla>,
    pub trailing: Option<Hsla>,
    pub overline: Option<Hsla>,
    pub supporting: Option<Hsla>,
}

/// Per-row palette. Omitted colors use the current window's theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListItemColors {
    pub normal: ListItemStateColors,
    pub disabled: ListItemStateColors,
    pub selected: ListItemStateColors,
    pub dragged: ListItemStateColors,
}

pub(super) struct RowColors {
    pub container: Hsla,
    pub content: Hsla,
    pub leading: Option<Hsla>,
    pub trailing: Option<Hsla>,
    pub overline: Hsla,
    pub supporting: Hsla,
}

impl ListItemColors {
    pub(super) fn resolve(
        self,
        selected: bool,
        selection: f32,
        state: InteractionState,
        tone: impl Fn(ColorRole) -> Hsla,
    ) -> RowColors {
        let normal_container = self
            .normal
            .container
            .unwrap_or_else(|| tone(ColorRole::Surface));
        let selected_container = self
            .selected
            .container
            .unwrap_or_else(|| tone(ColorRole::SecondaryContainer));
        let (colors, content, secondary, side, background) = if state.disabled {
            let tint = tone(ColorRole::OnSurface).opacity(0.38);
            let background = self
                .disabled
                .container
                .unwrap_or_else(|| crate::motion::mix(normal_container, tint, selection));
            (self.disabled, tint, tint, Some(tint), background)
        } else if state.dragged {
            let background = self.dragged.container.unwrap_or_else(|| {
                if selected {
                    selected_container
                } else {
                    tone(ColorRole::SurfaceContainerHighest)
                }
            });
            let content = tone(if selected {
                ColorRole::OnSecondaryContainer
            } else {
                ColorRole::OnSurface
            });
            (self.dragged, content, content, Some(content), background)
        } else {
            let background = crate::motion::mix(normal_container, selected_container, selection);
            if selected {
                let tint = tone(ColorRole::OnSecondaryContainer);
                (self.selected, tint, tint, None, background)
            } else {
                (
                    self.normal,
                    tone(ColorRole::OnSurface),
                    tone(ColorRole::OnSurfaceVariant),
                    None,
                    background,
                )
            }
        };
        RowColors {
            container: background,
            content: colors.content.unwrap_or(content),
            leading: colors.leading.or(side),
            trailing: colors.trailing.or(side),
            overline: colors.overline.unwrap_or(secondary),
            supporting: colors.supporting.unwrap_or(secondary),
        }
    }
}

/// Per-state shadow levels. Rows default to Level0, or Level4 while dragged.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListItemElevations {
    pub resting: Option<Elevation>,
    pub hovered: Option<Elevation>,
    pub focused: Option<Elevation>,
    pub pressed: Option<Elevation>,
    pub dragged: Option<Elevation>,
    pub disabled: Option<Elevation>,
}

impl ListItemElevations {
    pub(super) fn resolve(self, state: InteractionState) -> Elevation {
        let resting = self.resting.unwrap_or(Elevation::Level0);
        if state.disabled {
            self.disabled.unwrap_or(resting)
        } else if state.dragged {
            self.dragged.unwrap_or(Elevation::Level4)
        } else if state.pressed {
            self.pressed.unwrap_or(resting)
        } else if state.focus_visible {
            self.focused.unwrap_or(resting)
        } else if state.hovered {
            self.hovered.unwrap_or(resting)
        } else {
            resting
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_material_theme::{CornerRole, Direction};

    use super::*;

    #[test]
    fn selected_and_active_shapes_are_logical_and_disabled_uses_selected_base() {
        let shape = CornerShape::Radii([4., 8., 12., 16.]);
        let selected = CornerRole::Large.shape();
        let styles = ListItemShapes {
            dragged: Some(shape),
            ..Default::default()
        };
        let mut state = InteractionState::default();
        state.dragged = true;
        state.hovered = true;
        assert_eq!(
            styles
                .resolve(CornerRole::None.shape(), selected, true, state)
                .radii(100., 60., Direction::Rtl),
            [8., 4., 16., 12.],
        );
        state.disabled = true;
        assert_eq!(
            styles.resolve(CornerRole::None.shape(), selected, true, state),
            selected,
        );
    }

    #[test]
    fn all_slot_colors_follow_the_state_and_dragged_elevation_yields_to_disabled() {
        let palette = |value| ListItemStateColors {
            container: Some(gpui::rgb(value).into()),
            content: Some(gpui::rgb(value + 1).into()),
            leading: Some(gpui::rgb(value + 2).into()),
            trailing: Some(gpui::rgb(value + 3).into()),
            overline: Some(gpui::rgb(value + 4).into()),
            supporting: Some(gpui::rgb(value + 5).into()),
        };
        let colors = ListItemColors {
            normal: palette(0x113300),
            selected: palette(0x224400),
            dragged: palette(0x335500),
            disabled: palette(0x446600),
        };
        let elevations = ListItemElevations {
            dragged: Some(Elevation::Level3),
            disabled: Some(Elevation::Level1),
            ..Default::default()
        };
        let mut state = InteractionState::default();
        for (selected, dragged, disabled, expected, level) in [
            (false, false, false, colors.normal, Elevation::Level0),
            (true, false, false, colors.selected, Elevation::Level0),
            (true, true, false, colors.dragged, Elevation::Level3),
            (true, true, true, colors.disabled, Elevation::Level1),
        ] {
            state.disabled = disabled;
            state.dragged = dragged;
            let actual = colors.resolve(selected, f32::from(selected), state, |_| {
                gpui::transparent_black()
            });
            let expected_container = expected.container.unwrap();
            let actual_container = actual.container;
            assert!(
                [
                    actual_container.h - expected_container.h,
                    actual_container.s - expected_container.s,
                    actual_container.l - expected_container.l,
                    actual_container.a - expected_container.a
                ]
                .into_iter()
                .all(|delta| delta.abs() < 1e-6)
            );
            assert_eq!(Some(actual.content), expected.content);
            assert_eq!(actual.leading, expected.leading);
            assert_eq!(actual.trailing, expected.trailing);
            assert_eq!(Some(actual.overline), expected.overline);
            assert_eq!(Some(actual.supporting), expected.supporting);
            assert_eq!(elevations.resolve(state), level);
        }
    }
}
