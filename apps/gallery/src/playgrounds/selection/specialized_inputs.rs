use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, div, px};
use gpui_material::{
    ColorPicker, ColorPickerState, NumberInput, NumberInputState, NumberInputStepper, OtpInput,
    OtpInputState, OtpInputVariant, TextFieldVariant, theme::Argb,
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

// --- NumberInput ---
pub static NUMBER_INPUT: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Outlined trailing",
            values: &[
                ("variant", Set::Option("Outlined")),
                ("stepper", Set::Option("Trailing")),
            ],
        },
        Scenario {
            name: "Error state",
            values: &[("error", Set::Bool(true))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "variant",
                    label: "Variant",
                    options: &["Filled", "Outlined"],
                    default: 0,
                },
                Control::Choice {
                    id: "stepper",
                    label: "Stepper layout",
                    options: &["Split", "Trailing", "None"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Quantity",
                },
            ],
        },
        Section {
            title: "State",
            controls: &[
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
    ],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<NumberInputState>>("number-input-state", || {
                cx.new(|cx| {
                    NumberInputState::new(window, cx)
                        .bounds(0.0, 100.0)
                        .step(1.0)
                        .default_value(5.0, window, cx)
                })
            })
            .clone();
        let variant = if cfg.option("variant") == "Outlined" {
            TextFieldVariant::Outlined
        } else {
            TextFieldVariant::Filled
        };
        let stepper = match cfg.option("stepper") {
            "Trailing" => NumberInputStepper::Trailing,
            "None" => NumberInputStepper::None,
            _ => NumberInputStepper::Split,
        };
        let mut input = NumberInput::new("pg-number-input", cfg.text("label").to_owned(), &state)
            .variant(variant)
            .stepper(stepper)
            .disabled(cfg.bool("disabled"));
        if cfg.bool("error") {
            input = input.error("Value out of range");
        } else {
            input = input.supporting_text("Use buttons or Up/Down keys");
        }
        div().w(px(300.)).child(input).into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "let state = cx.new(|cx| {{\n    NumberInputState::new(window, cx)\n        .bounds(0.0, 100.0)\n        .step(1.0)\n        .default_value(5.0, window, cx)\n}});\nNumberInput::new(\"qty\", {:?}, &state)\n    .variant(TextFieldVariant::{})\n    .stepper(NumberInputStepper::{})",
            cfg.text("label"),
            cfg.option("variant"),
            cfg.option("stepper"),
        );
        if cfg.bool("disabled") {
            s.push_str("\n    .disabled(true)");
        }
        if cfg.bool("error") {
            s.push_str("\n    .error(\"Value out of range\")");
        }
        s
    },
};

// --- OtpInput ---
pub static OTP_INPUT: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Filled masked",
            values: &[
                ("variant", Set::Option("Filled")),
                ("masked", Set::Bool(true)),
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
                Control::Switch {
                    id: "masked",
                    label: "Masked",
                    default: false,
                },
                Control::Switch {
                    id: "separator",
                    label: "Middle dash",
                    default: true,
                },
            ],
        },
        Section {
            title: "State",
            controls: &[
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
    ],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<OtpInputState>>("otp-input-state", || {
                cx.new(|cx| OtpInputState::new(6, window, cx).default_value("123", window, cx))
            })
            .clone();
        let variant = if cfg.option("variant") == "Filled" {
            OtpInputVariant::Filled
        } else {
            OtpInputVariant::Outlined
        };
        let mut input = OtpInput::new("pg-otp-input", "Verification code", &state)
            .variant(variant)
            .masked(cfg.bool("masked"))
            .disabled(cfg.bool("disabled"));
        if cfg.bool("separator") {
            input = input.separator_after(3);
        }
        if cfg.bool("error") {
            input = input.error("Invalid verification code");
        } else {
            input = input.supporting_text("Enter the 6-digit code");
        }
        input.into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "let state = cx.new(|cx| OtpInputState::new(6, window, cx));\nOtpInput::new(\"code\", \"Verification code\", &state)\n    .variant(OtpInputVariant::{})\n    .masked({})",
            cfg.option("variant"),
            cfg.bool("masked"),
        );
        if cfg.bool("separator") {
            s.push_str("\n    .separator_after(3)");
        }
        if cfg.bool("disabled") {
            s.push_str("\n    .disabled(true)");
        }
        if cfg.bool("error") {
            s.push_str("\n    .error(\"Invalid verification code\")");
        }
        s
    },
};

// --- ColorPicker ---
pub static COLOR_PICKER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Popover + alpha",
            values: &[("mode", Set::Option("Popover")), ("alpha", Set::Bool(true))],
        },
        Scenario {
            name: "Swatches only",
            values: &[("sliders", Set::Bool(false)), ("hex", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Presentation",
        controls: &[
            Control::Choice {
                id: "mode",
                label: "Mode",
                options: &["Inline", "Popover"],
                default: 0,
            },
            Control::Switch {
                id: "presets",
                label: "Swatches",
                default: true,
            },
            Control::Switch {
                id: "sliders",
                label: "HSLA sliders",
                default: true,
            },
            Control::Switch {
                id: "alpha",
                label: "Alpha slider",
                default: false,
            },
            Control::Switch {
                id: "hex",
                label: "Hex input",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<ColorPickerState>>("color-picker-state", || {
                cx.new(|cx| {
                    ColorPickerState::new(window, cx).default_argb(
                        Argb::from_rgb(0x67, 0x50, 0xA4),
                        window,
                        cx,
                    )
                })
            })
            .clone();
        let picker = if cfg.option("mode") == "Popover" {
            ColorPicker::new("pg-color-picker", "Seed color", &state)
        } else {
            ColorPicker::inline("pg-color-picker", "Seed color", &state)
        };
        picker
            .show_presets(cfg.bool("presets"))
            .show_sliders(cfg.bool("sliders"))
            .show_alpha(cfg.bool("alpha"))
            .show_hex_input(cfg.bool("hex"))
            .into_any_element()
    },
    code: |cfg| {
        let ctor = if cfg.option("mode") == "Popover" {
            "ColorPicker::new"
        } else {
            "ColorPicker::inline"
        };
        format!(
            "let state = cx.new(|cx| {{\n    ColorPickerState::new(window, cx)\n        .default_argb(Argb::from_rgb(0x67, 0x50, 0xA4), window, cx)\n}});\n{ctor}(\"seed\", \"Seed color\", &state)\n    .show_presets({})\n    .show_sliders({})\n    .show_alpha({})\n    .show_hex_input({})",
            cfg.bool("presets"),
            cfg.bool("sliders"),
            cfg.bool("alpha"),
            cfg.bool("hex"),
        )
    },
};
