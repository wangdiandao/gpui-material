use gpui::{
    AppContext, Axis, ElementId, Entity, IntoElement, ParentElement, SharedString, Styled, div, px,
};
use gpui_material::{
    Button, ButtonVariant, Checkbox, CheckboxState, Chip, ChipGroup, ChipGroupItem, ChipVariant,
    ControlSize, DropdownOption, ExposedDropdown, ExposedDropdownState, Icon, IconName, Menu,
    MenuItem, Radio, RadioGroup, Slider, SliderState, StepItem, StepStatus, Stepper,
    StepperOrientation, Switch, TextField, TextFieldState, TextFieldVariant,
    date_time::{
        DatePicker, DatePickerState, DatePickerVariant, DateSelection, NaiveDate, NaiveTime,
        TimeFormat, TimePicker, TimePickerState, TimePickerVariant,
    },
    foundation::{color, window_material_theme},
    theme::ColorRole,
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

// --- Checkbox ---
pub static CHECKBOX: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Indeterminate",
            values: &[("state", Set::Option("Indeterminate"))],
        },
        Scenario {
            name: "Error state",
            values: &[
                ("state", Set::Option("Unchecked")),
                ("error", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "State",
            controls: &[
                Control::Choice {
                    id: "state",
                    label: "State",
                    options: &["Unchecked", "Checked", "Indeterminate"],
                    default: 1,
                },
                Control::Switch {
                    id: "error",
                    label: "Error",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[Control::Text {
                id: "label",
                label: "Label",
                default: "Accept release terms",
            }],
        },
    ],
    preview: |cfg, ctx, window, cx| {
        let theme = window_material_theme(window, cx);
        let state = match cfg.option("state") {
            "Unchecked" => CheckboxState::Unchecked,
            "Indeterminate" => CheckboxState::Indeterminate,
            _ => CheckboxState::Checked,
        };
        let on_choice = ctx.on_choice("state", cx);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded(px(12.))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLow)))
            .child(
                Checkbox::new("pg-checkbox", cfg.text("label").to_owned(), state)
                    .error(cfg.bool("error"))
                    .disabled(cfg.bool("disabled"))
                    .on_change(move |next, window, cx| {
                        let ix = match next {
                            CheckboxState::Unchecked => 0,
                            CheckboxState::Checked => 1,
                            CheckboxState::Indeterminate => 2,
                        };
                        on_choice(ix, window, cx);
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                    .child(format!("Current state: {}", cfg.option("state"))),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Checkbox::new(\"accept\", {:?}, CheckboxState::{})\n    .error({})\n    .disabled({})\n    .on_change(|next, _, _| {{}})",
            cfg.text("label"),
            cfg.option("state"),
            cfg.bool("error"),
            cfg.bool("disabled")
        )
    },
};

// --- Chip ---
pub static CHIP: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Selected filter",
            values: &[
                ("variant", Set::Option("Filter")),
                ("selected", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Elevated assist",
            values: &[
                ("variant", Set::Option("Assist")),
                ("elevated", Set::Bool(true)),
                ("icon", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Input chip",
            values: &[
                ("variant", Set::Option("Input")),
                ("label", Set::Text("design-system")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "variant",
                    label: "Variant",
                    options: &["Assist", "Filter", "Input", "Suggestion"],
                    default: 1,
                },
                Control::Switch {
                    id: "elevated",
                    label: "Elevated",
                    default: false,
                },
                Control::Switch {
                    id: "icon",
                    label: "Leading icon",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content & state",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Material 3",
                },
                Control::Switch {
                    id: "selected",
                    label: "Selected",
                    default: true,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
            ],
        },
    ],
    preview: |cfg, ctx, _, cx| {
        let variant = match cfg.option("variant") {
            "Assist" => ChipVariant::Assist,
            "Input" => ChipVariant::Input,
            "Suggestion" => ChipVariant::Suggestion,
            _ => ChipVariant::Filter,
        };
        let selected = cfg.bool("selected");
        let on_change = ctx.on_bool("selected", cx);
        let on_activate = ctx.on_bool("selected", cx);
        let mut chip = Chip::new("pg-chip", cfg.text("label").to_owned())
            .variant(variant)
            .elevated(cfg.bool("elevated"))
            .selected(selected)
            .disabled(cfg.bool("disabled"))
            .on_change(on_change)
            .on_activate(move |window, cx| on_activate(!selected, window, cx));
        if cfg.bool("icon") {
            chip = chip.leading_icon(Icon::new(IconName::Favorite));
        }
        chip.into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "Chip::new(\"tag\", {:?})\n    .variant(ChipVariant::{})\n    .elevated({})\n    .selected({})\n    .disabled({})",
            cfg.text("label"),
            cfg.option("variant"),
            cfg.bool("elevated"),
            cfg.bool("selected"),
            cfg.bool("disabled")
        );
        if cfg.bool("icon") {
            s.push_str("\n    .leading_icon(Icon::new(IconName::Favorite))");
        }
        s
    },
};

// --- ChipGroup ---
pub static CHIP_GROUP: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Elevated",
            values: &[("elevated", Set::Bool(true))],
        },
        Scenario {
            name: "Tonal",
            values: &[("tonal", Set::Bool(true))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Options",
        controls: &[
            Control::Choice {
                id: "selected",
                label: "Selected chip",
                options: &["All", "Unread", "Starred", "Archived"],
                default: 0,
            },
            Control::Switch {
                id: "elevated",
                label: "Elevated",
                default: false,
            },
            Control::Switch {
                id: "tonal",
                label: "Tonal",
                default: false,
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let selected: SharedString = cfg.option("selected").to_ascii_lowercase().into();
        let on_choice = ctx.on_choice("selected", cx);
        ChipGroup::new(
            "pg-chip-group",
            "Filter messages",
            [
                ChipGroupItem::new("all", "All"),
                ChipGroupItem::new("unread", "Unread"),
                ChipGroupItem::new("starred", "Starred"),
                ChipGroupItem::new("archived", "Archived"),
            ],
        )
        .elevated(cfg.bool("elevated"))
        .tonal(cfg.bool("tonal"))
        .disabled(cfg.bool("disabled"))
        .single_select(selected, move |next, window, cx| {
            let ix = match next.as_ref() {
                "unread" => 1,
                "starred" => 2,
                "archived" => 3,
                _ => 0,
            };
            on_choice(ix, window, cx);
        })
        .into_any_element()
    },
    code: |cfg| {
        format!(
            "ChipGroup::new(\"filters\", \"Filter messages\", [\n    ChipGroupItem::new(\"all\", \"All\"),\n    ChipGroupItem::new(\"unread\", \"Unread\"),\n    ChipGroupItem::new(\"starred\", \"Starred\"),\n    ChipGroupItem::new(\"archived\", \"Archived\"),\n])\n.elevated({})\n.tonal({})\n.disabled({})",
            cfg.bool("elevated"),
            cfg.bool("tonal"),
            cfg.bool("disabled")
        )
    },
};

// --- Radio ---
pub static RADIO: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Horizontal",
            values: &[("layout", Set::Option("Horizontal"))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Options",
        controls: &[
            Control::Choice {
                id: "selected",
                label: "Selected",
                options: &["Compact", "Comfortable", "Spacious"],
                default: 1,
            },
            Control::Choice {
                id: "layout",
                label: "Direction",
                options: &["Vertical", "Horizontal"],
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
        let theme = window_material_theme(window, cx);
        let selected = ElementId::from(match cfg.option("selected") {
            "Compact" => "compact",
            "Spacious" => "spacious",
            _ => "comfortable",
        });
        let axis = if cfg.option("layout") == "Horizontal" {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };
        let disabled = cfg.bool("disabled");
        let on_choice = ctx.on_choice("selected", cx);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded(px(12.))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLow)))
            .child(
                div()
                    .text_xs()
                    .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                    .child("Workspace density"),
            )
            .child(
                RadioGroup::new("pg-radio-group", "Density")
                    .axis(axis)
                    .selected(selected)
                    .options([
                        Radio::new("compact", "Compact").disabled(disabled),
                        Radio::new("comfortable", "Comfortable").disabled(disabled),
                        Radio::new("spacious", "Spacious").disabled(disabled),
                    ])
                    .on_select(move |next, window, cx| {
                        let ix = if *next == ElementId::from("compact") {
                            0
                        } else if *next == ElementId::from("spacious") {
                            2
                        } else {
                            1
                        };
                        on_choice(ix, window, cx);
                    }),
            )
            .into_any_element()
    },
    code: |cfg| {
        let axis = if cfg.option("layout") == "Horizontal" {
            "Axis::Horizontal"
        } else {
            "Axis::Vertical"
        };
        format!(
            "RadioGroup::new(\"density\", \"Density\")\n    .axis({axis})\n    .selected({:?})\n    .options([\n        Radio::new(\"compact\", \"Compact\"),\n        Radio::new(\"comfortable\", \"Comfortable\"),\n        Radio::new(\"spacious\", \"Spacious\"),\n    ])",
            cfg.option("selected").to_ascii_lowercase()
        )
    },
};

// --- Slider ---
pub static SLIDER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Large",
            values: &[("size", Set::Option("L")), ("value", Set::Number(75.))],
        },
        Scenario {
            name: "Extra small",
            values: &[("size", Set::Option("XS")), ("value", Set::Number(25.))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Configuration",
        controls: &[
            Control::Choice {
                id: "size",
                label: "Size",
                options: &["XS", "S", "M", "L", "XL"],
                default: 2,
            },
            Control::Slider {
                id: "value",
                label: "Value",
                min: 0.,
                max: 100.,
                step: 5.,
                default: 40.,
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let size = match cfg.option("size") {
            "XS" => ControlSize::ExtraSmall,
            "S" => ControlSize::Small,
            "L" => ControlSize::Large,
            "XL" => ControlSize::ExtraLarge,
            _ => ControlSize::Medium,
        };
        let target = cfg.number("value");
        let state = ctx
            .local::<Entity<SliderState>>("slider-state", || {
                cx.new(|_| SliderState::new().max(100.).step(5.).default_value(target))
            })
            .clone();
        state.update(cx, |s, cx| {
            if (s.value().end() - target).abs() > 1e-4 {
                s.set_value(target, window, cx);
            }
        });
        let live = state.read(cx).value().end();
        let theme = window_material_theme(window, cx);
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_sm().child("Audio volume"))
                    .child(
                        div()
                            .px_2()
                            .py(px(2.))
                            .rounded_full()
                            .bg(color(theme.colors.color(ColorRole::SecondaryContainer)))
                            .text_xs()
                            .text_color(color(theme.colors.color(ColorRole::OnSecondaryContainer)))
                            .child(format!("{live:.0}%")),
                    ),
            )
            .child(
                Slider::new("pg-slider", "Volume", &state)
                    .size(size)
                    .disabled(cfg.bool("disabled")),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "let state = cx.new(|_| SliderState::new().max(100.).step(5.).default_value({:.0}.));\nSlider::new(\"volume\", \"Volume\", &state)\n    .disabled({})",
            cfg.number("value"),
            cfg.bool("disabled")
        )
    },
};

// --- Switch ---
pub static SWITCH: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Off",
            values: &[("checked", Set::Bool(false))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "State & label",
        controls: &[
            Control::Text {
                id: "label",
                label: "Label",
                default: "Desktop notifications",
            },
            Control::Switch {
                id: "checked",
                label: "Checked",
                default: true,
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let theme = window_material_theme(window, cx);
        let checked = cfg.bool("checked");
        let on_change = ctx.on_bool("checked", cx);
        div()
            .w(px(320.))
            .p_4()
            .rounded(px(12.))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLow)))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_sm().child(cfg.text("label").to_owned()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                            .child(if checked { "Enabled" } else { "Muted" }),
                    ),
            )
            .child(
                Switch::new("pg-switch", cfg.text("label").to_owned(), checked)
                    .show_label(false)
                    .disabled(cfg.bool("disabled"))
                    .on_change(on_change),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Switch::new(\"notifications\", {:?}, {})\n    .disabled({})\n    .on_change(|checked, _, _| {{}})",
            cfg.text("label"),
            cfg.bool("checked"),
            cfg.bool("disabled")
        )
    },
};

