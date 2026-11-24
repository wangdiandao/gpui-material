use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, RenderOnce, Role, SharedString,
    Window, accesskit::Toggled,
};
use gpui_material_foundation::InteractionSource;

use crate::{Checkbox, CheckboxColors, CheckboxState, ListItem, ListItemColors, ListVariant};

use super::common::{TileFields, apply_standalone_variant};

type CheckboxTileChange = Rc<dyn Fn(CheckboxState, &mut Window, &mut App)>;

/// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Checkbox`].
///
/// Clicking anywhere on the row or pressing Enter or Space requests the next [`CheckboxState`]
/// with a single focus stop and `Role::CheckBox` semantics.
#[derive(IntoElement)]
pub struct CheckboxTile {
    fields: TileFields,
    state: CheckboxState,
    trailing: Option<AnyElement>,
    trailing_checkbox: bool,
    error: bool,
    checkbox_colors: CheckboxColors,
    on_change: Option<CheckboxTileChange>,
}

impl CheckboxTile {
    /// Creates a checkbox settings row with an element ID, headline label, and controlled [`CheckboxState`].
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: CheckboxState,
    ) -> Self {
        Self {
            fields: TileFields::new(id, label),
            state,
            trailing: None,
            trailing_checkbox: true,
            error: false,
            checkbox_colors: CheckboxColors::default(),
            on_change: None,
        }
    }

    /// Convenience constructor for a two-state boolean checkbox tile.
    pub fn bool(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self::new(
            id,
            label,
            if checked {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            },
        )
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

    /// Sets an optional leading icon or graphic when the checkbox is in the trailing slot.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.fields.leading = Some(leading.into_any_element());
        self
    }

    /// Trailing decorative content used when `.trailing_checkbox(false)` places the checkbox
    /// in the leading slot.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Places the decorative checkbox in the trailing slot (`true`, default) or leading slot (`false`).
    pub fn trailing_checkbox(mut self, trailing: bool) -> Self {
        self.trailing_checkbox = trailing;
        self
    }

    /// Marks the checkbox tile as invalid, applying error styling and `aria-invalid` semantics.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Disables row interaction and dims the checkbox when `true`.
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

    /// Overrides checkmark, box, and border colors on the decorative [`Checkbox`].
    pub fn checkbox_colors(mut self, colors: CheckboxColors) -> Self {
        self.checkbox_colors = colors;
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

    /// Invoked with the requested next [`CheckboxState`] when the tile is toggled.
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub(super) fn into_list_item(mut self) -> ListItem {
        let checkbox = Checkbox::new(
            (self.fields.id.clone(), "checkbox"),
            self.fields.label.clone(),
            self.state,
        )
        .error(self.error)
        .disabled(self.fields.disabled)
        .colors(self.checkbox_colors)
        .decorative(true);

        let toggled = match self.state {
            CheckboxState::Unchecked => Toggled::False,
            CheckboxState::Checked => Toggled::True,
            CheckboxState::Indeterminate => Toggled::Mixed,
        };

        let leading = self.fields.leading.take();
        let mut item = self
            .fields
            .into_list_item()
            .checked_toggled(toggled)
            .checked_role(Role::CheckBox)
            .checked_selects_row(false)
            .invalid(self.error);

        if self.trailing_checkbox {
            item = item.trailing(checkbox);
            if let Some(leading) = leading {
                item = item.leading(leading);
            }
        } else {
            item = item.leading(checkbox);
            if let Some(trailing) = self.trailing {
                item = item.trailing(trailing);
            }
        }

        if let Some(on_change) = self.on_change {
            let next = if self.state == CheckboxState::Checked {
                CheckboxState::Unchecked
            } else {
                CheckboxState::Checked
            };
            item = item.on_activate(move |window, cx| on_change(next, window, cx));
        }
        item
    }
}

impl RenderOnce for CheckboxTile {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let variant = self.fields.variant;
        apply_standalone_variant(self.into_list_item(), variant)
    }
}
