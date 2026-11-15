use gpui::{Hsla, Pixels, px};
use gpui_material_foundation::InteractionState;
use gpui_material_theme::{CornerShape, Elevation};

/// Button corner overrides in logical top-start, top-end, bottom-end,
/// bottom-start order. Unspecified states keep their size and shape presets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonShapes {
    pub resting: Option<CornerShape>,
    pub pressed: Option<CornerShape>,
    pub checked: Option<CornerShape>,
}

/// Per-instance pigments. Each omitted field keeps the variant's color in
/// that state. Checked colors apply to enabled toggles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub disabled_container: Option<Hsla>,
    pub disabled_content: Option<Hsla>,
    pub checked_container: Option<Hsla>,
    pub checked_content: Option<Hsla>,
}

impl ButtonColors {
    pub(crate) fn container(self, disabled: bool, checked: bool) -> Option<Hsla> {
        match (disabled, checked) {
            (true, _) => self.disabled_container,
            (false, true) => self.checked_container,
            (false, false) => self.container,
        }
    }

    pub(crate) fn content(self, disabled: bool, checked: bool) -> Option<Hsla> {
        match (disabled, checked) {
            (true, _) => self.disabled_content,
            (false, true) => self.checked_content,
            (false, false) => self.content,
        }
    }
}

/// A stroke for any button variant. A zero width removes its outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonBorder {
    pub width: Pixels,
    pub color: Hsla,
}

/// Insets around the label or custom content, including leading and trailing icons.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonPadding {
    pub top: Pixels,
    pub end: Pixels,
    pub bottom: Pixels,
    pub start: Pixels,
}

/// Optional Material elevation levels. Disabled takes precedence, followed by
/// pressed, keyboard focus, hover and rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonElevations {
    pub resting: Option<Elevation>,
    pub pressed: Option<Elevation>,
    pub focused: Option<Elevation>,
    pub hovered: Option<Elevation>,
    pub disabled: Option<Elevation>,
}

impl ButtonElevations {
    pub(crate) fn for_state(self, state: InteractionState) -> Option<Elevation> {
        if state.disabled {
            self.disabled
        } else if state.pressed {
            self.pressed
        } else if state.focus_visible {
            self.focused
        } else if state.hovered {
            self.hovered
        } else {
            self.resting
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ButtonStyle {
    pub(crate) colors: ButtonColors,
    pub(crate) shapes: ButtonShapes,
    pub(crate) border: Option<ButtonBorder>,
    pub(crate) elevations: ButtonElevations,
}

pub(crate) fn finite_length(length: Pixels) -> Pixels {
    if f32::from(length).is_finite() {
        length.max(px(0.))
    } else {
        px(0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_overrides_are_independent_and_follow_state_priority() {
        use Elevation::*;
        let levels = ButtonElevations {
            resting: Some(Level1),
            hovered: Some(Level2),
            focused: Some(Level3),
            pressed: Some(Level4),
            disabled: Some(Level5),
        };
        let mut state = InteractionState::default();
        assert_eq!(levels.for_state(state), Some(Level1));
        state.hovered = true;
        assert_eq!(levels.for_state(state), Some(Level2));
        state.focus_visible = true;
        assert_eq!(levels.for_state(state), Some(Level3));
        state.pressed = true;
        assert_eq!(levels.for_state(state), Some(Level4));
        state.disabled = true;
        assert_eq!(levels.for_state(state), Some(Level5));
        let resting_only = ButtonElevations {
            resting: Some(Level2),
            ..Default::default()
        };
        assert_eq!(resting_only.for_state(state), None);
    }
}
