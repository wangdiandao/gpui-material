use std::rc::{Rc, Weak};

use gpui::{
    AppContext, Context, ElementId, Entity, EntityInputHandler, FocusHandle, Focusable,
    ListAlignment, ListOffset, ListState, Pixels, SharedString, Subscription, Window, px,
};

use super::{
    super::{TextFieldEvent, TextFieldState},
    Dismiss, DropdownOption, OpenChange,
};

#[derive(Default)]
pub(super) struct Visibility {
    pub(super) controlled: Option<bool>,
    pub(super) change: Option<OpenChange>,
    pub(super) dismiss: Option<Dismiss>,
}

#[derive(Clone, Copy)]
pub(super) enum Opening {
    Editor,
    PopupFromPrimary,
    PopupFromSecondary,
}

/// Retains the filtered model independently of the visible rows. Comparisons
/// use stable IDs and labels; only a changed collection or query is filtered.
#[derive(Default)]
pub(super) struct OptionSnapshot {
    pub(super) source: Rc<[DropdownOption]>,
    query: Option<SharedString>,
    pub(super) visible: Rc<[DropdownOption]>,
}

impl OptionSnapshot {
    pub(super) fn reconcile(
        &mut self,
        source: &Rc<[DropdownOption]>,
        query: Option<SharedString>,
    ) -> bool {
        let source_unchanged =
            Rc::ptr_eq(&self.source, source) || self.source.as_ref() == source.as_ref();
        if !Rc::ptr_eq(&self.source, source) {
            self.source = source.clone();
        }
        if source_unchanged && self.query == query {
            return false;
        }
        self.visible = if let Some(query) = &query {
            let query_lower = query.to_lowercase();
            self.source
                .iter()
                .filter(|option| contains_ignore_case(&option.label, query, &query_lower))
                .cloned()
                .collect()
        } else {
            self.source.clone()
        };
        self.query = query;
        true
    }
}

pub(super) fn contains_ignore_case(haystack: &str, needle_raw: &str, needle_lower: &str) -> bool {
    if haystack.is_ascii() && needle_raw.is_ascii() {
        needle_raw.is_empty()
            || haystack
                .as_bytes()
                .windows(needle_raw.len())
                .any(|window| window.eq_ignore_ascii_case(needle_raw.as_bytes()))
    } else {
        haystack.to_lowercase().contains(needle_lower)
    }
}

/// Keeps the editable query and pending keyboard choice across renders.
pub struct ExposedDropdownState {
    pub(super) input: Entity<TextFieldState>,
    pub(super) committed: SharedString,
    pub(super) editable: bool,
    pub(super) open: bool,
    presented_open: bool,
    visibility: Weak<Visibility>,
    primary_enabled: bool,
    opening: Option<Opening>,
    pub(super) popup_focus: FocusHandle,
    pub(super) secondary_focus: FocusHandle,
    pub(super) restore_focus: Option<FocusHandle>,
    focus_epoch: u64,
    pub(super) active: Option<ElementId>,
    active_index: Option<usize>,
    pub(super) selected: Option<ElementId>,
    pub(super) scroll: ListState,
    presented: Rc<[DropdownOption]>,
    metrics: Option<(Pixels, f32, Pixels, gpui_material_theme::Typography, u64)>,
    options: OptionSnapshot,
    _input: Subscription,
}

impl ExposedDropdownState {
    pub fn new(
        value: impl Into<SharedString>,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let value = value.into();
        let input = cx.new(|cx| TextFieldState::new(window, cx).default_value(value));
        Self::with_editor(&input, editable, window, cx)
    }

