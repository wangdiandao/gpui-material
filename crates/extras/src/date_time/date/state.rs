use std::{cell::RefCell, ops::RangeInclusive, rc::Rc, sync::Arc};

use gpui::{
    App, AppContext as _, Context, Entity, EntityInputHandler as _, FocusHandle, Focusable as _,
    SharedString, WeakFocusHandle, Window,
};
use gpui_base::{CalendarState, Date as BaseDate, Matcher};
use gpui_material_components::{DialogCloseReason, DialogState, TextFieldState};
use gpui_material_theme::Direction;
use jiff::{
    ToSpan as _,
    civil::{Date, Weekday},
};

use super::types::{
    DateFilter, DateFormatter, DatePickerDisplayMode, DatePickerDraft, DatePickerLocale,
    DatePickerVariant, DateSelection, YearFilter, complete,
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
    pub(super) bounds: RangeInclusive<Date>,
    pub(super) disabled: DateFilter,
    pub(super) disabled_years: YearFilter,
}

impl DateLimits {
    pub(super) fn accepts(&self, date: Date) -> bool {
        self.bounds.contains(&date)
            && self.accepts_year(i32::from(date.year()))
            && !(self.disabled)(date)
    }

    pub(super) fn accepts_year(&self, year: i32) -> bool {
        (i32::from(self.bounds.start().year())..=i32::from(self.bounds.end().year()))
            .contains(&year)
            && !(self.disabled_years)(year)
    }

    pub(super) fn accepts_draft(&self, draft: BaseDate) -> bool {
        let start = draft.start();
        let end = draft.end();
        start.is_none_or(|date| self.accepts(date))
            && end.is_none_or(|date| start.is_some_and(|start| start <= date) && self.accepts(date))
    }
}

pub(super) fn first_of_month(date: Date) -> Date {
    date.first_of_month()
}

