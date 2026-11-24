use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

use crate::action::{Action, InvokeAction};

use super::{
    appearance::button_appearance,
    appearance_builders, common_builders, slot_builders,
    slots::{Slots, loading_overlay},
    types::{ButtonVariant, Common},
};

/// A labeled Material action. Activation uses Base's pointer, keyboard and
/// accessibility behavior and invokes the application callback once.
#[derive(IntoElement)]
pub struct Button {
    common: Common,
    pub(super) variant: ButtonVariant,
    slots: Slots,
    on_activate: Option<InvokeAction>,
    scale: f32,
}

impl Button {
    /// Creates a button with the default [`ButtonVariant::Filled`] emphasis.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            common: Common::new(id, label),
            variant: ButtonVariant::default(),
            slots: Slots::default(),
            on_activate: None,
            scale: 1.,
        }
    }

    /// Creates a Filled button.
    pub fn filled(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Filled)
    }

    /// Creates a Filled Tonal button.
    pub fn tonal(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::FilledTonal)
    }

    /// Creates an Elevated button.
    pub fn elevated(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Elevated)
    }

    /// Creates an Outlined button.
    pub fn outlined(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Outlined)
    }

    /// Creates a Text button.
    pub fn text(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Text)
    }

    common_builders!();
    appearance_builders!();
    slot_builders!();

    /// Sets the visual emphasis variant. Defaults to [`ButtonVariant::Filled`].
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a centered inline circular spinner while suppressing activation and preserving the
    /// button's resting width. Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
        self
    }

    #[cfg(feature = "feedback")]
    pub(crate) fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Invoked once when the button is activated via pointer click, keyboard Enter/Space, or assistive technology.
    ///
    /// Replaces any previously registered `.on_activate(...)` or `.on_accessible_activate(...)` handler.
    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(InvokeAction::Full(Rc::new(handler)));
        self
    }

    /// Registers only the accessibility click action (`AccessibleAction::Click`), leaving pointer
    /// and keyboard activation to an enclosing trigger container such as `Menu::composed` or
    /// `gpui_base::Popover`.
    ///
    /// Press feedback and focus remain active while avoiding `on_click` conflicts with container
    /// `on_mouse_down` handlers. Replaces any previously registered `.on_activate(...)` or
    /// `.on_accessible_activate(...)` handler.
    pub fn on_accessible_activate(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(InvokeAction::AccessibleOnly(Rc::new(handler)));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let metrics = self.common.size.metrics();
        let appearance = button_appearance(self.variant, None, self.common.tone);
        let style = appearance.style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            58.,
        );
        let tint = self
            .common
            .appearance
            .colors
            .content(self.common.disabled, false)
            .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
        let resting_content = self.slots.render(
            self.common.label.clone(),
            metrics,
            self.scale,
            window,
            tint,
            cx,
        );
        let content = if self.common.loading {
            loading_overlay(
                &self.common.id,
                resting_content,
                metrics.icon * self.scale,
                tint,
                window,
                cx,
            )
        } else {
            resting_content
        };
        self.common.render_scaled(
            content,
            style,
            Action::Invoke(self.on_activate),
            self.scale,
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_activation_modes_replace_each_other() {
        let button = Button::new("b", "Open").on_accessible_activate(|_, _| {});
        assert!(matches!(
            button.on_activate,
            Some(InvokeAction::AccessibleOnly(_))
        ));
        let button = button.on_activate(|_, _| {});
        assert!(matches!(button.on_activate, Some(InvokeAction::Full(_))));
        let button = button.on_accessible_activate(|_, _| {});
        assert!(matches!(
            button.on_activate,
            Some(InvokeAction::AccessibleOnly(_))
        ));
    }
}
