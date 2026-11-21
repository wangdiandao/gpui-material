use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, Entity, EntityInputHandler as _, FocusHandle,
    Focusable as _, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled as _, Window, canvas,
    div, prelude::FluentBuilder as _, px,
};
use gpui_base::{Date, Popup, TestSupportExt as _};
use gpui_material_components::{
    Button, ButtonVariant, Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties,
    DialogVariant, Icon, IconButton, IconName, Text, TextField, TextFieldColors, TextFieldState,
    TextFieldVariant,
};
use gpui_material_foundation::{Surface, ThemeScope, current_material_theme, logical_anchor};
use gpui_material_theme::{ColorRole, Elevation, TypeRole};

use super::{Confirm, NaiveDate, Weekday, status};

mod calendar;
mod state;

pub use state::DatePickerState;
use state::parse_selection;

/// A committed picker value. The empty variants preserve the selection kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelection {
    Single(Option<NaiveDate>),
    Range(Option<(NaiveDate, NaiveDate)>),
}

impl DateSelection {
    fn draft(self) -> Date {
        match self {
            Self::Single(value) => Date::Single(value),
            Self::Range(Some((start, end))) => Date::Range(Some(start), Some(end)),
            Self::Range(None) => Date::Range(None, None),
        }
    }
}

fn complete(draft: Date) -> Option<DateSelection> {
    match draft {
        Date::Single(Some(date)) => Some(DateSelection::Single(Some(date))),
        Date::Range(Some(start), Some(end)) if start <= end => {
            Some(DateSelection::Range(Some((start, end))))
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerVariant {
    Docked,
    #[default]
    Modal,
    ModalInput,
    /// Uses a full-screen dialog in compact windows.
    FullScreen,
}

/// A draft can retain the first endpoint before the second is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePickerDraft {
    Single(Option<NaiveDate>),
    Range(Option<NaiveDate>, Option<NaiveDate>),
}

impl DatePickerDraft {
    /// Returns a complete candidate selection, when the draft has all its dates.
    pub fn selection(self) -> Option<DateSelection> {
        complete(self.into())
    }
}

impl From<DateSelection> for DatePickerDraft {
    fn from(value: DateSelection) -> Self {
        value.draft().into()
    }
}

impl From<Date> for DatePickerDraft {
    fn from(date: Date) -> Self {
        match date {
            Date::Single(value) => Self::Single(value),
            Date::Range(start, end) => Self::Range(start, end),
        }
    }
}

impl From<DatePickerDraft> for Date {
    fn from(draft: DatePickerDraft) -> Self {
        match draft {
            DatePickerDraft::Single(value) => Self::Single(value),
            DatePickerDraft::Range(start, end) => Self::Range(start, end),
        }
    }
}

/// The visible editor in a date picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerDisplayMode {
    #[default]
    Calendar,
    Input,
}

/// Local date picker and editor colors. Unset values use the active Material theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct DatePickerColors {
    pub container: Option<Hsla>,
    pub title: Option<Hsla>,
    pub headline: Option<Hsla>,
    pub weekday: Option<Hsla>,
    pub subheading: Option<Hsla>,
    pub navigation: Option<Hsla>,
    pub year: Option<Hsla>,
    pub current_year: Option<Hsla>,
    pub disabled_year: Option<Hsla>,
    pub selected_year: Option<Hsla>,
    pub selected_year_container: Option<Hsla>,
    pub disabled_selected_year: Option<Hsla>,
    pub disabled_selected_year_container: Option<Hsla>,
    pub day: Option<Hsla>,
    pub disabled_day: Option<Hsla>,
    pub selected_day: Option<Hsla>,
    pub selected_day_container: Option<Hsla>,
    pub disabled_selected_day: Option<Hsla>,
    pub disabled_selected_day_container: Option<Hsla>,
    pub today: Option<Hsla>,
    pub today_outline: Option<Hsla>,
    pub range_day: Option<Hsla>,
    pub range_container: Option<Hsla>,
    pub focus_outline: Option<Hsla>,
    pub input: Option<TextFieldColors>,
}

