use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, div, px};
use gpui_material::{
    Button, ButtonVariant, DropdownOption, ExposedDropdown, ExposedDropdownState, Icon, IconName,
    Menu, MenuItem, TextField, TextFieldState, TextFieldVariant,
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

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
