use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Role, SharedString, StatefulInteractiveElement as _, Window, div,
};
use gpui_base::TestSupportExt as _;
use gpui_material::{
    Text,
    date_time::{
        DatePicker, DatePickerLocale, DatePickerState, DatePickerVariant, DateSelection, NaiveDate,
        NaiveTime, TimeFormat, TimePicker, TimePickerLayout, TimePickerState, TimePickerVariant,
        Weekday,
    },
    theme::TypeRole,
};

use crate::story::{Story, section, stack};

pub(crate) struct DateTimeExamples {
    story: Story,
    states: PickerStates,
    first_date: DateSelection,
    second_date: DateSelection,
    first_time: NaiveTime,
    second_time: NaiveTime,
    result: SharedString,
}

enum PickerStates {
    Dates([Entity<DatePickerState>; 2]),
    Times([Entity<TimePickerState>; 2]),
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

impl DateTimeExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let states = if story == Story::TimePicker {
            PickerStates::Times(std::array::from_fn(|_| {
                let state = cx.new(|cx| TimePickerState::new(window, cx));
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                state
            }))
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

    fn dates(&self, states: &[Entity<DatePickerState>; 2], cx: &Context<Self>) -> AnyElement {
        let range = self.story == Story::DateRangePicker;
        let locale = DatePickerLocale::default()
            .first_day(Weekday::Mon)
            .input_format(
                |date| date.format("%d/%m/%Y").to_string().into(),
                |text| NaiveDate::parse_from_str(text, "%d/%m/%Y").ok(),
            );
        stack()
            .child(
                section(if range {
                    "Modal calendar"
                } else {
                    "Docked calendar"
                })
                .child(
                    DatePicker::new(
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
                    .on_confirm(Self::on_date(0, cx)),
                ),
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
}

impl Render for DateTimeExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.states {
            PickerStates::Dates(states) => self.dates(states, cx),
            PickerStates::Times(states) => self.times(states, cx),
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
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{AnyWindowHandle, ElementId, InputEvent as _, TestAppContext, WindowOptions};
    use gpui_base::test_support::{ElementSnapshot, find};
    use gpui_material::MaterialRoot;

    use super::*;

    fn fixture(
        story: Story,
        cx: &mut TestAppContext,
    ) -> (AnyWindowHandle, Entity<DateTimeExamples>) {
        cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let mut view = None;
            let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let entity = cx.new(|cx| DateTimeExamples::new(story, window, cx));
                view = Some(entity.clone());
                entity
            })
            .unwrap();
            (handle.into(), view.unwrap())
        })
    }

    fn frame(handle: AnyWindowHandle, cx: &mut TestAppContext) {
        for _ in 0..2 {
            cx.update_window(handle, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
    }

    fn element(
        handle: AnyWindowHandle,
        id: impl Into<ElementId>,
        cx: &mut TestAppContext,
    ) -> ElementSnapshot {
        let id = id.into();
        cx.update_window(handle, |_, window, _| find(window, &[], &id))
            .unwrap()
            .unwrap_or_else(|| panic!("missing {id:?}"))
    }

    fn click(handle: AnyWindowHandle, id: impl Into<ElementId>, cx: &mut TestAppContext) {
        frame(handle, cx);
        let position = element(handle, id, cx).bounds().center();
        cx.update_window(handle, |_, window, cx| {
            window.activate_window();
            window.dispatch_event(
                gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        frame(handle, cx);
    }

    fn part(picker: &'static str, part: &'static str) -> ElementId {
        (ElementId::from(picker), part).into()
    }

    #[gpui::test]
    fn docked_calendar_rejects_disabled_day_and_restores_focus_after_key_selection(
        cx: &mut TestAppContext,
    ) {
        let (window, view) = fixture(Story::DatePicker, cx);
        let opener = part("calendar-picker", "open");
        click(window, opener.clone(), cx);
        let grid = part("calendar-picker", "calendar-grid");
        assert_eq!(element(window, grid.clone(), cx).focused(), Some(true));
        cx.simulate_keystrokes(window, "right enter");
        frame(window, cx);
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).first_date,
                DateSelection::Single(Some(date(2026, 11, 14)))
            )
        });
        assert_eq!(element(window, grid, cx).focused(), Some(true));

        cx.simulate_keystrokes(window, "right enter");
        frame(window, cx);
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).first_date,
                DateSelection::Single(Some(date(2026, 11, 16)))
            )
        });
        assert_eq!(element(window, opener, cx).focused(), Some(true));
    }

    #[gpui::test]
    fn modal_range_commits_across_months_only_on_confirm(cx: &mut TestAppContext) {
        let (window, view) = fixture(Story::DateRangePicker, cx);
        let opener = part("calendar-picker", "open");
        click(window, opener.clone(), cx);
        cx.simulate_keystrokes(window, "enter pagedown enter");
        frame(window, cx);
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).first_date,
                DateSelection::Range(Some((date(2026, 11, 14), date(2026, 11, 17))))
            )
        });
        click(window, "confirm", cx);
        assert_eq!(element(window, opener, cx).focused(), Some(true));
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).first_date,
                DateSelection::Range(Some((date(2026, 11, 14), date(2026, 12, 14))))
            )
        });
    }

    #[gpui::test]
    fn dial_can_choose_24_hour_inner_ring_then_adjust_minute_by_keyboard(cx: &mut TestAppContext) {
        let (window, view) = fixture(Story::TimePicker, cx);
        let opener = part("dial-picker", "open");
        click(window, opener.clone(), cx);
        let dial = part("dial-picker", "dial");
        assert_eq!(element(window, dial.clone(), cx).focused(), Some(true));
        click(window, part("dial-picker", "mark-18"), cx);
        assert_eq!(element(window, dial, cx).focused(), Some(true));
        cx.simulate_keystrokes(window, "right");
        frame(window, cx);
        click(window, "confirm", cx);
        assert_eq!(element(window, opener, cx).focused(), Some(true));
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).first_time,
                NaiveTime::from_hms_opt(18, 36, 0).unwrap()
            )
        });
    }

    #[gpui::test]
    fn invalid_time_stays_open_and_cancel_preserves_value(cx: &mut TestAppContext) {
        let (window, view) = fixture(Story::TimePicker, cx);
        let opener = part("time-input-picker", "open");
        click(window, opener.clone(), cx);
        let hour = part("time-input-picker", "hour");
        assert_eq!(element(window, hour.clone(), cx).focused(), Some(true));
        let select_all = if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        };
        cx.simulate_keystrokes(window, select_all);
        cx.simulate_input(window, "13");
        click(window, "confirm", cx);
        let alert = element(window, "picker-status", cx);
        assert_eq!(alert.role(), Some(Role::Alert));
        cx.simulate_keystrokes(window, "escape");
        frame(window, cx);
        assert_eq!(element(window, opener.clone(), cx).focused(), Some(true));
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).second_time,
                NaiveTime::from_hms_opt(9, 15, 0).unwrap()
            )
        });

        click(window, opener.clone(), cx);
        cx.simulate_keystrokes(window, select_all);
        cx.simulate_input(window, "12");
        click(window, part("time-input-picker", "pm"), cx);
        click(window, "confirm", cx);
        assert_eq!(element(window, opener, cx).focused(), Some(true));
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).second_time,
                NaiveTime::from_hms_opt(12, 15, 0).unwrap()
            )
        });
    }
}
