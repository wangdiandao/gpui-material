use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{App, ElementId, EntityId, FocusHandle, Window};

/// Shared open/overflow state controller for responsive overflow strips (`AppBarRow`/`AppBarColumn`, `Breadcrumbs`).
#[derive(Clone)]
pub(super) struct OverflowMenuController {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    view: EntityId,
}

impl OverflowMenuController {
    pub(super) fn is_showing(&self) -> bool {
        self.shown.get()
    }

    pub(super) fn show(&self, window: &mut Window, cx: &mut App) {
        if self.has_overflow.get() {
            self.set(true, window, cx);
        }
    }

    pub(super) fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.set(false, window, cx);
    }

    pub(super) fn set(&self, shown: bool, window: &mut Window, cx: &mut App) {
        if self.shown.replace(shown) != shown {
            cx.notify(self.view);
            window.refresh();
        }
    }
}

struct OverflowState<K> {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    focus: FocusHandle,
    displaced: Rc<RefCell<Option<(K, FocusHandle)>>>,
}

/// Active per-render handle to a keyed overflow strip's menu and displaced-focus state.
pub(super) struct OverflowStripHandle<K> {
    pub(super) controller: OverflowMenuController,
    pub(super) focus: FocusHandle,
    displaced: Rc<RefCell<Option<(K, FocusHandle)>>>,
}

impl<K: Clone + PartialEq + 'static> OverflowStripHandle<K> {
    pub(super) fn use_state(
        owner_id: &ElementId,
        suffix: &'static str,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let state =
            window.use_keyed_state((owner_id.clone(), suffix), cx, |_, cx| OverflowState::<K> {
                shown: Rc::new(Cell::new(false)),
                has_overflow: Rc::new(Cell::new(false)),
                focus: cx.focus_handle(),
                displaced: Rc::new(RefCell::new(None)),
            });
        let state = state.read(cx);
        Self {
            controller: OverflowMenuController {
                shown: state.shown.clone(),
                has_overflow: state.has_overflow.clone(),
                view: window.current_view(),
            },
            focus: state.focus.clone(),
            displaced: state.displaced.clone(),
        }
    }

    /// Transfers focus to the overflow trigger when a focused inline item is hidden,
    /// and restores focus to the displaced item (or first fallback) when space returns.
    pub(super) fn transfer_focus(
        &self,
        has_overflow: bool,
        hidden_focused: Option<(K, FocusHandle)>,
        is_visible_enabled: impl Fn(&K) -> bool,
        first_visible_fallback: impl FnOnce() -> Option<FocusHandle>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let in_overflow = self.focus.is_focused(window);
        let previous_overflow = self.controller.has_overflow.replace(has_overflow);
        let menu_was_open = self.controller.is_showing();
        let mut displaced = self.displaced.borrow_mut();

        if has_overflow && let Some((hidden_id, fallback_focus)) = hidden_focused {
            let old_focus = window.focused(cx).unwrap_or(fallback_focus);
            *displaced = Some((hidden_id, old_focus));
            focus_after_layout(self.focus.clone(), window, cx);
            return;
        }

        if !has_overflow && previous_overflow {
            self.controller.shown.set(false);
        }

        let returned = displaced
            .as_ref()
            .filter(|(id, _)| is_visible_enabled(id))
            .map(|(_, focus)| focus.clone());

        let restore = if !has_overflow && (in_overflow || menu_was_open) {
            returned.or_else(first_visible_fallback)
        } else if in_overflow {
            returned
        } else {
            None
        };

        if let Some(focus) = restore {
            displaced.take();
            focus_after_layout(focus, window, cx);
        } else if !has_overflow {
            displaced.take();
        }
    }
}

pub(super) fn focus_after_layout(target: FocusHandle, window: &mut Window, cx: &mut App) {
    let origin = window.focused(cx);
    window.on_next_frame(move |window, cx| {
        let current = window.focused(cx);
        if current.is_none() || current == origin {
            target.focus(window, cx);
        }
    });
}
