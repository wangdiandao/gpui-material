use gpui::{
    App, AppContext, Context, Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    ScrollHandle, SharedString, Subscription, Window,
};
use gpui_material_components::{DialogCloseReason, DialogState, TextFieldEvent, TextFieldState};

use super::CommandItem;

/// Events emitted by [`CommandPaletteState`].
#[derive(Clone, Debug)]
pub enum CommandPaletteEvent {
    /// Emitted when an enabled [`CommandItem`] is executed via keyboard or pointer activation.
    Execute(CommandItem),
    /// Emitted when the palette's search query changes.
    QueryChange(SharedString),
    /// Emitted when the palette closes due to user dismissal.
    Close(DialogCloseReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NavStep {
    Next,
    Previous,
    First,
    Last,
}

/// Retains modal visibility, search editor, scroll position, and active selection state
/// for a [`super::CommandPalette`].
pub struct CommandPaletteState {
    dialog: Entity<DialogState>,
    input: Entity<TextFieldState>,
    scroll: ScrollHandle,
    query: SharedString,
    active_id: Option<SharedString>,
    clear_on_open: bool,
    _input_sub: Subscription,
    _dialog_sub: Subscription,
}

impl EventEmitter<CommandPaletteEvent> for CommandPaletteState {}

impl CommandPaletteState {
    /// Creates a closed command palette state with an owned [`TextFieldState`] and [`DialogState`].
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextFieldState::new(window, cx));
        Self::with_editor(&input, window, cx)
    }

