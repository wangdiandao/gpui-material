use gpui::{Hsla, Pixels, Rgba, px};
use gpui_material_foundation::InteractionState;
use gpui_material_theme::{CornerShape, Elevation};

/// Material Expressive sizes, from extra small to extra large.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlSize {
    /// Extra-small control tier (32 dp height).
    ExtraSmall,
    /// Small control tier (40 dp height, default).
    #[default]
    Small,
    /// Medium control tier (56 dp height).
    Medium,
    /// Large control tier (96 dp height).
    Large,
    /// Extra-large control tier (136 dp height).
    ExtraLarge,
}

/// Control corner overrides in logical top-start, top-end, bottom-end,
/// bottom-start order. Unspecified states keep their size and shape presets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ControlShapes {
    /// Corner shape while resting and unselected.
    pub resting: Option<CornerShape>,
    /// Corner shape while pressed.
    pub pressed: Option<CornerShape>,
    /// Corner shape while checked/selected on toggle controls and chips.
    pub checked: Option<CornerShape>,
}

/// Per-instance button pigments. Each omitted field keeps the variant's color in
/// that state. Checked colors apply to enabled toggles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonColors {
    /// Enabled container background color.
    pub container: Option<Hsla>,
    /// Enabled foreground label and icon color.
    pub content: Option<Hsla>,
    /// Disabled container background color.
    pub disabled_container: Option<Hsla>,
    /// Disabled foreground label and icon color.
    pub disabled_content: Option<Hsla>,
    /// Enabled container background color when a toggle button is selected.
    pub checked_container: Option<Hsla>,
    /// Enabled foreground label and icon color when a toggle button is selected.
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

/// A stroke for any control variant. A zero width removes its outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlBorder {
    /// Stroke thickness in logical pixels.
    pub width: Pixels,
    /// Stroke color.
    pub color: Hsla,
}

/// Optional Material elevation levels. Disabled takes precedence, followed by
/// dragged, pressed, keyboard focus, hover, and rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlElevations {
    /// Elevation level while resting.
    pub resting: Option<Elevation>,
    /// Elevation level while pressed.
    pub pressed: Option<Elevation>,
    /// Elevation level while keyboard focus is visible.
    pub focused: Option<Elevation>,
    /// Elevation level while hovered.
    pub hovered: Option<Elevation>,
    /// Elevation level while dragged.
    pub dragged: Option<Elevation>,
    /// Elevation level while disabled.
    pub disabled: Option<Elevation>,
}

impl ControlElevations {
    pub(crate) fn for_state(self, state: InteractionState) -> Option<Elevation> {
        if state.disabled {
            self.disabled
        } else if state.dragged {
            self.dragged
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
    pub(crate) shapes: ControlShapes,
    pub(crate) border: Option<ControlBorder>,
    pub(crate) elevations: ControlElevations,
}

pub(crate) fn finite_length(length: Pixels) -> Pixels {
    if f32::from(length).is_finite() {
        length.max(px(0.))
    } else {
        px(0.)
    }
}

pub(crate) fn mix_color(from: Hsla, to: Hsla, fraction: f32) -> Hsla {
    if fraction <= 0. {
        return from;
    }
    if fraction >= 1. {
        return to;
    }
    let from = Rgba::from(from);
    let to = Rgba::from(to);
    let mix = |a, b| a + (b - a) * fraction;
    Rgba {
        r: mix(from.r, to.r),
        g: mix(from.g, to.g),
        b: mix(from.b, to.b),
        a: mix(from.a, to.a),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_overrides_are_independent_and_follow_state_priority() {
        use Elevation::*;
        let levels = ControlElevations {
            resting: Some(Level1),
            hovered: Some(Level2),
            focused: Some(Level3),
            pressed: Some(Level4),
            dragged: Some(Level5),
            disabled: Some(Level0),
        };
        let mut state = InteractionState::default();
        assert_eq!(levels.for_state(state), Some(Level1));
        state.hovered = true;
        assert_eq!(levels.for_state(state), Some(Level2));
        state.focus_visible = true;
        assert_eq!(levels.for_state(state), Some(Level3));
        state.pressed = true;
        assert_eq!(levels.for_state(state), Some(Level4));
        state.dragged = true;
        assert_eq!(levels.for_state(state), Some(Level5));
        state.disabled = true;
        assert_eq!(levels.for_state(state), Some(Level0));
    }
}