type DateFormatter = Rc<dyn Fn(NaiveDate) -> SharedString>;
type DateParser = Rc<dyn Fn(&str) -> Option<NaiveDate>>;
type WeekdayFormatter = Rc<dyn Fn(Weekday) -> SharedString>;
type DateFilter = Arc<dyn Fn(NaiveDate) -> bool + Send + Sync>;
type YearFilter = Arc<dyn Fn(i32) -> bool + Send + Sync>;
type OnClose = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;
type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
type CustomAnchor = Box<dyn FnOnce(TextField) -> AnyElement>;
type CustomAction = Box<dyn FnOnce(DialogAction) -> AnyElement>;

/// Formatting and parsing belong to the application locale. Default editing
/// uses an ISO pattern and the calendar uses English headings.
#[derive(Clone)]
pub struct DatePickerLocale {
    first_day: Weekday,
    day: DateFormatter,
    month: DateFormatter,
    weekday: WeekdayFormatter,
    format: DateFormatter,
    parse: DateParser,
}

impl Default for DatePickerLocale {
    fn default() -> Self {
        Self {
            first_day: Weekday::Sun,
            day: Rc::new(|date| date.format("%A, %B %-d, %Y").to_string().into()),
            month: Rc::new(|date| date.format("%B %Y").to_string().into()),
            weekday: Rc::new(|day| day.to_string().into()),
            format: Rc::new(|date| date.format("%Y-%m-%d").to_string().into()),
            parse: Rc::new(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()),
        }
    }
}

impl DatePickerLocale {
    pub fn first_day(mut self, day: Weekday) -> Self {
        self.first_day = day;
        self
    }

    /// Formats the initial editable text and parses a finished edit.
    pub fn input_format(
        mut self,
        format: impl Fn(NaiveDate) -> SharedString + 'static,
        parse: impl Fn(&str) -> Option<NaiveDate> + 'static,
    ) -> Self {
        self.format = Rc::new(format);
        self.parse = Rc::new(parse);
        self
    }

    /// Sets the spoken date format independently of editable text.
    pub fn accessible_date_format(
        mut self,
        format: impl Fn(NaiveDate) -> SharedString + 'static,
    ) -> Self {
        self.day = Rc::new(format);
        self
    }

    /// Formats day accessibility names, month headings, and weekday labels.
    pub fn calendar_format(
        mut self,
        day: impl Fn(NaiveDate) -> SharedString + 'static,
        month: impl Fn(NaiveDate) -> SharedString + 'static,
        weekday: impl Fn(Weekday) -> SharedString + 'static,
    ) -> Self {
        self.day = Rc::new(day);
        self.month = Rc::new(month);
        self.weekday = Rc::new(weekday);
        self
    }
}

#[derive(Clone)]
struct DatePickerOptions {
    value: DateSelection,
    variant: DatePickerVariant,
    locale: DatePickerLocale,
    focus_target: Option<FocusHandle>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Presentation {
    Triggered,
    Panel,
    Dialog,
}

#[derive(Clone)]
struct DockedActions {
    state: Entity<DatePickerState>,
    generation: u64,
    properties: DialogProperties,
    confirm: Option<Confirm<DateSelection>>,
    request: Option<DismissRequest>,
    closed: Option<OnClose>,
}

impl DockedActions {
    fn current(&self, cx: &App) -> bool {
        let state = self.state.read(cx);
        state.docked_open && state.docked_generation == self.generation
    }

    fn dismiss(&self, reason: DialogCloseReason, window: &mut Window, cx: &mut App) {
        let enabled = match reason {
            DialogCloseReason::Escape => self.properties.dismiss_on_escape,
            DialogCloseReason::OutsideClick => self.properties.dismiss_on_outside_click,
            _ => true,
        };
        if enabled
            && self.current(cx)
            && self
                .request
                .as_ref()
                .is_none_or(|callback| callback(reason, window, cx))
        {
            self.finish(reason, None, window, cx);
        }
    }

