use std::{collections::HashMap, rc::Rc};

use gpui::{AnyElement, App, ElementId, Entity, IntoElement, Pixels, SharedString, Window, px};
use gpui_material_theme::LogicalEdges;

use crate::{DialogCloseReason, DialogProperties, DialogState};

use super::items::NavigationItems;

mod render;
mod state;
mod types;

pub use state::NavigationRailState;
pub(in crate::navigation) use types::RailPresentation;
pub use types::{
    NavigationRailArrangement, NavigationRailColors, NavigationRailShapes, RailModalOptions,
    RailMode,
};

use types::{Closed, Destinations, DismissRequest};

/// Material wide navigation rail. The optional modal uses the side-sheet session.
#[derive(IntoElement)]
pub struct NavigationRail {
    pub(super) id: ElementId,
    pub(super) items: NavigationItems,
    pub(super) mode: RailMode,
    pub(super) state: Option<Entity<NavigationRailState>>,
    pub(super) title: Option<SharedString>,
    pub(super) header: Option<AnyElement>,
    pub(super) fab: Option<AnyElement>,
    pub(super) footer: Option<AnyElement>,
    pub(super) expanded_width: Pixels,
    pub(super) modal: Option<Entity<DialogState>>,
    pub(super) modal_options: RailModalOptions,
    pub(super) destination_content: Option<Destinations>,
    pub(super) dividers: HashMap<SharedString, AnyElement>,
    pub(super) arrangement: NavigationRailArrangement,
    pub(super) destination_spacing: Pixels,
    pub(super) content_padding: Option<LogicalEdges>,
    pub(super) header_spacing: Pixels,
    pub(super) expanded_header_top_padding: Pixels,
    pub(super) shapes: NavigationRailShapes,
    pub(super) colors: NavigationRailColors,
    pub(super) dismiss: Option<DismissRequest>,
    pub(super) closed: Option<Closed>,
}

impl NavigationRail {
    /// Creates a collapsed navigation rail with the given element ID and controlled [`NavigationItems`].
    pub fn new(id: impl Into<ElementId>, items: NavigationItems) -> Self {
        Self {
            id: id.into(),
            items,
            mode: RailMode::Collapsed,
            state: None,
            title: None,
            header: None,
            fab: None,
            footer: None,
            expanded_width: px(240.),
            modal: None,
            modal_options: RailModalOptions::default(),
            destination_content: None,
            dividers: HashMap::new(),
            arrangement: NavigationRailArrangement::default(),
            destination_spacing: px(4.),
            content_padding: None,
            header_spacing: px(4.),
            expanded_header_top_padding: px(0.),
            shapes: NavigationRailShapes::default(),
            colors: NavigationRailColors::default(),
            dismiss: None,
            closed: None,
        }
    }

    /// Sets the docked rail mode (default [`RailMode::Collapsed`]). Ignored when [`Self::state`] is bound.
    pub fn mode(mut self, mode: RailMode) -> Self {
        self.mode = mode;
        self
    }

    /// Uses retained presentation and transitions in place of the supplied mode.
    pub fn state(mut self, state: &Entity<NavigationRailState>) -> Self {
        self.state = Some(state.clone());
        self
    }

    /// Sets the accessible title used for the rail and its optional modal sheet (default `"Navigation"`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the rail headline displayed in expanded mode and used as the modal sheet's accessible name.
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.title = Some(headline.into());
        self
    }

    /// Places a custom header element at the top of the rail (such as a menu toggle button).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Places a floating action button below the header.
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }

    /// Places custom footer content anchored beneath the scrollable destination list.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Sets the expanded rail width in logical pixels, clamped to `220..=360` px (default 240 px).
    pub fn expanded_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() {
            self.expanded_width = width.clamp(px(220.), px(360.));
        }
        self
    }

    /// Replaces the standard destination list. The caller owns its navigation semantics.
    pub fn destination_content<E: IntoElement + 'static>(
        mut self,
        build: impl FnOnce(RailMode) -> E + 'static,
    ) -> Self {
        self.destination_content = Some(Box::new(move |mode| build(mode).into_any_element()));
        self
    }

    /// Places a divider or separator after one standard destination.
    pub fn divider_after(
        mut self,
        destination: impl Into<SharedString>,
        divider: impl IntoElement,
    ) -> Self {
        self.dividers
            .insert(destination.into(), divider.into_any_element());
        self
    }

    /// Sets how destinations are vertically distributed within the rail (default [`NavigationRailArrangement::Start`]).
    pub fn arrangement(mut self, arrangement: NavigationRailArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    /// Sets the vertical gap between destinations in logical pixels (default 4 px).
    pub fn destination_spacing(mut self, spacing: Pixels) -> Self {
        self.destination_spacing = length(spacing);
        self
    }

    /// Overrides the logical inner padding of the rail container.
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.content_padding = Some(padding.sanitize());
        self
    }

    /// Sets the vertical spacing below the header and FAB slots (default 4 px).
    pub fn header_spacing(mut self, spacing: Pixels) -> Self {
        self.header_spacing = length(spacing);
        self
    }

    /// Sets extra top padding above the header when the rail is expanded.
    pub fn expanded_header_top_padding(mut self, padding: Pixels) -> Self {
        self.expanded_header_top_padding = length(padding);
        self
    }

    /// Overrides corner shapes for collapsed, expanded, and modal presentations.
    pub fn shapes(mut self, shapes: NavigationRailShapes) -> Self {
        self.shapes = shapes;
        self
    }

    /// Overrides container, content, and modal scrim colors.
    pub fn colors(mut self, colors: NavigationRailColors) -> Self {
        self.colors = colors;
        self
    }

    /// Attaches a caller-owned [`DialogState`] for modal visibility and focus restoration.
    pub fn modal_state(mut self, state: &Entity<DialogState>) -> Self {
        self.modal = Some(state.clone());
        self
    }

    /// Configures Escape and outside-click dismissal rules for the modal rail sheet.
    pub fn modal_properties(mut self, properties: DialogProperties) -> Self {
        self.modal_options.properties = properties;
        self
    }

    /// Configures side-sheet presentation and dismissal policy for the modal rail.
    pub fn modal_options(mut self, options: RailModalOptions) -> Self {
        self.modal_options = options;
        self
    }

    /// Intercepts user-initiated modal dismissal requests; return `false` to veto closing.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.dismiss = Some(Rc::new(callback));
        self
    }

    /// Invoked once when an accepted modal dismissal closes the rail sheet.
    pub fn on_closed(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.closed = Some(Rc::new(callback));
        self
    }
}

pub(super) fn length(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() {
        value.max(px(0.))
    } else {
        px(0.)
    }
}
