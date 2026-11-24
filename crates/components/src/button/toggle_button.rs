use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

use crate::action::{Action, Change};

use super::{
    appearance::button_appearance,
    appearance_builders, common_builders, slot_builders,
    slots::{Slots, loading_overlay},
    types::{ButtonVariant, Common, ToggleButtonVariant},
};

/// A controlled labeled toggle. The caller re-renders with the selected value
/// after accepting a requested change.
#[derive(IntoElement)]
pub struct ToggleButton {
    common: Common,
    pub(super) variant: ToggleButtonVariant,
    selected: bool,
    slots: Slots,
    on_change: Option<Change>,
}

impl ToggleButton {
    /// Creates a controlled toggle button with the default [`ToggleButtonVariant::Filled`] emphasis.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            common: Common::new(id, label),
            variant: ToggleButtonVariant::default(),
            selected,
            slots: Slots::default(),
            on_change: None,
        }
    }

    /// Creates a Filled toggle button.
    pub fn filled(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
    ) -> Self {
        Self::new(id, label, selected).variant(ToggleButtonVariant::Filled)
    }

    /// Creates a Filled Tonal toggle button.
    pub fn tonal(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self::new(id, label, selected).variant(ToggleButtonVariant::FilledTonal)
    }

    /// Creates an Elevated toggle button.
    pub fn elevated(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
    ) -> Self {
        Self::new(id, label, selected).variant(ToggleButtonVariant::Elevated)
    }

    /// Creates an Outlined toggle button.
    pub fn outlined(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
    ) -> Self {
        Self::new(id, label, selected).variant(ToggleButtonVariant::Outlined)
    }

    common_builders!();
    appearance_builders!();
    slot_builders!();

    /// Sets the visual emphasis variant. Defaults to [`ToggleButtonVariant::Filled`].
    pub fn variant(mut self, variant: ToggleButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a centered inline circular spinner while suppressing activation and preserving the
    /// button's resting width. Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
        self
    }

    /// Invoked with the requested next selected state when the toggle button is activated.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let metrics = self.common.size.metrics();
        let variant = match self.variant {
            ToggleButtonVariant::Elevated => ButtonVariant::Elevated,
            ToggleButtonVariant::Filled => ButtonVariant::Filled,
            ToggleButtonVariant::FilledTonal => ButtonVariant::FilledTonal,
            ToggleButtonVariant::Outlined => ButtonVariant::Outlined,
        };
        let mut style = button_appearance(variant, Some(self.selected), self.common.tone).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            0.,
        );
        style.selected_corner = Some(self.common.shape.corner(metrics.square, true));
        let tint = self
            .common
            .appearance
            .colors
            .content(self.common.disabled, self.selected)
            .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
        let resting_content =
            self.slots
                .render(self.common.label.clone(), metrics, 1., window, tint, cx);
        let content = if self.common.loading {
            loading_overlay(
                &self.common.id,
                resting_content,
                metrics.icon,
                tint,
                window,
                cx,
            )
        } else {
            resting_content
        };
        self.common.render(
            content,
            style,
            Action::Toggle {
                selected: self.selected,
                on_change: self.on_change,
            },
            window,
            cx,
        )
    }
}
