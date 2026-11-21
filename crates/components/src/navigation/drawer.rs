use std::{collections::HashSet, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Window, px,
};
use gpui_material_theme::CornerShape;

use crate::{DialogCloseReason, DialogProperties, DialogState};

use super::items::{NavigationItem, NavigationItems};

mod render;

/// Presentation mode for a standalone [`NavigationDrawer`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NavigationDrawerMode {
    #[default]
    Standard,
    Modal,
}

/// Ordered entry inside a [`NavigationDrawer`] destination list.
pub enum NavigationDrawerEntry {
    Item(Box<NavigationItem>),
    Header(SharedString),
    Divider,
}

impl NavigationDrawerEntry {
    /// Creates a destination item entry.
    pub fn item(item: NavigationItem) -> Self {
        Self::Item(Box::new(item))
    }
}

impl From<NavigationItem> for NavigationDrawerEntry {
    fn from(item: NavigationItem) -> Self {
        Self::Item(Box::new(item))
    }
}

/// Convenience section grouping an optional heading with a list of [`NavigationItem`]s.
pub struct NavigationDrawerSection {
    pub header: Option<SharedString>,
    pub items: Vec<NavigationItem>,
}

impl NavigationDrawerSection {
    pub fn new(items: impl IntoIterator<Item = NavigationItem>) -> Self {
        Self {
            header: None,
            items: items.into_iter().collect(),
        }
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.header = Some(title.into());
        self
    }
}

/// Per-instance pigment overrides for [`NavigationDrawer`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationDrawerColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub header: Option<Hsla>,
    pub divider: Option<Hsla>,
    pub active_indicator: Option<Hsla>,
    pub active_content: Option<Hsla>,
    pub inactive_content: Option<Hsla>,
    pub scrim: Option<Hsla>,
    pub scrim_opacity: Option<f32>,
}

pub(super) type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
pub(super) type Closed = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;

/// Full-width Material 3 Navigation Drawer (360 dp default width) with Standard (docked) and
/// Modal (side-sheet) presentations, sections, dividers, and 56 dp pill destination indicators.
#[derive(IntoElement)]
pub struct NavigationDrawer {
    pub(super) id: ElementId,
    pub(super) items: NavigationItems,
    pub(super) entries: Vec<NavigationDrawerEntry>,
    pub(super) dividers_before: HashSet<SharedString>,
    pub(super) mode: NavigationDrawerMode,
    pub(super) width: Pixels,
    pub(super) headline: Option<SharedString>,
    pub(super) header: Option<AnyElement>,
    pub(super) footer: Option<AnyElement>,
    pub(super) shape: Option<CornerShape>,
    pub(super) colors: NavigationDrawerColors,
    pub(super) modal_state: Option<Entity<DialogState>>,
    pub(super) modal_properties: DialogProperties,
    pub(super) on_dismiss_request: Option<DismissRequest>,
    pub(super) on_closed: Option<Closed>,
}

impl NavigationDrawer {
    pub const DEFAULT_WIDTH: Pixels = px(360.);

    pub fn new(id: impl Into<ElementId>, items: NavigationItems) -> Self {
        Self {
            id: id.into(),
            items,
            entries: Vec::new(),
            dividers_before: HashSet::new(),
            mode: NavigationDrawerMode::default(),
            width: Self::DEFAULT_WIDTH,
            headline: None,
            header: None,
            footer: None,
            shape: None,
            colors: NavigationDrawerColors::default(),
            modal_state: None,
            modal_properties: DialogProperties::default(),
            on_dismiss_request: None,
            on_closed: None,
        }
    }

    /// Appends ordered destination, header, or divider entries to the drawer list.
    pub fn entries(mut self, entries: impl IntoIterator<Item = NavigationDrawerEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    /// Appends a section heading followed by its navigation items.
    pub fn section(
        mut self,
        header: impl Into<SharedString>,
        items: impl IntoIterator<Item = NavigationItem>,
    ) -> Self {
        self.entries
            .push(NavigationDrawerEntry::Header(header.into()));
        self.entries
            .extend(items.into_iter().map(NavigationDrawerEntry::item));
        self
    }

    /// Appends a [`NavigationDrawerSection`] with an optional header and its items.
    pub fn push_section(mut self, section: NavigationDrawerSection) -> Self {
        if let Some(header) = section.header {
            self.entries.push(NavigationDrawerEntry::Header(header));
        }
        self.entries
            .extend(section.items.into_iter().map(NavigationDrawerEntry::item));
        self
    }

    /// Appends a horizontal divider entry to the ordered entries list.
    pub fn divider(mut self) -> Self {
        self.entries.push(NavigationDrawerEntry::Divider);
        self
    }

    /// Inserts a horizontal divider immediately before the destination with matching `item_id`.
    pub fn divider_before(mut self, item_id: impl Into<SharedString>) -> Self {
        self.dividers_before.insert(item_id.into());
        self
    }

    /// Chooses Standard (docked, default) or Modal (side-sheet overlay) mode.
    pub fn mode(mut self, mode: NavigationDrawerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the drawer width in logical pixels (defaults to 360 dp, clamped to viewport).
    pub fn width(mut self, width: Pixels) -> Self {
        let value = f32::from(width);
        if value.is_finite() && value > 0. {
            self.width = width.max(px(200.));
        }
        self
    }

    /// Sets the drawer title displayed in Standard mode and used as the modal sheet's accessible name.
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    /// Supplies custom header content above the destination list.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Supplies custom footer content anchored beneath the scrolling destination list.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Overrides the container corner shape.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides container, content, header, divider, indicator, and modal scrim colors independently.
    pub fn colors(mut self, colors: NavigationDrawerColors) -> Self {
        self.colors = colors;
        self
    }

    /// Attaches a caller-owned `DialogState` for `NavigationDrawerMode::Modal`.
    pub fn modal_state(mut self, state: &Entity<DialogState>) -> Self {
        self.modal_state = Some(state.clone());
        self
    }

    /// Configures Escape and outside-click dismissal rules in Modal mode.
    pub fn modal_properties(mut self, properties: DialogProperties) -> Self {
        self.modal_properties = properties;
        self
    }

    /// Intercepts user-initiated modal dismissal requests; return `false` to veto closing.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(callback));
        self
    }

    /// Invoked once when an accepted modal dismissal closes the drawer.
    pub fn on_closed(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_closed = Some(Rc::new(callback));
        self
    }
}