// --- TextField ---
pub static TEXT_FIELD: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Filled + Icon",
            values: &[
                ("variant", Set::Option("Filled")),
                ("leading_icon", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Password",
            values: &[
                ("label", Set::Text("Password")),
                ("password", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Error state",
            values: &[("error", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "variant",
                    label: "Variant",
                    options: &["Outlined", "Filled"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Email address",
                },
                Control::Switch {
                    id: "leading_icon",
                    label: "Leading icon",
                    default: false,
                },
            ],
        },
        Section {
            title: "Behavior & state",
            controls: &[
                Control::Switch {
                    id: "password",
                    label: "Password toggle",
                    default: false,
                },
                Control::Switch {
                    id: "clearable",
                    label: "Clear button",
                    default: true,
                },
                Control::Switch {
                    id: "error",
                    label: "Error",
                    default: false,
                },
            ],
        },
    ],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<TextFieldState>>("text-field-state", || {
                cx.new(|cx| TextFieldState::new(window, cx))
            })
            .clone();
        let variant = if cfg.option("variant") == "Filled" {
            TextFieldVariant::Filled
        } else {
            TextFieldVariant::Outlined
        };
        let mut field = TextField::new("pg-text-field", cfg.text("label").to_owned(), &state)
            .variant(variant)
            .clearable(cfg.bool("clearable"));
        if cfg.bool("leading_icon") {
            field = field.leading(Icon::new(IconName::Search));
        }
        if cfg.bool("password") {
            let visible = *ctx.local::<bool>("pw-visible", || false);
            let owner = cx.weak_entity();
            field = field.visibility_toggle(visible, move |next, _, cx| {
                let _ = owner.update(cx, |_, cx| {
                    let _ = next;
                    cx.notify();
                });
            });
        }
        if cfg.bool("error") {
            field = field.error("Enter a valid value");
        } else {
            field = field.supporting_text("We only use this for account recovery.");
        }
        div().w(px(320.)).child(field).into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "let state = cx.new(|cx| TextFieldState::new(window, cx));\nTextField::new(\"field\", {:?}, &state)\n    .variant(TextFieldVariant::{})\n    .clearable({})",
            cfg.text("label"),
            cfg.option("variant"),
            cfg.bool("clearable")
        );
        if cfg.bool("leading_icon") {
            s.push_str("\n    .leading(Icon::new(IconName::Search))");
        }
        if cfg.bool("error") {
            s.push_str("\n    .error(\"Enter a valid value\")");
        }
        s
    },
};

