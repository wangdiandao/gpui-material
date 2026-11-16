use gpui::{
    App, ElementId, Entity, EntityId, IntoElement, Pixels, RenderOnce, SharedString, Subscription,
    Window,
};
use gpui_material_foundation::InteractionSource;

use super::{Tooltip, TooltipCaret, TooltipPlacement};

/// An overlay annotation for arbitrary anchor content and its own interaction source.
#[derive(IntoElement)]
pub struct Label {
    tooltip: Tooltip,
    source: Option<(Entity<InteractionSource>, ElementId)>,
    persistent: bool,
}

impl Label {
    pub fn new(
        id: impl Into<ElementId>,
        anchor: impl IntoElement,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            tooltip: Tooltip::plain(id, anchor, description),
            source: None,
            persistent: false,
        }
    }

    /// Supplies decorative content. The constructor's description names the overlay.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.tooltip = self.tooltip.body_content(content);
        self
    }

    /// Observes one control under the given ID in this label's window.
    pub fn interaction_source(
        mut self,
        source: &Entity<InteractionSource>,
        control_id: impl Into<ElementId>,
    ) -> Self {
        self.source = Some((source.clone(), control_id.into()));
        self
    }

    /// Shows the label continuously while mounted.
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.tooltip = self.tooltip.placement(placement);
        self
    }

    pub fn offset(mut self, offset: Pixels) -> Self {
        self.tooltip = self.tooltip.offset(offset);
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.tooltip = self.tooltip.max_width(width);
        self
    }

    pub fn caret(mut self, caret: TooltipCaret) -> Self {
        self.tooltip = self.tooltip.caret(caret);
        self
    }
}

impl RenderOnce for Label {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let observed = self.source.as_ref().filter(|_| !self.persistent);
        let observation = window.use_keyed_state(
            (self.tooltip.id.clone(), "label-interaction-source"),
            cx,
            |_, _| None::<(EntityId, Subscription)>,
        );
        let source_id = observed.map(|(source, _)| source.entity_id());
        observation.update(cx, |observation, cx| {
            if observation.as_ref().map(|(id, _)| *id) != source_id {
                *observation = observed.map(|(source, _)| {
                    (
                        source.entity_id(),
                        window.observe(source, cx, |_, window, _| window.refresh()),
                    )
                });
            }
        });
        let window_id = window.window_handle().window_id();
        let active = observed
            .and_then(|(source, control)| source.read(cx).state(window_id, control))
            .is_some_and(|state| {
                !state.disabled
                    && (state.hovered || state.pressed || state.dragged || state.focus_visible)
            });
        self.tooltip.label_visible = Some(self.persistent || active);
        self.tooltip.render(window, cx)
    }
}