pub(super) fn calendar_start(month: Date, first_day: Weekday) -> Date {
    let offset = (i32::from(month.weekday().to_monday_zero_offset()) + 7
        - i32::from(first_day.to_monday_zero_offset()))
        % 7;
    month.checked_sub(offset.days()).unwrap_or(month)
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

pub(super) fn parse_docked_range(
    text: &str,
    locale: &DatePickerLocale,
    limits: &DateLimits,
) -> Option<DateSelection> {
    text.split_once(" – ")
        .or_else(|| text.split_once(" - "))
        .and_then(|(start, end)| parse_selection(start, Some(end), locale, limits).ok())
}

pub(super) struct MonthCache {
    month: Date,
    day_formatter: DateFormatter,
    month_formatter: DateFormatter,
    pub(super) month_label: SharedString,
    pub(super) day_ids: Vec<SharedString>,
    pub(super) day_labels: Vec<SharedString>,
}

impl MonthCache {
    fn matches(&self, month: Date, locale: &DatePickerLocale) -> bool {
        self.month == month
            && Rc::ptr_eq(&self.day_formatter, &locale.day)
            && Rc::ptr_eq(&self.month_formatter, &locale.month)
    }

    fn build(month: Date, locale: &DatePickerLocale) -> Self {
        let mut day_ids = Vec::with_capacity(31);
        let mut day_labels = Vec::with_capacity(31);
        let mut current = month;
        while current.month() == month.month() {
            day_ids.push(SharedString::from(format!("day-{current}")));
            day_labels.push((locale.day)(current));
            let Ok(next) = current.tomorrow() else {
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
    pub(super) cursor: Date,
    pub(super) docked_open: bool,
    pub(super) docked_generation: u64,
    docked_previous: Option<WeakFocusHandle>,
    docked_reason: Option<DialogCloseReason>,
    variant: DatePickerVariant,
    pub(super) locale: DatePickerLocale,
    pub(super) locale_initialized: bool,
    month_cache: RefCell<Option<MonthCache>>,
    initial_month: Option<Date>,
    pub(super) year_page: Option<i32>,
    pub(super) input_mode: bool,
    pub(super) input_attempted: bool,
    pub(super) limits: DateLimits,
}

impl DatePickerState {
    /// Creates a new date picker state with default bounds (`1900-01-01` through `2100-12-31`).
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let limits = DateLimits {
            bounds: Date::constant(1900, 1, 1)..=Date::constant(2100, 12, 31),
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
    pub fn date_bounds(mut self, bounds: RangeInclusive<Date>) -> Self {
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
        disabled: impl Fn(Date) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.limits.disabled = Arc::new(disabled);
        self
    }

    /// Selects which month is initially displayed on each open.
    pub fn initial_month(mut self, date: Date) -> Self {
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

    /// Returns the retained focus handle of the built-in picker trigger.
    pub fn trigger_focus(&self) -> &FocusHandle {
        &self.trigger
    }

    /// Returns whether the docked popup or modal dialog is currently open.
    pub fn is_open(&self, cx: &App) -> bool {
        self.docked_open || self.dialog.read(cx).is_open()
    }

    /// Returns the reason the picker was most recently closed, if any.
    pub fn last_close_reason(&self, cx: &App) -> Option<DialogCloseReason> {
        if self.variant == DatePickerVariant::Docked {
            self.docked_reason
        } else {
            self.dialog.read(cx).last_close_reason()
        }
    }

    /// Updates presentation and cursor without changing the draft.
    pub fn set_displayed_month(&mut self, date: Date, cx: &mut Context<Self>) {
        let clamped = date.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
        let month_changed = self.show_month(clamped, cx);
        let changed = self.cursor != clamped || self.year_page.take().is_some() || month_changed;
        self.cursor = clamped;
        if changed {
            cx.notify();
        }
    }

    /// Returns the first day of the month currently shown by the calendar cursor.
    pub fn displayed_month(&self) -> Date {
        first_of_month(self.cursor)
    }

    pub(super) fn month_cache(
        &self,
        month: Date,
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

    /// Reads the current in-progress draft from the active display mode.
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
        let draft: BaseDate = draft.into();
        if !self.limits.accepts_draft(draft) {
            return false;
        }
        self.apply_draft(draft, cx);
        if let Some(active) = draft.start() {
            self.cursor = active.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
            self.year_page = None;
        }
        self.show_month(self.cursor, cx);
        self.reset_input(draft, &self.locale, window, cx);
        self.input_attempted = false;
        cx.notify();
        true
    }

    /// Returns the active display mode (`Calendar` or `Input`).
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

    fn apply_draft(&self, draft: BaseDate, cx: &mut Context<Self>) {
        let limits = self.limits.clone();
        self.calendar.update(cx, |calendar, _| {
            calendar.set_disabled_matcher_shared(Some(Rc::new(Matcher::custom(move |date| {
                !limits.accepts(*date)
            }))));
            calendar.apply_date(draft);
        });
    }

    fn show_month(&self, date: Date, cx: &mut Context<Self>) -> bool {
        self.calendar.update(cx, |calendar, _| {
            let year = i32::from(date.year());
            let month = date.month() as u32;
            let changed = calendar.offset_year_month(0) != (year, month);
            if changed {
                calendar.select_year(year);
                calendar.select_month(month as u8);
            }
            changed
        })
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
            BaseDate::Single(None)
        } else {
            BaseDate::Range(None, None)
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
        draft: BaseDate,
        locale: &DatePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let format = |date: Option<Date>| {
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

    fn input_draft(&self, cx: &App) -> BaseDate {
        let parse = |field: &Entity<TextFieldState>| {
            let text = field.read(cx).value();
            (self.locale.parse)(text.trim()).filter(|date| self.limits.accepts(*date))
        };
        let start = parse(&self.start);
        if self.calendar.read(cx).date().is_single() {
            return BaseDate::Single(start);
        }
        let end = parse(&self.end).filter(|end| start.is_some_and(|start| start <= *end));
        BaseDate::Range(start, end)
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
        date: Date,
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
        let offset = (i32::from(self.cursor.weekday().to_monday_zero_offset()) + 7
            - i32::from(first_day.to_monday_zero_offset()))
            % 7;
        let candidate = match key {
            "left" | "right" => {
                if (key == "left") != rtl {
                    self.cursor.yesterday().ok()
                } else {
                    self.cursor.tomorrow().ok()
                }
            }
            "up" => self.cursor.checked_sub(7.days()).ok(),
            "down" => self.cursor.checked_add(7.days()).ok(),
            "home" => self.cursor.checked_sub(offset.days()).ok(),
            "end" => self.cursor.checked_add((6 - offset).days()).ok(),
            "pageup" => self
                .cursor
                .checked_sub(if control { 12.months() } else { 1.month() })
                .ok(),
            "pagedown" => self
                .cursor
                .checked_add(if control { 12.months() } else { 1.month() })
                .ok(),
            _ => return false,
        };
        if let Some(date) = candidate {
            let clamped = date.clamp(*self.limits.bounds.start(), *self.limits.bounds.end());
            let month_changed = self.show_month(clamped, cx);
            if self.cursor != clamped || month_changed {
                self.cursor = clamped;
                cx.notify();
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_dmy(text: &str) -> Option<Date> {
        let mut parts = text.split('/');
        let day: i8 = parts.next()?.parse().ok()?;
        let month: i8 = parts.next()?.parse().ok()?;
        let year: i16 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Date::new(year, month, day).ok()
    }

    #[test]
    fn local_date_parsing_rejects_invalid_disabled_and_reversed_endpoints() {
        let first = Date::constant(2024, 2, 1);
        let last = Date::constant(2024, 2, 29);
        let locale = DatePickerLocale::default().input_format(
            |date| format!("{:02}/{:02}/{:04}", date.day(), date.month(), date.year()).into(),
            parse_dmy,
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

    #[test]
    fn docked_range_parsing_accepts_ascii_hyphen_and_unicode_en_dash() {
        let start = Date::constant(2026, 6, 10);
        let end = Date::constant(2026, 6, 20);
        let locale = DatePickerLocale::default();
        let limits = DateLimits {
            bounds: Date::constant(2026, 1, 1)..=Date::constant(2026, 12, 31),
            disabled: Arc::new(|_| false),
            disabled_years: Arc::new(|_| false),
        };
        assert_eq!(
            parse_docked_range("2026-06-10 – 2026-06-20", &locale, &limits),
            Some(DateSelection::Range(Some((start, end))))
        );
        assert_eq!(
            parse_docked_range("2026-06-10 - 2026-06-20", &locale, &limits),
            Some(DateSelection::Range(Some((start, end))))
        );
        assert_eq!(
            parse_docked_range("2026-06-20 - 2026-06-10", &locale, &limits),
            None
        );
    }

    #[test]
    fn jiff_date_bounds_and_default_locale_formatting() {
        let locale = DatePickerLocale::default();
        let date = Date::constant(2026, 6, 12);
        assert_eq!((locale.format)(date).as_ref(), "2026-06-12");
        assert_eq!((locale.parse)("2026-06-12"), Some(date));
        assert_eq!((locale.parse)("2026-02-30"), None);
        assert_eq!((locale.month)(date).as_ref(), "June 2026");
        assert_eq!((locale.day)(date).as_ref(), "Friday, June 12, 2026");
        assert_eq!((locale.weekday)(Weekday::Sunday).as_ref(), "Sun");
        assert_eq!(first_of_month(date), Date::constant(2026, 6, 1));
        assert_eq!(
            calendar_start(Date::constant(2026, 6, 1), Weekday::Sunday),
            Date::constant(2026, 5, 31)
        );
        assert_eq!(
            calendar_start(Date::constant(2026, 6, 1), Weekday::Monday),
            Date::constant(2026, 6, 1)
        );
    }

    #[cfg(feature = "test-support")]
    #[gpui::test]
    fn set_draft_syncs_cursor_month_and_date_picker_on_closed_fires(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;

        use gpui::{IntoElement, Render, WindowOptions};
        use gpui_material_foundation::MaterialRoot;
        use gpui_material_theme::MaterialTheme;

        use crate::date_time::date::DatePicker;

        struct Fixture {
            state: Entity<DatePickerState>,
            closed: Rc<RefCell<Vec<DialogCloseReason>>>,
        }

        impl Render for Fixture {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let closed = self.closed.clone();
                DatePicker::new(
                    "draft-sync-picker",
                    "Pick date",
                    &self.state,
                    DateSelection::Single(Some(Date::constant(2026, 6, 12))),
                )
                .variant(DatePickerVariant::Docked)
                .on_closed(move |reason, _, _| closed.borrow_mut().push(reason))
            }
        }

        let closed = Rc::new(RefCell::new(Vec::new()));
        let mut state = None;
        let window: gpui::AnyWindowHandle = cx
            .update(|cx| {
                MaterialRoot::init(MaterialTheme::default(), cx);
                gpui_material_components::init_components(cx);
                cx.set_reduce_motion(true);
                MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                    let picker = cx.new(|cx| {
                        DatePickerState::new(window, cx)
                            .date_bounds(Date::constant(2024, 1, 1)..=Date::constant(2028, 12, 31))
                            .initial_month(Date::constant(2026, 6, 1))
                    });
                    state = Some(picker.clone());
                    cx.new(|cx| {
                        cx.observe(&picker, |_, _, cx| cx.notify()).detach();
                        Fixture {
                            state: picker,
                            closed: closed.clone(),
                        }
                    })
                })
                .unwrap()
            })
            .into();
        let state = state.unwrap();

        cx.update_window(window, |_, window, cx| {
            state.update(cx, |picker, cx| {
                assert_eq!(picker.displayed_month(), Date::constant(2026, 6, 1));
                assert!(picker.set_draft(
                    DatePickerDraft::Single(Some(Date::constant(2027, 10, 15))),
                    window,
                    cx,
                ));
                assert_eq!(picker.displayed_month(), Date::constant(2027, 10, 1));
                picker.open(
                    DateSelection::Single(Some(Date::constant(2026, 6, 12))),
                    DatePickerVariant::Docked,
                    &DatePickerLocale::default(),
                    window,
                    cx,
                );
            });
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();

        cx.simulate_keystrokes(window, "escape");
        cx.update_window(window, |_, window, cx| {
            window.draw(cx).clear(cx);
            assert!(!state.read(cx).is_open(cx));
        })
        .unwrap();
        assert_eq!(*closed.borrow(), [DialogCloseReason::Escape]);
    }
}
