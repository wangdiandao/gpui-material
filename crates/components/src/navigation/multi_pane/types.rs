use gpui::{AnyElement, ElementId, FocusHandle, IntoElement, Pixels, SharedString, Size, px};
use gpui_material_theme::LogicalEdges;

use crate::VerticalDragHandle;

use super::{
    PaneAdaptStrategy, PaneExpansionAnchor, PaneLayoutDirective, PaneRole, geometry::Measures,
};

type AdaptationRule = Box<dyn Fn(Size<Pixels>, &PaneLayoutDirective) -> PaneAdaptStrategy>;
type HandleBuilder = Box<dyn FnOnce(VerticalDragHandle) -> VerticalDragHandle>;

/// A pane slot with stable identity, logical geometry preferences and adaptation.
pub struct AdaptivePane {
    pub(super) id: ElementId,
    pub(super) content: AnyElement,
    pub(super) measures: Measures,
    pub(super) adapt: Option<PaneAdaptStrategy>,
    pub(super) rule: Option<AdaptationRule>,
    pub(super) scrim: Option<AnyElement>,
    pub(super) entry_focus: Option<FocusHandle>,
}

impl AdaptivePane {
    /// Creates an adaptive pane with a stable element ID and body content.
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            measures: Measures::default(),
            adapt: None,
            rule: None,
            scrim: None,
            entry_focus: None,
        }
    }

    /// Sets the pane's preferred width in logical pixels.
    pub fn preferred_width(mut self, width: Pixels) -> Self {
        self.measures.preferred_width = Some(width);
        self
    }

    /// Sets the pane's preferred height in logical pixels (used for levitated or stacked presentations).
    pub fn preferred_height(mut self, height: Pixels) -> Self {
        self.measures.preferred_height = Some(height);
        self
    }

    /// Sets the minimum width below which the pane will not shrink.
    pub fn minimum_width(mut self, width: Pixels) -> Self {
        self.measures.minimum_width = width;
        self
    }

    /// Sets logical outer margins around the pane.
    pub fn margins(mut self, margins: LogicalEdges) -> Self {
        self.measures.margins = margins.sanitize();
        self
    }

    /// Sets a fixed adaptation strategy when the pane cannot be shown as an inline column.
    pub fn adapt(mut self, strategy: PaneAdaptStrategy) -> Self {
        self.adapt = Some(strategy);
        self.rule = None;
        self
    }

    /// Calculates the adaptation strategy from the available size and directive.
    pub fn adapt_with(
        mut self,
        rule: impl Fn(Size<Pixels>, &PaneLayoutDirective) -> PaneAdaptStrategy + 'static,
    ) -> Self {
        self.rule = Some(Box::new(rule));
        self
    }

    /// Supplies a modal backdrop for the levitated presentation.
    pub fn levitate_scrim(mut self, scrim: impl IntoElement) -> Self {
        self.scrim = Some(scrim.into_any_element());
        self
    }

    /// Sets the focus target focused when this pane opens as an active or levitated overlay.
    pub fn entry_focus(mut self, focus: &FocusHandle) -> Self {
        self.entry_focus = Some(focus.clone());
        self
    }

    pub(in crate::navigation) fn has_adaptation(&self) -> bool {
        self.rule.is_some() || self.adapt.is_some()
    }

    pub(super) fn strategy(
        &self,
        size: Size<Pixels>,
        directive: &PaneLayoutDirective,
    ) -> PaneAdaptStrategy {
        self.rule
            .as_ref()
            .map_or(self.adapt.unwrap_or_default(), |rule| rule(size, directive))
    }
}

/// An opt-in resize handle between two adjacent logical roles.
pub struct PaneSeparator {
    pub(super) label: SharedString,
    pub(super) anchors: Vec<PaneExpansionAnchor>,
    pub(super) keyboard_step: Pixels,
    pub(super) build: Option<HandleBuilder>,
}

impl PaneSeparator {
    /// Creates a resize separator with the given accessible label and an 8 px default keyboard step.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            anchors: Vec::new(),
            keyboard_step: px(8.),
            build: None,
        }
    }

    /// Sets snap anchors offered when dragging or stepping across the split boundary.
    pub fn anchors(mut self, anchors: impl IntoIterator<Item = PaneExpansionAnchor>) -> Self {
        self.anchors = anchors.into_iter().collect();
        self
    }

    /// Sets the logical pixel distance moved per arrow-key press (default 8 px).
    pub fn keyboard_step(mut self, step: Pixels) -> Self {
        self.keyboard_step = step;
        self
    }

    /// Configures the existing Material handle's appearance or interaction source.
    pub fn handle(
        mut self,
        build: impl FnOnce(VerticalDragHandle) -> VerticalDragHandle + 'static,
    ) -> Self {
        self.build = Some(Box::new(build));
        self
    }
}

pub(super) fn pair_index(start: PaneRole, end: PaneRole) -> usize {
    start.index() * 3 + end.index()
}
