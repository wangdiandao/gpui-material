use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyWindowHandle, AppContext, Context, ElementId, Entity, EntityInputHandler as _, FocusHandle,
    InputEvent, InteractiveElement, IntoElement, ParentElement, Render, Role, Styled,
    TestAppContext, Window, WindowOptions, div, px, size,
};
use gpui_base::{
    TestSupportExt,
    test_support::{ElementSnapshot, find},
};
use gpui_material::{
    Button, DialogCloseReason, DialogProperties, MaterialRoot, Text,
    date_time::{
        DatePicker, DatePickerColors, DatePickerDisplayMode as Mode, DatePickerDraft as Draft,
        DatePickerLocale, DatePickerState, DatePickerVariant as Variant, DateSelection, Weekday,
    },
};

#[derive(Clone, Copy)]
enum Example {
    Panel,
    Triggered(Variant),
    DialogOnly,
}

#[derive(Default)]
struct Events {
    selected: RefCell<Vec<DateSelection>>,
    closed: RefCell<Vec<DialogCloseReason>>,
}

struct Probe {
    example: Example,
    value: DateSelection,
    state: Entity<DatePickerState>,
    locale: DatePickerLocale,
    target: FocusHandle,
    opener: FocusHandle,
    events: Rc<Events>,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let locale = self.locale.clone();
        let value = self.value;
        let picker = match self.example {
            Example::Panel => DatePicker::panel("date-probe", "Pick dates", &state),
            Example::Triggered(variant) => {
                DatePicker::new("date-probe", "Pick dates", &state, value).variant(variant)
            }
            Example::DialogOnly => DatePicker::dialog("date-probe", "Pick dates", &state, value)
                .variant(Variant::FullScreen),
        };
        let picker = match self.example {
            Example::Triggered(Variant::Docked) => picker
                .docked_anchor(|field| div().w(px(255.)).child(field.label_always_minimized(true))),
            Example::Triggered(Variant::ModalInput) => picker.focus_target(&self.target),
            _ => picker,
        };
        let confirm = self.events.clone();
        let dismiss = self.events.clone();
        let close_focus = self.target.clone();
        div()
            .size_full()
            .children(matches!(self.example, Example::DialogOnly).then(|| {
                Button::new("external-open", "Open date planner")
                    .track_focus(&self.opener)
                    .on_activate(move |window, cx| {
                        state.update(cx, |state, cx| {
                            state.open(value, Variant::FullScreen, &locale, window, cx)
                        });
                    })
            }))
            .child(
                picker
                    .locale(self.locale.clone())
                    .show_mode_toggle(false)
                    .title_content(Text::new("Review dates").id("custom-date-title"))
                    .headline_content(
                        div()
                            .id("custom-date-headline")
                            .test_support()
                            .child("Scheduled date"),
                    )
                    .colors(DatePickerColors {
                        day: Some(gpui::rgb(0x32426a).into()),
                        selected_day: Some(gpui::rgb(0xffffee).into()),
                        selected_day_container: Some(gpui::rgb(0x32426a).into()),
                        ..DatePickerColors::default()
                    })
                    .properties(DialogProperties {
                        dismiss_on_outside_click: false,
                        ..DialogProperties::default()
                    })
                    .confirm_content(|confirm| {
                        Button::new("date-accept", "Apply selection")
                            .on_activate(move |window, cx| confirm.activate(window, cx))
                    })
                    .dismiss_content(move |action| {
                        Button::new("date-cancel", "Discard dates")
                            .track_focus(&close_focus)
                            .on_activate(move |window, cx| action.activate(window, cx))
                    })
                    .on_confirm(move |value, _, _| confirm.selected.borrow_mut().push(value))
                    .on_closed(move |reason, _, _| dismiss.closed.borrow_mut().push(reason)),
            )
    }
}

struct Harness {
    window: AnyWindowHandle,
    state: Entity<DatePickerState>,
    target: FocusHandle,
    opener: FocusHandle,
    events: Rc<Events>,
}

