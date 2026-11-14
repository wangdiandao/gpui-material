use std::{cell::Cell, f32::consts::TAU, rc::Rc};

use chrono::Timelike as _;
use gpui::{
    AccessibleAction, AnyElement, App, AppContext as _, Bounds, Context, ElementId, Entity,
    FocusHandle, Focusable as _, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, PathBuilder, Pixels, Point, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt as _};
use gpui_material_components::{
    Button, ButtonVariant, Dialog, DialogState, Text, TextField, TextFieldState, TextFieldVariant,
};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, TypeRole};

use super::{Confirm, NaiveTime, status};

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
}

impl Default for TimePickerLocale {
    fn default() -> Self {
        Self {
            number: Rc::new(|value| format!("{value:02}").into()),
            parse: Rc::new(|text| text.parse().ok()),
            am: "AM".into(),
            pm: "PM".into(),
            separator: ":".into(),
            hour: "Hour".into(),
            minute: "Minute".into(),
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Segment {
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

fn parse_time(
    hour: &str,
    minute: &str,
    format: TimeFormat,
    pm: bool,
    locale: &TimePickerLocale,
) -> Option<NaiveTime> {
    let mut hour = (locale.parse)(hour.trim())?;
    let minute = (locale.parse)(minute.trim())?;
    if format == TimeFormat::TwelveHour {
        if !(1..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + if pm { 12 } else { 0 };
    }
    NaiveTime::from_hms_opt(hour, minute, 0)
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

/// Owns the editable time, active dial segment and modal focus. Opening the
/// picker starts a fresh draft from the supplied application value.
pub struct TimePickerState {
    draft: NaiveTime,
    segment: Segment,
    variant: TimePickerVariant,
    input_attempted: bool,
    hour: Entity<TextFieldState>,
    minute: Entity<TextFieldState>,
    dialog: Entity<DialogState>,
    trigger: FocusHandle,
    dial: FocusHandle,
}

impl TimePickerState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let hour = cx.new(|cx| TextFieldState::new(window, cx));
        let minute = cx.new(|cx| TextFieldState::new(window, cx));
        let dialog = cx.new(DialogState::new);
        cx.observe(&hour, |_, _, cx| cx.notify()).detach();
        cx.observe(&minute, |_, _, cx| cx.notify()).detach();
        cx.observe(&dialog, |_, _, cx| cx.notify()).detach();
        Self {
            draft: NaiveTime::default(),
            segment: Segment::Hour,
            variant: TimePickerVariant::Dial,
            input_attempted: false,
            hour,
            minute,
            dialog,
            trigger: cx.focus_handle(),
            dial: cx.focus_handle(),
        }
    }

    fn sync_fields(
        &self,
        format: TimeFormat,
        locale: &TimePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.hour.update(cx, |field, cx| {
            field.set_value(
                (locale.number)(display_hour(self.draft, format)),
                window,
                cx,
            )
        });
        self.minute.update(cx, |field, cx| {
            field.set_value((locale.number)(self.draft.minute()), window, cx)
        });
    }

    fn input_value(
        &self,
        format: TimeFormat,
        locale: &TimePickerLocale,
        cx: &App,
    ) -> Option<NaiveTime> {
        parse_time(
            &self.hour.read(cx).value(),
            &self.minute.read(cx).value(),
            format,
            self.draft.hour() >= 12,
            locale,
        )
    }

    fn set_mode(
        &mut self,
        mode: TimePickerVariant,
        format: TimeFormat,
        locale: &TimePickerLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.variant == TimePickerVariant::Input
            && let Some(value) = self.input_value(format, locale, cx)
        {
            self.draft = value;
        }
        self.variant = mode;
        self.input_attempted = false;
        if mode == TimePickerVariant::Input {
            self.sync_fields(format, locale, window, cx);
        }
        self.focus_editor(window, cx);
        cx.notify();
    }

    fn focus_editor(&self, window: &mut Window, cx: &mut App) {
        let focus = if self.variant == TimePickerVariant::Input {
            self.hour.read(cx).focus_handle(cx)
        } else {
            self.dial.clone()
        };
        focus.focus(window, cx);
    }

    fn select(&mut self, value: u32, format: TimeFormat, advance: bool, cx: &mut Context<Self>) {
        self.draft = match self.segment {
            Segment::Hour => {
                let value = if format == TimeFormat::TwelveHour {
                    value % 12 + (self.draft.hour() / 12) * 12
                } else {
                    value
                };
                if advance {
                    self.segment = Segment::Minute;
                }
                self.draft.with_hour(value)
            }
            Segment::Minute => self.draft.with_minute(value),
        }
        .unwrap_or(self.draft);
        cx.notify();
    }
}

#[derive(Clone)]
struct TimeOptions {
    value: NaiveTime,
    format: TimeFormat,
    variant: TimePickerVariant,
    layout: TimePickerLayout,
    locale: TimePickerLocale,
}

/// Material time dialog with a clock face and direct keyboard entry.
///
/// The callback receives minute precision and runs only after confirmation.
#[derive(IntoElement)]
pub struct TimePicker {
    id: ElementId,
    label: SharedString,
    state: Entity<TimePickerState>,
    options: TimeOptions,
    confirm: Option<Confirm<NaiveTime>>,
}

impl TimePicker {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TimePickerState>,
        value: NaiveTime,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: TimeOptions {
                value,
                format: TimeFormat::default(),
                variant: TimePickerVariant::default(),
                layout: TimePickerLayout::default(),
                locale: TimePickerLocale::default(),
            },
            confirm: None,
        }
    }

    pub fn format(mut self, format: TimeFormat) -> Self {
        self.options.format = format;
        self
    }

    pub fn variant(mut self, variant: TimePickerVariant) -> Self {
        self.options.variant = variant;
        self
    }

    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.options.layout = layout;
        self
    }

    pub fn locale(mut self, locale: TimePickerLocale) -> Self {
        self.options.locale = locale;
        self
    }

    pub fn on_confirm(
        mut self,
        callback: impl Fn(NaiveTime, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let open = model.dialog.read(cx).is_open();
        let state = self.state.clone();
        let opts = self.options.clone();
        let trigger = div()
            .id((self.id.clone(), "trigger-group"))
            .role(Role::Group)
            .aria_expanded(open)
            .child(
                Button::new((self.id.clone(), "open"), self.label.clone())
                    .variant(ButtonVariant::Outlined)
                    .track_focus(&model.trigger)
                    .on_activate(move |window, cx| {
                        state.update(cx, |state, cx| {
                            state.draft =
                                NaiveTime::from_hms_opt(opts.value.hour(), opts.value.minute(), 0)
                                    .unwrap();
                            state.segment = Segment::Hour;
                            state.variant = opts.variant;
                            state.input_attempted = false;
                            state.sync_fields(opts.format, &opts.locale, window, cx);
                            state.trigger.focus(window, cx);
                            state
                                .dialog
                                .update(cx, |dialog, cx| dialog.open(window, cx));
                            state.focus_editor(window, cx);
                            cx.notify();
                        })
                    }),
            );
        let (format, locale) = (self.options.format, &self.options.locale);
        let input = model.variant == TimePickerVariant::Input;
        let controls = if input {
            let fields = div()
                .flex()
                .items_start()
                .gap(px(12.))
                .w_full()
                .child(
                    div().flex_1().min_w_0().child(
                        TextField::new((self.id.clone(), "hour"), locale.hour.clone(), &model.hour)
                            .variant(TextFieldVariant::Outlined),
                    ),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        TextField::new(
                            (self.id.clone(), "minute"),
                            locale.minute.clone(),
                            &model.minute,
                        )
                        .variant(TextFieldVariant::Outlined),
                    ),
                );
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(fields)
                .when(
                    model.input_attempted && model.input_value(format, locale, cx).is_none(),
                    |column| column.child(status("Enter a valid time".into(), true)),
                )
                .into_any_element()
        } else {
            time_selector(&self.id, &self.state, format, locale, window, cx)
        };
        let period = (format == TimeFormat::TwelveHour)
            .then(|| period_selector(&self.id, &self.state, locale, window, cx));
        let horizontal = !input
            && window.viewport_size().width >= px(600.)
            && match self.options.layout {
                TimePickerLayout::Vertical => false,
                TimePickerLayout::Horizontal => true,
                TimePickerLayout::Auto => window.viewport_size().height < px(650.),
            };
        let body = div()
            .flex()
            .gap(px(12.))
            .items_center()
            .when(!horizontal, gpui::Styled::flex_col)
            .child(
                div()
                    .min_w_0()
                    .w(px(216.))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(controls)
                    .children(period),
            )
            .children((!input).then(|| dial(&self.id, &self.state, format, locale, window, cx)));
        let switch = self.state.clone();
        let locale = locale.clone();
        let switch_locale = locale.clone();
        let toggle = Button::new(
            (self.id.clone(), "display-mode"),
            if input { "Show clock" } else { "Enter time" },
        )
        .variant(ButtonVariant::Text)
        .on_activate(move |window, cx| {
            switch.update(cx, |state, cx| {
                state.set_mode(
                    if input {
                        TimePickerVariant::Dial
                    } else {
                        TimePickerVariant::Input
                    },
                    format,
                    &switch_locale,
                    window,
                    cx,
                );
            })
        });
        let dialog = model.dialog.clone();
        let state = self.state;
        let confirm = self.confirm;
        div().child(trigger).child(
            Dialog::new(self.id, &dialog, self.label)
                .body(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.))
                        .child(body)
                        .child(toggle),
                )
                .confirm("OK", move |window, cx| {
                    let model = state.read(cx);
                    let value = if model.variant == TimePickerVariant::Input {
                        model.input_value(format, &locale, cx)
                    } else {
                        Some(model.draft)
                    };
                    let Some(value) = value else {
                        state.update(cx, |state, cx| {
                            state.input_attempted = true;
                            cx.notify();
                        });
                        return false;
                    };
                    if let Some(confirm) = &confirm {
                        confirm(value, window, cx);
                    }
                    true
                }),
        )
    }
}

