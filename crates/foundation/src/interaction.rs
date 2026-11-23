use std::{cell::Cell, collections::HashMap};

use gpui::{App, ElementId, Entity, EntityId, EventEmitter, WeakEntity, Window, WindowId};

use crate::InteractionState;

/// Observes the presentation state of controls with a shared GPUI entity.
///
/// A source can observe multiple controls. Give each control a distinct
/// element ID within each window that uses the source.
#[derive(Default)]
pub struct InteractionSource {
    windows: HashMap<WindowId, HashMap<ElementId, (EntityId, InteractionState)>>,
}

/// One deferred change from an observed control.
///
/// For a fast sequence, the event's snapshot can precede the latest state
/// returned by [`InteractionSource::state`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InteractionUpdate {
    /// Window containing the observed control.
    pub window: WindowId,
    /// Element identifier of the observed control within [`Self::window`].
    pub id: ElementId,
    /// `None` means the control was detached from this source.
    pub state: Option<InteractionState>,
}

impl EventEmitter<InteractionUpdate> for InteractionSource {}

impl InteractionSource {
    /// Creates an empty source for use in a caller-owned GPUI entity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the most recently delivered state for a control in a window.
    pub fn state(&self, window: WindowId, id: &ElementId) -> Option<InteractionState> {
        self.windows.get(&window)?.get(id).map(|(_, state)| *state)
    }

    /// Binds an observer to a control during its rendering.
    ///
    /// Record the control's initial state during rendering and later changes
    /// from its existing input handlers. The tracker expires when the control
    /// stops rendering with this source.
    pub fn track(
        source: &Entity<Self>,
        id: &ElementId,
        window: &mut Window,
        cx: &mut App,
    ) -> InteractionTracker {
        let window_id = window.window_handle().window_id();
        let source_id = source.entity_id();
        let attachment = window.with_id((id.clone(), "material-interaction-source"), |window| {
            window.use_keyed_state(("source", source_id), cx, |_, cx| {
                cx.on_release(|this: &mut Attachment, cx| {
                    let source = this.source.clone();
                    let window = this.window;
                    let id = this.id.clone();
                    let owner = this.owner;
                    cx.defer(move |cx| {
                        source
                            .update(cx, |source, cx| {
                                if source.detach(window, &id, owner) {
                                    cx.emit(InteractionUpdate {
                                        window,
                                        id,
                                        state: None,
                                    });
                                    cx.notify();
                                }
                            })
                            .ok();
                    });
                })
                .detach();
                Attachment {
                    source: source.downgrade(),
                    window: window_id,
                    id: id.clone(),
                    owner: cx.entity_id(),
                    last: Cell::new(None),
                }
            })
        });
        InteractionTracker(attachment.downgrade())
    }

    fn detach(&mut self, window: WindowId, id: &ElementId, owner: EntityId) -> bool {
        let Some(controls) = self.windows.get_mut(&window) else {
            return false;
        };
        if controls
            .get(id)
            .is_none_or(|(current, _)| *current != owner)
        {
            return false;
        }
        controls.remove(id);
        if controls.is_empty() {
            self.windows.remove(&window);
        }
        true
    }
}

struct Attachment {
    source: WeakEntity<InteractionSource>,
    window: WindowId,
    id: ElementId,
    owner: EntityId,
    last: Cell<Option<InteractionState>>,
}

/// A control's binding to an interaction source.
/// The tracker observes an existing input path and has no effect on activation.
#[derive(Clone)]
pub struct InteractionTracker(WeakEntity<Attachment>);

impl InteractionTracker {
    /// Records one changed snapshot. Events are delivered in recording order
    /// after the current GPUI operation, even when several changes share a draw.
    pub fn record(&self, state: InteractionState, cx: &mut App) {
        let Some(attachment) = self.0.upgrade() else {
            return;
        };
        if attachment.read(cx).last.replace(Some(state)) == Some(state) {
            return;
        }
        let attachment = self.0.clone();
        cx.defer(move |cx| {
            let Some(attachment) = attachment.upgrade() else {
                return;
            };
            let attachment = attachment.read(cx);
            let source = attachment.source.clone();
            let window = attachment.window;
            let id = attachment.id.clone();
            let owner = attachment.owner;
            source
                .update(cx, |source, cx| {
                    source
                        .windows
                        .entry(window)
                        .or_default()
                        .insert(id.clone(), (owner, state));
                    cx.emit(InteractionUpdate {
                        window,
                        id,
                        state: Some(state),
                    });
                    cx.notify();
                })
                .ok();
        });
    }
}
