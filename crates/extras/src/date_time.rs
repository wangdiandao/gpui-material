//! Optional Material date and time editors. Confirm callbacks request committed
//! values from the application. Picker states retain the draft and focus.

mod date;
mod scroll_field;
mod time;

use gpui::{
    App, InteractiveElement, ParentElement, Role, SharedString, StatefulInteractiveElement, Window,
    div,
};
use gpui_base::TestSupportExt as _;
use gpui_material_components::Text;
use gpui_material_theme::{ColorRole, TypeRole};
use std::rc::Rc;

pub use chrono::{NaiveDate, NaiveTime, Weekday};
pub use date::{
    DatePicker, DatePickerColors, DatePickerDisplayMode, DatePickerDraft, DatePickerLocale,
    DatePickerState, DatePickerVariant, DateSelection,
};
pub use scroll_field::{ScrollField, ScrollFieldColors, ScrollFieldState};
pub use time::{
    TimeFormat, TimePicker, TimePickerDialog, TimePickerLayout, TimePickerLocale,
    TimePickerModeAction, TimePickerPanel, TimePickerSegment, TimePickerSelectionColors,
    TimePickerState, TimePickerStyle, TimePickerVariant, TimeScroll, VibrantTimePickerDialog,
};

type Confirm<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

const TWO_DIGIT: [&str; 60] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
];

fn format_two_digits(value: usize) -> SharedString {
    TWO_DIGIT.get(value).map_or_else(
        || format!("{value:02}").into(),
        |&text| SharedString::new_static(text),
    )
}

fn status(message: SharedString, error: bool) -> impl gpui::IntoElement {
    div()
        .id("picker-status")
        .test_support()
        .role(if error { Role::Alert } else { Role::Status })
        .aria_label(message.clone())
        .child(
            Text::new(message)
                .role(TypeRole::BodyMedium)
                .color(if error {
                    ColorRole::Error
                } else {
                    ColorRole::OnSurfaceVariant
                }),
        )
}
