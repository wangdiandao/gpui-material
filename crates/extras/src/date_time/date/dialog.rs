use gpui::{
    AnyElement, App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    Role, StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _,
    px,
};
use gpui_base::{Date as BaseDate, TestSupportExt as _};
use gpui_material_components::{
    Button, ButtonVariant, Dialog, DialogCloseReason, DialogColors, DialogState, DialogVariant,
    Text, TextField, TextFieldState, TextFieldVariant,
};
use gpui_material_theme::TypeRole;

use super::{
    DatePicker, calendar,
    types::{
        AnchorBounds, DatePickerDisplayMode, DatePickerVariant, DockedActions, complete,
        selection_label,
    },
};
use crate::date_time::status;

impl DatePicker {
    pub(super) fn render_dialog(
        mut self,
        trigger: Option<(AnyElement, AnchorBounds)>,
        overlay: Entity<DialogState>,
        open: bool,
        width: Pixels,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let body = if open {
            self.body(width, None, false, window, cx)
        } else {
            div().into_any_element()
        };
        let state = self.state.clone();
        let request_state = self.state;
        let locale = self.options.locale;
        let callback = self.confirm;
        let request = self.request;
        let mut dialog = Dialog::new(self.id, &overlay, self.label)
            .variant(if self.options.variant == DatePickerVariant::FullScreen {
                DialogVariant::FullScreen
            } else {
                DialogVariant::Basic
            })
            .properties(self.properties)
            .colors(DialogColors {
                container: self.colors.container,
                title: self.colors.title,
                ..DialogColors::default()
            })
            .when_some(self.title, Dialog::title_content)
            .body(body)
            .on_dismiss_request(move |reason, window, cx| {
                let composing = reason == DialogCloseReason::Escape
                    && request_state.update(cx, |state, cx| {
                        state.input_mode && state.input_composing(window, cx)
                    });
                !composing
                    && request
                        .as_ref()
                        .is_none_or(|request| request(reason, window, cx))
            })
            .confirm("OK", move |window, cx| {
                let value = state.update(cx, |model, cx| {
                    if model.input_mode && model.input_composing(window, cx) {
                        return None;
                    }
                    let value = if model.input_mode {
                        model.input_value(&locale, cx).ok()
                    } else {
                        complete(model.calendar.read(cx).date())
                    };
                    if value.is_none() {
                        model.input_attempted = true;
                        cx.notify();
                    }
                    value
                });
                if let (Some(value), Some(callback)) = (value, &callback) {
                    callback(value, window, cx);
                }
                value.is_some()
            });
        if let Some(build) = self.confirm_content {
            dialog = dialog.confirm_content(build);
        }
        if let Some(build) = self.dismiss_content {
            dialog = dialog.dismiss_content(build);
        }
        if let Some(closed) = self.closed {
            dialog = dialog.on_closed(move |reason, window, cx| closed(reason, window, cx));
        }
        div()
            .children(trigger.map(|(trigger, _)| trigger))
            .child(dialog)
            .into_any_element()
    }

    pub(super) fn body(
        &mut self,
        width: Pixels,
        docked: Option<DockedActions>,
        inline_title: bool,
        window: &Window,
        cx: &App,
    ) -> AnyElement {
        let model = self.state.read(cx);
        let input = model.input_mode;
        let draft = BaseDate::from(model.draft(cx));
        let locale = &self.options.locale;
        let value = selection_label(draft, &locale.format);
        let spoken = selection_label(draft, &locale.day);
        let heading = div()
            .id((self.id.clone(), "headline"))
            .test_support()
            .role(Role::Status)
            .aria_label(spoken)
            .a11y_synthetic_children(|tree| {
                tree.parent_node().set_live(gpui::accesskit::Live::Polite)
            })
            .when_some(self.colors.headline, gpui::Styled::text_color)
            .child(self.headline.take().unwrap_or_else(|| {
                Text::new(value)
                    .id("date-headline-text")
                    .role(TypeRole::TitleMedium)
                    .into_any_element()
            }));
        let title = inline_title.then(|| self.title.take()).flatten();
        let attempted = model.input_attempted;
        let body = if input {
            self.input_fields(cx)
        } else {
            calendar::calendar(
                &self.id,
                self.state.clone(),
                locale.clone(),
                docked.clone(),
                (self.colors, width),
                window,
                cx,
            )
        };
        let state = self.state.clone();
        let mode = if input {
            DatePickerDisplayMode::Calendar
        } else {
            DatePickerDisplayMode::Input
        };
        let toggle = (self.show_mode_toggle && docked.is_none()).then(|| {
            Button::new(
                (self.id.clone(), "display-mode"),
                if input { "Show calendar" } else { "Enter date" },
            )
            .variant(ButtonVariant::Text)
            .on_activate(move |window, cx| {
                state.update(cx, |state, cx| {
                    state.set_display_mode(mode, window, cx);
                });
            })
        });
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .children(title.map(|title| {
                div()
                    .when_some(self.colors.title, gpui::Styled::text_color)
                    .child(title)
            }))
            .child(heading)
            .child(body)
            .children(toggle)
            .children(
                (!input && attempted && complete(draft).is_none())
                    .then(|| status("Select a date before confirming".into(), true)),
            )
            .into_any_element()
    }

    fn input_fields(&self, cx: &App) -> AnyElement {
        let model = self.state.read(cx);
        let range = !model.calendar.read(cx).date().is_single();
        let result = model.input_value(&self.options.locale, cx);
        let field = |id, name, state: &Entity<TextFieldState>| {
            TextField::new((self.id.clone(), id), name, state)
                .variant(TextFieldVariant::Outlined)
                .when_some(self.colors.input, TextField::colors)
        };
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(field(
                "start",
                if range { "Start date" } else { "Date" },
                &model.start,
            ))
            .when(range, |column| {
                column.child(field("end", "End date", &model.end))
            })
            .when_some(
                result.err().filter(|_| model.input_attempted),
                |column, error| column.child(status(error.message().into(), true)),
            )
            .into_any_element()
    }
}
