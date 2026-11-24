use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{AnyElement, App, Bounds, Entity, FocusHandle, Hsla, Pixels, SharedString, Window};
use gpui_base::Date as BaseDate;
use gpui_material_components::{
    DialogAction, DialogCloseReason, DialogProperties, TextField, TextFieldColors,
};
use jiff::civil::{Date, Weekday};

use super::state::DatePickerState;
use crate::date_time::Confirm;

pub(super) type DateFormatter = Rc<dyn Fn(Date) -> SharedString>;
pub(super) type DateParser = Rc<dyn Fn(&str) -> Option<Date>>;
pub(super) type WeekdayFormatter = Rc<dyn Fn(Weekday) -> SharedString>;
pub(super) type DateFilter = Arc<dyn Fn(Date) -> bool + Send + Sync>;
pub(super) type YearFilter = Arc<dyn Fn(i32) -> bool + Send + Sync>;
pub(super) type OnClose = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;
pub(super) type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
pub(super) type CustomAnchor = Box<dyn FnOnce(TextField) -> AnyElement>;
pub(super) type CustomAction = Box<dyn FnOnce(DialogAction) -> AnyElement>;
pub(super) type AnchorBounds = Rc<Cell<Option<Bounds<Pixels>>>>;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn month_name(month: i8) -> &'static str {
    MONTH_NAMES
        .get(usize::try_from(month - 1).unwrap_or(0))
        .copied()
        .unwrap_or("")
}

fn weekday_full_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

fn weekday_short_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

/// A committed picker value. The empty variants preserve the selection kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelection {
    /// A single-date selection, or `None` when cleared.
    Single(Option<Date>),
    /// An inclusive start-and-end date range, or `None` when cleared.
    Range(Option<(Date, Date)>),
}

impl DateSelection {
    pub(super) fn draft(self) -> BaseDate {
        match self {
            Self::Single(value) => BaseDate::Single(value),
            Self::Range(Some((start, end))) => BaseDate::Range(Some(start), Some(end)),
            Self::Range(None) => BaseDate::Range(None, None),
        }
    }
}

pub(super) fn complete(draft: BaseDate) -> Option<DateSelection> {
    match draft {
        BaseDate::Single(Some(date)) => Some(DateSelection::Single(Some(date))),
        BaseDate::Range(Some(start), Some(end)) if start <= end => {
            Some(DateSelection::Range(Some((start, end))))
        }
        _ => None,
    }
}

/// Presentation style of a [`DatePicker`](super::DatePicker).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerVariant {
    /// Anchors an inline text field with a popup calendar beneath it.
    Docked,
    /// Opens a centered modal dialog starting in calendar mode.
    #[default]
    Modal,
    /// Opens a centered modal dialog starting in text-input mode.
    ModalInput,
    /// Uses a full-screen dialog in compact windows.
    FullScreen,
}

/// A draft can retain the first endpoint before the second is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePickerDraft {
    /// Draft for a single-date picker.
    Single(Option<Date>),
    /// Draft for a date-range picker with optional start and end endpoints.
    Range(Option<Date>, Option<Date>),
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

impl From<BaseDate> for DatePickerDraft {
    fn from(date: BaseDate) -> Self {
        match date {
            BaseDate::Single(value) => Self::Single(value),
            BaseDate::Range(start, end) => Self::Range(start, end),
        }
    }
}

impl From<DatePickerDraft> for BaseDate {
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
    /// Interactive month grid and year selector.
    #[default]
    Calendar,
    /// Direct text-field date entry.
    Input,
}

/// Local date picker and editor colors. Unset values use the active Material theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct DatePickerColors {
    /// Background color of the picker surface or dialog container.
    pub container: Option<Hsla>,
    /// Text color of the picker title.
    pub title: Option<Hsla>,
    /// Text color of the selected-date headline.
    pub headline: Option<Hsla>,
    /// Text color of weekday column headers.
    pub weekday: Option<Hsla>,
    /// Text color of month and year subheadings.
    pub subheading: Option<Hsla>,
    /// Icon and label color of month/year navigation controls.
    pub navigation: Option<Hsla>,
    /// Text color of unselected year items.
    pub year: Option<Hsla>,
    /// Text color of the current calendar year item.
    pub current_year: Option<Hsla>,
    /// Text color of disabled year items.
    pub disabled_year: Option<Hsla>,
    /// Text color of the selected year item.
    pub selected_year: Option<Hsla>,
    /// Container fill color behind the selected year item.
    pub selected_year_container: Option<Hsla>,
    /// Text color of a selected year item that is disabled.
    pub disabled_selected_year: Option<Hsla>,
    /// Container fill color behind a disabled selected year item.
    pub disabled_selected_year_container: Option<Hsla>,
    /// Text color of unselected enabled day cells.
    pub day: Option<Hsla>,
    /// Text color of disabled day cells.
    pub disabled_day: Option<Hsla>,
    /// Text color of selected day endpoints.
    pub selected_day: Option<Hsla>,
    /// Container fill color behind selected day endpoints.
    pub selected_day_container: Option<Hsla>,
    /// Text color of a selected day that is disabled.
    pub disabled_selected_day: Option<Hsla>,
    /// Container fill color behind a disabled selected day.
    pub disabled_selected_day_container: Option<Hsla>,
    /// Text color of today's date when not selected.
    pub today: Option<Hsla>,
    /// Outline color drawn around today's date.
    pub today_outline: Option<Hsla>,
    /// Text color of intermediate days inside a selected range.
    pub range_day: Option<Hsla>,
    /// Background fill color connecting intermediate days in a selected range.
    pub range_container: Option<Hsla>,
    /// Focus ring outline color for keyboard navigation.
    pub focus_outline: Option<Hsla>,
    /// Color overrides applied to the date text input fields.
    pub input: Option<TextFieldColors>,
}