// --- ExposedDropdown ---
pub static EXPOSED_DROPDOWN: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Custom label",
            values: &[("label", Set::Text("Deployment region"))],
        },
        Scenario {
            name: "Editable filter",
            values: &[("editable", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Configuration",
        controls: &[
            Control::Text {
                id: "label",
                label: "Field label",
                default: "Region",
            },
            Control::Switch {
                id: "editable",
                label: "Editable filter",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let editable = cfg.bool("editable");
        let state = ctx
            .local::<Entity<ExposedDropdownState>>("dropdown-state", || {
                cx.new(|cx| ExposedDropdownState::new("US West", editable, window, cx))
            })
            .clone();
        let on_status = ctx.on_status(cx);
        div()
            .w(px(300.))
            .child(
                ExposedDropdown::new(
                    "pg-dropdown",
                    cfg.text("label").to_owned(),
                    &state,
                    [
                        DropdownOption::new("us-west", "US West (Oregon)"),
                        DropdownOption::new("eu-central", "EU Central (Frankfurt)"),
                        DropdownOption::new("ap-east", "Asia Pacific (Tokyo)"),
                        DropdownOption::new("sa-east", "South America (Sao Paulo)"),
                    ],
                )
                .on_select(move |choice, _, cx| {
                    on_status(format!("Selected: {}", choice.label), cx);
                }),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "let state = cx.new(|cx| ExposedDropdownState::new(\"US West\", {}, window, cx));\nExposedDropdown::new(\"region\", {:?}, &state, [\n    DropdownOption::new(\"us-west\", \"US West (Oregon)\"),\n    DropdownOption::new(\"eu-central\", \"EU Central (Frankfurt)\"),\n    DropdownOption::new(\"ap-east\", \"Asia Pacific (Tokyo)\"),\n])",
            cfg.bool("editable"),
            cfg.text("label")
        )
    },
};

// --- Menu ---
pub static MENU: Playground = Playground {
    scenarios: &[Scenario {
        name: "Disabled",
        values: &[("disabled", Set::Bool(true))],
    }],
    sections: &[Section {
        title: "State",
        controls: &[
            Control::Text {
                id: "label",
                label: "Trigger label",
                default: "Document actions",
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let s1 = ctx.on_status(cx);
        let s2 = ctx.on_status(cx);
        let s3 = ctx.on_status(cx);
        Menu::new(
            "pg-menu",
            Button::new("pg-menu-trigger", cfg.text("label").to_owned())
                .variant(ButtonVariant::FilledTonal)
                .trailing_icon(Icon::new(IconName::ArrowDropDown)),
            [
                MenuItem::section(
                    "edit-group",
                    "Edit",
                    [
                        MenuItem::new("rename", "Rename")
                            .leading(|| Icon::new(IconName::Edit))
                            .shortcut("F2")
                            .on_activate(move |_, cx| s1("Menu: Rename".into(), cx)),
                        MenuItem::new("archive", "Archive")
                            .leading(|| Icon::new(IconName::Archive))
                            .shortcut("Ctrl+E")
                            .on_activate(move |_, cx| s2("Menu: Archive".into(), cx)),
                    ],
                ),
                MenuItem::divider("sep"),
                MenuItem::new("delete", "Delete document")
                    .destructive()
                    .on_activate(move |_, cx| s3("Menu: Delete document".into(), cx)),
            ],
        )
        .disabled(cfg.bool("disabled"))
        .into_any_element()
    },
    code: |cfg| {
        format!(
            "Menu::new(\"actions\", Button::new(\"trigger\", {:?}), [\n    MenuItem::new(\"rename\", \"Rename\").shortcut(\"F2\"),\n    MenuItem::new(\"archive\", \"Archive\").shortcut(\"Ctrl+E\"),\n    MenuItem::divider(\"sep\"),\n    MenuItem::new(\"delete\", \"Delete document\").destructive(),\n])\n.disabled({})",
            cfg.text("label"),
            cfg.bool("disabled")
        )
    },
};

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
            DateSelection::Single(NaiveDate::from_ymd_opt(2026, 11, 22)),
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
        let range =
            NaiveDate::from_ymd_opt(2026, 11, 20).zip(NaiveDate::from_ymd_opt(2026, 11, 25));
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
            NaiveTime::from_hms_opt(9, 30, 0).expect("valid time"),
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
