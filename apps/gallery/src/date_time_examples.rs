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
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{
        AnyWindowHandle, ElementId, InputEvent as _, ScrollDelta, ScrollEvent, TestAppContext,
        TouchPhase, WindowOptions, point, rgb,
    };
    use gpui_base::test_support::{ElementSnapshot, find};
    use gpui_material::{
        DialogCloseReason, MaterialRoot,
        date_time::{TimePickerDialog, TimePickerPanel, TimePickerSegment, TimeScroll},
    };

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

    const FIELD_COLORS: [u32; 6] = [0x28445a, 0x9bb6ca, 0xe9f7ff, 0x5a4539, 0x957c66, 0xd1b293];

    struct WheelFixture {
        state: Entity<ScrollFieldState>,
        source: Entity<gpui_material::InteractionSource>,
        disabled: bool,
    }

    #[derive(IntoElement)]
    struct RowInk {
        index: usize,
        selected: bool,
        enabled: bool,
    }

    impl gpui::RenderOnce for RowInk {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            div()
                .id((
                    if self.enabled && self.selected {
                        "chosen-option"
                    } else {
                        "option"
                    },
                    self.index,
                ))
                .test_support()
                .size(px(14.))
                .bg(window.text_style().color)
        }
    }

    impl Render for WheelFixture {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            use gpui_material::date_time::ScrollFieldColors;

            let [
                container,
                text,
                selected,
                disabled_container,
                disabled_text,
                disabled_selected,
            ] = FIELD_COLORS.map(|tint| Some(rgb(tint).into()));
            div().p_4().w(px(180.)).child(
                ScrollField::new("test-wheel", "Duration in hours", &self.state)
                    .value_description(|index| format!("{index} hours"))
                    .colors(ScrollFieldColors {
                        container,
                        content: text,
                        selected_content: selected,
                        disabled_container,
                        disabled_content: disabled_text,
                        disabled_selected_content: disabled_selected,
                    })
                    .interaction_source(&self.source)
                    .disabled(self.disabled)
                    .item_content(|index, selected, enabled| RowInk {
                        index,
                        selected,
                        enabled,
                    }),
            )
        }
    }

    fn wheel_fixture(
        cx: &mut TestAppContext,
        reduced: bool,
    ) -> (AnyWindowHandle, Entity<WheelFixture>) {
        cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(reduced);
            let state = cx.new(|cx| ScrollFieldState::new(9, 8, cx));
            let source = cx.new(|_| gpui_material::InteractionSource::new());
            let model = cx.new(|_| WheelFixture {
                state,
                source,
                disabled: false,
            });
            let handle =
                MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone())
                    .unwrap();
            (handle.into(), model)
        })
    }

    fn wheel_selection(
        state: &Entity<ScrollFieldState>,
        cx: &mut TestAppContext,
    ) -> (Option<usize>, Option<usize>, bool) {
        cx.update(|cx| {
            let state = state.read(cx);
            (
                state.selected_option(),
                state.target_option(),
                state.is_scroll_in_progress(),
            )
        })
    }

    fn send_scroll(
        handle: AnyWindowHandle,
        delta: ScrollDelta,
        touch_phase: TouchPhase,
        cx: &mut TestAppContext,
    ) {
        cx.update_window(handle, |_, window, cx| {
            let position = find(window, &[], &"test-wheel".into())
                .unwrap()
                .bounds()
                .center();
            window.dispatch_event(
                ScrollEvent {
                    position,
                    delta,
                    touch_phase,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
    }

    fn assert_colors(handle: AnyWindowHandle, colors: &[u32], cx: &mut TestAppContext) {
        cx.update_window(handle, |_, window, _| {
            let quads = window.painted_quads();
            for &color in colors {
                let fill: gpui::Hsla = rgb(color).into();
                assert!(quads.iter().any(|quad| quad.background == fill.into()));
            }
        })
        .unwrap();
    }

    #[gpui::test]
    fn scroll_field_accepts_pointer_wheel_touchpad_and_keyboard_with_dynamic_count(
        cx: &mut TestAppContext,
    ) {
        let (handle, model) = wheel_fixture(cx, true);
        let (state, source) =
            cx.update(|cx| (model.read(cx).state.clone(), model.read(cx).source.clone()));
        frame(handle, cx);
        let field = element(handle, "test-wheel", cx);
        assert_eq!(field.role(), Some(Role::Slider));
        assert_eq!(field.label(), Some("Duration in hours"));
        assert_eq!(field.value(), Some("8 hours"));
        assert_eq!(field.bounds().size.height, px(200.));
        let offset = element(handle, ("chosen-option", 8_usize), cx)
            .bounds()
            .center()
            - field.bounds().center();
        assert!(offset.x.abs() <= px(1.) && offset.y.abs() <= px(1.));
        assert_colors(handle, &FIELD_COLORS[..3], cx);

        click(handle, ("scroll-field-row", 1_usize), cx);
        assert_eq!(wheel_selection(&state, cx), (Some(7), Some(7), false));
        assert_eq!(element(handle, "test-wheel", cx).focused(), Some(true));
        let key = |name, cx: &mut TestAppContext| {
            cx.simulate_keystrokes(handle, name);
            frame(handle, cx);
        };
        key("down", cx);
        assert_eq!(wheel_selection(&state, cx), (Some(8), Some(8), false));
        let wheel_down = ScrollDelta::Lines(point(0., -1.));
        send_scroll(handle, wheel_down, TouchPhase::Moved, cx);
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(0), Some(0), false));

        let touchpad = ScrollDelta::Pixels(point(px(0.), px(-20.)));
        send_scroll(handle, touchpad, TouchPhase::Started, cx);
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(0), Some(0), true));
        cx.update(|cx| {
            assert!(
                source
                    .read(cx)
                    .state(handle.window_id(), &"test-wheel".into())
                    .unwrap()
                    .dragged
            );
        });
        send_scroll(handle, touchpad, TouchPhase::Moved, cx);
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), true));
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(140));
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));
        send_scroll(
            handle,
            ScrollDelta::Pixels(point(px(0.), px(-60.))),
            TouchPhase::Started,
            cx,
        );
        send_scroll(
            handle,
            ScrollDelta::Pixels(point(px(0.), px(0.))),
            TouchPhase::Cancelled,
            cx,
        );
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));

        cx.update(|cx| {
            state.update(cx, |state, cx| {
                state.scroll_to_option(usize::MAX, cx);
                state.set_item_count(3, cx);
                assert_eq!(state.item_count(), 3);
            })
        });
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(2), Some(2), false));
        cx.update(|cx| state.update(cx, |state, cx| state.set_item_count(0, cx)));
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (None, None, false));
        assert_eq!(element(handle, "test-wheel", cx).value(), None);
        cx.update(|cx| {
            state.update(cx, |state, cx| {
                state.set_item_count(12, cx);
                state.animate_scroll_to_option(usize::MAX, cx);
            })
        });
        assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));

        frame(handle, cx);
        send_scroll(handle, touchpad, TouchPhase::Started, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), true));

        cx.update(|cx| {
            model.update(cx, |model, cx| {
                model.disabled = true;
                cx.notify();
            })
        });
        frame(handle, cx);
        assert_colors(handle, &FIELD_COLORS[3..], cx);
        assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(140));
        frame(handle, cx);
        click(handle, ("scroll-field-row", 1_usize), cx);
        send_scroll(handle, wheel_down, TouchPhase::Moved, cx);
        key("up", cx);
        assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));
        cx.update(|cx| {
            let snapshot = source
                .read(cx)
                .state(handle.window_id(), &"test-wheel".into())
                .unwrap();
            assert!(snapshot.disabled && !snapshot.dragged && !snapshot.pressed);
        });
    }

    #[gpui::test]
    fn scroll_field_reports_animated_target_and_settles_after_retargeting(cx: &mut TestAppContext) {
        let (handle, model) = wheel_fixture(cx, false);
        let state = cx.update(|cx| model.read(cx).state.clone());
        frame(handle, cx);
        cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(6, cx)));
        assert_eq!(wheel_selection(&state, cx), (Some(8), Some(6), true));
        frame(handle, cx);
        assert_eq!(element(handle, "test-wheel", cx).value(), Some("6 hours"));
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(50));
        frame(handle, cx);
        cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(4, cx)));
        assert!(wheel_selection(&state, cx).2);
        frame(handle, cx);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(2));
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(4), Some(4), false));

        send_scroll(
            handle,
            ScrollDelta::Lines(point(0., -1.)),
            TouchPhase::Moved,
            cx,
        );
        assert_eq!(wheel_selection(&state, cx), (Some(4), Some(5), true));
        cx.update(|cx| {
            model.update(cx, |model, cx| {
                model.disabled = true;
                cx.notify();
            })
        });
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(5), Some(5), false));
        cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(1, cx)));
        let (_, target, in_progress) = wheel_selection(&state, cx);
        assert_eq!(target, Some(1));
        assert!(in_progress);
        frame(handle, cx);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(2));
        frame(handle, cx);
        assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));
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
    #[gpui::test]
    fn vibrant_time_scroll_wraps_validates_and_restores_focus(cx: &mut TestAppContext) {
        let (window, view) = fixture(Story::TimePicker, cx);
        click(window, "open-vibrant-time", cx);
        let hour = part("vibrant-time", "hour-scroll");
        let minute = part("vibrant-time", "minute-scroll");
        let hour_value = element(window, hour.clone(), cx);
        assert_eq!(hour_value.role(), Some(Role::Slider));
        assert_eq!(hour_value.label(), Some("Meeting hour"));
        assert_eq!(hour_value.value(), Some("11"));
        assert_eq!(hour_value.focused(), Some(true));
        assert_eq!(element(window, minute.clone(), cx).value(), Some("59"));
        cx.simulate_keystrokes(window, "down");
        frame(window, cx);
        assert_eq!(element(window, hour.clone(), cx).value(), Some("12"));
        cx.simulate_keystrokes(window, "down tab down");
        frame(window, cx);
        assert_eq!(element(window, hour, cx).value(), Some("01"));
        assert_eq!(element(window, minute, cx).value(), Some("00"));
        cx.update(|cx| {
            let PickerStates::Times { vibrant, .. } = &view.read(cx).states else {
                unreachable!()
            };
            let state = vibrant.read(cx);
            assert_eq!((state.hour(), state.minute()), (13, 0));
            assert_eq!(state.segment(), TimePickerSegment::Minute);
        });

        click(window, "meeting-mode", cx);
        let minute_input = part("vibrant-time", "minute");
        assert_eq!(
            element(window, minute_input.clone(), cx).focused(),
            Some(true)
        );
        let select_all = if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        };
        cx.simulate_keystrokes(window, select_all);
        cx.simulate_input(window, "60");
        click(window, "meeting-confirm", cx);
        assert_eq!(
            element(window, "picker-status", cx).label(),
            Some("Enter a valid meeting time")
        );
        assert_eq!(element(window, minute_input, cx).focused(), Some(true));
        cx.simulate_keystrokes(window, select_all);
        cx.simulate_input(window, "08");
        click(window, "meeting-confirm", cx);
        assert_eq!(
            element(window, "open-vibrant-time", cx).focused(),
            Some(true)
        );
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).vibrant_time,
                NaiveTime::from_hms_opt(13, 8, 0).unwrap()
            )
        });

        click(window, "open-vibrant-time", cx);
        click(window, part("vibrant-time", "am"), cx);
        click(window, "meeting-dismiss", cx);
        assert_eq!(
            element(window, "open-vibrant-time", cx).focused(),
            Some(true)
        );
        cx.update(|cx| {
            assert_eq!(
                view.read(cx).vibrant_time,
                NaiveTime::from_hms_opt(13, 8, 0).unwrap()
            )
        });
    }

    struct StandaloneTime {
        state: Entity<TimePickerState>,
    }

    impl Render for StandaloneTime {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let style = TimePickerStyle {
                time_field_shape: Some(CornerShape::Radii([4., 12., 16., 8.])),
                clock_dial: Some(rgb(0x281e4a).into()),
                selector: Some(rgb(0xef9540).into()),
                scroll: gpui_material::date_time::ScrollFieldColors {
                    container: Some(rgb(0x345a85).into()),
                    ..Default::default()
                },
                ..Default::default()
            };
            div().p_4().child(match self.state.read(cx).mode() {
                TimePickerVariant::Dial => TimePickerPanel::dial("standalone-time", &self.state)
                    .style(style)
                    .into_any_element(),
                TimePickerVariant::Input => TimePickerPanel::input("standalone-time", &self.state)
                    .style(style)
                    .into_any_element(),
                TimePickerVariant::Scroll => TimeScroll::new("standalone-time", &self.state)
                    .style(style)
                    .mode_toggle_content(|mode| {
                        Button::new("wheel-to-input", "Use text")
                            .on_activate(move |window, cx| mode.activate(window, cx))
                    })
                    .into_any_element(),
            })
        }
    }

    #[gpui::test]
    fn standalone_time_panels_share_external_state_locale_and_rtl(cx: &mut TestAppContext) {
        use gpui_material::{material_theme, set_material_theme, theme::Direction};

        let mut state = None;
        let window = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = Direction::Rtl;
            set_material_theme(theme, cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let picker = cx.new(|cx| {
                    let mut time = TimePickerState::new(window, cx);
                    time.set_mode(TimePickerVariant::Scroll, window, cx);
                    time.set_value(NaiveTime::from_hms_opt(23, 59, 45).unwrap(), window, cx);
                    time.set_locale(
                        TimePickerLocale::default()
                            .numbers(
                                |value| format!("{value:02} u").into(),
                                |text| text.strip_suffix(" u")?.parse().ok(),
                            )
                            .labels(
                                "Localized hour",
                                "Localized minute",
                                "h",
                                "Morning",
                                "Evening",
                            ),
                        window,
                        cx,
                    );
                    time
                });
                state = Some(picker.clone());
                cx.new(|cx| {
                    cx.observe(&picker, |_, _, cx| cx.notify()).detach();
                    StandaloneTime { state: picker }
                })
            })
            .unwrap()
        });
        let window = window.into();
        let state = state.unwrap();
        frame(window, cx);
        let hour = part("standalone-time", "hour-scroll");
        let minute = part("standalone-time", "minute-scroll");
        assert!(
            element(window, hour.clone(), cx).bounds().left()
                > element(window, minute.clone(), cx).bounds().left()
        );
        assert_eq!(element(window, hour.clone(), cx).value(), Some("23 u"));
        click(window, hour.clone(), cx);
        cx.simulate_keystrokes(window, "down tab down");
        frame(window, cx);
        assert_eq!(element(window, hour, cx).value(), Some("00 u"));
        assert_eq!(element(window, minute.clone(), cx).value(), Some("00 u"));
        cx.update_window(window, |_, window, cx| {
            state.update(cx, |state, cx| {
                assert_eq!((state.hour(), state.minute()), (0, 0));
                assert!(state.set_hour(14, window, cx));
                assert!(state.set_minute(7, window, cx));
                assert!(!state.set_hour(24, window, cx));
                assert!(!state.set_minute(60, window, cx));
            });
        })
        .unwrap();
        frame(window, cx);
        cx.update(|cx| cx.set_reduce_motion(false));
        click(window, minute.clone(), cx);
        cx.simulate_keystrokes(window, "down");
        frame(window, cx);
        assert_eq!(element(window, minute.clone(), cx).value(), Some("08 u"));
        cx.update(|cx| assert_eq!(state.read(cx).minute(), 7));
        cx.update_window(window, |_, window, cx| {
            state.update(cx, |state, cx| assert!(state.set_minute(7, window, cx)));
        })
        .unwrap();
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        frame(window, cx);
        assert_eq!(element(window, minute, cx).value(), Some("07 u"));
        cx.update(|cx| cx.set_reduce_motion(true));
        click(window, "wheel-to-input", cx);
        let minute_input = part("standalone-time", "minute");
        assert_eq!(element(window, minute_input, cx).focused(), Some(true));
        cx.update_window(window, |_, window, cx| {
            state.update(cx, |state, cx| {
                assert_eq!(state.mode(), TimePickerVariant::Input);
                assert_eq!(state.hour_input(cx).as_ref(), "14 u");
                state.set_hour_input("05 u", window, cx);
                state.set_minute_input("60 u", window, cx);
                assert_eq!((state.hour(), state.minute()), (5, 7));
                assert!(state.is_hour_input_valid(cx));
                assert!(!state.is_minute_input_valid(cx));
                assert!(!state.is_input_valid(cx));
                state.set_format(TimeFormat::TwelveHour, window, cx);
                assert!(state.is_input_valid(cx));
                state.set_segment(TimePickerSegment::Hour, cx);
                state.set_mode(TimePickerVariant::Dial, window, cx);
            });
        })
        .unwrap();
        frame(window, cx);
        let dial = element(window, part("standalone-time", "dial"), cx);
        assert_eq!(dial.label(), Some("Localized hour, 05 uh07 u Morning"));
        assert_eq!(dial.focused(), Some(true));
        assert_colors(window, &[0x281e4a, 0xef9540], cx);
        cx.simulate_keystrokes(window, "space right");
        frame(window, cx);
        cx.update(|cx| {
            let time = state.read(cx);
            assert_eq!((time.hour(), time.minute()), (5, 8));
            assert_eq!(time.segment(), TimePickerSegment::Minute);
        });
    }

    struct CustomTimeDialog {
        state: Entity<TimePickerState>,
        allow: std::rc::Rc<std::cell::Cell<bool>>,
        requests: std::rc::Rc<std::cell::RefCell<Vec<DialogCloseReason>>>,
        closes: std::rc::Rc<std::cell::RefCell<Vec<DialogCloseReason>>>,
    }

    impl Render for CustomTimeDialog {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let state = self.state.clone();
            let (allow, requests, closes) = (
                self.allow.clone(),
                self.requests.clone(),
                self.closes.clone(),
            );
            div()
                .p_4()
                .child(
                    Button::new("custom-time-trigger", "Open schedule").on_activate(
                        move |window, cx| {
                            state.update(cx, |state, cx| state.open(NaiveTime::MIN, window, cx));
                        },
                    ),
                )
                .child(
                    TimePickerDialog::new("custom-time-dialog", "Schedule", &self.state)
                        .body(TimePickerPanel::input("custom-time-body", &self.state))
                        .title_content(Text::new("Set schedule").id("custom-time-title"))
                        .show_mode_toggle(false)
                        .on_dismiss_request(move |reason, _, _| {
                            requests.borrow_mut().push(reason);
                            allow.get()
                        })
                        .on_close(move |reason, _, _| closes.borrow_mut().push(reason)),
                )
        }
    }

    #[gpui::test]
    fn time_dialog_reports_only_accepted_user_closes(cx: &mut TestAppContext) {
        use std::{
            cell::{Cell, RefCell},
            rc::Rc,
        };

        let allow = Rc::new(Cell::new(false));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let closes = Rc::new(RefCell::new(Vec::new()));
        let mut state = None;
        let window = cx
            .update(|cx| {
                crate::init_gallery(cx);
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
                        CustomTimeDialog {
                            state: picker,
                            allow: allow.clone(),
                            requests: requests.clone(),
                            closes: closes.clone(),
                        }
                    })
                })
                .unwrap()
            })
            .into();
        let state = state.unwrap();
        click(window, "custom-time-trigger", cx);
        assert_eq!(
            element(window, "custom-time-title", cx).label(),
            Some("Set schedule")
        );
        cx.simulate_keystrokes(window, "escape");
        frame(window, cx);
        cx.update(|cx| assert!(state.read(cx).is_open(cx)));
        assert_eq!(*requests.borrow(), [DialogCloseReason::Escape]);
        assert!(closes.borrow().is_empty());
        allow.set(true);
        cx.simulate_keystrokes(window, "escape");
        frame(window, cx);
        assert_eq!(
            element(window, "custom-time-trigger", cx).focused(),
            Some(true)
        );

        click(window, "custom-time-trigger", cx);
        click(window, "cancel", cx);
        assert_eq!(
            *requests.borrow(),
            [DialogCloseReason::Escape, DialogCloseReason::Escape]
        );
        assert_eq!(
            *closes.borrow(),
            [DialogCloseReason::Escape, DialogCloseReason::Cancel]
        );
        click(window, "custom-time-trigger", cx);
        cx.update_window(window, |_, window, cx| {
            state.update(cx, |state, cx| state.close(window, cx));
        })
        .unwrap();
        frame(window, cx);
        cx.update(|cx| assert!(!state.read(cx).is_open(cx)));
        assert_eq!(
            *closes.borrow(),
            [DialogCloseReason::Escape, DialogCloseReason::Cancel]
        );
    }
}
