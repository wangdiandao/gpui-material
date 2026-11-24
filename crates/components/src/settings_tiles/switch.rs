use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, FocusHandle, IntoElement, RenderOnce, Role, SharedString, Window,
};
use gpui_material_foundation::InteractionSource;

use crate::{ListItem, ListItemColors, ListVariant, Switch, SwitchColors, action::Change};

use super::common::{TileFields, apply_standalone_variant};

/// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Switch`].
///
/// Clicking anywhere on the row or pressing Enter or Space toggles once with a single focus
/// stop and `Role::Switch` semantics.
#[derive(IntoElement)]
pub struct SwitchTile {
    fields: TileFields,
    checked: bool,
    switch_colors: SwitchColors,
    on_change: Option<Change>,
}

impl SwitchTile {
    /// Creates a switch settings row with an element ID, headline label, and controlled checked state.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            fields: TileFields::new(id, label),
            checked,
            switch_colors: SwitchColors::default(),
            on_change: None,
        }
    }

    /// Sets secondary supporting text displayed below the headline.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.supporting = Some(text.into());
        self
    }

    /// Sets overline text displayed above the headline.
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.overline = Some(text.into());
        self
    }

    /// Sets an optional leading icon or graphic.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.fields.leading = Some(leading.into_any_element());
        self
    }

    /// Disables row interaction and dims the switch when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.fields.disabled = disabled;
        self
    }

    /// Sets standalone row styling (`Standard` or `Segmented`).
    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.fields.variant = variant;
        self
    }

    /// Overrides container and slot colors on the underlying [`ListItem`].
    pub fn colors(mut self, colors: ListItemColors) -> Self {
        self.fields.colors = colors;
        self
    }

    /// Overrides track, thumb, border, and content colors on the decorative [`Switch`].
    pub fn switch_colors(mut self, colors: SwitchColors) -> Self {
        self.switch_colors = colors;
        self
    }

    /// Uses a caller-owned focus handle for this tile.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.fields.focus = Some(focus.clone());
        self
    }

    /// Reports hover, focus, press, and disabled states through a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.fields.source = Some(source.clone());
        self
    }

    /// Invoked with the requested next checked state when the tile is toggled.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub(super) fn into_list_item(self) -> ListItem {
        let switch = Switch::new(
            (self.fields.id.clone(), "switch"),
            self.fields.label.clone(),
            self.checked,
        )
        .disabled(self.fields.disabled)
        .colors(self.switch_colors)
        .decorative(true);

        let mut item = self
            .fields
            .into_list_item()
            .checked(self.checked)
            .checked_role(Role::Switch)
            .checked_selects_row(false)
            .trailing(switch);

        if let Some(on_change) = self.on_change {
            item = item.on_checked_change(move |next, window, cx| on_change(next, window, cx));
        }
        item
    }
}

impl RenderOnce for SwitchTile {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let variant = self.fields.variant;
        apply_standalone_variant(self.into_list_item(), variant)
    }
}
