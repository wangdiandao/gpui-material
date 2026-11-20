use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Role, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material::{
    Button, ButtonVariant, Switch, Text,
    date_time::{
        DatePicker, DatePickerLocale, DatePickerState, DatePickerVariant, DateSelection, NaiveDate,
        NaiveTime, ScrollField, ScrollFieldState, TimeFormat, TimePicker, TimePickerLayout,
        TimePickerLocale, TimePickerState, TimePickerStyle, TimePickerVariant,
        VibrantTimePickerDialog, Weekday,
    },
    theme::{CornerShape, TypeRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod date_picker_tests;
#[cfg(all(test, feature = "test-support"))]
mod scroll_field_tests;
#[cfg(all(test, feature = "test-support"))]
mod time_picker_tests;

pub(crate) struct DateTimeExamples {
    story: Story,
    states: PickerStates,
    first_date: DateSelection,
    second_date: DateSelection,
    first_time: NaiveTime,
    second_time: NaiveTime,
    vibrant_time: NaiveTime,
    scroll_locked: bool,
    result: SharedString,
}

enum PickerStates {
    Dates([Entity<DatePickerState>; 3]),
    Times {
        dialogs: [Entity<TimePickerState>; 2],
        vibrant: Entity<TimePickerState>,
        hours: Entity<ScrollFieldState>,
        minutes: Entity<ScrollFieldState>,
    },
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

impl DateTimeExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let states = if story == Story::TimePicker {
            let mut scroll_field = |count, index| {
                let state = cx.new(|cx| ScrollFieldState::new(count, index, cx));
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                state
            };
            let hours = scroll_field(24, 14);
            let minutes = scroll_field(60, 35);
            let vibrant = cx.new(|cx| {
                let mut state = TimePickerState::new(window, cx);
                state.set_format(TimeFormat::TwelveHour, window, cx);
                state.set_locale(
                    TimePickerLocale::default()
                        .labels(
                            "Meeting hour",
                            "Meeting minute",
                            ":",
                            "Morning",
                            "Afternoon",
                        )
                        .validation_message("Enter a valid meeting time"),
                    window,
                    cx,
                );
                state
            });
            cx.observe(&vibrant, |_, _, cx| cx.notify()).detach();
            PickerStates::Times {
                vibrant,
                dialogs: std::array::from_fn(|_| {
                    let state = cx.new(|cx| TimePickerState::new(window, cx));
                    cx.observe(&state, |_, _, cx| cx.notify()).detach();
                    state
                }),
                hours,
                minutes,
            }
        } else {
            PickerStates::Dates(std::array::from_fn(|_| {
                let state = cx.new(|cx| {
                    DatePickerState::new(window, cx)
                        .date_bounds(date(2024, 1, 1)..=date(2028, 12, 31))
                        .disabled_dates(|unavailable| unavailable == date(2026, 11, 15))
                });
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                state
            }))
        };
        let initial_date = if story == Story::DateRangePicker {
            DateSelection::Range(Some((date(2026, 11, 14), date(2026, 11, 17))))
        } else {
            DateSelection::Single(Some(date(2026, 11, 14)))
        };
        Self {
            story,
            states,
            first_date: initial_date,
            second_date: initial_date,
            first_time: NaiveTime::from_hms_opt(14, 35, 0).unwrap(),
            second_time: NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
            vibrant_time: NaiveTime::from_hms_opt(23, 59, 0).unwrap(),
            scroll_locked: false,
            result: "No changes yet".into(),
        }
    }

    fn on_date(
        index: usize,
        cx: &Context<Self>,
    ) -> impl Fn(DateSelection, &mut Window, &mut App) + 'static {
        let view = cx.weak_entity();
        move |value, _, cx| {
            let _ = view.update(cx, |view, cx| {
                if index == 0 {
                    view.first_date = value;
                } else {
                    view.second_date = value;
                }
                view.result = match value {
                    DateSelection::Single(Some(date)) => format!("Selected {date}").into(),
                    DateSelection::Range(Some((start, end))) => {
                        format!("Selected {start} to {end}").into()
                    }
                    _ => "Selection cleared".into(),
                };
                cx.notify();
            });
        }
    }

    fn on_time(
        index: usize,
        cx: &Context<Self>,
    ) -> impl Fn(NaiveTime, &mut Window, &mut App) + 'static {
        let view = cx.weak_entity();
        move |value, _, cx| {
            let _ = view.update(cx, |view, cx| {
                if index == 0 {
                    view.first_time = value;
                } else {
                    view.second_time = value;
                }
                view.result = format!("Selected {}", value.format("%H:%M")).into();
                cx.notify();
            });
        }
    }

    fn dates(&self, states: &[Entity<DatePickerState>; 3], cx: &Context<Self>) -> AnyElement {
        let range = self.story == Story::DateRangePicker;
        let locale = DatePickerLocale::default()
            .first_day(Weekday::Mon)
            .input_format(
                |date| date.format("%d/%m/%Y").to_string().into(),
                |text| NaiveDate::parse_from_str(text, "%d/%m/%Y").ok(),
            );
        let calendar = DatePicker::new(
            "calendar-picker",
            if range {
                "Choose travel dates"
            } else {
                "Choose a date"
            },
            &states[0],
            self.first_date,
        )
        .variant(if range {
            DatePickerVariant::Modal
        } else {
            DatePickerVariant::Docked
        })
        .on_confirm(Self::on_date(0, cx));
        let calendar = if range {
            calendar
        } else {
            calendar
                .docked_anchor(|field| div().w(px(260.)).child(field.label_always_minimized(true)))
        };
        stack()
            .child(
                section(if range {
                    "Modal calendar"
                } else {
                    "Docked calendar"
                })
                .child(calendar),
            )
            .child(
                section("Date entry")
                    .description("Enter dates in day/month/year format.")
                    .child(
                        DatePicker::new(
                            "input-picker",
                            if range {
                                "Enter travel dates"
                            } else {
                                "Enter a date"
                            },
                            &states[1],
                            self.second_date,
                        )
                        .variant(DatePickerVariant::ModalInput)
                        .locale(locale)
                        .on_confirm(Self::on_date(1, cx)),
                    ),
            )
            .child(
                section("Full-screen date picker")
                    .description(
                        "The calendar fills compact windows and uses a dialog at wider sizes.",
                    )
                    .child(
                        DatePicker::new(
                            "full-date-picker",
                            "Choose dates full screen",
                            &states[2],
                            self.second_date,
                        )
                        .variant(DatePickerVariant::FullScreen)
                        .title_content(
                            Text::new("Plan your dates")
                                .id("plan-date-title")
                                .role(TypeRole::TitleMedium),
                        )
                        .confirm_content(|confirm| {
                            Button::new("apply-dates", "Apply dates")
                                .variant(ButtonVariant::Text)
                                .on_activate(move |window, cx| confirm.activate(window, cx))
                        })
                        .on_confirm(Self::on_date(1, cx)),
                    ),
            )
            .into_any_element()
    }

    fn times(&self, states: &[Entity<TimePickerState>; 2], cx: &Context<Self>) -> AnyElement {
        stack()
            .child(
                section("24-hour dial").child(
                    TimePicker::new("dial-picker", "Choose a time", &states[0], self.first_time)
                        .layout(TimePickerLayout::Horizontal)
                        .on_confirm(Self::on_time(0, cx)),
                ),
            )
            .child(
                section("12-hour input")
                    .description("Enter the hour and minutes, then choose AM or PM.")
                    .child(
                        TimePicker::new(
                            "time-input-picker",
                            "Enter a time",
                            &states[1],
                            self.second_time,
                        )
                        .format(TimeFormat::TwelveHour)
                        .variant(TimePickerVariant::Input)
                        .layout(TimePickerLayout::Vertical)
                        .on_confirm(Self::on_time(1, cx)),
                    ),
            )
            .into_any_element()
    }
    fn vibrant_time(&self, state: &Entity<TimePickerState>, cx: &Context<Self>) -> AnyElement {
        let initial = self.vibrant_time;
        let opener = state.clone();
        let view = cx.weak_entity();
        section("Vibrant time picker")
            .child(
                Button::new("open-vibrant-time", "Choose a meeting time")
                    .variant(ButtonVariant::Outlined)
                    .on_activate(move |window, cx| {
                        opener.update(cx, |state, cx| {
                            state.set_mode(TimePickerVariant::Scroll, window, cx);
                            state.open(initial, window, cx);
                        });
                    }),
            )
            .child(
                VibrantTimePickerDialog::new("vibrant-time", "Meeting time", state)
                    .title_content(Text::new("Meeting time"))
                    .style(TimePickerStyle {
                        time_field_shape: Some(CornerShape::Radii([12., 20., 20., 12.])),
                        period_selector_shape: Some(CornerShape::Full),
                        ..Default::default()
                    })
                    .mode_toggle_content(|action| {
                        Button::new(
                            "meeting-mode",
                            if action.target() == TimePickerVariant::Input {
                                "Type meeting time"
                            } else {
                                "Use meeting wheels"
                            },
                        )
                        .variant(ButtonVariant::Text)
                        .on_activate(move |window, cx| action.activate(window, cx))
                    })
                    .confirm_content(|action| {
                        Button::new("meeting-confirm", "Save meeting time")
                            .on_activate(move |window, cx| action.activate(window, cx))
                    })
                    .dismiss_content(|action| {
                        Button::new("meeting-dismiss", "Discard meeting time")
                            .variant(ButtonVariant::Text)
                            .on_activate(move |window, cx| action.activate(window, cx))
                    })
                    .on_confirm(move |time, _, cx| {
                        let _ = view.update(cx, |view, cx| {
                            view.vibrant_time = time;
                            view.result = format!("Meeting at {}", time.format("%H:%M")).into();
                            cx.notify();
                        });
                    }),
            )
            .into_any_element()
    }
}

