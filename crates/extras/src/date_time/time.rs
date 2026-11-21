use std::rc::Rc;

use chrono::Timelike;
use gpui::{
    App, AppContext as _, Context, Entity, EntityInputHandler as _, FocusHandle, Focusable as _,
    SharedString, Window,
};
use gpui_material_components::{DialogState, TextFieldEvent, TextFieldState};

use super::{NaiveTime, ScrollFieldState};

mod dialog;
mod panel;

pub use dialog::{TimePicker, TimePickerDialog, TimePickerModeAction, VibrantTimePickerDialog};
pub use panel::{TimePickerPanel, TimePickerSelectionColors, TimePickerStyle, TimeScroll};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    TwelveHour,
    #[default]
    TwentyFourHour,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerVariant {
    #[default]
    Dial,
    Input,
    Scroll,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerLayout {
    #[default]
    Auto,
    Vertical,
    Horizontal,
}

type NumberFormatter = Rc<dyn Fn(u32) -> SharedString>;
type NumberParser = Rc<dyn Fn(&str) -> Option<u32>>;

/// Application locale hooks for numbers, period names, separator, and labels.
#[derive(Clone)]
pub struct TimePickerLocale {
    number: NumberFormatter,
    parse: NumberParser,
    am: SharedString,
    pm: SharedString,
    separator: SharedString,
    hour: SharedString,
    minute: SharedString,
    validation_message: SharedString,
}

impl Default for TimePickerLocale {
    fn default() -> Self {
        Self {
            number: Rc::new(|value| super::format_two_digits(value as usize)),
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
    pub fn numbers(
        mut self,
        format: impl Fn(u32) -> SharedString + 'static,
        parse: impl Fn(&str) -> Option<u32> + 'static,
    ) -> Self {
        self.number = Rc::new(format);
        self.parse = Rc::new(parse);
        self
    }

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

    pub fn validation_message(mut self, message: impl Into<SharedString>) -> Self {
        self.validation_message = message.into();
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerSegment {
    #[default]
    Hour,
    Minute,
}

fn display_hour(time: NaiveTime, format: TimeFormat) -> u32 {
    if format == TimeFormat::TwelveHour {
        time.hour12().1
    } else {
        time.hour()
    }
}

fn parse_segment(
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

fn format_time(time: NaiveTime, format: TimeFormat, locale: &TimePickerLocale) -> SharedString {
    let hour = (locale.number)(display_hour(time, format));
    let minute = (locale.number)(time.minute());
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

/// Owns one editable time, its input fields, selection wheels, and dialog session.
pub struct TimePickerState {
    draft: NaiveTime,
    segment: TimePickerSegment,
    mode: TimePickerVariant,
    picker_mode: TimePickerVariant,
    format: TimeFormat,
    locale: TimePickerLocale,
    input_attempted: bool,
    mode_revision: u64,
    hour: Entity<TextFieldState>,
    minute: Entity<TextFieldState>,
    hours: Entity<ScrollFieldState>,
    minutes: Entity<ScrollFieldState>,
    dialog: Entity<DialogState>,
    trigger: FocusHandle,
    panel: FocusHandle,
    dial: FocusHandle,
}

impl TimePickerState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let hour = cx.new(|cx| TextFieldState::new(window, cx));
        let minute = cx.new(|cx| TextFieldState::new(window, cx));
        let hours = cx.new(|cx| ScrollFieldState::new(24, 0, cx));
        let minutes = cx.new(|cx| ScrollFieldState::new(60, 0, cx));
        let dialog = cx.new(DialogState::new);
        for (segment, editor, wheel) in [
            (TimePickerSegment::Hour, &hour, &hours),
            (TimePickerSegment::Minute, &minute, &minutes),
        ] {
            cx.subscribe_in(editor, window, move |state, editor, event, window, cx| {
                if matches!(event, TextFieldEvent::Change) && !has_marked_text(editor, window, cx) {
                    let text = editor.read(cx).value();
                    state.apply_input(segment, &text, cx);
                }
            })
            .detach();
            cx.observe_in(wheel, window, move |state, wheel, window, cx| {
                let wheel = wheel.read(cx);
                if !wheel.is_scroll_in_progress()
                    && let Some(index) = wheel.selected_option()
                    && index != state.wheel_index(segment)
                {
                    let mut value = index as u32;
                    if segment == TimePickerSegment::Hour && state.format == TimeFormat::TwelveHour
                    {
                        value = (value + 1) % 12 + state.hour() / 12 * 12;
                    }
                    if !state
                        .segment_focus(state.segment, TimePickerVariant::Scroll, cx)
                        .is_focused(window)
                    {
                        state.set_segment(segment, cx);
                    }
                    state.update_draft(segment, value);
                    state.sync_field(segment, window, cx);
                    cx.notify();
                }
            })
            .detach();
            for focus in [
                editor.read(cx).focus_handle(cx),
                wheel.read(cx).focus_handle().clone(),
            ] {
                cx.on_focus(&focus, window, move |state, _, cx| {
                    state.set_segment(segment, cx)
                })
                .detach();
            }
        }
        cx.observe(&dialog, |_, _, cx| cx.notify()).detach();
        let state = Self {
            draft: NaiveTime::default(),
            segment: TimePickerSegment::Hour,
            mode: TimePickerVariant::Dial,
            picker_mode: TimePickerVariant::Dial,
            format: TimeFormat::TwentyFourHour,
            locale: TimePickerLocale::default(),
            input_attempted: false,
            mode_revision: 0,
            hour,
            minute,
            hours,
            minutes,
            dialog,
            trigger: cx.focus_handle(),
            panel: cx.focus_handle(),
            dial: cx.focus_handle(),
        };
        state.sync_fields(window, cx);
        state
    }

    /// The latest valid draft, at minute precision.
    pub fn value(&self) -> NaiveTime {
        self.draft
    }

    pub fn hour(&self) -> u32 {
        self.draft.hour()
    }

    pub fn minute(&self) -> u32 {
        self.draft.minute()
    }

    pub fn is_pm(&self) -> bool {
        self.hour() >= 12
    }

    pub fn segment(&self) -> TimePickerSegment {
        self.segment
    }

    pub fn mode(&self) -> TimePickerVariant {
        self.mode
    }

    pub fn format(&self) -> TimeFormat {
        self.format
    }

    pub fn hour_input(&self, cx: &App) -> SharedString {
        self.hour.read(cx).value()
    }

    pub fn minute_input(&self, cx: &App) -> SharedString {
        self.minute.read(cx).value()
    }

    pub fn set_hour_input(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_input(TimePickerSegment::Hour, text.into(), window, cx);
    }

    pub fn set_minute_input(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_input(TimePickerSegment::Minute, text.into(), window, cx);
    }

    pub fn is_hour_input_valid(&self, cx: &App) -> bool {
        self.valid_input(TimePickerSegment::Hour, cx)
    }

    pub fn is_minute_input_valid(&self, cx: &App) -> bool {
        self.valid_input(TimePickerSegment::Minute, cx)
    }

    pub fn is_input_valid(&self, cx: &App) -> bool {
        self.invalid_segment(cx).is_none()
    }

    pub fn is_open(&self, cx: &App) -> bool {
        self.dialog.read(cx).is_open()
    }

    /// Replaces the editable draft and cancels outstanding wheel motion.
    pub fn set_value(&mut self, value: NaiveTime, window: &mut Window, cx: &mut Context<Self>) {
        self.draft = NaiveTime::from_hms_opt(value.hour(), value.minute(), 0).unwrap();
        self.input_attempted = false;
        self.sync_fields(window, cx);
        self.sync_wheels(cx);
        cx.notify();
    }

    /// Sets a 24-hour value. Returns whether the supplied hour was valid.
    pub fn set_hour(&mut self, hour: u32, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.set_part(TimePickerSegment::Hour, hour, window, cx)
    }

    /// Sets a minute in 0–59. Returns whether the supplied minute was valid.
    pub fn set_minute(&mut self, minute: u32, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.set_part(TimePickerSegment::Minute, minute, window, cx)
    }

    pub fn set_segment(&mut self, segment: TimePickerSegment, cx: &mut Context<Self>) {
        if segment != self.segment {
            self.segment = segment;
            cx.notify();
        }
    }

    /// Retains the active segment across mode changes. Reconciles editors with the last valid time.
    pub fn set_mode(
        &mut self,
        mode: TimePickerVariant,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if mode != self.mode && !self.has_composition(window, cx) {
            if self.mode == TimePickerVariant::Input {
                self.reconcile_input(cx);
            }
            let restore_focus = self.panel.contains_focused(window, cx);
            self.mode = mode;
            self.mode_revision = self.mode_revision.wrapping_add(1);
            if mode != TimePickerVariant::Input {
                self.picker_mode = mode;
            }
            self.input_attempted = false;
            self.sync_fields(window, cx);
            self.sync_wheels(cx);
            if restore_focus {
                self.focus_editor(window, cx);
            }
            cx.notify();
        }
    }

    pub fn set_format(&mut self, format: TimeFormat, window: &mut Window, cx: &mut Context<Self>) {
        if format != self.format {
            self.format = format;
            self.input_attempted = false;
            self.sync_fields(window, cx);
            self.sync_wheels(cx);
            cx.notify();
        }
    }

    pub fn set_locale(
        &mut self,
        locale: TimePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.locale = locale;
        self.input_attempted = false;
        self.sync_fields(window, cx);
        cx.notify();
    }

    /// Opens a new editing session initialized from the application's value.
    pub fn open(&mut self, value: NaiveTime, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_open(cx) {
            self.mode_revision = self.mode_revision.wrapping_add(1);
            self.set_value(value, window, cx);
            self.dialog.update(cx, |dialog, cx| dialog.open(window, cx));
            self.focus_editor(window, cx);
        }
    }

    /// Closes this session programmatically through the Material dialog.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open(cx) {
            self.discard_composition(window, cx);
            self.sync_wheels(cx);
            self.dialog
                .update(cx, |dialog, cx| dialog.close(window, cx));
        }
    }

    fn has_composition(&self, window: &mut Window, cx: &mut App) -> bool {
        [&self.hour, &self.minute]
            .into_iter()
            .any(|field| has_marked_text(field, window, cx))
    }

    fn finish_composition(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let mut composed = false;
        for editor in [&self.hour, &self.minute] {
            composed |= editor.update(cx, |editor, cx| {
                if editor.marked_text_range(window, cx).is_none() {
                    return false;
                }
                editor.unmark_text(window, cx);
                true
            });
        }
        if composed {
            self.reconcile_input(cx);
        }
        composed
    }

    fn discard_composition(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.has_composition(window, cx) {
            self.sync_fields(window, cx);
        }
    }

    fn reconcile_input(&mut self, cx: &mut Context<Self>) {
        for segment in [TimePickerSegment::Hour, TimePickerSegment::Minute] {
            let text = self.field(segment).read(cx).value();
            if let Some(value) = self.parse_segment(segment, &text) {
                self.update_draft(segment, value);
                self.sync_wheel(segment, cx);
            }
        }
        cx.notify();
    }

    fn parse_segment(&self, segment: TimePickerSegment, text: &str) -> Option<u32> {
        parse_segment(&self.locale, segment, text, self.format, self.is_pm())
    }

    fn valid_input(&self, segment: TimePickerSegment, cx: &App) -> bool {
        self.parse_segment(segment, &self.field(segment).read(cx).value())
            .is_some()
    }

    fn invalid_segment(&self, cx: &App) -> Option<TimePickerSegment> {
        [TimePickerSegment::Hour, TimePickerSegment::Minute]
            .into_iter()
            .find(|&segment| !self.valid_input(segment, cx))
    }

    fn apply_input(&mut self, segment: TimePickerSegment, text: &str, cx: &mut Context<Self>) {
        if let Some(value) = self.parse_segment(segment, text) {
            self.update_draft(segment, value);
            self.sync_wheel(segment, cx);
        }
        cx.notify();
    }

    fn set_input(
        &mut self,
        segment: TimePickerSegment,
        text: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.field(segment)
            .update(cx, |field, cx| field.set_value(text.clone(), window, cx));
        self.apply_input(segment, &text, cx);
    }

    fn field(&self, segment: TimePickerSegment) -> &Entity<TextFieldState> {
        match segment {
            TimePickerSegment::Hour => &self.hour,
            TimePickerSegment::Minute => &self.minute,
        }
    }

    fn wheel(&self, segment: TimePickerSegment) -> &Entity<ScrollFieldState> {
        match segment {
            TimePickerSegment::Hour => &self.hours,
            TimePickerSegment::Minute => &self.minutes,
        }
    }

    fn wheel_index(&self, segment: TimePickerSegment) -> usize {
        match segment {
            TimePickerSegment::Hour => {
                let hour = display_hour(self.draft, self.format) as usize;
                hour - usize::from(self.format == TimeFormat::TwelveHour)
            }
            TimePickerSegment::Minute => self.minute() as usize,
        }
    }

    fn sync_field(&self, segment: TimePickerSegment, window: &mut Window, cx: &mut Context<Self>) {
        let number = match segment {
            TimePickerSegment::Hour => display_hour(self.draft, self.format),
            TimePickerSegment::Minute => self.minute(),
        };
        let text = (self.locale.number)(number);
        let field = self.field(segment);
        if field.read(cx).value() != text || has_marked_text(field, window, cx) {
            field.update(cx, |field, cx| field.set_value(text, window, cx));
        }
    }

    fn sync_fields(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.sync_field(TimePickerSegment::Hour, window, cx);
        self.sync_field(TimePickerSegment::Minute, window, cx);
    }

    fn sync_wheel(&self, segment: TimePickerSegment, cx: &mut Context<Self>) {
        let count = match (segment, self.format) {
            (TimePickerSegment::Hour, TimeFormat::TwelveHour) => 12,
            (TimePickerSegment::Hour, TimeFormat::TwentyFourHour) => 24,
            (TimePickerSegment::Minute, _) => 60,
        };
        let index = self.wheel_index(segment);
        self.wheel(segment).update(cx, |wheel, cx| {
            wheel.set_item_count(count, cx);
            if wheel.target_option() != Some(index) || wheel.is_scroll_in_progress() {
                wheel.scroll_to_option(index, cx);
            }
        });
    }

    fn sync_wheels(&self, cx: &mut Context<Self>) {
        self.sync_wheel(TimePickerSegment::Hour, cx);
        self.sync_wheel(TimePickerSegment::Minute, cx);
    }

    fn update_draft(&mut self, segment: TimePickerSegment, value: u32) -> bool {
        let time = match segment {
            TimePickerSegment::Hour => self.draft.with_hour(value),
            TimePickerSegment::Minute => self.draft.with_minute(value),
        };
        if let Some(time) = time {
            self.draft = time;
        }
        time.is_some()
    }

    fn set_part(
        &mut self,
        segment: TimePickerSegment,
        value: u32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.update_draft(segment, value) {
            return false;
        }
        self.sync_field(segment, window, cx);
        self.sync_wheel(segment, cx);
        cx.notify();
        true
    }

    fn segment_focus(
        &self,
        segment: TimePickerSegment,
        mode: TimePickerVariant,
        cx: &App,
    ) -> FocusHandle {
        match mode {
            TimePickerVariant::Dial => self.dial.clone(),
            TimePickerVariant::Input => self.field(segment).read(cx).focus_handle(cx),
            TimePickerVariant::Scroll => self.wheel(segment).read(cx).focus_handle().clone(),
        }
    }

    fn editor_focus(&self, mode: TimePickerVariant, cx: &App) -> FocusHandle {
        self.segment_focus(self.segment, mode, cx)
    }

    fn focus_editor(&self, window: &mut Window, cx: &mut App) {
        self.editor_focus(self.mode, cx).focus(window, cx);
    }

    fn select(&mut self, value: u32, advance: bool, window: &mut Window, cx: &mut Context<Self>) {
        let segment = self.segment;
        let value = if segment == TimePickerSegment::Hour && self.format == TimeFormat::TwelveHour {
            value % 12 + self.hour() / 12 * 12
        } else {
            value
        };
        if self.set_part(segment, value, window, cx)
            && advance
            && segment == TimePickerSegment::Hour
        {
            self.set_segment(TimePickerSegment::Minute, cx);
        }
    }
}

fn has_marked_text(field: &Entity<TextFieldState>, window: &mut Window, cx: &mut App) -> bool {
    field.update(cx, |editor, cx| {
        editor.marked_text_range(window, cx).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_time_parsing_handles_midnight_noon_and_boundaries() {
        let locale = TimePickerLocale::default();
        let parse_hour = |text, pm| {
            parse_segment(
                &locale,
                TimePickerSegment::Hour,
                text,
                TimeFormat::TwelveHour,
                pm,
            )
        };
        assert_eq!(parse_hour("12", false), Some(0));
        assert_eq!(parse_hour("12", true), Some(12));
        assert_eq!(parse_hour("11", true), Some(23));
        assert_eq!(parse_hour("00", false), None);
        assert_eq!(
            parse_segment(
                &locale,
                TimePickerSegment::Minute,
                "60",
                TimeFormat::TwelveHour,
                false
            ),
            None
        );
        assert_eq!(
            parse_segment(
                &locale,
                TimePickerSegment::Hour,
                "24",
                TimeFormat::TwentyFourHour,
                false
            ),
            None
        );
    }
    #[cfg(feature = "test-support")]
    #[gpui::test]
    fn time_dialog_waits_for_ime_then_confirms_the_unmarked_value(cx: &mut gpui::TestAppContext) {
        use std::cell::{Cell, RefCell};

        use gpui::{IntoElement, Render, WindowOptions, div};
        use gpui_material_components::DialogAction;
        use gpui_material_foundation::MaterialRoot;
        use gpui_material_theme::MaterialTheme;

        struct Fixture {
            state: Entity<TimePickerState>,
            change_mode: Rc<RefCell<Option<TimePickerModeAction>>>,
            confirm: Rc<RefCell<Option<DialogAction>>>,
            accepted: Rc<Cell<Option<NaiveTime>>>,
        }

        impl Render for Fixture {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let change_mode = self.change_mode.clone();
                let confirm = self.confirm.clone();
                let accepted = self.accepted.clone();
                TimePickerDialog::new("ime-clock", "Input composition", &self.state)
                    .mode_toggle_content(move |action| {
                        change_mode.replace(Some(action));
                        div()
                    })
                    .confirm_content(move |action| {
                        confirm.replace(Some(action));
                        div()
                    })
                    .on_confirm(move |time, _, _| accepted.set(Some(time)))
            }
        }

        let mode = Rc::new(RefCell::new(None));
        let confirm = Rc::new(RefCell::new(None));
        let accepted = Rc::new(Cell::new(None));
        let mut state = None;
        let window = cx
            .update(|cx| {
                MaterialRoot::init(MaterialTheme::default(), cx);
                gpui_material_components::init_components(cx);
                cx.set_reduce_motion(true);
                MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                    let picker = cx.new(|cx| {
                        let mut picker = TimePickerState::new(window, cx);
                        picker.set_mode(TimePickerVariant::Input, window, cx);
                        picker
                    });
                    state = Some(picker.clone());
                    cx.new(|cx| {
                        cx.observe(&picker, |_, _, cx| cx.notify()).detach();
                        Fixture {
                            state: picker,
                            change_mode: mode.clone(),
                            confirm: confirm.clone(),
                            accepted: accepted.clone(),
                        }
                    })
                })
                .unwrap()
            })
            .into();
        let state = state.unwrap();
        let initial = NaiveTime::from_hms_opt(15, 25, 0).unwrap();
        cx.update_window(window, |_, window, cx| {
            state.update(cx, |state, cx| state.open(initial, window, cx));
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
        let change_mode = mode.borrow().clone().unwrap();
        let apply = confirm.borrow().clone().unwrap();

        cx.update_window(window, |_, window, cx| {
            let editor = state.read(cx).hour.clone();
            editor.update(cx, |editor, cx| {
                editor.replace_and_mark_text_in_range(Some(0..2), "19", Some(1..1), window, cx);
            });
            change_mode.activate(window, cx);
            apply.activate(window, cx);
            let model = state.read(cx);
            assert_eq!(model.mode(), TimePickerVariant::Input);
            assert_eq!(model.value(), initial);
            assert_eq!(model.hour_input(cx).as_ref(), "19");
            assert!(model.is_open(cx));
            assert!(accepted.get().is_none());
        })
        .unwrap();

        cx.simulate_keystrokes(window, "escape");
        cx.update_window(window, |_, window, cx| {
            let model = state.read(cx);
            let editor = model.hour.clone();
            assert!(model.is_open(cx));
            assert!(accepted.get().is_none());
            editor.update(cx, |editor, cx| {
                assert!(editor.marked_text_range(window, cx).is_none());
                editor.replace_and_mark_text_in_range(Some(0..2), "21", Some(1..1), window, cx);
                editor.unmark_text(window, cx);
            });
            apply.activate(window, cx);
            assert_eq!(accepted.get(), NaiveTime::from_hms_opt(21, 25, 0));
            assert!(!state.read(cx).is_open(cx));

            state.update(cx, |state, cx| state.open(initial, window, cx));
            let editor = state.read(cx).hour.clone();
            editor.update(cx, |editor, cx| {
                editor.replace_and_mark_text_in_range(Some(0..2), "22", Some(1..1), window, cx);
            });
            state.update(cx, |state, cx| state.close(window, cx));
            assert!(!state.read(cx).is_open(cx));
            assert_eq!(state.read(cx).value(), initial);
            assert_eq!(state.read(cx).hour_input(cx).as_ref(), "15");
        })
        .unwrap();
    }
}
