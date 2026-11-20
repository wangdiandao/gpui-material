use std::{cell::Cell, ops::RangeInclusive, rc::Rc, sync::Arc};

use chrono::{Datelike, Days, Months};
use gpui::{
    AccessibleAction, Anchor, AnyElement, App, AppContext as _, Bounds, Context, ElementId, Entity,
    EntityInputHandler as _, FocusHandle, Focusable as _, Hsla, InteractiveElement as _,
    IntoElement, ParentElement as _, Pixels, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, WeakFocusHandle, Window, canvas, div,
    prelude::FluentBuilder as _, px,
};
use gpui_base::{
    Button as BaseButton, CalendarState, Date, Matcher, Popup, TestSupportExt as _,
    actions::Confirm as ConfirmAction,
};
use gpui_material_components::{
    Button, ButtonColors, ButtonVariant, Dialog, DialogAction, DialogCloseReason, DialogColors,
    DialogProperties, DialogState, DialogVariant, Icon, IconButton, IconName, Text, TextField,
    TextFieldColors, TextFieldState, TextFieldVariant,
};
use gpui_material_foundation::{
    Surface, ThemeScope, color, current_material_theme, logical_anchor, text_style,
};
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
    disabled_years: YearFilter,
}

impl DateLimits {
    fn accepts(&self, date: NaiveDate) -> bool {
        self.bounds.contains(&date) && self.accepts_year(date.year()) && !(self.disabled)(date)
    }

    fn accepts_year(&self, year: i32) -> bool {
        (self.bounds.start().year()..=self.bounds.end().year()).contains(&year)
            && !(self.disabled_years)(year)
    }

