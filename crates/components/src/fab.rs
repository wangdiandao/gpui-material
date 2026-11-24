mod extended;
mod standard;
mod types;

pub use extended::ExtendedFab;
pub use standard::{Fab, FloatingAction};
pub use types::{FabAlignment, FabColor, FabSize, FabVisibilityAnimation};

macro_rules! fab_builders {
    () => {
        /// Sets the Expressive FAB size tier. Defaults to [`super::FabSize::Small`].
        pub fn size(mut self, size: super::FabSize) -> Self {
            self.content.size = size;
            self
        }

        /// Sets the FAB color palette preset. Defaults to [`super::FabColor::Primary`].
        pub fn color(mut self, color: super::FabColor) -> Self {
            self.content.color = color;
            self
        }

        /// Desktop compatibility extension. Material recommends hiding a FAB
        /// when its primary action is unavailable instead of disabling it.
        pub fn disabled(mut self, disabled: bool) -> Self {
            self.content.disabled = disabled;
            self
        }

        /// Uses a caller-owned focus handle for keyboard traversal and focus state.
        pub fn track_focus(mut self, focus: &gpui::FocusHandle) -> Self {
            self.content.focus = Some(focus.clone());
            self
        }

        /// Invoked once when the FAB is activated via pointer, keyboard, or assistive technology.
        pub fn on_activate(
            mut self,
            handler: impl Fn(&mut gpui::Window, &mut gpui::App) + 'static,
        ) -> Self {
            self.content.on_activate = Some(std::rc::Rc::new(handler));
            self
        }

        /// Overrides the container shape for all interaction states.
        pub fn shape(mut self, shape: gpui_material_theme::CornerShape) -> Self {
            self.appearance.style.shapes.resting = Some(shape);
            self.appearance.style.shapes.pressed = Some(shape);
            self
        }

        /// Sets the enabled and disabled container and content colors.
        pub fn colors(mut self, colors: crate::control_style::ButtonColors) -> Self {
            self.appearance.style.colors = colors;
            self
        }

        /// Overrides the elevation for each interaction state.
        pub fn elevations(mut self, elevations: crate::control_style::ControlElevations) -> Self {
            self.appearance.style.elevations = elevations;
            self
        }

        /// Reports interaction state through a caller-owned source.
        pub fn interaction_source(
            mut self,
            source: &gpui::Entity<gpui_material_foundation::InteractionSource>,
        ) -> Self {
            self.appearance.interaction_source = Some(source.clone());
            self
        }

        /// Applies the caller's visibility state. Use a stable ID across updates.
        pub fn visible(mut self, visible: bool) -> Self {
            self.appearance.visible = Some(visible);
            self
        }

        /// Sets the pivot, hidden scale, and independent scale and alpha motions.
        pub fn visibility_animation(mut self, animation: super::FabVisibilityAnimation) -> Self {
            self.appearance.animation = animation;
            self
        }
    };
}
pub(super) use fab_builders;
