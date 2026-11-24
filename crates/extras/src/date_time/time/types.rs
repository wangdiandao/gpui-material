use std::rc::Rc;

use gpui::{Hsla, SharedString};
use gpui_material_components::TextFieldColors;
use gpui_material_foundation::color;
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, MaterialTheme};
use jiff::civil::Time;

use crate::date_time::ScrollFieldColors;

/// Hour display convention for a time picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    /// 12-hour clock (`1`–`12`) with an AM/PM period selector.
    TwelveHour,
    /// 24-hour clock (`00`–`23`) without a period selector.
    #[default]
    TwentyFourHour,
}

/// Interactive editor mode for a time picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerVariant {
    /// Circular clock dial with clickable and draggable hands.
    #[default]
    Dial,
    /// Direct numeric text fields for hours and minutes.
    Input,
    /// Wrapping vertical scroll wheels for hours and minutes.
    Scroll,
}

/// Orientation of the dial relative to the time header in a time picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerLayout {
    /// Switches to horizontal when the viewport is wide (`>= 600px`) and short (`< 650px`).
    #[default]
    Auto,
    /// Always stacks the time header above the dial.
    Vertical,
    /// Places the time header beside the dial when the viewport is at least `600px` wide.
    Horizontal,
}

pub(super) type NumberFormatter = Rc<dyn Fn(u32) -> SharedString>;
pub(super) type NumberParser = Rc<dyn Fn(&str) -> Option<u32>>;

/// Application locale hooks for numbers, period names, separator, and labels.
#[derive(Clone)]
pub struct TimePickerLocale {
    pub(super) number: NumberFormatter,
    pub(super) parse: NumberParser,
    pub(super) am: SharedString,
    pub(super) pm: SharedString,
    pub(super) separator: SharedString,
    pub(super) hour: SharedString,
    pub(super) minute: SharedString,
    pub(super) validation_message: SharedString,
}

impl Default for TimePickerLocale {
    fn default() -> Self {
        Self {
            number: Rc::new(|value| crate::date_time::format_two_digits(value as usize)),
            parse: Rc::new(|text| text.parse().ok()),
            am: SharedString::new_static("AM"),
            pm: SharedString::new_static("PM"),
            separator: SharedString::new_static(":"),
            hour: SharedString::new_static("Hour"),
            minute: SharedString::new_static("Minute"),
            validation_message: SharedString::new_static("Enter a valid time"),
        }
    }
}

impl TimePickerLocale {
    /// Sets custom formatting and parsing functions for hour and minute numerals.
    pub fn numbers(
        mut self,
        format: impl Fn(u32) -> SharedString + 'static,
        parse: impl Fn(&str) -> Option<u32> + 'static,
    ) -> Self {
        self.number = Rc::new(format);
        self.parse = Rc::new(parse);
        self
    }

    /// Sets localized labels for the hour field, minute field, separator, and AM/PM periods.
    pub fn labels(
        mut self,
        hour: impl Into<SharedString>,
        minute: impl Into<SharedString>,
        separator: impl Into<SharedString>,
        am: impl Into<SharedString>,
        pm: impl Into<SharedString>,
    ) -> Self {
        self.hour = hour.into();
        self.minute = minute.into();
        self.separator = separator.into();
        self.am = am.into();
        self.pm = pm.into();
        self
    }

    /// Sets the error message shown when text input cannot be parsed as a valid time.
    pub fn validation_message(mut self, message: impl Into<SharedString>) -> Self {
        self.validation_message = message.into();
        self
    }
}

/// The time component currently targeted for editing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerSegment {
    /// Editing the hour component.
    #[default]
    Hour,
    /// Editing the minute component.
    Minute,
}

/// Container and content colors for selected and unselected controls.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimePickerSelectionColors {
    /// Background fill color when the control is selected.
    pub selected_container: Option<Hsla>,
    /// Background fill color when the control is unselected.
    pub container: Option<Hsla>,
    /// Foreground text color when the control is selected.
    pub selected_content: Option<Hsla>,
    /// Foreground text color when the control is unselected.
    pub content: Option<Hsla>,
}