impl Harness {
    fn new(example: Example, value: DateSelection, cx: &mut TestAppContext) -> Self {
        cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let events = Rc::new(Events::default());
            let target = cx.focus_handle();
            let opener = cx.focus_handle();
            const MONTHS: [&str; 12] = [
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
            let locale = DatePickerLocale::default()
                .first_day(Weekday::Monday)
                .input_format(
                    |date| {
                        format!("{:02}/{:02}/{:04}", date.day(), date.month(), date.year()).into()
                    },
                    super::parse_dmy,
                )
                .accessible_date_format(|date| {
                    format!(
                        "Date {:02} {} {:04}",
                        date.day(),
                        MONTHS[(date.month() - 1) as usize],
                        date.year(),
                    )
                    .into()
                });
            let mut retained = None;
            let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let state = cx.new(|cx| {
                    DatePickerState::new(window, cx)
                        .date_bounds(super::date(2024, 5, 4)..=super::date(2028, 12, 29))
                        .initial_month(super::date(2026, 6, 18))
                        .disabled_years(|year| year == 2025)
                        .disabled_dates(|date| date == super::date(2026, 6, 15))
                });
                retained = Some(state.clone());
                cx.new(|cx| {
                    cx.observe(&state, |_, _, cx| cx.notify()).detach();
                    Probe {
                        example,
                        value,
                        state,
                        locale,
                        target: target.clone(),
                        opener: opener.clone(),
                        events: events.clone(),
                    }
                })
            })
            .unwrap()
            .into();
            Self {
                window,
                state: retained.unwrap(),
                target,
                opener,
                events,
            }
        })
    }

    fn frame(&self, cx: &mut TestAppContext) {
        for _ in 0..2 {
            cx.update_window(self.window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                window.simulate_next_frame(cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
    }

    fn node(&self, key: impl Into<ElementId>, cx: &mut TestAppContext) -> Option<ElementSnapshot> {
        cx.update_window(self.window, |_, window, _| find(window, &[], &key.into()))
            .unwrap()
    }

    fn click(&self, key: impl Into<ElementId>, cx: &mut TestAppContext) {
        self.frame(cx);
        let key = key.into();
        let position = self
            .node(key.clone(), cx)
            .unwrap_or_else(|| panic!("missing {key:?}"))
            .bounds()
            .center();
        cx.update_window(self.window, |_, window, cx| {
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
        self.frame(cx);
    }

    fn set_draft(&self, draft: Draft, cx: &mut TestAppContext) -> bool {
        let accepted = cx
            .update_window(self.window, |_, window, cx| {
                self.state
                    .update(cx, |state, cx| state.set_draft(draft, window, cx))
            })
            .unwrap();
        self.frame(cx);
        accepted
    }

    fn set_mode(&self, mode: Mode, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            self.state.update(cx, |state, cx| {
                assert!(state.set_display_mode(mode, window, cx))
            })
        })
        .unwrap();
        self.frame(cx);
    }
}

fn part(name: &'static str) -> ElementId {
    (ElementId::from("date-probe"), name).into()
}

#[gpui::test]
fn date_panel_separates_partial_draft_month_and_year_validity(cx: &mut TestAppContext) {
    let picker = Harness::new(Example::Panel, DateSelection::Range(None), cx);
    let start = super::date(2026, 6, 12);
    assert_eq!(
        cx.update(|cx| picker.state.read(cx).displayed_month()),
        super::date(2026, 6, 1)
    );
    let partial = Draft::Range(Some(start), None);
    assert!(picker.set_draft(partial, cx));
    assert!(!picker.set_draft(Draft::Range(None, Some(start)), cx));
    assert!(!picker.set_draft(Draft::Single(Some(super::date(2025, 6, 12))), cx));
    assert!(!picker.set_draft(Draft::Single(Some(super::date(2024, 5, 2))), cx));
    let headline = picker.node(part("headline"), cx).unwrap();
    assert_eq!(headline.role(), Some(Role::Status));
    assert_eq!(headline.label(), Some("Date 12 June 2026"));
    assert!(picker.node("custom-date-title", cx).is_some());
    assert!(picker.node("custom-date-headline", cx).is_some());

    picker.set_mode(Mode::Input, cx);
    assert_eq!(cx.update(|cx| picker.state.read(cx).draft(cx)), partial);
    assert!(picker.node(part("end"), cx).is_some());
    assert!(picker.node(part("display-mode"), cx).is_none());
    picker.set_mode(Mode::Calendar, cx);
    assert_eq!(cx.update(|cx| picker.state.read(cx).draft(cx)), partial);
    cx.update(|cx| {
        picker.state.update(cx, |state, cx| {
            state.set_displayed_month(super::date(2025, 7, 27), cx)
        })
    });
    picker.frame(cx);
    assert_eq!(
        cx.update(|cx| picker.state.read(cx).displayed_month()),
        super::date(2025, 7, 1)
    );
    picker.click(part("year-toggle"), cx);
    let blocked = (ElementId::from("date-probe"), "year-2025");
    assert_eq!(
        picker.node(blocked.clone(), cx).unwrap().disabled(),
        Some(true)
    );
    picker.click(blocked, cx);
    assert!(picker.node(part("year-list"), cx).is_some());
    picker.click((ElementId::from("date-probe"), "year-2027"), cx);
    assert_eq!(
        cx.update(|cx| picker.state.read(cx).displayed_month()),
        super::date(2027, 7, 1)
    );
    assert_eq!(cx.update(|cx| picker.state.read(cx).draft(cx)), partial);
}

#[gpui::test]
fn fullscreen_date_dialog_uses_custom_actions_and_restores_external_focus(cx: &mut TestAppContext) {
    let picker = Harness::new(Example::DialogOnly, DateSelection::Range(None), cx);
    cx.simulate_window_resize(picker.window, size(px(410.), px(700.)));
    picker.click("external-open", cx);
    let panel = picker.node("date-probe", cx).unwrap().bounds();
    assert_eq!(panel.size, size(px(410.), px(700.)));
    assert!(picker.node("custom-date-title", cx).is_some());
    picker.click("date-accept", cx);
    assert!(cx.update(|cx| picker.state.read(cx).is_open(cx)));
    assert!(picker.events.selected.borrow().is_empty());

    let start = super::date(2026, 6, 16);
    let end = super::date(2026, 6, 19);
    assert!(picker.set_draft(Draft::Range(Some(start), Some(end)), cx));
    cx.simulate_window_resize(picker.window, size(px(900.), px(700.)));
    picker.frame(cx);
    assert!(picker.node("date-probe", cx).unwrap().bounds().size.width < px(900.));
    assert_eq!(
        cx.update(|cx| picker.state.read(cx).draft(cx)),
        Draft::Range(Some(start), Some(end))
    );
    picker.click("date-accept", cx);
    assert_eq!(
        *picker.events.selected.borrow(),
        [DateSelection::Range(Some((start, end)))]
    );
    assert_eq!(*picker.events.closed.borrow(), [DialogCloseReason::Confirm]);
    assert!(!cx.update(|cx| picker.state.read(cx).is_open(cx)));
    assert!(
        cx.update_window(picker.window, |_, window, _| picker
            .opener
            .is_focused(window))
            .unwrap()
    );

    picker.click("external-open", cx);
    picker.click("date-cancel", cx);
    assert_eq!(
        *picker.events.closed.borrow(),
        [DialogCloseReason::Confirm, DialogCloseReason::Cancel]
    );
    assert_eq!(picker.events.selected.borrow().len(), 1);
}

#[gpui::test]
fn date_input_defers_mode_and_confirmation_until_ime_commits(cx: &mut TestAppContext) {
    let picker = Harness::new(
        Example::Triggered(Variant::ModalInput),
        DateSelection::Single(None),
        cx,
    );
    picker.click(part("open"), cx);
    assert!(
        cx.update_window(picker.window, |_, window, _| picker
            .target
            .is_focused(window))
            .unwrap()
    );
    let editor = cx.update(|cx| picker.state.read(cx).input_editors(cx).0.clone());
    cx.update_window(picker.window, |_, window, cx| {
        editor.update(cx, |editor, cx| {
            editor.replace_and_mark_text_in_range(None, "16/06/2026", Some(10..10), window, cx);
            assert!(editor.marked_text_range(window, cx).is_some());
        });
        picker.state.update(cx, |state, cx| {
            assert!(!state.set_display_mode(Mode::Calendar, window, cx));
        });
    })
    .unwrap();
    picker.click("date-accept", cx);
    assert!(picker.events.selected.borrow().is_empty());
    assert!(cx.update(|cx| picker.state.read(cx).is_open(cx)));
    cx.update_window(picker.window, |_, window, cx| {
        editor.update(cx, |editor, cx| {
            editor.replace_text_in_range(None, "16/06/2026", window, cx)
        });
    })
    .unwrap();
    picker.click("date-accept", cx);
    assert_eq!(
        *picker.events.selected.borrow(),
        [DateSelection::Single(Some(super::date(2026, 6, 16)))]
    );
    assert_eq!(*picker.events.closed.borrow(), [DialogCloseReason::Confirm]);
}

#[gpui::test]
fn docked_date_anchor_keeps_localized_editor_and_reports_an_accepted_range(
    cx: &mut TestAppContext,
) {
    let picker = Harness::new(
        Example::Triggered(Variant::Docked),
        DateSelection::Range(None),
        cx,
    );
    picker.frame(cx);
    let anchor = picker.node(part("anchor"), cx).unwrap();
    assert_eq!(anchor.label(), Some("Pick dates"));
    picker.click(part("open"), cx);
    assert!(cx.update(|cx| picker.state.read(cx).is_open(cx)));
    let day = |day: i32| {
        (
            ElementId::from("date-probe"),
            format!("day-2026-06-{day:02}"),
        )
    };
    picker.click(day(15), cx);
    assert!(picker.events.selected.borrow().is_empty());
    picker.click(day(16), cx);
    assert_eq!(
        cx.update(|cx| picker.state.read(cx).draft(cx)),
        Draft::Range(Some(super::date(2026, 6, 16)), None)
    );
    picker.click(day(18), cx);
    assert_eq!(*picker.events.closed.borrow(), [DialogCloseReason::Confirm]);
    assert!(!cx.update(|cx| picker.state.read(cx).is_open(cx)));
}

#[gpui::test]
fn editable_docked_anchor_commits_localized_single_text(cx: &mut TestAppContext) {
    let picker = Harness::new(
        Example::Triggered(Variant::Docked),
        DateSelection::Single(None),
        cx,
    );
    picker.click(part("anchor"), cx);
    cx.simulate_input(picker.window, "16/06/2026");
    picker.frame(cx);
    assert_eq!(
        picker.node(part("anchor"), cx).unwrap().value(),
        Some("16/06/2026")
    );
    cx.simulate_keystrokes(picker.window, "enter");
    assert_eq!(
        *picker.events.selected.borrow(),
        [DateSelection::Single(Some(super::date(2026, 6, 16)))]
    );
    assert!(picker.events.closed.borrow().is_empty());
}

#[gpui::test]
fn docked_calendar_rejects_disabled_day_and_restores_focus_after_key_selection(
    cx: &mut TestAppContext,
) {
    use super::time_picker_tests::{click, element, fixture, frame, part as picker_part};
    use crate::story::Story;

    let (window, view) = fixture(Story::DatePicker, cx);
    let opener = picker_part("calendar-picker", "open");
    click(window, opener.clone(), cx);
    let grid = picker_part("calendar-picker", "calendar-grid");
    assert_eq!(element(window, grid.clone(), cx).focused(), Some(true));
    cx.simulate_keystrokes(window, "right enter");
    frame(window, cx);
    cx.update(|cx| {
        assert_eq!(
            view.read(cx).first_date,
            DateSelection::Single(Some(super::date(2026, 11, 14)))
        )
    });
    assert_eq!(element(window, grid, cx).focused(), Some(true));

    cx.simulate_keystrokes(window, "right enter");
    frame(window, cx);
    cx.update(|cx| {
        assert_eq!(
            view.read(cx).first_date,
            DateSelection::Single(Some(super::date(2026, 11, 16)))
        )
    });
    assert_eq!(element(window, opener, cx).focused(), Some(true));
}

#[gpui::test]
fn modal_range_commits_across_months_only_on_confirm(cx: &mut TestAppContext) {
    use super::time_picker_tests::{click, element, fixture, frame, part as picker_part};
    use crate::story::Story;

    let (window, view) = fixture(Story::DateRangePicker, cx);
    let opener = picker_part("calendar-picker", "open");
    click(window, opener.clone(), cx);
    cx.simulate_keystrokes(window, "enter pagedown enter");
    frame(window, cx);
    cx.update(|cx| {
        assert_eq!(
            view.read(cx).first_date,
            DateSelection::Range(Some((super::date(2026, 11, 14), super::date(2026, 11, 17))))
        )
    });
    click(window, "confirm", cx);
    assert_eq!(element(window, opener, cx).focused(), Some(true));
    cx.update(|cx| {
        assert_eq!(
            view.read(cx).first_date,
            DateSelection::Range(Some((super::date(2026, 11, 14), super::date(2026, 12, 14))))
        )
    });
}

#[gpui::test]
fn date_picker_disabled_blocks_trigger_and_docked_enter_and_exposes_trigger_focus(
    cx: &mut TestAppContext,
) {
    struct DisabledDates {
        modal_state: Entity<DatePickerState>,
        docked_state: Entity<DatePickerState>,
        committed: Vec<DateSelection>,
    }

    impl Render for DisabledDates {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity();
            let owner_docked = cx.entity();
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    DatePicker::new(
                        "disabled-modal",
                        "Modal trigger",
                        &self.modal_state,
                        DateSelection::Single(None),
                    )
                    .disabled(true)
                    .on_confirm(move |val, _, cx| {
                        owner.update(cx, |this, _| this.committed.push(val));
                    }),
                )
                .child(
                    DatePicker::new(
                        "disabled-docked",
                        "Docked date",
                        &self.docked_state,
                        DateSelection::Single(None),
                    )
                    .variant(Variant::Docked)
                    .docked_anchor(|field| div().w(px(220.)).child(field))
                    .disabled(true)
                    .on_confirm(move |val, _, cx| {
                        owner_docked.update(cx, |this, _| this.committed.push(val));
                    }),
                )
        }
    }

    let (mut modal_state, mut docked_state) = (None, None);
    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let ms = cx.new(|cx| DatePickerState::new(window, cx));
            let ds = cx.new(|cx| DatePickerState::new(window, cx));
            modal_state = Some(ms.clone());
            docked_state = Some(ds.clone());
            let e = cx.new(|_| DisabledDates {
                modal_state: ms,
                docked_state: ds,
                committed: Vec::new(),
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    let modal_state = modal_state.unwrap();
    let docked_state = docked_state.unwrap();
    let window = handle.into();
    cx.simulate_window_resize(window, size(px(500.), px(420.)));

    cx.update_window(window, |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        let modal_open = find(
            window,
            &[],
            &(ElementId::from("disabled-modal"), "open").into(),
        )
        .unwrap();
        assert_eq!(modal_open.disabled(), Some(true));

        let pos = modal_open.bounds().center();
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position: pos,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            gpui::MouseUpEvent {
                button: gpui::MouseButton::Left,
                position: pos,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(!modal_state.read(cx).is_open(cx));

        // Focusing trigger_focus() does not throw and focuses the docked trigger icon button.
        let trigger_focus = docked_state.read(cx).trigger_focus().clone();
        trigger_focus.focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        let docked_open = find(
            window,
            &[],
            &(ElementId::from("disabled-docked"), "open").into(),
        )
        .unwrap();
        assert_eq!(docked_open.disabled(), Some(true));

        // Pressing Enter inside docked anchor while disabled must NOT commit.
        let anchor_center = find(
            window,
            &[],
            &(ElementId::from("disabled-docked"), "anchor").into(),
        )
        .unwrap()
        .bounds()
        .center();
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position: anchor_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            gpui::MouseUpEvent {
                button: gpui::MouseButton::Left,
                position: anchor_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.simulate_input(window, "16/06/2026");
    cx.simulate_keystrokes(window, "enter");
    cx.update(|cx| assert!(view.read(cx).committed.is_empty()));
}
