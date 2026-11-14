use std::{ops::RangeInclusive, rc::Rc, sync::Arc};

use chrono::{Datelike, Days, Months};
use gpui::{
    AccessibleAction, Anchor, AnyElement, App, AppContext as _, Context, ElementId, Entity,
    FocusHandle, Focusable as _, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_base::{
    Button as BaseButton, CalendarState, Date, Matcher, Popup, TestSupportExt as _,
    actions::Confirm as ConfirmAction,
};
use gpui_material_components::{
    Button, ButtonVariant, Dialog, DialogState, Icon, IconButton, IconName, Text, TextField,
    TextFieldState, TextFieldVariant,
};
use gpui_material_foundation::{Surface, color, logical_anchor, text_style, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, Elevation, TypeRole, TypographyEmphasis};

use super::{Confirm, NaiveDate, Weekday, status};

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
}

type DateFormatter = Rc<dyn Fn(NaiveDate) -> SharedString>;
type DateParser = Rc<dyn Fn(&str) -> Option<NaiveDate>>;
type WeekdayFormatter = Rc<dyn Fn(Weekday) -> SharedString>;
type DateFilter = Arc<dyn Fn(NaiveDate) -> bool + Send + Sync>;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateError {
    Invalid,
    Disabled,
    Order,
}

impl DateError {
    fn message(self) -> &'static str {
        match self {
            Self::Invalid => "Enter a valid date",
            Self::Disabled => "Choose an available date",
            Self::Order => "End date must be on or after start date",
        }
    }
}

#[derive(Clone)]
struct DateLimits {
    bounds: RangeInclusive<NaiveDate>,
    disabled: DateFilter,
}

impl DateLimits {
    fn accepts(&self, date: NaiveDate) -> bool {
        self.bounds.contains(&date) && !(self.disabled)(date)
    }
}

fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("every month has a first day")
}

fn calendar_start(month: NaiveDate, first_day: Weekday) -> NaiveDate {
    let offset =
        (month.weekday().num_days_from_monday() + 7 - first_day.num_days_from_monday()) % 7;
    month
        .checked_sub_days(Days::new(u64::from(offset)))
        .unwrap_or(month)
}

fn parse_selection(
    start: &str,
    end: Option<&str>,
    locale: &DatePickerLocale,
    limits: &DateLimits,
) -> Result<DateSelection, DateError> {
    let parse = |text: &str| {
        let date = (locale.parse)(text.trim()).ok_or(DateError::Invalid)?;
        if !limits.accepts(date) {
            return Err(DateError::Disabled);
        }
        Ok(date)
    };
    let start = parse(start)?;
    let Some(end) = end.map(parse).transpose()? else {
        return Ok(DateSelection::Single(Some(start)));
    };
    if start > end {
        return Err(DateError::Order);
    }
    Ok(DateSelection::Range(Some((start, end))))
}

/// Owns the Base calendar, input editors, and draft focus. The application owns
/// the value passed to the picker. Every open copies that value into the draft.
pub struct DatePickerState {
    calendar: Entity<CalendarState>,
    start: Entity<TextFieldState>,
    end: Entity<TextFieldState>,
    dialog: Entity<DialogState>,
    trigger: FocusHandle,
    docked: FocusHandle,
    day: FocusHandle,
    cursor: NaiveDate,
    docked_open: bool,
    input_mode: bool,
    input_attempted: bool,
    limits: DateLimits,
}

