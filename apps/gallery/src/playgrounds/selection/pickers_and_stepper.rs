use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, div};
use gpui_material::{
    StepItem, StepStatus, Stepper, StepperOrientation,
    date_time::{
        Date, DatePicker, DatePickerState, DatePickerVariant, DateSelection, Time, TimeFormat,
        TimePicker, TimePickerState, TimePickerVariant,
    },
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

// --- DatePicker ---
pub static DATE_PICKER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Modal input",
            values: &[("variant", Set::Option("Modal input"))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Presentation",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Modal", "Modal input"],
                default: 0,
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<DatePickerState>>("date-picker-state", || {
                cx.new(|cx| DatePickerState::new(window, cx))
            })
            .clone();
        let variant = if cfg.option("variant") == "Modal input" {
            DatePickerVariant::ModalInput
        } else {
            DatePickerVariant::Modal
        };
        let on_status = ctx.on_status(cx);
        DatePicker::new(
            "pg-date-picker",
            "Choose release date",
            &state,
            DateSelection::Single(Date::new(2026, 11, 22).ok()),
        )
        .variant(variant)
        .disabled(cfg.bool("disabled"))
        .on_confirm(move |sel, _, cx| on_status(format!("Confirmed: {sel:?}"), cx))
        .into_any_element()
    },
    code: |cfg| {
        let variant = if cfg.option("variant") == "Modal input" {
            "DatePickerVariant::ModalInput"
        } else {
            "DatePickerVariant::Modal"
        };
        format!(
            "DatePicker::new(\"date\", \"Choose release date\", &state, DateSelection::Single(date))\n    .variant({variant})\n    .disabled({})",
            cfg.bool("disabled")
        )
    },
};

// --- DateRangePicker ---
pub static DATE_RANGE_PICKER: Playground = Playground {
    scenarios: &[Scenario {
        name: "Disabled",
        values: &[("disabled", Set::Bool(true))],
    }],
    sections: &[Section {
        title: "Presentation",
        controls: &[Control::Switch {
            id: "disabled",
            label: "Disabled",
            default: false,
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<DatePickerState>>("date-range-state", || {
                cx.new(|cx| DatePickerState::new(window, cx))
            })
            .clone();
        let on_status = ctx.on_status(cx);
        let range = Date::new(2026, 11, 20)
            .ok()
            .zip(Date::new(2026, 11, 25).ok());
        DatePicker::new(
            "pg-date-range",
            "Choose sprint dates",
            &state,
            DateSelection::Range(range),
        )
        .variant(DatePickerVariant::Modal)
        .disabled(cfg.bool("disabled"))
        .on_confirm(move |sel, _, cx| on_status(format!("Range: {sel:?}"), cx))
        .into_any_element()
    },
    code: |cfg| {
        format!(
            "DatePicker::new(\"range\", \"Choose sprint dates\", &state, DateSelection::Range(Some((start, end))))\n    .variant(DatePickerVariant::Modal)\n    .disabled({})",
            cfg.bool("disabled")
        )
    },
};

// --- TimePicker ---
pub static TIME_PICKER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "12-hour input",
            values: &[
                ("format", Set::Option("12-hour")),
                ("variant", Set::Option("Input")),
            ],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Presentation",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Dial", "Input"],
                default: 0,
            },
            Control::Choice {
                id: "format",
                label: "Format",
                options: &["24-hour", "12-hour"],
                default: 0,
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<TimePickerState>>("time-picker-state", || {
                cx.new(|cx| TimePickerState::new(window, cx))
            })
            .clone();
        let variant = if cfg.option("variant") == "Input" {
            TimePickerVariant::Input
        } else {
            TimePickerVariant::Dial
        };
        let format = if cfg.option("format") == "12-hour" {
            TimeFormat::TwelveHour
        } else {
            TimeFormat::TwentyFourHour
        };
        let on_status = ctx.on_status(cx);
        TimePicker::new(
            "pg-time-picker",
            "Schedule reminder",
            &state,
            Time::new(9, 30, 0, 0).expect("valid time"),
        )
        .variant(variant)
        .format(format)
        .disabled(cfg.bool("disabled"))
        .on_confirm(move |time, _, cx| on_status(format!("Time: {time}"), cx))
        .into_any_element()
    },
    code: |cfg| {
        let format = if cfg.option("format") == "12-hour" {
            "TimeFormat::TwelveHour"
        } else {
            "TimeFormat::TwentyFourHour"
        };
        format!(
            "TimePicker::new(\"time\", \"Schedule reminder\", &state, time)\n    .variant(TimePickerVariant::{})\n    .format({format})\n    .disabled({})",
            cfg.option("variant"),
            cfg.bool("disabled")
        )
    },
};

// --- Stepper ---
pub static STEPPER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Vertical",
            values: &[
                ("axis", Set::Option("Vertical")),
                ("step", Set::Option("Profile")),
            ],
        },
        Scenario {
            name: "Error on step 2",
            values: &[
                ("step_error", Set::Bool(true)),
                ("step", Set::Option("Profile")),
            ],
        },
    ],
    sections: &[Section {
        title: "Configuration",
        controls: &[
            Control::Choice {
                id: "step",
                label: "Active step",
                options: &["Account", "Profile", "Review"],
                default: 0,
            },
            Control::Choice {
                id: "axis",
                label: "Orientation",
                options: &["Horizontal", "Vertical"],
                default: 0,
            },
            Control::Switch {
                id: "step_error",
                label: "Flag step 2 error",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let active = match cfg.option("step") {
            "Profile" => 1,
            "Review" => 2,
            _ => 0,
        };
        let orientation = if cfg.option("axis") == "Vertical" {
            StepperOrientation::Vertical
        } else {
            StepperOrientation::Horizontal
        };
        let on_choice = ctx.on_choice("step", cx);
        let mut step2 = StepItem::new("profile", "Profile").supporting("Team details");
        if cfg.bool("step_error") {
            step2 = step2.status(StepStatus::Error);
        }
        div()
            .w_full()
            .child(
                Stepper::new(
                    "pg-stepper",
                    [
                        StepItem::new("account", "Account").supporting("Credentials"),
                        step2,
                        StepItem::new("review", "Review").supporting("Confirm setup"),
                    ],
                )
                .orientation(orientation)
                .active_step(active)
                .on_select(move |ix, window, cx| on_choice(ix, window, cx)),
            )
            .into_any_element()
    },
    code: |cfg| {
        let active = match cfg.option("step") {
            "Profile" => 1,
            "Review" => 2,
            _ => 0,
        };
        let orientation = if cfg.option("axis") == "Vertical" {
            "StepperOrientation::Vertical"
        } else {
            "StepperOrientation::Horizontal"
        };
        format!(
            "Stepper::new(\"setup\", [\n    StepItem::new(\"account\", \"Account\").supporting(\"Credentials\"),\n    StepItem::new(\"profile\", \"Profile\").supporting(\"Team details\"),\n    StepItem::new(\"review\", \"Review\").supporting(\"Confirm setup\"),\n])\n.orientation({orientation})\n.active_step({active})"
        )
    },
};
