use gpui::{Hsla, Pixels, px};
use gpui_material_foundation::InteractionState;
use gpui_material_theme::{ColorRole, CornerShape, Elevation};

use crate::SemanticTone;

/// Alignment of the slots within one row.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListItemAlignment {
    /// Adapts automatically based on line count (centered for 1–2 lines, top-aligned for 3+ lines).
    #[default]
    Auto,
    /// Aligns slots to the top edge of the row.
    Start,
    /// Vertically centers slots within the row.
    Center,
    /// Aligns slots to the bottom edge of the row.
    End,
}

/// Logical padding in pixels. Using the builder replaces the adaptive defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListItemPadding {
    /// Leading-edge padding (left in LTR, right in RTL). Defaults to `16 px`.
    pub start: Pixels,
    /// Trailing-edge padding (right in LTR, left in RTL). Defaults to `16 px`.
    pub end: Pixels,
    /// Top padding in logical pixels. Defaults to `10 px`.
    pub top: Pixels,
    /// Bottom padding in logical pixels. Defaults to `10 px`.
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
    /// Corner shape while resting and unselected.
    pub resting: Option<CornerShape>,
    /// Corner shape while selected.
    pub selected: Option<CornerShape>,
    /// Corner shape while pressed.
    pub pressed: Option<CornerShape>,
    /// Corner shape while keyboard focus is visible.
    pub focused: Option<CornerShape>,
    /// Corner shape while hovered.
    pub hovered: Option<CornerShape>,
    /// Corner shape while dragged.
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
    /// Row container background color.
    pub container: Option<Hsla>,
    /// Primary headline text color.
    pub content: Option<Hsla>,
    /// Leading slot foreground color.
    pub leading: Option<Hsla>,
    /// Trailing slot foreground color.
    pub trailing: Option<Hsla>,
    /// Overline text color.
    pub overline: Option<Hsla>,
    /// Supporting text color.
    pub supporting: Option<Hsla>,
}

/// Per-row palette. Omitted colors use the current window's theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListItemColors {
    /// Colors in the default enabled, unselected state.
    pub normal: ListItemStateColors,
    /// Colors when the row is disabled.
    pub disabled: ListItemStateColors,
    /// Colors when the row is selected.
    pub selected: ListItemStateColors,
    /// Colors when the row is dragged.
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
        semantic_tone: SemanticTone,
        tone: impl Fn(ColorRole) -> Hsla,
    ) -> RowColors {
        let normal_container = self
            .normal
            .container
            .unwrap_or_else(|| tone(ColorRole::Surface));
        let selected_container = self.selected.container.unwrap_or_else(|| {
            tone(
                semantic_tone
                    .container_role()
                    .unwrap_or(ColorRole::SecondaryContainer),
            )
        });
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
                semantic_tone
                    .on_container_role()
                    .unwrap_or(ColorRole::OnSecondaryContainer)
            } else {
                semantic_tone.role().unwrap_or(ColorRole::OnSurface)
            });
            (self.dragged, content, content, Some(content), background)
        } else {
            let background = crate::motion::mix(normal_container, selected_container, selection);
            if selected {
                let tint = tone(
                    semantic_tone
                        .on_container_role()
                        .unwrap_or(ColorRole::OnSecondaryContainer),
                );
                let side = semantic_tone.on_container_role().map(&tone);
                (self.selected, tint, tint, side, background)
            } else {
                let accent = semantic_tone.role().map(&tone);
                (
                    self.normal,
                    accent.unwrap_or_else(|| tone(ColorRole::OnSurface)),
                    tone(ColorRole::OnSurfaceVariant),
                    accent,
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
    /// Elevation while resting.
    pub resting: Option<Elevation>,
    /// Elevation while hovered.
    pub hovered: Option<Elevation>,
    /// Elevation while keyboard focus is visible.
    pub focused: Option<Elevation>,
    /// Elevation while pressed.
    pub pressed: Option<Elevation>,
    /// Elevation while dragged.
    pub dragged: Option<Elevation>,
    /// Elevation while disabled.
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
            let actual = colors.resolve(
                selected,
                f32::from(selected),
                state,
                SemanticTone::Default,
                |_| gpui::transparent_black(),
            );
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

    #[test]
    fn semantic_tone_tints_enabled_and_selected_roles_without_overriding_disabled_or_explicit_colors()
     {
        let role_color = |role: ColorRole| -> Hsla {
            match role {
                ColorRole::Error => gpui::rgb(0xB3261E).into(),
                ColorRole::ErrorContainer => gpui::rgb(0xF9DEDC).into(),
                ColorRole::OnErrorContainer => gpui::rgb(0x410E0B).into(),
                ColorRole::OnSurface => gpui::rgb(0x1C1B1F).into(),
                ColorRole::OnSurfaceVariant => gpui::rgb(0x49454F).into(),
                _ => gpui::rgb(0xFFFFFF).into(),
            }
        };
        let state = InteractionState::default();
        let normal =
            ListItemColors::default().resolve(false, 0., state, SemanticTone::Danger, role_color);
        assert_eq!(normal.content, role_color(ColorRole::Error));
        assert_eq!(normal.leading, Some(role_color(ColorRole::Error)));
        assert_eq!(normal.trailing, Some(role_color(ColorRole::Error)));

        let selected =
            ListItemColors::default().resolve(true, 1., state, SemanticTone::Danger, role_color);
        assert_eq!(selected.content, role_color(ColorRole::OnErrorContainer));
        assert_eq!(
            selected.leading,
            Some(role_color(ColorRole::OnErrorContainer))
        );
        assert_eq!(
            selected.trailing,
            Some(role_color(ColorRole::OnErrorContainer))
        );

        let mut disabled_state = InteractionState::default();
        disabled_state.disabled = true;
        let disabled = ListItemColors::default().resolve(
            false,
            0.,
            disabled_state,
            SemanticTone::Danger,
            role_color,
        );
        assert_eq!(
            disabled.content,
            role_color(ColorRole::OnSurface).opacity(0.38)
        );

        let custom: Hsla = gpui::rgb(0x123456).into();
        let overridden = ListItemColors {
            normal: ListItemStateColors {
                content: Some(custom),
                leading: Some(custom),
                ..Default::default()
            },
            ..Default::default()
        }
        .resolve(false, 0., state, SemanticTone::Danger, role_color);
        assert_eq!(overridden.content, custom);
        assert_eq!(overridden.leading, Some(custom));
        assert_eq!(overridden.trailing, Some(role_color(ColorRole::Error)));
    }
}
