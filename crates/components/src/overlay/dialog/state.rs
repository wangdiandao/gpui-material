use gpui::{
    AnyWindowHandle, App, Context, Entity, FocusHandle, IntoElement, ParentElement, Render,
    ScrollHandle, Styled, Subscription, WeakEntity, Window, div,
};
use gpui_base::active_focus_trap;

use super::{Dialog, DialogCloseReason, ModalClosed, ModalDismissRequest};

pub(super) struct NativeDialog {
    pub(super) parent: AnyWindowHandle,
    pub(super) child: AnyWindowHandle,
    pub(super) _closed: Subscription,
}

/// Retains a dialog's visibility and focus scope across renders.
pub struct DialogState {
    pub(in crate::overlay) focus: FocusHandle,
    pub(super) previous: Option<FocusHandle>,
    pub(super) cancel_focus: FocusHandle,
    pub(super) confirm_focus: FocusHandle,
    pub(super) open: bool,
    generation: u64,
    pub(super) last_close_reason: Option<DialogCloseReason>,
    pub(super) native: Option<NativeDialog>,
    pub(in crate::overlay) scroll: ScrollHandle,
}

impl DialogState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous: None,
            cancel_focus: cx.focus_handle(),
            confirm_focus: cx.focus_handle(),
            open: false,
            generation: 0,
            last_close_reason: None,
            native: None,
            scroll: ScrollHandle::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the most recent accepted close, including programmatic closes.
    pub fn last_close_reason(&self) -> Option<DialogCloseReason> {
        self.last_close_reason
    }

    pub(super) fn begin(
        &mut self,
        previous: Option<FocusHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.previous = previous;
        self.open = true;
        self.generation = self.generation.wrapping_add(1);
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Focuses the modal container. A caller can then focus a child editor.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.begin(window.focused(cx), window, cx);
        }
    }

    /// Closes the dialog and restores its previous focus if it still owns focus.
    /// A programmatic close updates the reason without firing the user close callback.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finish(DialogCloseReason::Programmatic, window, cx);
    }

    fn finish(&mut self, reason: DialogCloseReason, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        self.last_close_reason = Some(reason);
        let previous = self.previous.take();
        if let Some(native) = self.native.take() {
            if reason != DialogCloseReason::WindowClose {
                if native.child.window_id() == window.window_handle().window_id() {
                    window.remove_window();
                } else {
                    native
                        .child
                        .update(cx, |_, window, _| window.remove_window())
                        .ok();
                }
            }
            restore_parent(native.parent, previous, cx);
        } else if self.focus.contains_focused(window, cx) || self.focus.is_focused(window) {
            if let Some(previous) = previous {
                previous.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        cx.notify();
    }
}

pub(super) fn restore_parent(parent: AnyWindowHandle, previous: Option<FocusHandle>, cx: &mut App) {
    if let Some(previous) = previous {
        cx.defer(move |cx| {
            parent
                .update(cx, |_, window, cx| {
                    if window.focused(cx).is_none() {
                        previous.focus(window, cx);
                    }
                })
                .ok();
        });
    }
}

#[derive(Clone)]
pub(in crate::overlay) struct ModalSession {
    state: WeakEntity<DialogState>,
    generation: u64,
    dismiss_request: Option<ModalDismissRequest>,
    on_close: Option<ModalClosed>,
}

impl ModalSession {
    pub(in crate::overlay) fn new(
        state: &Entity<DialogState>,
        dismiss_request: Option<ModalDismissRequest>,
        on_close: Option<ModalClosed>,
        cx: &App,
    ) -> Self {
        Self {
            state: state.downgrade(),
            generation: state.read(cx).generation,
            dismiss_request,
            on_close,
        }
    }

    pub(super) fn current(&self, window: &Window, cx: &App) -> bool {
        self.state.upgrade().is_some_and(|state| {
            let dialog = state.read(cx);
            dialog.open
                && dialog.generation == self.generation
                && dialog.focus.contains_focused(window, cx)
                && active_focus_trap(window, cx).is_none_or(|focus| focus == dialog.focus)
        })
    }

    pub(in crate::overlay) fn dismiss(
        &self,
        reason: DialogCloseReason,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        self.current(window, cx)
            && self
                .dismiss_request
                .as_ref()
                .is_none_or(|request| request(reason, window, cx))
            && self.finish(reason, window, cx)
    }

    pub(super) fn finish(
        &self,
        reason: DialogCloseReason,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        let accepted = state.update(cx, |state, cx| {
            if !state.open || state.generation != self.generation {
                return false;
            }
            state.finish(reason, window, cx);
            true
        });
        if accepted && let Some(on_close) = &self.on_close {
            on_close(reason, window, cx);
        }
        accepted
    }
}

pub(super) type BuildNativeDialog =
    Box<dyn Fn(&Entity<DialogState>, &mut Window, &mut App) -> Dialog>;

pub(super) struct NativeDialogView {
    pub(super) state: Entity<DialogState>,
    pub(super) build: BuildNativeDialog,
}

impl Render for NativeDialogView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child((self.build)(&self.state, window, cx))
    }
}