fn time_selector(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    format: TimeFormat,
    locale: &TimePickerLocale,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let theme = window_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let segment = |part, label: &SharedString, value| {
        let selected = part == model.segment;
        let state = state.clone();
        let text = (locale.number)(value);
        let action = Rc::new(move |window: &mut Window, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.segment = part;
                state.dial.focus(window, cx);
                cx.notify();
            });
        });
        let click = action.clone();
        BaseButton::new((
            id.clone(),
            if part == Segment::Hour {
                "select-hour"
            } else {
                "select-minute"
            },
        ))
        .role(Role::RadioButton)
        .aria_toggled(if selected {
            gpui::Toggled::True
        } else {
            gpui::Toggled::False
        })
        .accessibility_label(format!("{label} {text}"))
        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            action(window, cx)
        })
        .on_click(move |_, window, cx| click(window, cx))
        .w(px(96.))
        .h(px(80.))
        .max_w_full()
        .min_w_0()
        .flex_shrink(1.)
        .rounded(px(8.))
        .bg(tone(if selected {
            ColorRole::PrimaryContainer
        } else {
            ColorRole::SurfaceContainerHighest
        }))
        .child(
            Text::new(text)
                .role(TypeRole::DisplayMedium)
                .color(if selected {
                    ColorRole::OnPrimaryContainer
                } else {
                    ColorRole::OnSurface
                }),
        )
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(segment(
            Segment::Hour,
            &locale.hour,
            display_hour(model.draft, format),
        ))
        .child(Text::new(locale.separator.clone()).role(TypeRole::DisplayMedium))
        .child(segment(
            Segment::Minute,
            &locale.minute,
            model.draft.minute(),
        ))
        .into_any_element()
}

