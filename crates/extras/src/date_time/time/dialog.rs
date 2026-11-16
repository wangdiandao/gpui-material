use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled as _, WeakEntity,
    Window, div, px,
};
use gpui_material_components::{
    Button, ButtonVariant, Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties,
};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, CornerShape};

use super::{
    NaiveTime, TimeFormat, TimePickerLayout, TimePickerLocale, TimePickerPanel, TimePickerSegment,
    TimePickerState, TimePickerStyle, TimePickerVariant,
};
use crate::date_time::Confirm;

type ActionSlot = Box<dyn FnOnce(DialogAction) -> AnyElement>;
pub(super) type ModeSlot = Box<dyn FnOnce(TimePickerModeAction) -> AnyElement>;
type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
type OnClose = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;

/// A request to switch between the current input and picker modes.
#[derive(Clone)]
pub struct TimePickerModeAction {
    state: WeakEntity<TimePickerState>,
    revision: u64,
    target: TimePickerVariant,
    displayed: TimePickerVariant,
    dialog: bool,
}

impl TimePickerModeAction {
    pub fn target(&self) -> TimePickerVariant {
        self.target
    }

    /// Changes mode and moves focus to the active segment of the new editor.
    pub fn activate(&self, window: &mut Window, cx: &mut App) {
        let _ = self.state.update(cx, |state, cx| {
            if state.mode_revision == self.revision && (!self.dialog || state.is_open(cx)) {
                if self.target == TimePickerVariant::Input {
                    state.picker_mode = self.displayed;
                }
                state.set_mode(self.target, window, cx);
                state.focus_editor(window, cx);
            }
        });
    }
}

pub(super) fn mode_action(
    state: &Entity<TimePickerState>,
    model: &TimePickerState,
    displayed: TimePickerVariant,
    dialog: bool,
) -> TimePickerModeAction {
    TimePickerModeAction {
        state: state.downgrade(),
        revision: model.mode_revision,
        target: if displayed == TimePickerVariant::Input {
            model.picker_mode
        } else {
            TimePickerVariant::Input
        },
        displayed,
        dialog,
    }
}

/// A Material dialog for an application-owned time picker trigger.
pub type TimePickerDialog = TimeDialog<false>;
/// A Material time dialog with vibrant colors and an optional visible title.
pub type VibrantTimePickerDialog = TimeDialog<true>;

/// Shared dialog composition used by both Material time picker presets.
#[derive(IntoElement)]
pub struct TimeDialog<const VIBRANT: bool> {
    id: ElementId,
    name: SharedString,
    state: Entity<TimePickerState>,
    layout: TimePickerLayout,
    style: TimePickerStyle,
    shape: Option<CornerShape>,
    colors: DialogColors,
    body: Option<AnyElement>,
    title: Option<AnyElement>,
    toggle: Option<ModeSlot>,
    confirm: Option<ActionSlot>,
    dismiss: Option<ActionSlot>,
    show_toggle: bool,
    show_dismiss: bool,
    properties: DialogProperties,
    on_confirm: Option<Confirm<NaiveTime>>,
    on_dismiss_request: Option<DismissRequest>,
    on_close: Option<OnClose>,
}

impl<const VIBRANT: bool> TimeDialog<VIBRANT> {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        state: &Entity<TimePickerState>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state: state.clone(),
            layout: TimePickerLayout::Auto,
            style: TimePickerStyle::default(),
            shape: None,
            colors: DialogColors::default(),
            body: None,
            title: None,
            toggle: None,
            confirm: None,
            dismiss: None,
            show_toggle: true,
            show_dismiss: true,
            properties: DialogProperties::default(),
            on_confirm: None,
            on_dismiss_request: None,
            on_close: None,
        }
    }

    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn colors(mut self, colors: DialogColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    pub fn title_content(mut self, content: impl IntoElement) -> Self {
        self.title = Some(content.into_any_element());
        self
    }

    pub fn mode_toggle_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(TimePickerModeAction) -> E + 'static,
    ) -> Self {
        self.toggle = Some(Box::new(move |action| content(action).into_any_element()));
        self
    }

    pub fn confirm_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.confirm = Some(Box::new(move |action| content(action).into_any_element()));
        self
    }

    pub fn dismiss_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.dismiss = Some(Box::new(move |action| content(action).into_any_element()));
        self
    }

    pub fn show_mode_toggle(mut self, show: bool) -> Self {
        self.show_toggle = show;
        self
    }

    pub fn show_dismiss(mut self, show: bool) -> Self {
        self.show_dismiss = show;
        self
    }

    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn on_confirm(
        mut self,
        confirm: impl Fn(NaiveTime, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm = Some(Rc::new(confirm));
        self
    }

    pub fn on_dismiss_request(
        mut self,
        request: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(request));
        self
    }

    pub fn on_close(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(callback));
        self
    }
}

