mod action_button;
mod appearance;
/// Internal composition hooks shared with the optional Material extras crate.
#[doc(hidden)]
pub mod composition;
mod icon_button;
mod slots;
mod toggle_button;
mod types;

pub use action_button::Button;
pub use icon_button::{IconButton, IconToggleButton};
pub use toggle_button::ToggleButton;
pub use types::{
    ButtonShape, ButtonVariant, IconButtonVariant, IconButtonWidth, ToggleButtonVariant,
};

// All four buttons expose the same shape, size, and focus contract.
macro_rules! common_builders {
    () => {
        /// Sets the Material size tier. Small is the default.
        pub fn size(mut self, size: crate::control_style::ControlSize) -> Self {
            self.common.size = size;
            self
        }

        /// Sets the resting shape. Round is the default.
        pub fn shape(mut self, shape: super::types::ButtonShape) -> Self {
            self.common.shape = shape;
            self
        }

        /// Applies a Material 3 semantic tone shortcut. Explicit `.colors` and `.border`
        /// overrides still take precedence.
        pub fn tone(mut self, tone: crate::SemanticTone) -> Self {
            self.common.tone = tone;
            self
        }

        /// Controls availability and keyboard traversal.
        pub fn disabled(mut self, disabled: bool) -> Self {
            self.common.disabled = disabled;
            self
        }

        /// Uses the caller's stable focus handle.
        pub fn track_focus(mut self, focus: &gpui::FocusHandle) -> Self {
            self.common.focus = Some(focus.clone());
            self
        }
    };
}
pub(super) use common_builders;

macro_rules! appearance_builders {
    () => {
        /// Observes this button's interaction presentation through a caller-owned source.
        /// The source does not handle input or change selection.
        pub fn interaction_source(
            mut self,
            source: &gpui::Entity<gpui_material_foundation::InteractionSource>,
        ) -> Self {
            self.common.interaction_source = Some(source.clone());
            self
        }

        /// Overrides state-specific corners. Unspecified states use the size and variant defaults.
        pub fn shapes(mut self, shapes: crate::control_style::ControlShapes) -> Self {
            self.common.appearance.shapes = shapes;
            self
        }

        /// Overrides container and content colors for individual states.
        pub fn colors(mut self, colors: crate::control_style::ButtonColors) -> Self {
            self.common.appearance.colors = colors;
            self
        }

        /// Supplies a border in place of the variant's outline.
        pub fn border(mut self, border: crate::control_style::ControlBorder) -> Self {
            self.common.appearance.border = Some(border);
            self
        }
    };
}
pub(super) use appearance_builders;

macro_rules! slot_builders {
    () => {
        /// Replaces the visible label with an element. The constructor's label
        /// remains the accessible name. Keep interactive children outside the button.
        pub fn content(mut self, content: impl gpui::IntoElement) -> Self {
            self.slots.content = Some(content.into_any_element());
            self
        }

        /// Sets logical insets around the label, content and icon slots.
        pub fn content_padding(mut self, padding: gpui_material_theme::LogicalEdges) -> Self {
            self.slots.padding = Some(padding);
            self
        }

        /// Overrides elevation levels for individual interaction states.
        pub fn elevations(mut self, elevations: crate::control_style::ControlElevations) -> Self {
            self.common.appearance.elevations = elevations;
            self
        }

        /// Supplies a decorative leading icon in the Material icon slot.
        pub fn leading_icon(mut self, icon: impl gpui::IntoElement) -> Self {
            self.slots.leading = Some(icon.into_any_element());
            self
        }

        /// Supplies a decorative trailing icon in the Material icon slot.
        pub fn trailing_icon(mut self, icon: impl gpui::IntoElement) -> Self {
            self.slots.trailing = Some(icon.into_any_element());
            self
        }
    };
}
pub(super) use slot_builders;

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn button_and_icon_button_variant_shorthands_set_expected_variants() {
        assert_eq!(
            Button::filled("b1", "Filled").variant,
            ButtonVariant::Filled
        );
        assert_eq!(
            Button::tonal("b2", "Tonal").variant,
            ButtonVariant::FilledTonal
        );
        assert_eq!(
            Button::elevated("b3", "Elevated").variant,
            ButtonVariant::Elevated
        );
        assert_eq!(
            Button::outlined("b4", "Outlined").variant,
            ButtonVariant::Outlined
        );
        assert_eq!(Button::text("b5", "Text").variant, ButtonVariant::Text);

        assert_eq!(
            ToggleButton::filled("tb1", "Filled", true).variant,
            ToggleButtonVariant::Filled
        );
        assert_eq!(
            ToggleButton::tonal("tb2", "Tonal", false).variant,
            ToggleButtonVariant::FilledTonal
        );
        assert_eq!(
            ToggleButton::elevated("tb3", "Elevated", true).variant,
            ToggleButtonVariant::Elevated
        );
        assert_eq!(
            ToggleButton::outlined("tb4", "Outlined", false).variant,
            ToggleButtonVariant::Outlined
        );

        assert_eq!(
            IconButton::standard("i1", "Std", div()).variant,
            IconButtonVariant::Standard
        );
        assert_eq!(
            IconButton::filled("i2", "Fill", div()).variant,
            IconButtonVariant::Filled
        );
        assert_eq!(
            IconButton::tonal("i3", "Tonal", div()).variant,
            IconButtonVariant::FilledTonal
        );
        assert_eq!(
            IconButton::outlined("i4", "Out", div()).variant,
            IconButtonVariant::Outlined
        );

        assert_eq!(
            IconToggleButton::standard("it1", "Std", div(), false).variant,
            IconButtonVariant::Standard
        );
        assert_eq!(
            IconToggleButton::filled("it2", "Fill", div(), true).variant,
            IconButtonVariant::Filled
        );
        assert_eq!(
            IconToggleButton::tonal("it3", "Tonal", div(), false).variant,
            IconButtonVariant::FilledTonal
        );
        assert_eq!(
            IconToggleButton::outlined("it4", "Out", div(), true).variant,
            IconButtonVariant::Outlined
        );
    }
}