impl DatePickerState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let limits = DateLimits {
            bounds: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
                ..=NaiveDate::from_ymd_opt(2100, 12, 31).unwrap(),
            disabled: Arc::new(|_| false),
        };
        let calendar = cx.new(|cx| CalendarState::new(window, cx));
        let cursor = calendar.read(cx).today();
        let start = cx.new(|cx| TextFieldState::new(window, cx));
        let end = cx.new(|cx| TextFieldState::new(window, cx));
        let dialog = cx.new(DialogState::new);
        cx.observe(&start, |_, _, cx| cx.notify()).detach();
        cx.observe(&end, |_, _, cx| cx.notify()).detach();
        cx.observe(&dialog, |_, _, cx| cx.notify()).detach();
        Self {
            calendar,
            start,
            end,
            dialog,
            trigger: cx.focus_handle(),
            docked: cx.focus_handle(),
            day: cx.focus_handle(),
            cursor,
            docked_open: false,
            input_mode: false,
            input_attempted: false,
            limits,
        }
    }

    /// Restricts the displayed months and selectable dates. A reversed range
    /// is ordered before it is applied on the next open.
    pub fn date_bounds(mut self, bounds: RangeInclusive<NaiveDate>) -> Self {
        let (min, max) = bounds.into_inner();
        self.limits.bounds = min.min(max)..=min.max(max);
        self
    }

    /// Disables individual days and range endpoints.
    pub fn disabled_dates(
        mut self,
        disabled: impl Fn(NaiveDate) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.limits.disabled = Arc::new(disabled);
        self
    }

    fn month(&self, cx: &App) -> NaiveDate {
        let calendar = self.calendar.read(cx);
        NaiveDate::from_ymd_opt(
            calendar.current_year(),
            u32::from(calendar.current_month()),
            1,
        )
        .expect("calendar displays a valid month")
    }

    fn show_month(&self, date: NaiveDate, cx: &mut Context<Self>) {
        self.calendar.update(cx, |calendar, _| {
            calendar.select_year(date.year());
            calendar.select_month(date.month() as u8);
        });
    }

    fn open(&mut self, picker: &DatePickerOptions, window: &mut Window, cx: &mut Context<Self>) {
        let draft = match picker.value.draft() {
            Date::Single(Some(day)) if !self.limits.accepts(day) => Date::Single(None),
            Date::Range(Some(start), Some(end))
                if start > end || !self.limits.accepts(start) || !self.limits.accepts(end) =>
            {
                Date::Range(None, None)
            }
            draft => draft,
        };
        let limits = self.limits.clone();
        self.calendar.update(cx, |calendar, _| {
            calendar.set_disabled_matcher_shared(Some(Rc::new(Matcher::custom(move |date| {
                !limits.accepts(*date)
            }))));
            calendar.apply_date(draft);
        });
        let today = self.calendar.read(cx).today();
        self.cursor = draft
            .start()
            .unwrap_or(today)
            .clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
        self.show_month(self.cursor, cx);
        self.reset_input(draft, &picker.locale, window, cx);
        self.input_mode = picker.variant == DatePickerVariant::ModalInput;
        self.input_attempted = false;
        self.trigger.focus(window, cx);
        if picker.variant == DatePickerVariant::Docked {
            self.docked_open = true;
            self.docked.focus(window, cx);
        } else {
            self.dialog.update(cx, |dialog, cx| dialog.open(window, cx));
        }
        let focus = if self.input_mode {
            self.start.read(cx).focus_handle(cx)
        } else {
            self.day.clone()
        };
        focus.focus(window, cx);
        cx.notify();
    }

    fn reset_input(
        &self,
        draft: Date,
        locale: &DatePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let format = |date: Option<NaiveDate>| {
            date.map_or_else(SharedString::default, |date| (locale.format)(date))
        };
        self.start.update(cx, |field, cx| {
            field.set_value(format(draft.start()), window, cx)
        });
        self.end.update(cx, |field, cx| {
            field.set_value(format(draft.end()), window, cx)
        });
    }

    fn input_value(&self, locale: &DatePickerLocale, cx: &App) -> Result<DateSelection, DateError> {
        let start = self.start.read(cx).value();
        let end = self.end.read(cx).value();
        let range = !self.calendar.read(cx).date().is_single();
        parse_selection(&start, range.then_some(end.as_ref()), locale, &self.limits)
    }

    fn toggle_mode(
        &mut self,
        locale: &DatePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.input_attempted = false;
        if self.input_mode {
            if let Ok(value) = self.input_value(locale, cx) {
                self.calendar
                    .update(cx, |calendar, _| calendar.apply_date(value.draft()));
                if let Some(day) = value.draft().start() {
                    self.cursor = day;
                }
            }
        } else {
            self.reset_input(self.calendar.read(cx).date(), locale, window, cx);
        }
        self.input_mode = !self.input_mode;
        cx.notify();
        let focus = if self.input_mode {
            self.start.read(cx).focus_handle(cx)
        } else {
            self.day.clone()
        };
        focus.focus(window, cx);
    }

    fn dismiss_docked(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.docked_open {
            self.docked_open = false;
            if self.docked.contains_focused(window, cx) {
                self.trigger.focus(window, cx);
            }
            cx.notify();
        }
    }

    fn select_day(&mut self, date: NaiveDate, cx: &mut Context<Self>) -> Option<DateSelection> {
        if !self.limits.accepts(date) {
            return None;
        }
        self.cursor = date;
        let result = self.calendar.update(cx, |calendar, _| {
            calendar.select_date(date);
            complete(calendar.date())
        });
        // Base owns the range transition. Keep the second endpoint's month
        // visible when completing a range across two months.
        self.show_month(date, cx);
        self.input_attempted = false;
        cx.notify();
        result
    }

    fn navigate(
        &mut self,
        key: &str,
        control: bool,
        direction: Direction,
        first_day: Weekday,
        cx: &mut Context<Self>,
    ) -> bool {
        let rtl = direction == Direction::Rtl;
        let offset = (self.cursor.weekday().num_days_from_monday() + 7
            - first_day.num_days_from_monday())
            % 7;
        let candidate = match key {
            "left" | "right" => {
                if (key == "left") != rtl {
                    self.cursor.pred_opt()
                } else {
                    self.cursor.succ_opt()
                }
            }
            "up" => self.cursor.checked_sub_days(Days::new(7)),
            "down" => self.cursor.checked_add_days(Days::new(7)),
            "home" => self.cursor.checked_sub_days(Days::new(u64::from(offset))),
            "end" => self
                .cursor
                .checked_add_days(Days::new(u64::from(6 - offset))),
            "pageup" => self
                .cursor
                .checked_sub_months(Months::new(if control { 12 } else { 1 })),
            "pagedown" => self
                .cursor
                .checked_add_months(Months::new(if control { 12 } else { 1 })),
            _ => return false,
        };
        if let Some(date) = candidate {
            self.cursor = date.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
            self.show_month(self.cursor, cx);
            cx.notify();
        }
        true
    }
}