    /// Uses a caller-retained single-line editor and its current value.
    pub fn with_editor(
        input: &Entity<TextFieldState>,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let committed = input.read(cx).value();
        input.update(cx, |input, cx| input.set_readonly(!editable, cx));
        let subscription = cx.subscribe_in(input, window, |state, input, event, window, cx| {
            if !state.editable || !matches!(event, TextFieldEvent::Change) {
                return;
            }
            if input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            }) {
                return;
            }
            if input.read(cx).value() != state.committed {
                state.active = None;
                state.active_index = None;
                if state.primary_enabled {
                    state.request_open(true, Some(Opening::Editor), window, cx);
                }
            }
            cx.notify();
        });
        Self {
            input: input.clone(),
            committed,
            editable,
            open: false,
            presented_open: false,
            visibility: Weak::new(),
            primary_enabled: true,
            opening: None,
            popup_focus: cx.focus_handle(),
            secondary_focus: cx.focus_handle(),
            restore_focus: None,
            focus_epoch: 0,
            active: None,
            active_index: None,
            selected: None,
            scroll: ListState::new(0, ListAlignment::Top, px(280.)),
            presented: Rc::default(),
            metrics: None,
            options: OptionSnapshot::default(),
            _input: subscription,
        }
    }

    /// Synchronizes the label and dismisses an internally controlled list.
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        if self.committed != value {
            self.selected = None;
        }
        self.committed = value;
        if self
            .visibility
            .upgrade()
            .is_none_or(|policy| policy.controlled.is_none())
        {
            self.open = false;
        }
        self.reset_draft(window, cx);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn selected_option_id(&self) -> Option<&ElementId> {
        self.selected.as_ref()
    }

    /// The retained variable-height list position, including while closed.
    pub fn scroll_state(&self) -> &ListState {
        &self.scroll
    }

    /// Uses a caller-owned list position. The dropdown owns its item count.
    pub fn set_scroll_state(&mut self, scroll: &ListState, cx: &mut Context<Self>) {
        let anchor = scroll.logical_scroll_top();
        scroll.reset(self.presented.len());
        scroll.scroll_to(anchor);
        self.scroll = scroll.clone();
        self.metrics = None;
        cx.notify();
    }

    pub(super) fn reset_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = None;
        self.active_index = None;
        self.input.update(cx, |input, cx| {
            if input.value() != self.committed {
                input.set_value(self.committed.clone(), window, cx);
            }
        });
        cx.notify();
    }

    pub(super) fn request_open(
        &mut self,
        open: bool,
        opening: Option<Opening>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open == open {
            return;
        }
        self.opening = opening;
        let visibility = self.visibility.upgrade();
        if visibility
            .as_ref()
            .is_none_or(|policy| policy.controlled.is_none())
        {
            self.open = open;
            if !open {
                self.reset_draft(window, cx);
            }
        }
        if let Some(visibility) = visibility
            && (visibility.change.is_some() || !open && visibility.dismiss.is_some())
        {
            window.defer(cx, move |window, cx| {
                if let Some(change) = &visibility.change {
                    change(open, window, cx);
                }
                if !open && let Some(dismiss) = &visibility.dismiss {
                    dismiss(window, cx);
                }
            });
        }
        cx.notify();
    }

    pub(super) fn present(
        &mut self,
        visibility: &Rc<Visibility>,
        primary_enabled: bool,
        secondary_enabled: bool,
        primary: Opening,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.primary_enabled = primary_enabled;
        self.open = visibility.controlled.unwrap_or(self.open);
        self.visibility = Rc::downgrade(visibility);
        let opening = self.opening.take();
        if self.presented_open == self.open {
            if !self.open {
                self.active = None;
                self.active_index = None;
            }
            return;
        }
        self.presented_open = self.open;
        self.focus_epoch = self.focus_epoch.wrapping_add(1);
        let epoch = self.focus_epoch;
        if self.open {
            let editor = self.input.read(cx).focus_handle(cx);
            let (focus, restore) = match opening.unwrap_or(primary) {
                Opening::Editor => (editor.clone(), None),
                Opening::PopupFromPrimary => (self.popup_focus.clone(), Some(editor)),
                Opening::PopupFromSecondary => {
                    (self.popup_focus.clone(), Some(self.secondary_focus.clone()))
                }
            };
            self.restore_focus = restore;
            if !focus.is_focused(window) {
                let state = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| state.read(cx).focus_epoch == epoch)
                    {
                        focus.focus(window, cx);
                    }
                });
            }
        } else {
            self.reset_draft(window, cx);
            if let Some(mut restore) = self.restore_focus.take()
                && self.popup_focus.contains_focused(window, cx)
            {
                if !secondary_enabled && restore == self.secondary_focus {
                    restore = self.input.read(cx).focus_handle(cx);
                }
                let popup = self.popup_focus.clone();
                let state = cx.weak_entity();
                window.defer(cx, move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| state.read(cx).focus_epoch == epoch)
                        && popup.contains_focused(window, cx)
                    {
                        restore.focus(window, cx);
                    }
                });
            }
        }
    }

    fn active_position(&self, choices: &[DropdownOption]) -> Option<usize> {
        let id = self.active.as_ref()?;
        self.active_index
            .filter(|&index| choices.get(index).is_some_and(|choice| &choice.id == id))
            .or_else(|| choices.iter().position(|choice| &choice.id == id))
    }

    pub(super) fn reconcile_options(
        &mut self,
        source: &Rc<[DropdownOption]>,
        query: Option<SharedString>,
    ) -> (Rc<[DropdownOption]>, Option<ElementId>, Option<ElementId>) {
        if self.options.reconcile(source, query) {
            self.active_index = self.active_position(&self.options.visible);
            if self.active_index.is_none() {
                self.active = None;
            }
        }
        (
            self.options.visible.clone(),
            self.active.clone(),
            self.selected.clone(),
        )
    }

    pub(super) fn sync_presented_list(
        &mut self,
        options: &Rc<[DropdownOption]>,
        active: Option<&ElementId>,
        metrics: (Pixels, f32, Pixels, gpui_material_theme::Typography, u64),
    ) {
        if !Rc::ptr_eq(&self.presented, options) {
            let old_top = self.scroll.logical_scroll_top();
            let old_id = self.presented.get(old_top.item_ix).map(|option| &option.id);
            let anchor = if active.is_some() && Rc::ptr_eq(options, &self.options.visible) {
                self.active_position(options)
            } else {
                active
                    .or(old_id)
                    .and_then(|id| options.iter().position(|option| &option.id == id))
            };
            self.scroll.reset(options.len());
            if let Some(index) = anchor {
                self.scroll.scroll_to(ListOffset {
                    item_ix: index,
                    offset_in_item: if active.is_some() {
                        px(0.)
                    } else {
                        old_top.offset_in_item
                    },
                });
            }
            self.presented = options.clone();
        } else if self.metrics.as_ref() != Some(&metrics) {
            let anchor = self.scroll.logical_scroll_top();
            self.scroll.remeasure();
            self.scroll.scroll_to(anchor);
        }
        self.metrics = Some(metrics);
    }

    pub(super) fn navigate(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let choices = &self.options.visible;
        let index = self.active_position(choices);
        let next = index.map_or_else(
            || {
                if backwards {
                    choices.len().saturating_sub(1)
                } else {
                    0
                }
            },
            |index| {
                if backwards {
                    index
                        .checked_sub(1)
                        .unwrap_or(choices.len().saturating_sub(1))
                } else {
                    (index + 1) % choices.len()
                }
            },
        );
        if let Some(choice) = choices.get(next) {
            self.active = Some(choice.id.clone());
            self.active_index = Some(next);
        } else {
            self.active = None;
            self.active_index = None;
        }
        if self.scroll.bounds_for_item(next).is_some() {
            self.scroll.scroll_to_reveal_item(next);
        } else {
            self.scroll.scroll_to(ListOffset {
                item_ix: next,
                offset_in_item: px(0.),
            });
        }
        cx.notify();
    }

    pub(super) fn pending(&self) -> Option<DropdownOption> {
        let choices = &self.options.visible;
        let index = self.active_position(choices).unwrap_or(0);
        choices.get(index).cloned()
    }
}
