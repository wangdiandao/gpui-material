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
pub use date::{DatePicker, DatePickerLocale, DatePickerState, DatePickerVariant, DateSelection};
pub use scroll_field::{ScrollField, ScrollFieldColors, ScrollFieldState};
pub use time::{
    TimeFormat, TimePicker, TimePickerLayout, TimePickerLocale, TimePickerState, TimePickerVariant,
};

type Confirm<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

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