/// Instance appearance for the dial, input, and scroll panels.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimePickerStyle {
    /// Corner shape override for hour and minute selector boxes and inputs.
    pub time_field_shape: Option<CornerShape>,
    /// Corner shape override for the AM/PM period selector.
    pub period_selector_shape: Option<CornerShape>,
    /// Background fill color for the panel container.
    pub container: Option<Hsla>,
    /// Background fill color for the circular clock dial face.
    pub clock_dial: Option<Hsla>,
    /// Text color for unselected numbers on the clock dial.
    pub clock_dial_content: Option<Hsla>,
    /// Text color for the currently selected number on the clock dial.
    pub clock_dial_selected_content: Option<Hsla>,
    /// Accent color for the clock dial hand and selection handle.
    pub selector: Option<Hsla>,
    /// Colors for the hour and minute header selector tiles.
    pub time_selector: TimePickerSelectionColors,
    /// Colors for the AM/PM period selector buttons.
    pub period_selector: TimePickerSelectionColors,
    /// Outline border color around the AM/PM period selector.
    pub period_selector_border: Option<Hsla>,
    /// Color overrides applied to the text-input fields.
    pub input: TextFieldColors,
    /// Color overrides applied to the scroll-wheel fields.
    pub scroll: ScrollFieldColors,
}

#[derive(Clone, Copy)]
pub(super) struct Presentation {
    pub(super) style: TimePickerStyle,
    pub(super) vibrant: bool,
}

impl Presentation {
    pub(super) fn selector_shape(self) -> CornerShape {
        self.style.time_field_shape.unwrap_or_else(|| {
            if self.vibrant {
                CornerRole::Large
            } else {
                CornerRole::Small
            }
            .shape()
        })
    }

    pub(super) fn period_shape(self) -> CornerShape {
        self.style.period_selector_shape.unwrap_or_else(|| {
            if self.vibrant {
                CornerRole::Full
            } else {
                CornerRole::Small
            }
            .shape()
        })
    }

    pub(super) fn input_colors(self, theme: &MaterialTheme) -> TextFieldColors {
        let mut input = self.style.input;
        if self.vibrant {
            let tone = |role| color(theme.colors.color(role));
            for state in [&mut input.unfocused, &mut input.focused] {
                state
                    .container
                    .get_or_insert_with(|| tone(ColorRole::SurfaceContainerLowest));
            }
            input
                .focused
                .text
                .get_or_insert_with(|| tone(ColorRole::Primary));
            input
                .invalid
                .container
                .get_or_insert_with(|| tone(ColorRole::ErrorContainer));
            input
                .invalid
                .text
                .get_or_insert_with(|| tone(ColorRole::OnErrorContainer));
        }
        input
    }
}

pub(super) fn display_hour(time: Time, format: TimeFormat) -> u32 {
    let hour = u32::from(time.hour() as u8);
    if format == TimeFormat::TwelveHour {
        match hour % 12 {
            0 => 12,
            h => h,
        }
    } else {
        hour
    }
}

pub(super) fn parse_segment(
    locale: &TimePickerLocale,
    segment: TimePickerSegment,
    text: &str,
    format: TimeFormat,
    pm: bool,
) -> Option<u32> {
    let value = (locale.parse)(text.trim())?;
    match (segment, format) {
        (TimePickerSegment::Hour, TimeFormat::TwelveHour) if (1..=12).contains(&value) => {
            Some(value % 12 + u32::from(pm) * 12)
        }
        (TimePickerSegment::Hour, TimeFormat::TwentyFourHour) if value < 24 => Some(value),
        (TimePickerSegment::Minute, _) if value < 60 => Some(value),
        _ => None,
    }
}

pub(super) fn format_time(
    time: Time,
    format: TimeFormat,
    locale: &TimePickerLocale,
) -> SharedString {
    let hour = (locale.number)(display_hour(time, format));
    let minute = (locale.number)(u32::from(time.minute() as u8));
    let suffix = if format == TimeFormat::TwelveHour {
        format!(
            " {}",
            if time.hour() < 12 {
                &locale.am
            } else {
                &locale.pm
            }
        )
    } else {
        String::new()
    };
    format!("{hour}{}{minute}{suffix}", locale.separator).into()
}