fn period_selector(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    locale: &TimePickerLocale,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let theme = window_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let button = |pm, label: SharedString| {
        let selected = (model.draft.hour() >= 12) == pm;
        let state = state.clone();
        let activate = Rc::new(move |_: &mut Window, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.draft = state
                    .draft
                    .with_hour(state.draft.hour() % 12 + if pm { 12 } else { 0 })
                    .unwrap();
                cx.notify();
            });
        });
        let click = activate.clone();
        BaseButton::new((id.clone(), if pm { "pm" } else { "am" }))
            .role(Role::RadioButton)
            .aria_toggled(if selected {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            })
            .accessibility_label(label.clone())
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                activate(window, cx)
            })
            .on_click(move |_, window, cx| click(window, cx))
            .flex_1()
            .h(px(38.))
            .rounded(px(8.))
            .when(selected, |button| {
                button.bg(tone(ColorRole::TertiaryContainer))
            })
            .child(
                Text::new(label)
                    .role(TypeRole::TitleMedium)
                    .color(if selected {
                        ColorRole::OnTertiaryContainer
                    } else {
                        ColorRole::OnSurfaceVariant
                    }),
            )
    };
    div()
        .flex()
        .w_full()
        .rounded(px(8.))
        .border_1()
        .border_color(tone(ColorRole::Outline))
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(button(false, locale.am.clone()))
        .child(button(true, locale.pm.clone()))
        .into_any_element()
}

