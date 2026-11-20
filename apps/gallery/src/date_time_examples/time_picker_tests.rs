use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyWindowHandle, AppContext as _, Context, ElementId, Entity, InputEvent as _, IntoElement,
    ParentElement as _, Render, Role, Styled as _, TestAppContext, Window, WindowOptions, div, rgb,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::{
    Button, DialogCloseReason, MaterialRoot, Text,
    date_time::{
        NaiveTime, ScrollFieldColors, TimeFormat, TimePickerDialog, TimePickerLocale,
        TimePickerPanel, TimePickerSegment, TimePickerState, TimePickerStyle, TimePickerVariant,
        TimeScroll,
    },
    material_theme, set_material_theme,
    theme::{CornerShape, Direction},
};

use super::{DateTimeExamples, PickerStates};
use crate::story::Story;

pub(super) fn fixture(
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

pub(super) fn frame(handle: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..2 {
        cx.update_window(handle, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

pub(super) fn element(
    handle: AnyWindowHandle,
    id: impl Into<ElementId>,
    cx: &mut TestAppContext,
) -> ElementSnapshot {
    let id = id.into();
    cx.update_window(handle, |_, window, _| find(window, &[], &id))
        .unwrap()
        .unwrap_or_else(|| panic!("missing {id:?}"))
}

pub(super) fn click(handle: AnyWindowHandle, id: impl Into<ElementId>, cx: &mut TestAppContext) {
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

pub(super) fn part(picker: &'static str, part: &'static str) -> ElementId {
    (ElementId::from(picker), part).into()
}

pub(super) fn assert_colors(handle: AnyWindowHandle, colors: &[u32], cx: &mut TestAppContext) {
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
            scroll: ScrollFieldColors {
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
    allow: Rc<Cell<bool>>,
    requests: Rc<RefCell<Vec<DialogCloseReason>>>,
    closes: Rc<RefCell<Vec<DialogCloseReason>>>,
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