    fn accepts_draft(&self, draft: Date) -> bool {
        let start = draft.start();
        let end = draft.end();
        start.is_none_or(|date| self.accepts(date))
            && end.is_none_or(|date| start.is_some_and(|start| start <= date) && self.accepts(date))
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
    anchor: Entity<TextFieldState>,
    anchor_value: Option<SharedString>,
    dialog: Entity<DialogState>,
    trigger: FocusHandle,
    docked: FocusHandle,
    day: FocusHandle,
    cursor: NaiveDate,
    docked_open: bool,
    docked_generation: u64,
    docked_previous: Option<WeakFocusHandle>,
    docked_reason: Option<DialogCloseReason>,
    variant: DatePickerVariant,
    locale: DatePickerLocale,
    locale_initialized: bool,
    initial_month: Option<NaiveDate>,
    year_page: Option<i32>,
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
            disabled_years: Arc::new(|_| false),
        };
        let calendar = cx.new(|cx| CalendarState::new(window, cx));
        let cursor = calendar.read(cx).today();
        let start = cx.new(|cx| TextFieldState::new(window, cx));
        let end = cx.new(|cx| TextFieldState::new(window, cx));
        let anchor = cx.new(|cx| TextFieldState::new(window, cx));
        cx.observe(&anchor, |_, _, cx| cx.notify()).detach();
        let dialog = cx.new(DialogState::new);
        cx.observe(&start, |_, _, cx| cx.notify()).detach();
        cx.observe(&end, |_, _, cx| cx.notify()).detach();
        cx.observe(&dialog, |_, _, cx| cx.notify()).detach();
        Self {
            calendar,
            start,
            end,
            anchor,
            anchor_value: None,
            dialog,
            trigger: cx.focus_handle(),
            docked: cx.focus_handle(),
            day: cx.focus_handle(),
            cursor,
            docked_open: false,
            docked_generation: 0,
            docked_previous: None,
            docked_reason: None,
            variant: DatePickerVariant::default(),
            locale: DatePickerLocale::default(),
            locale_initialized: false,
            initial_month: None,
            year_page: None,
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
        self.cursor = self
            .cursor
            .clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
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

    /// Selects which month is initially displayed on each open.
    pub fn initial_month(mut self, date: NaiveDate) -> Self {
        self.initial_month = Some(date);
        self.cursor = date.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
        self
    }

    /// Marks every date in matching years unavailable, independently of day filtering.
    pub fn disabled_years(
        mut self,
        disabled: impl Fn(i32) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.limits.disabled_years = Arc::new(disabled);
        self
    }

    pub fn is_open(&self, cx: &App) -> bool {
        self.docked_open || self.dialog.read(cx).is_open()
    }

    pub fn last_close_reason(&self, cx: &App) -> Option<DialogCloseReason> {
        if self.variant == DatePickerVariant::Docked {
            self.docked_reason
        } else {
            self.dialog.read(cx).last_close_reason()
        }
    }

    /// Updates presentation and cursor without changing the draft.
    pub fn set_displayed_month(&mut self, date: NaiveDate, cx: &mut Context<Self>) {
        self.cursor = date.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
        self.show_month(self.cursor, cx);
        self.year_page = None;
        cx.notify();
    }

    pub fn displayed_month(&self) -> NaiveDate {
        first_of_month(self.cursor)
    }

    pub fn draft(&self, cx: &App) -> DatePickerDraft {
        if self.input_mode {
            self.input_draft(cx).into()
        } else {
            self.calendar.read(cx).date().into()
        }
    }

    /// Returns the retained start editor and the end editor for a range draft.
    pub fn input_editors(
        &self,
        cx: &App,
    ) -> (&Entity<TextFieldState>, Option<&Entity<TextFieldState>>) {
        let range = !self.calendar.read(cx).date().is_single();
        (&self.start, range.then_some(&self.end))
    }

    /// Applies a valid draft, including empty values and a range with only a start.
    pub fn set_draft(
        &mut self,
        draft: DatePickerDraft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let draft = draft.into();
        if !self.limits.accepts_draft(draft) {
            return false;
        }
        self.apply_draft(draft, cx);
        self.show_month(self.cursor, cx);
        self.reset_input(draft, &self.locale, window, cx);
        self.input_attempted = false;
        cx.notify();
        true
    }

    pub fn display_mode(&self) -> DatePickerDisplayMode {
        if self.input_mode {
            DatePickerDisplayMode::Input
        } else {
            DatePickerDisplayMode::Calendar
        }
    }

    /// Changes display mode after any active IME composition has finished.
    pub fn set_display_mode(
        &mut self,
        mode: DatePickerDisplayMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let input = mode == DatePickerDisplayMode::Input;
        if self.input_mode == input {
            return true;
        }
        if self.input_mode && self.input_composing(window, cx) {
            return false;
        }
        self.input_attempted = false;
        if self.input_mode {
            let draft = self.input_draft(cx);
            self.apply_draft(draft, cx);
            if let Some(day) = draft.start() {
                self.set_displayed_month(day, cx);
            }
        } else {
            self.reset_input(self.calendar.read(cx).date(), &self.locale, window, cx);
        }
        self.input_mode = input;
        self.year_page = None;
        if self.is_open(cx) {
            self.default_focus(cx).focus(window, cx);
        }
        cx.notify();
        true
    }

    fn default_focus(&self, cx: &App) -> FocusHandle {
        if self.input_mode {
            self.start.read(cx).focus_handle(cx)
        } else {
            self.day.clone()
        }
    }

    fn input_composing(&self, window: &mut Window, cx: &mut App) -> bool {
        [&self.start, &self.end].into_iter().any(|field| {
            field.update(cx, |field, cx| {
                field.marked_text_range(window, cx).is_some()
            })
        })
    }

    fn apply_draft(&self, draft: Date, cx: &mut Context<Self>) {
        let limits = self.limits.clone();
        self.calendar.update(cx, |calendar, _| {
            calendar.set_disabled_matcher_shared(Some(Rc::new(Matcher::custom(move |date| {
                !limits.accepts(*date)
            }))));
            calendar.apply_date(draft);
        });
    }

    fn show_month(&self, date: NaiveDate, cx: &mut Context<Self>) {
        self.calendar.update(cx, |calendar, _| {
            calendar.select_year(date.year());
            calendar.select_month(date.month() as u8);
        });
    }

    /// Opens a picker rendered by a caller-owned trigger and copies the committed value.
    pub fn open(
        &mut self,
        value: DateSelection,
        variant: DatePickerVariant,
        locale: &DatePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_open(cx) {
            return;
        }
        self.variant = variant;
        self.locale = locale.clone();
        self.locale_initialized = true;
        let draft = value.draft();
        let draft = if self.limits.accepts_draft(draft) {
            draft
        } else if draft.is_single() {
            Date::Single(None)
        } else {
            Date::Range(None, None)
        };
        self.apply_draft(draft, cx);
        let today = self.calendar.read(cx).today();
        self.set_displayed_month(self.initial_month.or(draft.start()).unwrap_or(today), cx);
        self.reset_input(draft, locale, window, cx);
        self.input_mode = variant == DatePickerVariant::ModalInput;
        self.input_attempted = false;
        if variant == DatePickerVariant::Docked {
            self.docked_previous = window.focused(cx).map(|focus| focus.downgrade());
            self.docked_open = true;
            self.docked_generation = self.docked_generation.wrapping_add(1);
        } else {
            self.dialog.update(cx, |dialog, cx| dialog.open(window, cx));
        }
        self.default_focus(cx).focus(window, cx);
        cx.notify();
    }

    /// Closes either presentation and restores focus when the picker still owns it.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.docked_open {
            self.dismiss_docked(DialogCloseReason::Programmatic, window, cx);
        } else {
            self.dialog
                .update(cx, |dialog, cx| dialog.close(window, cx));
        }
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