    fn finish(
        &self,
        reason: DialogCloseReason,
        value: Option<DateSelection>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.current(cx) {
            self.state
                .update(cx, |model, cx| model.dismiss_docked(reason, window, cx));
            if let (Some(value), Some(confirm)) = (value, &self.confirm) {
                confirm(value, window, cx);
            }
            if let Some(closed) = &self.closed {
                closed(reason, window, cx);
            }
        }
    }
}

/// A single or range date picker with a trigger, a standalone panel, or a modal dialog.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    label: SharedString,
    state: Entity<DatePickerState>,
    options: DatePickerOptions,
    presentation: Presentation,
    anchor: Option<CustomAnchor>,
    show_mode_toggle: bool,
    title: Option<AnyElement>,
    headline: Option<AnyElement>,
    confirm_content: Option<CustomAction>,
    dismiss_content: Option<CustomAction>,
    confirm: Option<Confirm<DateSelection>>,
    properties: DialogProperties,
    request: Option<DismissRequest>,
    closed: Option<OnClose>,
    colors: DatePickerColors,
}

impl DatePicker {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
        value: DateSelection,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: DatePickerOptions {
                value,
                variant: DatePickerVariant::default(),
                locale: DatePickerLocale::default(),
                focus_target: None,
            },
            presentation: Presentation::Triggered,
            anchor: None,
            show_mode_toggle: true,
            title: None,
            headline: None,
            confirm_content: None,
            dismiss_content: None,
            confirm: None,
            properties: DialogProperties::default(),
            request: None,
            closed: None,
            colors: DatePickerColors::default(),
        }
    }

    /// Embeds the retained draft and display mode directly in another layout.
    pub fn panel(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
    ) -> Self {
        Self {
            presentation: Presentation::Panel,
            ..Self::new(id, label, state, DateSelection::Single(None))
        }
    }

    /// Renders a modal opened by a caller-owned control with `DatePickerState::open`.
    pub fn dialog(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
        value: DateSelection,
    ) -> Self {
        Self {
            presentation: Presentation::Dialog,
            ..Self::new(id, label, state, value)
        }
    }

    pub fn variant(mut self, variant: DatePickerVariant) -> Self {
        self.options.variant = variant;
        self
    }

    pub fn locale(mut self, locale: DatePickerLocale) -> Self {
        self.options.locale = locale;
        self
    }

    /// Chooses the initial focus when the built-in control opens the picker.
    pub fn focus_target(mut self, focus: &FocusHandle) -> Self {
        self.options.focus_target = Some(focus.clone());
        self
    }

    /// Wraps/customizes the retained docked editor and supplied trailing trigger.
    pub fn docked_anchor<E: IntoElement>(
        mut self,
        render: impl FnOnce(TextField) -> E + 'static,
    ) -> Self {
        self.anchor = Some(Box::new(move |field| render(field).into_any_element()));
        self
    }

    pub fn show_mode_toggle(mut self, show: bool) -> Self {
        self.show_mode_toggle = show;
        self
    }

    /// Replaces the visible title while preserving the accessible picker label.
    pub fn title_content(mut self, element: impl IntoElement) -> Self {
        self.title = Some(element.into_any_element());
        self
    }

    /// Replaces the selection headline while preserving its date announcement.
    pub fn headline_content(mut self, element: impl IntoElement) -> Self {
        self.headline = Some(element.into_any_element());
        self
    }

    pub fn confirm_content<E: IntoElement>(
        mut self,
        build: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.confirm_content = Some(Box::new(move |confirm| build(confirm).into_any_element()));
        self
    }

    pub fn dismiss_content<E: IntoElement>(
        mut self,
        build: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.dismiss_content = Some(Box::new(move |dismiss| build(dismiss).into_any_element()));
        self
    }

    pub fn on_confirm(
        mut self,
        callback: impl Fn(DateSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(callback));
        self
    }

    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.request = Some(Rc::new(callback));
        self
    }

    pub fn on_close(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.closed = Some(Rc::new(callback));
        self
    }

    pub fn colors(mut self, colors: DatePickerColors) -> Self {
        self.colors = colors;
        self
    }
}

fn selection_label(draft: Date, formatter: &DateFormatter) -> SharedString {
    match (draft.start(), draft.end()) {
        (Some(start), Some(end)) => format!("{} – {}", formatter(start), formatter(end)).into(),
        (Some(start), _) => formatter(start),
        _ if draft.is_single() => "No date selected".into(),
        _ => "No date range selected".into(),
    }
}