#[derive(Clone)]
struct DatePickerOptions {
    value: DateSelection,
    variant: DatePickerVariant,
    locale: DatePickerLocale,
}

/// An anchored or modal date picker. A completed docked selection commits
/// immediately. Modal editing commits only after a valid confirmation.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    label: SharedString,
    state: Entity<DatePickerState>,
    options: DatePickerOptions,
    confirm: Option<Confirm<DateSelection>>,
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
            },
            confirm: None,
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

    pub fn on_confirm(
        mut self,
        callback: impl Fn(DateSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(callback));
        self
    }
}

fn selection_label(draft: Date, locale: &DatePickerLocale) -> SharedString {
    match (draft.start(), draft.end()) {
        (Some(start), Some(end)) => {
            format!("{} – {}", (locale.format)(start), (locale.format)(end)).into()
        }
        (Some(start), _) => (locale.format)(start),
        _ if draft.is_single() => "No date selected".into(),
        _ => "No date range selected".into(),
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let docked = self.options.variant == DatePickerVariant::Docked;
        let open = if docked {
            model.docked_open
        } else {
            model.dialog.read(cx).is_open()
        };
        let trigger_focus = model.trigger.clone();
        let scope = model.docked.clone();
        let overlay = model.dialog.clone();
        let trigger_state = self.state.clone();
        let opts = self.options.clone();
        let trigger = div()
            .id((self.id.clone(), "trigger-group"))
            .role(Role::Group)
            .aria_expanded(open)
            .child(
                Button::new((self.id.clone(), "open"), self.label.clone())
                    .variant(ButtonVariant::Outlined)
                    .track_focus(&trigger_focus)
                    .on_activate(move |window, cx| {
                        trigger_state.update(cx, |state, cx| {
                            if docked && state.docked_open {
                                state.dismiss_docked(window, cx);
                            } else {
                                state.open(&opts, window, cx);
                            }
                        });
                    }),
            );
        if docked {
            let direction = window_material_theme(window, cx).layout.direction;
            let mut popup = Popup::new(self.id.clone(), trigger)
                .anchor(logical_anchor(Anchor::TopLeft, direction))
                .offset(px(4.));
            if open {
                let width = (window.viewport_size().width - px(16.))
                    .min(px(360.))
                    .max(px(1.));
                let close_key = self.state.clone();
                let close_outside = self.state.clone();
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
                    .on_mouse_down_out(move |_, window, cx| {
                        close_outside.update(cx, |state, cx| state.dismiss_docked(window, cx));
                    })
                    .on_key_down(move |event, window, cx| {
                        if event.keystroke.key == "escape" {
                            close_key.update(cx, |state, cx| state.dismiss_docked(window, cx));
                            cx.stop_propagation();
                        }
                    })
                    .child(
                        Surface::new(ColorRole::SurfaceContainerHigh)
                            .w(width)
                            .rounded(px(16.))
                            .shadow_elevation(Elevation::Level3)
                            .p(px(12.))
                            .child(calendar(
                                &self.id,
                                self.state,
                                self.options.locale,
                                self.confirm,
                                width - px(24.),
                                window,
                                cx,
                            )),
                    );
                popup = popup.content(panel);
            }
            return popup.into_any_element();
        }

        let input_mode = model.input_mode;
        let body = if !open {
            div().into_any_element()
        } else if input_mode {
            self.input_fields(cx)
        } else {
            let width = (window.viewport_size().width - px(96.)).clamp(px(100.), px(312.));
            calendar(
                &self.id,
                self.state.clone(),
                self.options.locale.clone(),
                None,
                width,
                window,
                cx,
            )
        };
        let toggle_state = self.state.clone();
        let locale = self.options.locale.clone();
        let toggle = Button::new(
            (self.id.clone(), "display-mode"),
            if input_mode {
                "Show calendar"
            } else {
                "Enter date"
            },
        )
        .variant(ButtonVariant::Text)
        .on_activate(move |window, cx| {
            toggle_state.update(cx, |state, cx| state.toggle_mode(&locale, window, cx))
        });
        let state = self.state;
        let locale = self.options.locale;
        let callback = self.confirm;
        div()
            .child(trigger)
            .child(
                Dialog::new(self.id, &overlay, self.label)
                    .body(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(body)
                            .child(toggle),
                    )
                    .confirm("OK", move |window, cx| {
                        let value = {
                            let model = state.read(cx);
                            if model.input_mode {
                                model.input_value(&locale, cx).ok()
                            } else {
                                complete(model.calendar.read(cx).date())
                            }
                        };
                        let Some(value) = value else {
                            state.update(cx, |state, cx| {
                                state.input_attempted = true;
                                cx.notify();
                            });
                            return false;
                        };
                        if let Some(callback) = &callback {
                            callback(value, window, cx);
                        }
                        true
                    }),
            )
            .into_any_element()
    }
}

