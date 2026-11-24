mod element;
mod state;

use std::{cell::RefCell, rc::Rc};

use gpui::InteractiveElement as _;
use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, ParentElement, RenderOnce,
    Styled, Subscription, Window, div,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::current_material_theme;

use element::SwipeContent;
use state::{Dismissed, SwipeOptions};

pub use state::{SwipeToDismissState, SwipeToDismissValue};

type OnDismiss = Rc<dyn Fn(SwipeToDismissValue, &mut Window, &mut App)>;

struct SwipeMount {
    callback: Rc<RefCell<Option<OnDismiss>>>,
    focus: FocusHandle,
    _dismiss: Subscription,
    _activation: Subscription,
}

/// A foreground and background with logical horizontal swipe gestures.
/// The caller owns the content, action commands, and resulting data changes.
#[derive(IntoElement)]
pub struct SwipeToDismiss {
    id: ElementId,
    state: Entity<SwipeToDismissState>,
    content: AnyElement,
    background: Option<AnyElement>,
    options: SwipeOptions,
    on_dismiss: Option<OnDismiss>,
}

impl SwipeToDismiss {
    /// Creates a swipe-to-dismiss container wrapping `content` and bound to `state`.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<SwipeToDismissState>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            content: content.into_any_element(),
            background: None,
            options: SwipeOptions::default(),
            on_dismiss: None,
        }
    }

    /// Sets the background element revealed behind the foreground while swiping.
    pub fn background(mut self, background: impl IntoElement) -> Self {
        self.background = Some(background.into_any_element());
        self
    }

    /// Enables or disables start-to-end dismissal gestures and programmatic dismissal. Defaults to `true`.
    pub fn enable_start_to_end(mut self, enabled: bool) -> Self {
        self.options.start_to_end = enabled;
        self
    }

    /// Enables or disables end-to-start dismissal gestures and programmatic dismissal. Defaults to `true`.
    pub fn enable_end_to_start(mut self, enabled: bool) -> Self {
        self.options.end_to_start = enabled;
        self
    }

    /// Enables direct gestures. Programmatic and accessible commands stay available.
    pub fn gestures_enabled(mut self, enabled: bool) -> Self {
        self.options.gestures_enabled = enabled;
        self
    }

    /// Runs once when the state settles at a new dismissal anchor.
    pub fn on_dismiss(
        mut self,
        on_dismiss: impl Fn(SwipeToDismissValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(on_dismiss));
        self
    }
}

impl RenderOnce for SwipeToDismiss {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.options.direction = current_material_theme(window, cx).layout.direction;
        let mount = window.with_element_namespace((self.id.clone(), "material-swipe"), |window| {
            window.use_keyed_state(("mount", self.state.entity_id()), cx, |window, cx| {
                let callback = Rc::new(RefCell::new(None::<OnDismiss>));
                let dismiss = window.subscribe(&self.state, cx, {
                    let callback = callback.clone();
                    move |_, event: &Dismissed, window, cx| {
                        let callback = callback.borrow().clone();
                        if let Some(callback) = callback {
                            callback(event.0, window, cx);
                        }
                    }
                });
                let state = self.state.downgrade();
                let activation = cx.observe_window_activation(window, {
                    let state = state.clone();
                    move |_, window, cx| {
                        if !window.is_window_active() {
                            let _ = state.update(cx, |state, cx| {
                                state.cancel_interaction(window, cx);
                            });
                        }
                    }
                });
                let handle = window.window_handle();
                let mount = cx.entity_id();
                cx.on_release(move |_: &mut SwipeMount, cx| {
                    let state = state.clone();
                    cx.defer(move |cx| {
                        let _ = handle.update(cx, |_, window, cx| {
                            let _ = state.update(cx, |state, cx| {
                                if state.mount.as_ref().is_some_and(|(id, _)| *id == mount) {
                                    state.mount = None;
                                    state.cancel_input(window, false);
                                    let settled = state.settled;
                                    state.settle(settled, window, cx);
                                }
                            });
                        });
                    });
                })
                .detach();
                SwipeMount {
                    callback,
                    focus: cx.focus_handle().tab_stop(false),
                    _dismiss: dismiss,
                    _activation: activation,
                }
            })
        });
        let mounted = mount.read(cx);
        mounted.callback.replace(self.on_dismiss);
        let focus = mounted.focus.clone();
        self.state.update(cx, |state, cx| {
            state.configure(self.options, (mount.entity_id(), focus.clone()), window, cx);
            state.sample_motion(&self.id, window, cx);
        });
        div()
            .id(self.id)
            .test_support()
            .overflow_hidden()
            .children(
                self.background
                    .map(|background| div().absolute().inset_0().child(background)),
            )
            .child(SwipeContent {
                content: self.content,
                state: self.state.downgrade(),
                focus,
            })
    }
}