fn polar(step: u32, divisions: u32, radius: Pixels) -> Point<Pixels> {
    let angle = step as f32 * TAU / divisions as f32;
    point(radius * angle.sin(), -radius * angle.cos())
}

fn dial_position(
    value: u32,
    segment: Segment,
    format: TimeFormat,
    diameter: Pixels,
) -> Point<Pixels> {
    if segment == Segment::Minute {
        return polar(value, 60, diameter * 0.365);
    }
    let inner = format == TimeFormat::TwentyFourHour && (value == 0 || value > 12);
    polar(value % 12, 12, diameter * if inner { 0.205 } else { 0.365 })
}

fn dial_value(
    offset: Point<Pixels>,
    segment: Segment,
    format: TimeFormat,
    diameter: Pixels,
) -> u32 {
    let (x, y) = (f32::from(offset.x), f32::from(offset.y));
    let turn = x.atan2(-y).rem_euclid(TAU) / TAU;
    if segment == Segment::Minute {
        return ((turn * 60.).round() as u32) % 60;
    }
    let hour = ((turn * 12.).round() as u32) % 12;
    let inner = format == TimeFormat::TwentyFourHour && x.hypot(y) < f32::from(diameter * 0.285);
    match (inner, hour) {
        (true, 0) => 0,
        (true, _) => hour + 12,
        (_, 0) => 12,
        _ => hour,
    }
}