    fn input_draft(&self, cx: &App) -> Date {
        let parse = |field: &Entity<TextFieldState>| {
            let text = field.read(cx).value();
            (self.locale.parse)(text.trim()).filter(|date| self.limits.accepts(*date))
        };
        let start = parse(&self.start);
        if self.calendar.read(cx).date().is_single() {
            return Date::Single(start);
        }
        let end = parse(&self.end).filter(|end| start.is_some_and(|start| start <= *end));
        Date::Range(start, end)
    }

    fn dismiss_docked(
        &mut self,
        reason: DialogCloseReason,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.docked_open {
            return false;
        }
        self.docked_open = false;
        self.docked_reason = Some(reason);
        let previous = self.docked_previous.take();
        if self.docked.contains_focused(window, cx) || self.docked.is_focused(window) {
            if let Some(focus) = previous.and_then(|focus| focus.upgrade()) {
                focus.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        cx.notify();
        true
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
            return popup.into_any_element();
        }

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
}

type AnchorBounds = Rc<Cell<Option<Bounds<Pixels>>>>;

impl DatePicker {
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
            calendar(
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

const YEARS_PER_PAGE: i32 = 12;

fn year_page(year: i32, first: i32) -> i32 {
    first + (year - first) / YEARS_PER_PAGE * YEARS_PER_PAGE
}

fn year_choices(
    id: &ElementId,
    state: &Entity<DatePickerState>,
    from: i32,
    colors: DatePickerColors,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let first = model.limits.bounds.start().year();
    let last = model.limits.bounds.end().year();
    let displayed = model.displayed_month();
    let today = model.calendar.read(cx).today();
    let selected_year = displayed.year();
    let direction = current_material_theme(window, cx).layout.direction;
    let rtl = direction == Direction::Rtl;
    let page_change = |delta: i32| {
        let state = state.clone();
        move |_: &mut Window, cx: &mut App| {
            state.update(cx, |model, cx| {
                model.year_page = Some((from + delta).clamp(first, last));
                cx.notify();
            });
        }
    };
    let items = (from..(from + YEARS_PER_PAGE).min(last + 1)).map(|year| {
        let state = state.clone();
        let selected = selected_year == year;
        let enabled = model.limits.accepts_year(year);
        let content = if selected {
            colors.selected_year
        } else if today.year() == year {
            colors.current_year
        } else {
            colors.year
        };
        let container = selected.then_some(colors.selected_year_container).flatten();
        Button::new((id.clone(), format!("year-{year}")), year.to_string())
            .variant(if selected {
                ButtonVariant::Filled
            } else {
                ButtonVariant::Text
            })
            .disabled(!enabled)
            .colors(ButtonColors {
                content,
                container,
                disabled_content: if selected {
                    colors.disabled_selected_year
                } else {
                    colors.disabled_year
                },
                disabled_container: selected
                    .then_some(colors.disabled_selected_year_container)
                    .flatten(),
                ..ButtonColors::default()
            })
            .on_activate(move |window, cx| {
                state.update(cx, |model, cx| {
                    if model.limits.accepts_year(year)
                        && let Some(target) = NaiveDate::from_ymd_opt(year, displayed.month(), 1)
                    {
                        model.set_displayed_month(target, cx);
                        model.day.focus(window, cx);
                    }
                });
            })
    });
    let switch = state.clone();
    div()
        .id((id.clone(), "year-list"))
        .test_support()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .when(rtl, gpui::Styled::flex_row_reverse)
                .child(
                    Button::new((id.clone(), "previous-years"), "Earlier years")
                        .variant(ButtonVariant::Text)
                        .disabled(from <= first)
                        .colors(ButtonColors {
                            content: colors.navigation,
                            ..ButtonColors::default()
                        })
                        .on_activate(page_change(-YEARS_PER_PAGE)),
                )
                .child(
                    Button::new((id.clone(), "return-month"), "Show month")
                        .variant(ButtonVariant::Text)
                        .colors(ButtonColors {
                            content: colors.subheading,
                            ..ButtonColors::default()
                        })
                        .on_activate(move |window, cx| {
                            switch.update(cx, |model, cx| {
                                model.year_page = None;
                                model.day.focus(window, cx);
                                cx.notify();
                            });
                        }),
                )
                .child(
                    Button::new((id.clone(), "next-years"), "Later years")
                        .variant(ButtonVariant::Text)
                        .disabled(from + YEARS_PER_PAGE > last)
                        .colors(ButtonColors {
                            content: colors.navigation,
                            ..ButtonColors::default()
                        })
                        .on_activate(page_change(YEARS_PER_PAGE)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(4.))
                .when(rtl, gpui::Styled::flex_row_reverse)
                .children(items),
        )
        .into_any_element()
}

fn activate_day(
    state: &Entity<DatePickerState>,
    docked: Option<&DockedActions>,
    date: NaiveDate,
    window: &mut Window,
    cx: &mut App,
) {
    if docked.is_some_and(|actions| !actions.current(cx)) {
        return;
    }
    let picked = state.update(cx, |model, cx| model.select_day(date, cx));
    if let (Some(actions), Some(value)) = (docked, picked) {
        actions.finish(DialogCloseReason::Confirm, Some(value), window, cx);
    } else {
        state.read(cx).day.clone().focus(window, cx);
    }
}

fn calendar(
    id: &ElementId,
    state: Entity<DatePickerState>,
    locale: DatePickerLocale,
    docked: Option<DockedActions>,
    presentation: (DatePickerColors, Pixels),
    window: &Window,
    cx: &App,
) -> AnyElement {
    let (colors, width) = presentation;
    let model = state.read(cx);
    if let Some(page) = model.year_page {
        return year_choices(id, &state, page, colors, window, cx);
    }
    let month = model.displayed_month();
    let draft = model.calendar.read(cx).date();
    let today = model.calendar.read(cx).today();
    let cursor = model.cursor;
    let start = calendar_start(month, locale.first_day);
    let theme = current_material_theme(window, cx);
    let direction = theme.layout.direction;
    let rtl = direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let cell = (width / 7.).clamp(px(16.), px(48.));
    let day_focus = model.day.clone();
    let keyboard_focus = day_focus.clone();
    let confirm_focus = day_focus.clone();
    let focus_cursor = day_focus.is_focused(window);
    let keys = state.clone();
    let key_actions = docked.clone();
    let key_selection = state.clone();
    let select = Rc::new(move |window: &mut Window, cx: &mut App| {
        let cursor = key_selection.read(cx).cursor;
        activate_day(&key_selection, key_actions.as_ref(), cursor, window, cx);
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
    let show_years = state.clone();
    let navigation = ButtonColors {
        content: colors.navigation,
        ..ButtonColors::default()
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
            .colors(navigation)
            .disabled(month <= first_of_month(*model.limits.bounds.start()))
            .on_activate(move_month(prev, "pageup")),
        )
        .child(
            Button::new(
                (id.clone(), "year-toggle"),
                format!("Choose year, {}", (locale.month)(month)),
            )
            .variant(ButtonVariant::Text)
            .colors(ButtonColors {
                content: colors.subheading,
                ..ButtonColors::default()
            })
            .content(
                Text::new((locale.month)(month))
                    .id("current-month")
                    .role(TypeRole::TitleSmall),
            )
            .on_activate(move |_, cx| {
                show_years.update(cx, |model, cx| {
                    model.year_page =
                        Some(year_page(month.year(), model.limits.bounds.start().year()));
                    cx.notify();
                });
            }),
        )
        .child(
            IconButton::new(
                (id.clone(), "next-month"),
                "Next month",
                Icon::new(IconName::ChevronRight).mirror_in_rtl(true),
            )
            .colors(navigation)
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
            .when_some(colors.weekday, gpui::Styled::text_color)
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
                let (ink, selected_background) = match (enabled, active) {
                    (true, true) => (
                        colors
                            .selected_day
                            .unwrap_or_else(|| tone(ColorRole::OnPrimary)),
                        colors
                            .selected_day_container
                            .unwrap_or_else(|| tone(ColorRole::Primary)),
                    ),
                    (false, true) => (
                        colors
                            .disabled_selected_day
                            .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.38)),
                        colors
                            .disabled_selected_day_container
                            .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.12)),
                    ),
                    (true, false) => (
                        if range {
                            colors
                                .range_day
                                .unwrap_or_else(|| tone(ColorRole::OnSecondaryContainer))
                        } else if date == today {
                            colors.today.unwrap_or_else(|| tone(ColorRole::Primary))
                        } else {
                            colors.day.unwrap_or_else(|| tone(ColorRole::OnSurface))
                        },
                        gpui::transparent_black(),
                    ),
                    (false, false) => (
                        colors
                            .disabled_day
                            .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.38)),
                        gpui::transparent_black(),
                    ),
                };
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
                            let actions = docked.clone();
                            Rc::new(move |window: &mut Window, cx: &mut App| {
                                activate_day(&state, actions.as_ref(), date, window, cx);
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
                                .bg(colors
                                    .range_container
                                    .unwrap_or_else(|| tone(ColorRole::SecondaryContainer))),
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
                                .when(active, |day| day.bg(selected_background))
                                .when(
                                    focus && focus_cursor && window.last_input_was_keyboard(),
                                    |day| {
                                        day.border_2().border_color(
                                            colors
                                                .focus_outline
                                                .unwrap_or_else(|| tone(ColorRole::Secondary)),
                                        )
                                    },
                                )
                                .when(
                                    date == today
                                        && !active
                                        && !(focus
                                            && focus_cursor
                                            && window.last_input_was_keyboard()),
                                    |day| {
                                        day.border_1().border_color(
                                            colors
                                                .today_outline
                                                .unwrap_or_else(|| tone(ColorRole::Primary)),
                                        )
                                    },
                                )
                                .text_color(ink)
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
            .child(grid),
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
            disabled_years: Arc::new(|_| false),
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
