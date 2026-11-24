use std::time::Duration;

use gpui::{ElementId, Entity, IntoElement, Pixels, Size};

mod geometry;
mod render;
mod state;
mod types;

pub use geometry::{PaneExpansionAnchor, PaneLayoutDirective, PaneOverlayAlignment};
pub use state::{
    AdaptivePaneState, PaneAdaptStrategy, PaneBackBehavior, PaneDestination, PaneLayoutValue,
    PanePlacement, PaneRole,
};
pub use types::{AdaptivePane, PaneSeparator};

use types::pair_index;

/// A three-role scaffold with caller-supplied panes, an explicit available viewport,
/// and retained layout, expansion and navigation state.
#[derive(IntoElement)]
pub struct AdaptivePaneLayout {
    pub(super) id: ElementId,
    pub(super) viewport: Size<Pixels>,
    pub(super) state: Entity<AdaptivePaneState>,
    pub(super) directive: Option<PaneLayoutDirective>,
    pub(super) panes: [Option<AdaptivePane>; 3],
    pub(super) order: [PaneRole; 3],
    pub(super) separators: [Option<PaneSeparator>; 9],
    pub(super) duration: Duration,
}

impl AdaptivePaneLayout {
    /// Creates an adaptive multi-pane layout for the given viewport size and retained [`AdaptivePaneState`].
    pub fn new(
        id: impl Into<ElementId>,
        viewport: Size<Pixels>,
        state: &Entity<AdaptivePaneState>,
    ) -> Self {
        Self {
            id: id.into(),
            viewport,
            state: state.clone(),
            directive: None,
            panes: Default::default(),
            order: PaneRole::ALL,
            separators: Default::default(),
            duration: Duration::from_millis(240),
        }
    }

    /// Assigns an [`AdaptivePane`] to the given logical role slot.
    pub fn pane(mut self, role: PaneRole, pane: AdaptivePane) -> Self {
        self.panes[role.index()] = Some(pane);
        self
    }

    /// Overrides the [`PaneLayoutDirective`] derived from the viewport size.
    pub fn directive(mut self, directive: PaneLayoutDirective) -> Self {
        self.directive = Some(directive);
        self
    }

    /// Logical order of the three roles. Duplicate roles leave the previous order intact.
    pub fn order(mut self, order: [PaneRole; 3]) -> Self {
        if PaneRole::ALL.into_iter().all(|role| order.contains(&role)) {
            self.order = order;
        }
        self
    }

    /// Registers a resize separator between two adjacent logical pane roles.
    pub fn separator(mut self, start: PaneRole, end: PaneRole, separator: PaneSeparator) -> Self {
        self.separators[pair_index(start, end)] = Some(separator);
        self
    }

    /// Sets the animation duration used for pane layout transitions (default 240 ms).
    pub fn transition_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}
