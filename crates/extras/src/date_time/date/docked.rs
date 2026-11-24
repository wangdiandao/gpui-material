use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, EntityInputHandler as _, FocusHandle, Focusable as _,
    InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, prelude::FluentBuilder as _,
    px,
};
use gpui_base::{Popup, TestSupportExt as _};
use gpui_material_components::{
    Button, ButtonVariant, DialogCloseReason, Icon, IconButton, IconName, TextField,
    TextFieldVariant,
};
use gpui_material_foundation::{Surface, ThemeScope, current_material_theme, logical_anchor};
use gpui_material_theme::{ColorRole, Elevation};

use super::{
    DatePicker,
    state::{parse_docked_range, parse_selection},
    types::{AnchorBounds, DockedActions, selection_label},
};

impl DatePicker {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_docked(
        mut self,
        trigger: AnyElement,
        bounds: AnchorBounds,
        open: bool,
        scope: FocusHandle,
        docked_actions: DockedActions,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let mut popup = Popup::new(self.id.clone(), trigger)
            .anchor(logical_anchor(Anchor::TopLeft, direction))
            .offset(px(4.));
        if open {
            let width = (window.viewport_size().width - px(16.)).clamp(px(1.), px(360.));
            let close_key = docked_actions.clone();
            let close_outside = docked_actions.clone();
            let body = self.body(
                (width - px(24.)).max(px(1.)),
                Some(docked_actions),
                true,
                window,
                cx,
            );
            let panel = div()
                .id((self.id.clone(), "docked"))
                .test_support()
                .role(Role::Dialog)
                .aria_label(self.label)
                .track_focus(&scope)
                .tab_group()
                .occlude()
                .max_h((window.viewport_size().height - px(16.)).max(px(1.)))
                .overflow_y_scroll()
                .on_mouse_down_out(move |event, window, cx| {
                    if bounds
                        .get()
                        .is_none_or(|bounds| !bounds.contains(&event.position))
                    {
                        close_outside.dismiss(DialogCloseReason::OutsideClick, window, cx);
                    }
                })
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified() {
                        let composing = close_key
                            .state
                            .update(cx, |state, cx| state.input_composing(window, cx));
                        if !composing {
                            close_key.dismiss(DialogCloseReason::Escape, window, cx);
                            cx.stop_propagation();
                        }
                    }
                })
                .child(
                    Surface::new(ColorRole::SurfaceContainerHigh)
                        .when_some(self.colors.container, Surface::container_color)
                        .w(width)
                        .rounded(px(16.))
                        .shadow_elevation(Elevation::Level3)
                        .p(px(12.))
                        .child(body),
                );
            popup = popup.content(ThemeScope::inherit(panel, window, cx));
        }
        popup.into_any_element()
    }

    pub(super) fn trigger(
        &mut self,
        docked: bool,
        open: bool,
        docked_actions: DockedActions,
        window: &mut Window,
        cx: &mut App,
    ) -> (AnyElement, AnchorBounds) {
        let model = self.state.read(cx);
        let focus = model.trigger.clone();
        let editor = model.anchor.clone();
        let state = self.state.clone();
        let options = self.options.clone();
        let typed_actions = docked_actions.clone();
        let disabled = self.disabled;
        let toggle = move |window: &mut Window, cx: &mut App| {
            if disabled {
                return;
            }
            if docked && state.read(cx).docked_open {
                docked_actions.dismiss(DialogCloseReason::Cancel, window, cx);
            } else {
                focus.focus(window, cx);
                state.update(cx, |state, cx| {
                    state.open(options.value, options.variant, &options.locale, window, cx);
                    if let Some(focus) = &options.focus_target {
                        focus.focus(window, cx);
                    }
                });
            }
        };
        let bounds = Rc::new(Cell::new(None));
        let capture = bounds.clone();
        let group = div()
            .id((self.id.clone(), "trigger-group"))
            .relative()
            .role(Role::Group)
            .aria_expanded(open)
            .child(
                canvas(
                    move |bounds, _, _| capture.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
        let control_id = (self.id.clone(), "open");
        let focus = self.state.read(cx).trigger.clone();
        if let Some(anchor) = self.anchor.take().filter(|_| docked) {
            let locale = self.options.locale.clone();
            let value = self.options.value;
            let formatted = if value.draft().start().is_none() {
                SharedString::default()
            } else {
                selection_label(value.draft(), &locale.format)
            };
            if self.state.read(cx).anchor_value.as_ref() != Some(&formatted) {
                self.state.update(cx, |state, cx| {
                    let synchronized = state.anchor.update(cx, |editor, cx| {
                        if editor.marked_text_range(window, cx).is_some() {
                            return false;
                        }
                        editor.set_value(formatted.clone(), window, cx);
                        true
                    });
                    if synchronized {
                        state.anchor_value = Some(formatted);
                    }
                });
            }
            let handle = editor.read(cx).focus_handle(cx);
            let state = self.state.clone();
            let commit = self.confirm.clone();
            let trailing = IconButton::new(
                control_id,
                "Open calendar",
                Icon::new(IconName::ArrowDropDown),
            )
            .disabled(disabled)
            .track_focus(&focus)
            .on_activate(toggle);
            let field = TextField::new((self.id.clone(), "anchor"), self.label.clone(), &editor)
                .variant(TextFieldVariant::Outlined)
                .when_some(self.colors.input, TextField::colors)
                .trailing(trailing);
            let element = group
                .on_key_down(move |event, window, cx| {
                    if disabled
                        || event.keystroke.key != "enter"
                        || event.keystroke.modifiers.modified()
                        || !handle.is_focused(window)
                    {
                        return;
                    }
                    let text = editor.update(cx, |editor, cx| {
                        editor
                            .marked_text_range(window, cx)
                            .is_none()
                            .then(|| editor.value())
                    });
                    let Some(text) = text else {
                        return;
                    };
                    let model = state.read(cx);
                    let selection = if value.draft().is_single() {
                        parse_selection(&text, None, &locale, &model.limits).ok()
                    } else {
                        parse_docked_range(&text, &locale, &model.limits)
                    };
                    if let Some(selection) = selection {
                        state.update(cx, |state, cx| {
                            state.set_draft(selection.draft().into(), window, cx);
                        });
                        if typed_actions.current(cx) {
                            typed_actions.finish(
                                DialogCloseReason::Confirm,
                                Some(selection),
                                window,
                                cx,
                            );
                        } else if let Some(commit) = &commit {
                            commit(selection, window, cx);
                        }
                        cx.stop_propagation();
                    }
                })
                .child(anchor(field))
                .into_any_element();
            (element, bounds)
        } else {
            let element = group
                .child(
                    Button::new(control_id, self.label.clone())
                        .variant(ButtonVariant::Outlined)
                        .disabled(disabled)
                        .track_focus(&focus)
                        .on_activate(toggle),
                )
                .into_any_element();
            (element, bounds)
        }
    }
}