impl DatePicker {
    fn input_fields(&self, cx: &App) -> AnyElement {
        let model = self.state.read(cx);
        let range = !model.calendar.read(cx).date().is_single();
        let result = model.input_value(&self.options.locale, cx);
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                TextField::new(
                    (self.id.clone(), "start"),
                    if range { "Start date" } else { "Date" },
                    &model.start,
                )
                .variant(TextFieldVariant::Outlined),
            )
            .when(range, |column| {
                column.child(
                    TextField::new((self.id.clone(), "end"), "End date", &model.end)
                        .variant(TextFieldVariant::Outlined),
                )
            })
            .when_some(
                result.err().filter(|_| model.input_attempted),
                |column, error| column.child(status(error.message().into(), true)),
            )
            .into_any_element()
    }
}

fn calendar(
    id: &ElementId,
    state: Entity<DatePickerState>,
    locale: DatePickerLocale,
    docked_commit: Option<Confirm<DateSelection>>,
    width: Pixels,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let month = model.month(cx);
    let draft = model.calendar.read(cx).date();
    let today = model.calendar.read(cx).today();
    let cursor = model.cursor;
    let start = calendar_start(month, locale.first_day);
    let theme = window_material_theme(window, cx);
    let direction = theme.layout.direction;
    let rtl = direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let cell = (width / 7.).clamp(px(16.), px(48.));
    let day_focus = model.day.clone();
    let keyboard_focus = day_focus.clone();
    let confirm_focus = day_focus.clone();
    let focus_cursor = day_focus.is_focused(window);
    let keys = state.clone();
    let key_commit = docked_commit.clone();
    let key_selection = state.clone();
    let select = Rc::new(move |window: &mut Window, cx: &mut App| {
        let result = key_selection.update(cx, |model, cx| {
            let result = model.select_day(model.cursor, cx);
            if result.is_some() && model.docked_open {
                model.dismiss_docked(window, cx);
            }
            result
        });
        if let (Some(value), Some(callback)) = (result, &key_commit) {
            callback(value, window, cx);
        }
    });
    let confirm = select.clone();
    let first_day = locale.first_day;
    let prev = state.clone();
    let next = state.clone();
    let move_month = |target: Entity<DatePickerState>, key| {
        move |_: &mut Window, cx: &mut App| {
            target.update(cx, |model, cx| {
                model.navigate(key, false, direction, first_day, cx);
            });
        }
    };
    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(
            IconButton::new(
                (id.clone(), "previous-month"),
                "Previous month",
                Icon::new(IconName::ChevronRight)
                    .rotate(gpui::Radians(std::f32::consts::PI))
                    .mirror_in_rtl(true),
            )
            .disabled(month <= first_of_month(*model.limits.bounds.start()))
            .on_activate(move_month(prev, "pageup")),
        )
        .child(
            Text::new((locale.month)(month))
                .id("current-month")
                .role(TypeRole::TitleSmall),
        )
        .child(
            IconButton::new(
                (id.clone(), "next-month"),
                "Next month",
                Icon::new(IconName::ChevronRight).mirror_in_rtl(true),
            )
            .disabled(month >= first_of_month(*model.limits.bounds.end()))
            .on_activate(move_month(next, "pagedown")),
        );
    let weekdays = (0..7).map(|offset| {
        let weekday = start
            .checked_add_days(Days::new(offset))
            .unwrap_or(start)
            .weekday();
        div()
            .size(cell)
            .flex()
            .items_center()
            .justify_center()
            .child(
                Text::new((locale.weekday)(weekday))
                    .id(("weekday", weekday.num_days_from_monday()))
                    .role(TypeRole::BodySmall),
            )
    });
    let rows = (0..6).map(|week| {
        div()
            .flex()
            .when(rtl, gpui::Styled::flex_row_reverse)
            .children((0..7).map(|day| {
                let Some(date) = start
                    .checked_add_days(Days::new(week * 7 + day))
                    .filter(|date| date.month() == month.month())
                else {
                    return div().size(cell).into_any_element();
                };
                let enabled = model.limits.accepts(date);
                let active = draft.is_active(&date);
                let range = draft.is_in_range(&date);
                let leading = range && draft.start() == Some(date);
                let trailing = range && draft.end() == Some(date);
                let focus = cursor == date;
                let ink = tone(if active {
                    ColorRole::OnPrimary
                } else if range {
                    ColorRole::OnSecondaryContainer
                } else if date == today {
                    ColorRole::Primary
                } else {
                    ColorRole::OnSurface
                });
                let background = tone(if active {
                    ColorRole::Primary
                } else {
                    ColorRole::SurfaceContainerHigh
                });
                let control = BaseButton::new((id.clone(), format!("day-{date}")))
                    .role(Role::ListBoxOption)
                    .aria_selected(range || active)
                    .aria_description(if date == today { "Today" } else { "" })
                    .accessibility_label((locale.day)(date))
                    .focusable(false)
                    .when(focus, |button| button.aria_active_descendant())
                    .disabled(!enabled)
                    .size(cell)
                    .when_some(
                        enabled.then(|| {
                            let state = state.clone();
                            let callback = docked_commit.clone();
                            Rc::new(move |window: &mut Window, cx: &mut App| {
                                let result = state.update(cx, |model, cx| {
                                    let result = model.select_day(date, cx);
                                    if result.is_some() && model.docked_open {
                                        model.dismiss_docked(window, cx);
                                    } else {
                                        model.day.focus(window, cx);
                                    }
                                    result
                                });
                                if let (Some(result), Some(callback)) = (result, &callback) {
                                    callback(result, window, cx);
                                }
                            })
                        }),
                        |button, activate| {
                            let click = activate.clone();
                            button
                                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                    activate(window, cx)
                                })
                                .on_click(move |_, window, cx| click(window, cx))
                        },
                    );
                div()
                    .size(cell)
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(range, |cell_box| {
                        cell_box.child(
                            div()
                                .absolute()
                                .h(cell.min(px(40.)))
                                .top((cell - cell.min(px(40.))) / 2.)
                                .left(if if rtl { trailing } else { leading } {
                                    cell / 2.
                                } else {
                                    px(0.)
                                })
                                .right(if if rtl { leading } else { trailing } {
                                    cell / 2.
                                } else {
                                    px(0.)
                                })
                                .bg(tone(ColorRole::SecondaryContainer)),
                        )
                    })
                    .child(
                        control.child(
                            div()
                                .size(cell.min(px(40.)))
                                .rounded_full()
                                .flex()
                                .justify_center()
                                .items_center()
                                .when(active, |day| day.bg(background))
                                .when(
                                    focus && focus_cursor && window.last_input_was_keyboard(),
                                    |day| day.border_2().border_color(tone(ColorRole::Secondary)),
                                )
                                .when(
                                    date == today
                                        && !active
                                        && !(focus
                                            && focus_cursor
                                            && window.last_input_was_keyboard()),
                                    |day| day.border_1().border_color(tone(ColorRole::Primary)),
                                )
                                .text_color(if enabled { ink } else { ink.opacity(0.38) })
                                .child(date.day().to_string()),
                        ),
                    )
                    .into_any_element()
            }))
    });
    let rows = rows.collect::<Vec<_>>();
    let grid = div()
        .id((id.clone(), "calendar-grid"))
        .test_support()
        .role(Role::ListBox)
        .aria_label((locale.month)(month))
        .track_focus(&day_focus)
        .tab_stop(true)
        .flex()
        .flex_col()
        .items_center()
        .on_action(move |_: &ConfirmAction, window, cx| {
            if confirm_focus.is_focused(window) {
                confirm(window, cx);
                cx.stop_propagation();
            } else {
                cx.propagate();
            }
        })
        .capture_key_down(move |event, window, cx| {
            if !keyboard_focus.is_focused(window) {
                return;
            }
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                select(window, cx);
                cx.stop_propagation();
                return;
            }
            if keys.update(cx, |model, cx| {
                model.navigate(
                    &event.keystroke.key,
                    event.keystroke.modifiers.control,
                    direction,
                    first_day,
                    cx,
                )
            }) {
                cx.stop_propagation();
            }
        })
        .child(
            div()
                .flex()
                .when(rtl, gpui::Styled::flex_row_reverse)
                .children(weekdays),
        )
        .children(rows);
    text_style(
        div()
            .w(width)
            .max_w_full()
            .mx_auto()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(header)
            .child(grid)
            .child(status(
                if model.input_attempted && complete(draft).is_none() {
                    "Select a date before confirming".into()
                } else {
                    selection_label(draft, &locale)
                },
                model.input_attempted && complete(draft).is_none(),
            )),
        TypeRole::BodyLarge.style(TypographyEmphasis::Baseline),
        1.,
        window,
        cx,
    )
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_date_parsing_rejects_invalid_disabled_and_reversed_endpoints() {
        let first = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let last = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let locale = DatePickerLocale::default().input_format(
            |date| date.format("%d/%m/%Y").to_string().into(),
            |text| NaiveDate::parse_from_str(text, "%d/%m/%Y").ok(),
        );
        let limits = DateLimits {
            bounds: first..=last,
            disabled: Arc::new(|date| date.day() == 10),
        };
        assert_eq!(
            parse_selection(" 29/02/2024 ", None, &locale, &limits),
            Ok(DateSelection::Single(Some(last)))
        );
        assert_eq!(
            parse_selection("30/02/2024", None, &locale, &limits),
            Err(DateError::Invalid)
        );
        assert_eq!(
            parse_selection("01/03/2024", None, &locale, &limits),
            Err(DateError::Disabled)
        );
        assert_eq!(
            parse_selection("01/02/2024", Some("10/02/2024"), &locale, &limits),
            Err(DateError::Disabled)
        );
        assert_eq!(
            parse_selection("29/02/2024", Some("01/02/2024"), &locale, &limits),
            Err(DateError::Order)
        );
        assert_eq!(
            parse_selection("01/02/2024", Some("29/02/2024"), &locale, &limits),
            Ok(DateSelection::Range(Some((first, last))))
        );
    }
}
