use std::rc::Rc;

use gpui::{App, Entity, EntityInputHandler, Focusable, KeyDownEvent, Window};

use super::{
    DropdownOption, Select,
    state::{ExposedDropdownState, Opening, Visibility},
};

pub(super) fn choose(
    state: &Entity<ExposedDropdownState>,
    choice: &DropdownOption,
    callback: &Option<Select>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        state.committed = choice.label.clone();
        state.selected = Some(choice.id.clone());
        state.reset_draft(window, cx);
        state.request_open(false, None, window, cx);
    });
    if let Some(callback) = callback {
        callback(choice, window, cx);
    }
}

#[derive(Clone)]
pub(super) struct Interaction {
    pub(super) state: Entity<ExposedDropdownState>,
    // The mounted handlers own the policy. The editor subscription observes it weakly.
    pub(super) _visibility: Rc<Visibility>,
    pub(super) select: Option<Select>,
    pub(super) primary: Opening,
    pub(super) primary_enabled: bool,
    pub(super) editable: bool,
    pub(super) disabled: bool,
    pub(super) parent: gpui::EntityId,
}

impl Interaction {
    pub(super) fn request(
        &self,
        open: Option<bool>,
        opening: Opening,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.disabled {
            return;
        }
        self.state.update(cx, |state, cx| {
            let open = open.unwrap_or(!state.open);
            if open && matches!(opening, Opening::Editor) {
                state.input.read(cx).focus_handle(cx).focus(window, cx);
            }
            state.request_open(open, open.then_some(opening), window, cx);
        });
        cx.notify(self.parent);
    }

    pub(super) fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| {
            state.request_open(false, None, window, cx);
        });
        cx.notify(self.parent);
    }

    pub(super) fn dismiss_outside(&self, window: &mut Window, cx: &mut App) {
        let editor_focus = self.state.read(cx).input.read(cx).focus_handle(cx);
        if editor_focus.is_focused(window) {
            window.defer(cx, move |window, cx| {
                if editor_focus.is_focused(window) {
                    window.blur(cx);
                }
            });
        }
        self.dismiss(window, cx);
    }

    pub(super) fn keys(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if key == "tab"
            && !modifiers.control
            && !modifiers.alt
            && !modifiers.platform
            && !modifiers.function
        {
            if self.state.read(cx).open {
                self.state.update(cx, |state, _| {
                    state.restore_focus = None;
                });
                self.dismiss(window, cx);
            }
            return;
        }
        if modifiers.modified()
            || !(matches!(key, "up" | "down" | "home" | "end" | "enter" | "escape")
                || key == "space" && !self.editable)
        {
            return;
        }
        let model = self.state.read(cx);
        let open = model.open;
        let input = model.input.clone();
        if !open && !self.primary_enabled
            || input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            })
        {
            return;
        }
        match key {
            "up" | "down" => self.state.update(cx, |state, cx| {
                state.navigate(key == "up", cx);
                if state.open {
                    return;
                }
                state.request_open(true, Some(self.primary), window, cx);
            }),
            "home" | "end" if open => self.state.update(cx, |state, cx| {
                state.navigate_edge(key == "end", cx);
            }),
            "escape" if open => self.dismiss(window, cx),
            "enter" if open => {
                if let Some(choice) = self.state.read(cx).pending() {
                    choose(&self.state, &choice, &self.select, window, cx);
                }
            }
            "enter" | "space" if !self.editable => self.request(None, self.primary, window, cx),
            _ => return,
        }
        cx.notify(self.parent);
        cx.stop_propagation();
    }
}