impl<const VIBRANT: bool> RenderOnce for TimeDialog<VIBRANT> {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let toggle = self.show_toggle.then(|| {
            let action = mode_action(&self.state, model, model.mode, true);
            if let Some(custom) = self.toggle {
                custom(action)
            } else {
                Button::new(
                    (self.id.clone(), "display-mode"),
                    match action.target() {
                        TimePickerVariant::Dial => "Show clock",
                        TimePickerVariant::Input => "Enter time",
                        TimePickerVariant::Scroll => "Show scroll wheels",
                    },
                )
                .variant(ButtonVariant::Text)
                .on_activate(move |window, cx| action.activate(window, cx))
                .into_any_element()
            }
        });
        let body = self.body.unwrap_or_else(|| {
            TimePickerPanel::new(self.id.clone(), &self.state)
                .layout(self.layout)
                .style(self.style)
                .vibrant(VIBRANT)
                .into_any_element()
        });
        if VIBRANT {
            let theme = window_material_theme(window, cx);
            self.colors
                .container
                .get_or_insert_with(|| color(theme.colors.color(ColorRole::SurfaceContainer)));
        }
        let state = self.state.clone();
        let dismiss_state = self.state.clone();
        let close_state = self.state;
        let confirm = self.on_confirm;
        let close = self.on_close;
        let dismiss_request = self.on_dismiss_request;
        let mut dialog = Dialog::new(self.id, &model.dialog, self.name)
            .show_title(!VIBRANT || self.title.is_some())
            .colors(self.colors)
            .properties(self.properties)
            .show_cancel(self.show_dismiss)
            .body(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.))
                    .child(body)
                    .children(toggle),
            )
            .confirm("OK", move |window, cx| {
                state.update(cx, |state, cx| {
                    if state.has_composition(window, cx) {
                        return false;
                    }
                    if state.mode != TimePickerVariant::Input {
                        return true;
                    }
                    if let Some(segment) = state.invalid_segment(cx) {
                        state.input_attempted = true;
                        state.set_segment(segment, cx);
                        state.focus_editor(window, cx);
                        cx.notify();
                        return false;
                    }
                    state.reconcile_input(cx);
                    true
                })
            })
            .on_dismiss_request(move |reason, window, cx| {
                if reason == DialogCloseReason::Escape
                    && dismiss_state.update(cx, |state, cx| state.finish_composition(window, cx))
                {
                    return false;
                }
                dismiss_request
                    .as_ref()
                    .is_none_or(|request| request(reason, window, cx))
            })
            .on_close(move |reason, window, cx| {
                let value = close_state.update(cx, |state, cx| {
                    state.discard_composition(window, cx);
                    state.sync_wheels(cx);
                    state.value()
                });
                if reason == DialogCloseReason::Confirm
                    && let Some(confirm) = &confirm
                {
                    confirm(value, window, cx);
                }
                if let Some(close) = &close {
                    close(reason, window, cx);
                }
            });
        if let Some(shape) = self.shape {
            dialog = dialog.shape(shape);
        }
        if let Some(title) = self.title {
            dialog = dialog.title_content(title);
        }
        if let Some(content) = self.confirm {
            dialog = dialog.confirm_content(content);
        }
        if self.show_dismiss
            && let Some(content) = self.dismiss
        {
            dialog = dialog.dismiss_content(content);
        }
        dialog
    }
}

struct TimeOptions {
    value: NaiveTime,
    format: TimeFormat,
    variant: TimePickerVariant,
    layout: TimePickerLayout,
    locale: TimePickerLocale,
}

/// A button that edits a committed time in a Material dialog.
#[derive(IntoElement)]
pub struct TimePicker {
    id: ElementId,
    label: SharedString,
    state: Entity<TimePickerState>,
    options: TimeOptions,
    confirm: Option<Confirm<NaiveTime>>,
    style: TimePickerStyle,
}

impl TimePicker {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TimePickerState>,
        value: NaiveTime,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: TimeOptions {
                value,
                format: TimeFormat::default(),
                variant: TimePickerVariant::default(),
                layout: TimePickerLayout::default(),
                locale: TimePickerLocale::default(),
            },
            confirm: None,
            style: TimePickerStyle::default(),
        }
    }

    pub fn format(mut self, format: TimeFormat) -> Self {
        self.options.format = format;
        self
    }

    pub fn variant(mut self, variant: TimePickerVariant) -> Self {
        self.options.variant = variant;
        self
    }

    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.options.layout = layout;
        self
    }

    pub fn locale(mut self, locale: TimePickerLocale) -> Self {
        self.options.locale = locale;
        self
    }

    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn on_confirm(
        mut self,
        callback: impl Fn(NaiveTime, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for TimePicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let state = self.state.clone();
        let options = self.options;
        let layout = options.layout;
        let trigger = div()
            .id((self.id.clone(), "trigger-group"))
            .role(Role::Group)
            .aria_expanded(model.is_open(cx))
            .child(
                Button::new((self.id.clone(), "open"), self.label.clone())
                    .variant(ButtonVariant::Outlined)
                    .track_focus(&model.trigger)
                    .on_activate(move |window, cx| {
                        state.update(cx, |state, cx| {
                            if !state.is_open(cx) {
                                state.trigger.focus(window, cx);
                                state.set_segment(TimePickerSegment::Hour, cx);
                                state.picker_mode = TimePickerVariant::Dial;
                                state.set_mode(options.variant, window, cx);
                                state.set_format(options.format, window, cx);
                                state.set_locale(options.locale.clone(), window, cx);
                                state.open(options.value, window, cx);
                            }
                        });
                    }),
            );
        let mut dialog = TimePickerDialog::new(self.id, self.label, &self.state)
            .layout(layout)
            .style(self.style);
        dialog.on_confirm = self.confirm;
        div().child(trigger).child(dialog)
    }
}
