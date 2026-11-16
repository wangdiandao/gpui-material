use std::time::Duration;

use gpui::{App, Context, Entity, EntityId, FocusHandle, Subscription, Task, Window};

use super::TooltipCallback;

/// Visibility retained by the owner of a persistent rich tooltip.
#[derive(Default)]
pub struct TooltipState {
    visible: bool,
}

impl TooltipState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the requested visibility.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.set_visible(true, cx);
    }

    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.set_visible(false, cx);
    }

    fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self.visible != visible {
            self.visible = visible;
            cx.notify();
        }
    }
}

pub(super) struct Interaction {
    open: bool,
    trigger_hover: bool,
    surface_hover: bool,
    keyboard: bool,
    suppress_focus: bool,
    surface_focus: bool,
    transient: bool,
    task: Option<Task<()>>,
    trigger: Option<(FocusHandle, [Subscription; 2])>,
    visibility: Option<(EntityId, Subscription)>,
    pub(super) surface: FocusHandle,
    _surface_tracking: [Subscription; 2],
    presented: bool,
    previous_focus: Option<FocusHandle>,
    pub(super) pressed: bool,
}

impl Interaction {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let surface = cx.focus_handle().tab_stop(false);
        let enter = cx.weak_entity();
        let leave = enter.clone();
        let on_enter = window.on_focus_in(&surface, cx, move |window, cx| {
            let _ = enter.update(cx, |state, cx| {
                state.surface_focus = true;
                state.schedule(window, cx);
            });
        });
        let on_leave = window.on_focus_out(&surface, cx, move |_, window, cx| {
            let _ = leave.update(cx, |state, cx| {
                state.surface_focus = false;
                state.schedule(window, cx);
            });
        });
        Self {
            open: false,
            trigger_hover: false,
            surface_hover: false,
            keyboard: false,
            suppress_focus: false,
            surface_focus: false,
            transient: false,
            task: None,
            trigger: None,
            visibility: None,
            surface,
            _surface_tracking: [on_enter, on_leave],
            presented: false,
            previous_focus: None,
            pressed: false,
        }
    }

    pub(super) fn configure(
        &mut self,
        transient: bool,
        trigger: Option<&FocusHandle>,
        visibility: Option<&Entity<TooltipState>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.transient != transient {
            self.transient = transient;
            self.open = false;
            self.task = None;
            self.trigger_hover = false;
            self.surface_hover = false;
            self.keyboard = false;
        }
        let tracked = trigger.filter(|_| transient);
        if self.trigger.as_ref().map(|(focus, _)| focus) != tracked {
            self.trigger = tracked.map(|focus| {
                let enter = cx.weak_entity();
                let leave = enter.clone();
                let on_enter = window.on_focus_in(focus, cx, move |window, cx| {
                    let _ = enter.update(cx, |state, cx| {
                        state.keyboard = !state.suppress_focus;
                        state.schedule(window, cx);
                    });
                });
                let on_leave = window.on_focus_out(focus, cx, move |_, window, cx| {
                    let _ = leave.update(cx, |state, cx| {
                        state.keyboard = false;
                        state.suppress_focus = false;
                        state.schedule(window, cx);
                    });
                });
                self.keyboard = focus.contains_focused(window, cx);
                (focus.clone(), [on_enter, on_leave])
            });
            if tracked.is_none() {
                self.keyboard = false;
            }
        }
        if self.visibility.as_ref().map(|(id, _)| *id) != visibility.map(Entity::entity_id) {
            self.visibility = visibility.map(|state| {
                (
                    state.entity_id(),
                    window.observe(state, cx, |_, window, _| window.refresh()),
                )
            });
        }
    }

    pub(super) fn hover(
        &mut self,
        surface: bool,
        hovering: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if surface {
            self.surface_hover = hovering;
        } else {
            self.trigger_hover = hovering;
        }
        self.schedule(window, cx);
    }

    fn schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.transient {
            return;
        }
        let open = self.trigger_hover || self.surface_hover || self.keyboard || self.surface_focus;
        self.task = None;
        if open == self.open {
            return;
        }
        if self.keyboard && open {
            self.open = true;
            window.refresh();
            return;
        }
        let delay = Duration::from_millis(if open { 500 } else { 200 });
        self.task = Some(cx.spawn_in(window, async move |state, cx| {
            cx.background_executor().timer(delay).await;
            let _ = state.update_in(cx, |state, window, _| {
                state.task = None;
                state.open = open;
                window.refresh();
            });
        }));
    }

    pub(super) fn is_open(&self, window: &Window, cx: &App) -> bool {
        self.transient
            && (self.open
                || self.keyboard
                || !self.suppress_focus
                    && (self.surface.contains_focused(window, cx)
                        || self
                            .trigger
                            .as_ref()
                            .is_some_and(|(trigger, _)| trigger.is_focused(window))))
    }

    fn dismiss(&mut self, window: &mut Window) {
        self.open = false;
        self.task = None;
        self.trigger_hover = false;
        self.surface_hover = false;
        self.keyboard = false;
        self.surface_focus = false;
        self.suppress_focus = true;
        window.refresh();
    }

    pub(super) fn present(
        &mut self,
        open: bool,
        trigger: Option<&FocusHandle>,
        enter: Option<FocusHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.presented == open {
            return;
        }
        self.presented = open;
        if open {
            self.previous_focus = window.focused(cx);
            if let Some(enter) = enter {
                let state = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| state.read(cx).presented)
                    {
                        enter.focus(window, cx);
                    }
                });
            }
        } else {
            let previous = self.previous_focus.take();
            if self.surface.contains_focused(window, cx)
                && let Some(focus) = trigger.cloned().or(previous)
            {
                self.suppress_focus = true;
                let state = cx.weak_entity();
                window.defer(cx, move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| !state.read(cx).presented)
                    {
                        focus.focus(window, cx);
                    }
                });
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct Dismiss {
    pub(super) transient: Entity<Interaction>,
    pub(super) persistent: Option<Entity<TooltipState>>,
    pub(super) request: Option<TooltipCallback>,
}

impl Dismiss {
    pub(super) fn request(&self, window: &mut Window, cx: &mut App) {
        if let Some(state) = &self.persistent {
            if !state.read(cx).is_visible() {
                return;
            }
            if let Some(callback) = &self.request {
                callback(window, cx);
            } else {
                state.update(cx, |state, cx| state.dismiss(cx));
            }
        } else if self.transient.read(cx).is_open(window, cx) {
            self.transient.update(cx, |state, _| state.dismiss(window));
            if let Some(callback) = &self.request {
                callback(window, cx);
            }
        }
    }
}