impl RenderOnce for DatePicker {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.state.update(cx, |state, cx| {
            state.locale = self.options.locale.clone();
            if !state.locale_initialized {
                state.locale_initialized = true;
                if self.presentation == Presentation::Panel {
                    let draft = state.calendar.read(cx).date();
                    state.reset_input(draft, &self.options.locale, window, cx);
                }
            }
        });
        let width = (window.viewport_size().width - px(96.)).clamp(px(70.), px(336.));
        if self.presentation == Presentation::Panel {
            let body = self.body(width, None, true, window, cx);
            return Surface::new(ColorRole::SurfaceContainerHigh)
                .when_some(self.colors.container, Surface::container_color)
                .child(
                    div()
                        .id(self.id)
                        .role(Role::Group)
                        .aria_label(self.label)
                        .child(body),
                )
                .into_any_element();
        }

        let docked = self.options.variant == DatePickerVariant::Docked
            && self.presentation == Presentation::Triggered;
        let model = self.state.read(cx);
        let open = if docked {
            model.docked_open
        } else {
            model.dialog.read(cx).is_open()
        };
        let scope = model.docked.clone();
        let overlay = model.dialog.clone();
        let docked_actions = DockedActions {
            state: self.state.clone(),
            generation: model.docked_generation,
            properties: self.properties,
            confirm: self.confirm.clone(),
            request: self.request.clone(),
            closed: self.closed.clone(),
        };

        let trigger = (self.presentation == Presentation::Triggered)
            .then(|| self.trigger(docked, open, docked_actions.clone(), window, cx));
        if docked {
            let (trigger, bounds) = trigger.expect("docked pickers have an anchor");
            return self.render_docked(trigger, bounds, open, scope, docked_actions, window, cx);
        }

        self.render_dialog(trigger, overlay, open, width, window, cx)
    }
}

type AnchorBounds = Rc<Cell<Option<Bounds<Pixels>>>>;

impl DatePicker {
    #[allow(clippy::too_many_arguments)]
    fn render_docked(
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
                    if event.keystroke.key == "escape" {
                        close_key.dismiss(DialogCloseReason::Escape, window, cx);
                        cx.stop_propagation();
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

    fn render_dialog(
        mut self,
        trigger: Option<(AnyElement, AnchorBounds)>,
        overlay: Entity<gpui_material_components::DialogState>,
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
            dialog = dialog.on_close(move |reason, window, cx| closed(reason, window, cx));
        }
        div()
            .children(trigger.map(|(trigger, _)| trigger))
            .child(dialog)
            .into_any_element()
    }

    fn trigger(
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
        let toggle = move |window: &mut Window, cx: &mut App| {
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
            self.state.update(cx, |state, cx| {
                if state.anchor_value.as_ref() != Some(&formatted) {
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
                }
            });
            let handle = editor.read(cx).focus_handle(cx);
            let state = self.state.clone();
            let commit = self.confirm.clone();
            let trailing = IconButton::new(
                control_id,
                "Open calendar",
                Icon::new(IconName::ArrowDropDown),
            )
            .track_focus(&focus)
            .on_activate(toggle);
            let field = TextField::new((self.id.clone(), "anchor"), self.label.clone(), &editor)
                .variant(TextFieldVariant::Outlined)
                .when_some(self.colors.input, TextField::colors)
                .trailing(trailing);
            let element = group
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.key != "enter" || !handle.is_focused(window) {
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
                        text.split_once(" – ").and_then(|(start, end)| {
                            parse_selection(start, Some(end), &locale, &model.limits).ok()
                        })
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
                        .track_focus(&focus)
                        .on_activate(toggle),
                )
                .into_any_element();
            (element, bounds)
        }
    }

    fn body(
        &mut self,
        width: Pixels,
        docked: Option<DockedActions>,
        inline_title: bool,
        window: &Window,
        cx: &App,
    ) -> AnyElement {
        let model = self.state.read(cx);
        let input = model.input_mode;
        let draft = Date::from(model.draft(cx));
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
