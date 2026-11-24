use gpui::{AnyElement, ElementId, Entity, FocusHandle, SharedString};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::CornerRole;

use crate::{ListItem, ListItemColors, ListItemShapes, ListVariant};

pub(super) struct TileFields {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) supporting: Option<SharedString>,
    pub(super) overline: Option<SharedString>,
    pub(super) leading: Option<AnyElement>,
    pub(super) disabled: bool,
    pub(super) variant: ListVariant,
    pub(super) colors: ListItemColors,
    pub(super) focus: Option<FocusHandle>,
    pub(super) source: Option<Entity<InteractionSource>>,
}

impl TileFields {
    pub(super) fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            overline: None,
            leading: None,
            disabled: false,
            variant: ListVariant::default(),
            colors: ListItemColors::default(),
            focus: None,
            source: None,
        }
    }

    pub(super) fn into_list_item(self) -> ListItem {
        let mut item = ListItem::new(self.id, self.label)
            .disabled(self.disabled)
            .colors(self.colors);
        if let Some(supporting) = self.supporting {
            item = item.supporting(supporting);
        }
        if let Some(overline) = self.overline {
            item = item.overline(overline);
        }
        if let Some(leading) = self.leading {
            item = item.leading(leading);
        }
        if let Some(focus) = self.focus {
            item = item.track_focus(&focus);
        }
        if let Some(source) = self.source {
            item = item.interaction_source(&source);
        }
        item
    }
}

pub(super) fn apply_standalone_variant(item: ListItem, variant: ListVariant) -> ListItem {
    if variant == ListVariant::Segmented {
        item.shapes(ListItemShapes {
            resting: Some(CornerRole::Large.shape()),
            selected: Some(CornerRole::Large.shape()),
            ..Default::default()
        })
    } else {
        item
    }
}