fn dial(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    format: TimeFormat,
    locale: &TimePickerLocale,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let segment = model.segment;
    let focus = model.dial.clone();
    let selected = if segment == Segment::Hour {
        display_hour(model.draft, format)
    } else {
        model.draft.minute()
    };
    let label = if segment == Segment::Hour {
        &locale.hour
    } else {
        &locale.minute
    };
    let theme = window_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let diameter = (window.viewport_size().width - px(110.)).clamp(px(120.), px(256.));
    let center = diameter / 2.;
    let handle = diameter * (48. / 256.);
    let selected_position = dial_position(selected, segment, format, diameter);
    let bounds = Rc::new(Cell::new(Bounds::default()));
    let measured = bounds.clone();
    let accent = tone(ColorRole::Primary);
    let track = canvas(
        move |bounds, _, _| {
            measured.set(bounds);
        },
        move |bounds, _, window, _| {
            let start = bounds.center();
            let mut line = PathBuilder::stroke(px(2.));
            line.move_to(start);
            line.line_to(start + selected_position);
            if let Ok(path) = line.build() {
                window.paint_path(path, accent);
            }
        },
    )
    .absolute()
    .size_full();
    let key_state = state.clone();
    let click_state = state.clone();
    let click_focus = focus.clone();
    let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
    let marks: Vec<u32> = match (segment, format) {
        (Segment::Minute, _) => (0..60).step_by(5).collect(),
        (_, TimeFormat::TwelveHour) => (1..=12).collect(),
        (_, TimeFormat::TwentyFourHour) => (0..24).collect(),
    };
    div()
        .id((id.clone(), "dial"))
        .test_support()
        .role(Role::Group)
        .aria_label(format!(
            "{label}, {}",
            format_time(model.draft, format, locale)
        ))
        .track_focus(&focus)
        .tab_stop(true)
        .size(diameter)
        .flex_none()
        .relative()
        .rounded_full()
        .bg(tone(ColorRole::SurfaceContainerHighest))
        .when(keyboard_focus, |dial| {
            dial.border_2().border_color(tone(ColorRole::Secondary))
        })
        .child(track)
        .child(
            div()
                .absolute()
                .size(px(8.))
                .left(center - px(4.))
                .top(center - px(4.))
                .rounded_full()
                .bg(accent),
        )
        .child(
            div()
                .absolute()
                .size(handle)
                .left(center + selected_position.x - handle / 2.)
                .top(center + selected_position.y - handle / 2.)
                .rounded_full()
                .bg(accent),
        )
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let offset = event.position - bounds.get().center();
            let distance = f32::from(offset.x).hypot(f32::from(offset.y));
            if distance <= f32::from(diameter / 2.) {
                click_focus.focus(window, cx);
                let value = dial_value(offset, segment, format, diameter);
                click_state.update(cx, |state, cx| state.select(value, format, true, cx));
            }
        })
        .capture_key_down(move |event, window, cx| {
            if !focus.is_focused(window) || event.keystroke.modifiers.modified() {
                return;
            }
            let divisions = match (segment, format) {
                (Segment::Minute, _) => 60,
                (_, TimeFormat::TwelveHour) => 12,
                (_, TimeFormat::TwentyFourHour) => 24,
            };
            let value = match event.keystroke.key.as_str() {
                "left" | "down" => Some((selected + divisions - 1) % divisions),
                "up" | "right" => Some((selected + 1) % divisions),
                "home" => Some(0),
                "end" => Some(divisions - 1),
                "space" if segment == Segment::Hour => Some(selected),
                _ => None,
            };
            if let Some(value) = value {
                key_state.update(cx, |state, cx| {
                    state.select(value, format, event.keystroke.key == "space", cx)
                });
                cx.stop_propagation();
            }
        })
        .children(marks.into_iter().map(|value| {
            let position = dial_position(value, segment, format, diameter);
            let active = value == selected;
            let label = (locale.number)(value);
            let state = state.clone();
            let activate = Rc::new(move |window: &mut Window, cx: &mut App| {
                state.update(cx, |state, cx| {
                    state.dial.focus(window, cx);
                    state.select(value, format, true, cx);
                });
            });
            let click = activate.clone();
            BaseButton::new((id.clone(), format!("mark-{value}")))
                .role(Role::ListBoxOption)
                .aria_selected(active)
                .accessibility_label(label.clone())
                .tab_stop(false)
                .absolute()
                .size(handle)
                .left(center + position.x - handle / 2.)
                .top(center + position.y - handle / 2.)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| cx.stop_propagation())
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    activate(window, cx)
                })
                .on_click(move |_, window, cx| click(window, cx))
                .child(Text::new(label).role(TypeRole::BodyLarge).color(if active {
                    ColorRole::OnPrimary
                } else {
                    ColorRole::OnSurface
                }))
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_time_parsing_handles_midnight_noon_and_boundaries() {
        let locale = TimePickerLocale::default();
        let parse =
            |hour, minute, pm| parse_time(hour, minute, TimeFormat::TwelveHour, pm, &locale);
        assert_eq!(parse("12", "59", false).unwrap().hour(), 0);
        assert_eq!(parse("12", "59", true).unwrap().hour(), 12);
        assert_eq!(parse("11", "59", true).unwrap().hour(), 23);
        assert_eq!(parse("00", "00", false), None);
        assert_eq!(parse("11", "60", false), None);
        assert_eq!(
            parse_time("24", "00", TimeFormat::TwentyFourHour, false, &locale),
            None
        );
    }

    #[test]
    fn dial_position_round_trips_both_hour_rings_and_every_minute() {
        let size = px(256.);
        for (segment, format, values) in [
            (Segment::Hour, TimeFormat::TwentyFourHour, 0..24),
            (Segment::Hour, TimeFormat::TwelveHour, 1..13),
            (Segment::Minute, TimeFormat::TwentyFourHour, 0..60),
        ] {
            for value in values {
                let point = dial_position(value, segment, format, size);
                assert_eq!(dial_value(point, segment, format, size), value);
            }
        }
    }
}
