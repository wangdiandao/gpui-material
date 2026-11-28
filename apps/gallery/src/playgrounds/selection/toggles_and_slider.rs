use gpui::{AppContext, Axis, ElementId, Entity, IntoElement, ParentElement, Styled, div, px};
use gpui_material::{
    Checkbox, CheckboxState, ControlSize, Radio, RadioGroup, Slider, SliderState, Switch,
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