impl Render for DateTimeExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.states {
            PickerStates::Dates(states) => self.dates(states, cx),
            PickerStates::Times {
                dialogs,
                vibrant,
                hours,
                minutes,
            } => {
                let hour = hours.read(cx).selected_option().unwrap_or_default();
                let minute = minutes.read(cx).selected_option().unwrap_or_default();
                let view = cx.weak_entity();
                stack()
                    .child(self.times(dialogs, cx))
                    .child(self.vibrant_time(vibrant, cx))
                    .child(
                        section("Scroll fields")
                            .description(
                                "Scroll or use the arrow keys to choose an hour and a minute.",
                            )
                            .child(
                                Switch::new(
                                    "lock-scroll",
                                    "Lock scroll fields",
                                    self.scroll_locked,
                                )
                                .on_change(move |locked, _, cx| {
                                    let _ = view.update(cx, |view, cx| {
                                        view.scroll_locked = locked;
                                        cx.notify();
                                    });
                                }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div().flex_1().child(
                                            ScrollField::new(
                                                "scroll-hour",
                                                "Hour, 00 to 23",
                                                hours,
                                            )
                                            .disabled(self.scroll_locked)
                                            .value_description(|hour| format!("{hour:02} hours")),
                                        ),
                                    )
                                    .child(
                                        Text::new(":")
                                            .id("scroll-separator")
                                            .role(TypeRole::DisplayMedium),
                                    )
                                    .child(
                                        div().flex_1().child(
                                            ScrollField::new(
                                                "scroll-minute",
                                                "Minute, 00 to 59",
                                                minutes,
                                            )
                                            .disabled(self.scroll_locked)
                                            .value_description(|minute| {
                                                format!("{minute:02} minutes")
                                            }),
                                        ),
                                    )
                                    .max_w(px(320.)),
                            )
                            .child(
                                Text::new(format!("Selected {hour:02}:{minute:02}"))
                                    .id("scroll-time-value")
                                    .role(TypeRole::BodyMedium),
                            ),
                    )
                    .into_any_element()
            }
        };
        stack().child(body).child(
            div()
                .id("date-time-result")
                .test_support()
                .role(Role::Status)
                .aria_label(self.result.clone())
                .child(Text::new(self.result.clone()).role(TypeRole::BodyMedium)),
        )
    }
}
