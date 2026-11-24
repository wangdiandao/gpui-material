use std::rc::Rc;

use gpui::{AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window};
use gpui_material_theme::{MotionKind, MotionSpeed};

use crate::action::{Action, Change, InvokeAction, icon_slot};

use super::{
    appearance::icon_appearance,
    appearance_builders, common_builders,
    slots::loading_icon_spinner,
    types::{Common, IconButtonVariant, IconButtonWidth},
};

/// An icon-only action with a required accessibility name.
#[derive(IntoElement)]
pub struct IconButton {
    common: Common,
    icon: AnyElement,
    pub(super) variant: IconButtonVariant,
    width: IconButtonWidth,
    on_activate: Option<InvokeAction>,
}

impl IconButton {
    /// Creates an icon button with the default [`IconButtonVariant::Standard`] emphasis.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            common: Common::new(id, name),
            icon: icon.into_any_element(),
            variant: IconButtonVariant::default(),
            width: IconButtonWidth::default(),
            on_activate: None,
        }
    }

    /// Creates a Standard icon button.
    pub fn standard(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Standard)
    }

    /// Creates a Filled icon button.
    pub fn filled(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Filled)
    }

    /// Creates a Filled Tonal icon button.
    pub fn tonal(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::FilledTonal)
    }

    /// Creates an Outlined icon button.
    pub fn outlined(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Outlined)
    }

    common_builders!();
    appearance_builders!();

    /// Sets the visual emphasis variant. Defaults to [`IconButtonVariant::Standard`].
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the horizontal width preset. Defaults to [`IconButtonWidth::Uniform`].
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    /// Shows an inline circular spinner in place of the icon while suppressing activation.
    /// Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
        self
    }

    /// Invoked once when the icon button is activated via pointer, keyboard, or assistive technology.
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

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (metrics, width) = self.common.size.icon_metrics(self.width);
        let mut style = icon_appearance(self.variant, None, self.common.tone).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            width,
        );
        style.corner_motion = (MotionKind::Effects, MotionSpeed::Default);
        let content = if self.common.loading {
            let tint = self
                .common
                .appearance
                .colors
                .content(self.common.disabled, false)
                .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
            loading_icon_spinner(&self.common.id, metrics.icon, tint, window, cx)
        } else {
            icon_slot(self.icon, metrics.icon).into_any_element()
        };
        self.common
            .render(content, style, Action::Invoke(self.on_activate), window, cx)
    }
}

/// An icon-only controlled toggle. The selected icon and native toggle state
/// both reflect the caller's value.
#[derive(IntoElement)]
pub struct IconToggleButton {
    common: Common,
    icon: AnyElement,
    selected_icon: Option<AnyElement>,
    selected: bool,
    pub(super) variant: IconButtonVariant,
    width: IconButtonWidth,
    on_change: Option<Change>,
}

impl IconToggleButton {
    /// Creates a controlled icon toggle button with the default [`IconButtonVariant::Standard`] emphasis.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self {
            common: Common::new(id, name),
            icon: icon.into_any_element(),
            selected_icon: None,
            selected,
            variant: IconButtonVariant::default(),
            width: IconButtonWidth::default(),
            on_change: None,
        }
    }

    /// Creates a Standard icon toggle button.
    pub fn standard(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self::new(id, name, icon, selected).variant(IconButtonVariant::Standard)
    }

    /// Creates a Filled icon toggle button.
    pub fn filled(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self::new(id, name, icon, selected).variant(IconButtonVariant::Filled)
    }

    /// Creates a Filled Tonal icon toggle button.
    pub fn tonal(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self::new(id, name, icon, selected).variant(IconButtonVariant::FilledTonal)
    }

    /// Creates an Outlined icon toggle button.
    pub fn outlined(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self::new(id, name, icon, selected).variant(IconButtonVariant::Outlined)
    }

    common_builders!();
    appearance_builders!();

    /// Supplies an alternate icon displayed when `selected` is `true`.
    pub fn selected_icon(mut self, icon: impl IntoElement) -> Self {
        self.selected_icon = Some(icon.into_any_element());
        self
    }

    /// Sets the visual emphasis variant. Defaults to [`IconButtonVariant::Standard`].
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the horizontal width preset. Defaults to [`IconButtonWidth::Uniform`].
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    /// Shows an inline circular spinner in place of the icon while suppressing activation.
    /// Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
        self
    }

    /// Invoked with the requested next selected state when the icon toggle button is activated.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for IconToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (metrics, width) = self.common.size.icon_metrics(self.width);
        let mut style = icon_appearance(self.variant, Some(self.selected), self.common.tone).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            width,
        );
        style.selected_corner = Some(self.common.shape.corner(metrics.square, true));
        style.corner_motion = (MotionKind::Effects, MotionSpeed::Default);
        let content = if self.common.loading {
            let tint = self
                .common
                .appearance
                .colors
                .content(self.common.disabled, self.selected)
                .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
            loading_icon_spinner(&self.common.id, metrics.icon, tint, window, cx)
        } else {
            let icon = if self.selected {
                self.selected_icon.unwrap_or(self.icon)
            } else {
                self.icon
            };
            icon_slot(icon, metrics.icon).into_any_element()
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

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn icon_button_activation_modes_replace_each_other() {
        let icon = IconButton::new("i", "More", div()).on_accessible_activate(|_, _| {});
        assert!(matches!(
            icon.on_activate,
            Some(InvokeAction::AccessibleOnly(_))
        ));
        let icon = icon.on_activate(|_, _| {});
        assert!(matches!(icon.on_activate, Some(InvokeAction::Full(_))));
        let icon = icon.on_accessible_activate(|_, _| {});
        assert!(matches!(
            icon.on_activate,
            Some(InvokeAction::AccessibleOnly(_))
        ));
    }
}