    /// Creates a closed command palette state reusing a caller-owned [`TextFieldState`].
    pub fn with_editor(
        input: &Entity<TextFieldState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let dialog = cx.new(DialogState::new);
        let dialog_sub = cx.observe(&dialog, |_, _, cx| cx.notify());
        let input_sub = cx.subscribe_in(input, window, |state, input, event, window, cx| {
            if !state.is_open(cx) || !matches!(event, TextFieldEvent::Change) {
                return;
            }
            if input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            }) {
                return;
            }
            let value = input.read(cx).value();
            if value != state.query {
                state.query = value.clone();
                state.active_id = None;
                cx.emit(CommandPaletteEvent::QueryChange(value));
                cx.notify();
                window.refresh();
            }
        });
        Self {
            dialog,
            input: input.clone(),
            scroll: ScrollHandle::new(),
            query: SharedString::default(),
            active_id: None,
            clear_on_open: true,
            _input_sub: input_sub,
            _dialog_sub: dialog_sub,
        }
    }

    /// Configures whether [`Self::open`] clears the previous query text (defaults to `true`).
    pub fn clear_on_open(mut self, clear: bool) -> Self {
        self.clear_on_open = clear;
        self
    }

    /// Returns `true` while the command palette dialog is open.
    pub fn is_open(&self, cx: &App) -> bool {
        self.dialog.read(cx).is_open()
    }

    /// Returns the latest committed query text.
    pub fn query(&self) -> &SharedString {
        &self.query
    }

    /// Returns the currently highlighted command ID, if explicitly selected.
    pub fn active_command_id(&self) -> Option<&SharedString> {
        self.active_id.as_ref()
    }

    /// Returns the underlying [`DialogState`] entity.
    pub fn dialog_state(&self) -> &Entity<DialogState> {
        &self.dialog
    }

    /// Returns the retained [`TextFieldState`] backing the search input.
    pub fn editor(&self) -> &Entity<TextFieldState> {
        &self.input
    }

    /// Returns the focus handle of the search input editor.
    pub fn input_focus(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }

    /// Returns the retained scroll handle bound to the results list.
    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    /// Opens the command palette, capturing current focus for restoration and focusing the search input.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open(cx) {
            self.input_focus(cx).focus(window, cx);
            return;
        }
        self.dialog.update(cx, |dialog, cx| dialog.open(window, cx));
        if self.clear_on_open {
            let had_query = !self.query.is_empty() || !self.input.read(cx).value().is_empty();
            self.query = SharedString::default();
            self.input.update(cx, |input, cx| {
                if !input.value().is_empty() {
                    input.set_value("", window, cx);
                }
            });
            if had_query {
                cx.emit(CommandPaletteEvent::QueryChange(SharedString::default()));
            }
        } else {
            let value = self.input.read(cx).value();
            if value != self.query {
                self.query = value;
            }
        }
        self.active_id = None;
        self.input_focus(cx).focus(window, cx);
        cx.notify();
        window.refresh();
    }

    /// Closes the command palette programmatically and restores prior focus.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_open(cx) {
            return;
        }
        self.dialog
            .update(cx, |dialog, cx| dialog.close(window, cx));
        cx.notify();
        window.refresh();
    }

    /// Toggles the command palette open or closed (convenient for `Ctrl+K` / `Cmd+K` bindings).
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open(cx) {
            self.close(window, cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Updates the query text programmatically and resets the active command highlight.
    pub fn set_query(
        &mut self,
        query: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let query = query.into();
        self.input.update(cx, |input, cx| {
            if input.value() != query {
                input.set_value(query.clone(), window, cx);
            }
        });
        if query != self.query {
            self.query = query.clone();
            self.active_id = None;
            cx.emit(CommandPaletteEvent::QueryChange(query));
            cx.notify();
            window.refresh();
        }
    }

    /// Explicitly highlights a command ID within the filtered list.
    pub fn set_active_command(&mut self, id: Option<SharedString>, cx: &mut Context<Self>) {
        if self.active_id != id {
            self.active_id = id;
            cx.notify();
        }
    }

    pub(super) fn emit_execute(&mut self, item: CommandItem, cx: &mut Context<Self>) {
        cx.emit(CommandPaletteEvent::Execute(item));
    }

    pub(super) fn emit_close(&mut self, reason: DialogCloseReason, cx: &mut Context<Self>) {
        cx.emit(CommandPaletteEvent::Close(reason));
    }
}

/// Default case-insensitive multi-token substring matcher across label, supporting text,
/// group heading, id, and keywords.
pub(super) fn matches_command(item: &CommandItem, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let label = item.label().to_lowercase();
    let id = item.id().to_lowercase();
    let supporting = item.supporting_text().map(|s| s.to_lowercase());
    let group = item.group_label().map(|s| s.to_lowercase());
    let keywords: Vec<String> = item
        .keywords_slice()
        .iter()
        .map(|k| k.to_lowercase())
        .collect();

    trimmed.split_whitespace().all(|token| {
        let needle = token.to_lowercase();
        label.contains(&needle)
            || id.contains(&needle)
            || supporting.as_deref().is_some_and(|s| s.contains(&needle))
            || group.as_deref().is_some_and(|g| g.contains(&needle))
            || keywords.iter().any(|k| k.contains(&needle))
    })
}

pub(super) fn resolve_active_id(
    filtered: &[CommandItem],
    current: Option<&SharedString>,
) -> Option<SharedString> {
    if let Some(current) = current
        && filtered
            .iter()
            .any(|item| !item.is_disabled() && item.id() == current)
    {
        return Some(current.clone());
    }
    filtered
        .iter()
        .find(|item| !item.is_disabled())
        .map(|item| item.id().clone())
}

pub(super) fn step_active_id(
    filtered: &[CommandItem],
    current: Option<&SharedString>,
    step: NavStep,
) -> Option<SharedString> {
    let enabled: Vec<&CommandItem> = filtered.iter().filter(|item| !item.is_disabled()).collect();
    if enabled.is_empty() {
        return None;
    }
    let active = resolve_active_id(filtered, current);
    let current_ix = active
        .as_ref()
        .and_then(|id| enabled.iter().position(|item| item.id() == id))
        .unwrap_or(0);
    let next_ix = match step {
        NavStep::First => 0,
        NavStep::Last => enabled.len() - 1,
        NavStep::Next => (current_ix + 1) % enabled.len(),
        NavStep::Previous => (current_ix + enabled.len() - 1) % enabled.len(),
    };
    Some(enabled[next_ix].id().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_matches_case_insensitively_across_fields_and_keywords() {
        let item = CommandItem::new("toggle-dark", "Switch appearance")
            .group("Theme")
            .supporting("Toggle between light and dark color schemes")
            .keywords(["night", "mode"]);

        assert!(matches_command(&item, ""));
        assert!(matches_command(&item, "   "));
        assert!(matches_command(&item, "APPEARANCE"));
        assert!(matches_command(&item, "theme dark"));
        assert!(matches_command(&item, "night mode"));
        assert!(!matches_command(&item, "theme export"));
    }

    #[test]
    fn navigation_steps_skip_disabled_commands_and_wrap() {
        let items = vec![
            CommandItem::new("open", "Open file"),
            CommandItem::new("locked", "Locked action").disabled(true),
            CommandItem::new("save", "Save file"),
            CommandItem::new("close", "Close tab"),
        ];

        assert_eq!(resolve_active_id(&items, None), Some("open".into()));
        assert_eq!(
            step_active_id(&items, Some(&"open".into()), NavStep::Next),
            Some("save".into())
        );
        assert_eq!(
            step_active_id(&items, Some(&"close".into()), NavStep::Next),
            Some("open".into())
        );
        assert_eq!(
            step_active_id(&items, Some(&"open".into()), NavStep::Previous),
            Some("close".into())
        );
        assert_eq!(
            step_active_id(&items, Some(&"save".into()), NavStep::First),
            Some("open".into())
        );
        assert_eq!(
            step_active_id(&items, Some(&"open".into()), NavStep::Last),
            Some("close".into())
        );
    }
}
