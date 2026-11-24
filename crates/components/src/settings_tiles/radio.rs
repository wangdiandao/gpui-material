use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, RenderOnce, Role, SharedString,
    Window, accesskit::Toggled,
};
use gpui_material_foundation::InteractionSource;

use crate::{ListItem, ListItemColors, ListVariant, Radio, RadioColors, action::Activate};

use super::common::{TileFields, apply_standalone_variant};

/// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Radio`].
///
/// Clicking anywhere on an unselected row or pressing Enter or Space invokes `.on_select`
/// with a single focus stop and `Role::RadioButton` semantics.
#[derive(IntoElement)]
pub struct RadioTile {
    fields: TileFields,
    selected: bool,
    trailing: Option<AnyElement>,
    trailing_radio: bool,
    radio_colors: RadioColors,
    on_select: Option<Activate>,
}

impl RadioTile {
    /// Creates a radio settings row with an element ID, headline label, and controlled selected state.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            fields: TileFields::new(id, label),
            selected,
            trailing: None,
            trailing_radio: true,
            radio_colors: RadioColors::default(),
            on_select: None,
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

    /// Sets an optional leading icon or graphic when the radio indicator is in the trailing slot.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.fields.leading = Some(leading.into_any_element());
        self
    }

    /// Trailing decorative content used when `.trailing_radio(false)` places the radio
    /// in the leading slot.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Places the decorative radio indicator in the trailing slot (`true`, default) or leading slot (`false`).
    pub fn trailing_radio(mut self, trailing: bool) -> Self {
        self.trailing_radio = trailing;
        self
    }

    /// Disables row interaction and dims the radio indicator when `true`.
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

    /// Overrides indicator colors on the decorative [`Radio`].
    pub fn radio_colors(mut self, colors: RadioColors) -> Self {
        self.radio_colors = colors;
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

    /// Invoked when an unselected radio tile is activated.
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub(super) fn into_list_item(mut self) -> ListItem {
        let radio = Radio::new((self.fields.id.clone(), "radio"), self.fields.label.clone())
            .selected(self.selected)
            .disabled(self.fields.disabled)
            .colors(self.radio_colors)
            .decorative(true);

        let toggled = if self.selected {
            Toggled::True
        } else {
            Toggled::False
        };

        let leading = self.fields.leading.take();
        let mut item = self
            .fields
            .into_list_item()
            .checked_toggled(toggled)
            .checked_role(Role::RadioButton)
            .checked_selects_row(false);

        if self.trailing_radio {
            item = item.trailing(radio);
            if let Some(leading) = leading {
                item = item.leading(leading);
            }
        } else {
            item = item.leading(radio);
            if let Some(trailing) = self.trailing {
                item = item.trailing(trailing);
            }
        }

        if let Some(on_select) = self.on_select {
            let already_selected = self.selected;
            item = item.on_activate(move |window, cx| {
                if !already_selected {
                    on_select(window, cx);
                }
            });
        }
        item
    }
}

impl RenderOnce for RadioTile {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let variant = self.fields.variant;
        apply_standalone_variant(self.into_list_item(), variant)
    }
}
