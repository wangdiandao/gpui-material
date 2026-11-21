use std::{cell::RefCell, ops::RangeInclusive, rc::Rc, sync::Arc};

use chrono::{Datelike, Days, Months};
use gpui::{
    App, AppContext as _, Context, Entity, EntityInputHandler as _, FocusHandle, Focusable as _,
    SharedString, WeakFocusHandle, Window,
};
use gpui_base::{CalendarState, Date, Matcher};
use gpui_material_components::{DialogCloseReason, DialogState, TextFieldState};
use gpui_material_theme::Direction;

use super::{
    DateFilter, DateFormatter, DatePickerDisplayMode, DatePickerDraft, DatePickerLocale,
    DatePickerVariant, DateSelection, NaiveDate, Weekday, YearFilter, complete,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DateError {
    Invalid,
    Disabled,
    Order,
}

impl DateError {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::Invalid => "Enter a valid date",
            Self::Disabled => "Choose an available date",
            Self::Order => "End date must be on or after start date",
        }
    }
}

#[derive(Clone)]
pub(super) struct DateLimits {
    pub(super) bounds: RangeInclusive<NaiveDate>,
    pub(super) disabled: DateFilter,
    pub(super) disabled_years: YearFilter,
}

impl DateLimits {
    pub(super) fn accepts(&self, date: NaiveDate) -> bool {
        self.bounds.contains(&date) && self.accepts_year(date.year()) && !(self.disabled)(date)
    }

    pub(super) fn accepts_year(&self, year: i32) -> bool {
        (self.bounds.start().year()..=self.bounds.end().year()).contains(&year)
            && !(self.disabled_years)(year)
    }

    pub(super) fn accepts_draft(&self, draft: Date) -> bool {
        let start = draft.start();
        let end = draft.end();
        start.is_none_or(|date| self.accepts(date))
            && end.is_none_or(|date| start.is_some_and(|start| start <= date) && self.accepts(date))
    }
}

pub(super) fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("every month has a first day")
}

pub(super) fn calendar_start(month: NaiveDate, first_day: Weekday) -> NaiveDate {
    let offset =
        (month.weekday().num_days_from_monday() + 7 - first_day.num_days_from_monday()) % 7;
    month
        .checked_sub_days(Days::new(u64::from(offset)))
        .unwrap_or(month)
}

pub(super) fn parse_selection(
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

pub(super) struct MonthCache {
    month: NaiveDate,
    day_formatter: DateFormatter,
    month_formatter: DateFormatter,
    pub(super) month_label: SharedString,
    pub(super) day_ids: Vec<SharedString>,
    pub(super) day_labels: Vec<SharedString>,
}

impl MonthCache {
    fn matches(&self, month: NaiveDate, locale: &DatePickerLocale) -> bool {
        self.month == month
            && Rc::ptr_eq(&self.day_formatter, &locale.day)
            && Rc::ptr_eq(&self.month_formatter, &locale.month)
    }

    fn build(month: NaiveDate, locale: &DatePickerLocale) -> Self {
        let mut day_ids = Vec::with_capacity(31);
        let mut day_labels = Vec::with_capacity(31);
        let mut current = month;
        while current.month() == month.month() {
            day_ids.push(SharedString::from(format!("day-{current}")));
            day_labels.push((locale.day)(current));
            let Some(next) = current.succ_opt() else {
                break;
            };
            current = next;
        }
        Self {
            month,
            day_formatter: locale.day.clone(),
            month_formatter: locale.month.clone(),
            month_label: (locale.month)(month),
            day_ids,
            day_labels,
        }
    }
}

/// Owns the Base calendar, input editors, and draft focus. The application owns
/// the value passed to the picker. Every open copies that value into the draft.
pub struct DatePickerState {
    pub(super) calendar: Entity<CalendarState>,
    pub(super) start: Entity<TextFieldState>,
    pub(super) end: Entity<TextFieldState>,
    pub(super) anchor: Entity<TextFieldState>,
    pub(super) anchor_value: Option<SharedString>,
    pub(super) dialog: Entity<DialogState>,
    pub(super) trigger: FocusHandle,
    pub(super) docked: FocusHandle,
    pub(super) day: FocusHandle,
    pub(super) cursor: NaiveDate,
    pub(super) docked_open: bool,
    pub(super) docked_generation: u64,
    docked_previous: Option<WeakFocusHandle>,
    docked_reason: Option<DialogCloseReason>,
    variant: DatePickerVariant,
    pub(super) locale: DatePickerLocale,
    pub(super) locale_initialized: bool,
    month_cache: RefCell<Option<MonthCache>>,
    initial_month: Option<NaiveDate>,
    pub(super) year_page: Option<i32>,
    pub(super) input_mode: bool,
    pub(super) input_attempted: bool,
    pub(super) limits: DateLimits,
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
            month_cache: RefCell::new(None),
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

    pub(super) fn month_cache(
        &self,
        month: NaiveDate,
        locale: &DatePickerLocale,
    ) -> std::cell::Ref<'_, MonthCache> {
        {
            let mut cache = self.month_cache.borrow_mut();
            if cache
                .as_ref()
                .is_none_or(|cached| !cached.matches(month, locale))
            {
                *cache = Some(MonthCache::build(month, locale));
            }
        }
        std::cell::Ref::map(self.month_cache.borrow(), |cache| {
            cache.as_ref().expect("month cache initialized")
        })
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

    pub(super) fn input_composing(&self, window: &mut Window, cx: &mut App) -> bool {
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

    pub(super) fn reset_input(
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

    pub(super) fn input_value(
        &self,
        locale: &DatePickerLocale,
        cx: &App,
    ) -> Result<DateSelection, DateError> {
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

    pub(super) fn dismiss_docked(
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

    pub(super) fn select_day(
        &mut self,
        date: NaiveDate,
        cx: &mut Context<Self>,
    ) -> Option<DateSelection> {
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

    pub(super) fn navigate(
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
