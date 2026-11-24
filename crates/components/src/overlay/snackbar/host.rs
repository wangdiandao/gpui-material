use std::{cell::Cell, collections::VecDeque, rc::Rc};

use gpui::{
    App, Context, FocusHandle, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Task, WeakFocusHandle, Window, canvas, div, point, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{OverlayMotion, ScaledOverlay};

use super::super::modal::ZLayer;
use super::render::snackbar_surface;
use super::{HostRenderer, Snackbar, SnackbarId, SnackbarParts, SnackbarResult};

const MAX_WIDTH: gpui::Pixels = px(672.);
const MAX_EXITING: usize = 4;

pub(super) struct Entry {
    pub(super) id: SnackbarId,
    pub(super) message: Snackbar,
    pub(super) controls: [FocusHandle; 2],
    return_focus: Option<WeakFocusHandle>,
}

/// One FIFO message queue per host. Mount the entity in the window layout.
/// A new message leaves keyboard focus wherever it is.
#[derive(Default)]
pub struct SnackbarHost {
    pub(super) active: Option<Entry>,
    exiting: Vec<Entry>,
    pending: VecDeque<(SnackbarId, Snackbar)>,
    next_id: usize,
    timer: Option<Task<()>>,
    hovered: bool,
    focused: bool,
    last_external_focus: Option<WeakFocusHandle>,
    _focus_subscriptions: Vec<Subscription>,
    renderer: Option<HostRenderer>,
}

impl SnackbarHost {
    /// Unified overlay z-layer priority for snackbars.
    #[allow(dead_code)]
    pub(crate) const Z_LAYER: ZLayer = ZLayer::SNACKBAR;

    /// Uses a custom container and layout for the host-owned message and controls.
    pub fn renderer<E: IntoElement>(
        mut self,
        render: impl Fn(&Snackbar, SnackbarParts, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.renderer = Some(Box::new(move |message, parts, window, cx| {
            render(message, parts, window, cx).into_any_element()
        }));
        self
    }

    /// Queues the message after any already pending messages.
    pub fn push(&mut self, snackbar: Snackbar, window: &mut Window, cx: &mut Context<Self>) {
        self.enqueue(snackbar, window, cx);
    }

    /// Queues a plain message snackbar and returns its opaque identity.
    pub fn push_message(
        &mut self,
        message: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SnackbarId {
        self.enqueue(Snackbar::new(message), window, cx)
    }

    /// Dismisses the currently active snackbar (if any, firing its dismissal
    /// completion cleanly) and immediately presents the new snackbar ahead of
    /// any pending entries.
    pub fn replace(
        &mut self,
        snackbar: Snackbar,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SnackbarId {
        self.next_id += 1;
        let id = SnackbarId {
            host: cx.entity_id(),
            sequence: self.next_id,
        };
        self.pending.push_front((id, snackbar));
        if let Some((active_id, _)) = self.current() {
            self.finish(active_id, SnackbarResult::Dismissed, window, cx);
        } else {
            self.show_next(window, cx);
        }
        id
    }

    /// Queues the message and returns an opaque identity scoped to this host.
    pub fn enqueue(
        &mut self,
        snackbar: Snackbar,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SnackbarId {
        self.next_id += 1;
        let id = SnackbarId {
            host: cx.entity_id(),
            sequence: self.next_id,
        };
        self.pending.push_back((id, snackbar));
        if self.active.is_none() {
            self.show_next(window, cx);
        } else {
            cx.notify();
        }
        id
    }

    /// Returns the active entry. Pending and exiting entries are excluded.
    pub fn current(&self) -> Option<(SnackbarId, &Snackbar)> {
        self.active.as_ref().map(|entry| (entry.id, &entry.message))
    }

    /// Returns the number of queued snackbars waiting behind the active entry.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Discards all queued snackbars waiting behind the active entry without
    /// disturbing the currently visible snackbar.
    pub fn clear_queue(&mut self, cx: &mut Context<Self>) {
        if !self.pending.is_empty() {
            self.pending.clear();
            cx.notify();
        }
    }

    /// Dismisses the active message. If any are queued, the next begins its own timeout.
    pub fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((id, _)) = self.current() {
            self.dismiss_message(id, window, cx);
        }
    }

    /// Dismisses the entry if it is still current.
    pub fn dismiss_message(
        &mut self,
        id: SnackbarId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.finish(id, SnackbarResult::Dismissed, window, cx)
    }

    /// Performs the entry's action if it is current and has an action.
    pub fn perform_action(
        &mut self,
        id: SnackbarId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.finish(id, SnackbarResult::ActionPerformed, window, cx)
    }

    fn show_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.hovered = false;
        self.focused = false;
        self._focus_subscriptions.clear();
        self.timer = None;
        self.last_external_focus = window.focused(cx).map(|focus| focus.downgrade());
        self.active = self.pending.pop_front().map(|(id, message)| Entry {
            id,
            message,
            controls: [cx.focus_handle(), cx.focus_handle()],
            return_focus: None,
        });
        if let Some(entry) = &self.active {
            let controls = entry.controls.clone();
            for handle in &controls {
                let enter = cx.weak_entity();
                let leave = enter.clone();
                self._focus_subscriptions.push(window.on_focus_in(
                    handle,
                    cx,
                    move |window, cx| {
                        let _ = enter.update(cx, |host, cx| host.refresh_focus(window, cx));
                    },
                ));
                self._focus_subscriptions.push(window.on_focus_out(
                    handle,
                    cx,
                    move |_, window, cx| {
                        let _ = leave.update(cx, |host, cx| host.refresh_focus(window, cx));
                    },
                ));
            }
        }
        self.sync_timer(window, cx);
        cx.notify();
    }

    pub(super) fn set_hovered(
        &mut self,
        hovered: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.hovered != hovered {
            self.hovered = hovered;
            self.sync_timer(window, cx);
        }
    }

    fn refresh_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focused = self
            .active
            .as_ref()
            .is_some_and(|entry| entry.controls.iter().any(|focus| focus.is_focused(window)));
        if self.focused != focused {
            self.focused = focused;
            if let Some(entry) = &mut self.active {
                if focused {
                    // Only capture return focus when focus actually enters the snackbar.
                    entry.return_focus = self.last_external_focus.clone();
                } else {
                    entry.return_focus = None;
                }
            }
            self.sync_timer(window, cx);
        }
    }

    fn sync_timer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.hovered || self.focused {
            self.timer = None;
            return;
        }
        if self.timer.is_some() {
            return;
        }
        self.timer = self.active.as_ref().and_then(|entry| {
            let id = entry.id;
            let timeout = entry.message.timeout?;
            Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(timeout).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.dismiss_message(id, window, cx);
                });
            }))
        });
    }

    pub(super) fn finish(
        &mut self,
        id: SnackbarId,
        result: SnackbarResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(active) = self.active.as_ref().filter(|entry| entry.id == id) else {
            return false;
        };
        let callback = match result {
            SnackbarResult::ActionPerformed => {
                let Some((_, callback)) = &active.message.action else {
                    return false;
                };
                Some(callback.clone())
            }
            SnackbarResult::Dismissed => active.message.on_dismiss.clone(),
        };
        let mut entry = self.active.take().expect("current entry checked above");
        let complete = entry.message.on_result.take();
        if entry.controls.iter().any(|focus| focus.is_focused(window)) {
            if let Some(restore) = entry.return_focus.take().and_then(|focus| focus.upgrade()) {
                restore.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        if !cx.reduce_motion() {
            if self.exiting.len() >= MAX_EXITING {
                self.exiting.remove(0);
            }
            self.exiting.push(entry);
        }
        self.show_next(window, cx);
        if callback.is_some() || complete.is_some() {
            window.defer(cx, move |window, cx| {
                if let Some(callback) = callback {
                    callback(window, cx);
                }
                if let Some(complete) = complete {
                    complete(result, window, cx);
                }
            });
        }
        true
    }
}

impl Render for SnackbarHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused
            && let Some(focused) = window.focused(cx)
            && self
                .active
                .as_ref()
                .is_none_or(|entry| !entry.controls.iter().any(|handle| handle == &focused))
        {
            self.last_external_focus = Some(focused.downgrade());
        }
        let host_width = window
            .use_keyed_state(("snackbar-host-width", cx.entity_id()), cx, |_, _| {
                Rc::new(Cell::new(px(0.)))
            })
            .read(cx)
            .clone();
        let measured_width = host_width.get();
        let width = if measured_width > px(0.) {
            measured_width
        } else {
            (window.viewport_size().width - px(32.)).clamp(px(0.), MAX_WIDTH)
        };
        let mut root = div().relative().w_full().child(
            canvas(
                move |bounds, window, _| {
                    if host_width.replace(bounds.size.width) != bounds.size.width {
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );
        let mut has_layout = self.active.is_some();
        let renderer = self.renderer.as_ref();
        self.exiting.retain(|entry| {
            let Some(motion) =
                OverlayMotion::sample(&("snackbar", entry.id.sequence).into(), false, window, cx)
            else {
                return false;
            };
            let content = snackbar_surface(entry, motion, width, false, renderer, window, cx);
            let scaled = ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width);
            let slot = div().when(has_layout, |div| div.absolute().inset_0());
            root.extend([slot.child(scaled).into_any_element()]);
            has_layout = true;
            true
        });
        if let Some(entry) = &self.active {
            let motion =
                OverlayMotion::sample(&("snackbar", entry.id.sequence).into(), true, window, cx)
                    .expect("active snackbar has motion");
            let content = snackbar_surface(entry, motion, width, true, renderer, window, cx);
            root =
                root.child(ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width));
        }
        root
    }
}