/// Formatting and parsing belong to the application locale. Default editing
/// uses an ISO pattern and the calendar uses English headings.
#[derive(Clone)]
pub struct DatePickerLocale {
    pub(super) first_day: Weekday,
    pub(super) day: DateFormatter,
    pub(super) month: DateFormatter,
    pub(super) weekday: WeekdayFormatter,
    pub(super) format: DateFormatter,
    pub(super) parse: DateParser,
}

thread_local! {
    static DEFAULT_DATE_LOCALE: DatePickerLocale = DatePickerLocale {
        first_day: Weekday::Sunday,
        day: Rc::new(|date| {
            format!(
                "{}, {} {}, {}",
                weekday_full_name(date.weekday()),
                month_name(date.month()),
                date.day(),
                date.year(),
            )
            .into()
        }),
        month: Rc::new(|date| format!("{} {}", month_name(date.month()), date.year()).into()),
        weekday: Rc::new(|day| SharedString::new_static(weekday_short_name(day))),
        format: Rc::new(|date| date.to_string().into()),
        parse: Rc::new(|text| text.parse::<Date>().ok()),
    };
}

impl Default for DatePickerLocale {
    fn default() -> Self {
        DEFAULT_DATE_LOCALE.with(Clone::clone)
    }
}

impl DatePickerLocale {
    /// Sets the first day of the week shown in the calendar grid. Defaults to [`Weekday::Sunday`].
    pub fn first_day(mut self, day: Weekday) -> Self {
        self.first_day = day;
        self
    }

    /// Formats the initial editable text and parses a finished edit.
    pub fn input_format(
        mut self,
        format: impl Fn(Date) -> SharedString + 'static,
        parse: impl Fn(&str) -> Option<Date> + 'static,
    ) -> Self {
        self.format = Rc::new(format);
        self.parse = Rc::new(parse);
        self
    }

    /// Sets the spoken date format independently of editable text.
    pub fn accessible_date_format(
        mut self,
        format: impl Fn(Date) -> SharedString + 'static,
    ) -> Self {
        self.day = Rc::new(format);
        self
    }

    /// Formats day accessibility names, month headings, and weekday labels.
    pub fn calendar_format(
        mut self,
        day: impl Fn(Date) -> SharedString + 'static,
        month: impl Fn(Date) -> SharedString + 'static,
        weekday: impl Fn(Weekday) -> SharedString + 'static,
    ) -> Self {
        self.day = Rc::new(day);
        self.month = Rc::new(month);
        self.weekday = Rc::new(weekday);
        self
    }
}

#[derive(Clone)]
pub(super) struct DatePickerOptions {
    pub(super) value: DateSelection,
    pub(super) variant: DatePickerVariant,
    pub(super) locale: DatePickerLocale,
    pub(super) focus_target: Option<FocusHandle>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Presentation {
    Triggered,
    Panel,
    Dialog,
}

#[derive(Clone)]
pub(super) struct DockedActions {
    pub(super) state: Entity<DatePickerState>,
    pub(super) generation: u64,
    pub(super) properties: DialogProperties,
    pub(super) confirm: Option<Confirm<DateSelection>>,
    pub(super) request: Option<DismissRequest>,
    pub(super) closed: Option<OnClose>,
}

impl DockedActions {
    pub(super) fn current(&self, cx: &App) -> bool {
        let state = self.state.read(cx);
        state.docked_open && state.docked_generation == self.generation
    }

    pub(super) fn dismiss(&self, reason: DialogCloseReason, window: &mut Window, cx: &mut App) {
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

    pub(super) fn finish(
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

pub(super) fn selection_label(draft: BaseDate, formatter: &DateFormatter) -> SharedString {
    match (draft.start(), draft.end()) {
        (Some(start), Some(end)) => format!("{} – {}", formatter(start), formatter(end)).into(),
        (Some(start), _) => formatter(start),
        _ if draft.is_single() => "No date selected".into(),
        _ => "No date range selected".into(),
    }
}
