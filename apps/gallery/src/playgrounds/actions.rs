use gpui::{IntoElement, SharedString};
use gpui_material::{
    Button, ButtonGroup, ButtonGroupItem, ButtonShape, ButtonVariant, ControlSize, ExtendedFab,
    Fab, FabColor, FabMenu, FabMenuItem, FabSize, Icon, IconButton, IconButtonVariant,
    IconButtonWidth, IconName, IconToggleButton, MenuItem, SemanticTone, SplitButton, ToggleButton,
    ToggleButtonVariant,
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

fn parse_size(value: &str) -> ControlSize {
    match value {
        "XS" => ControlSize::ExtraSmall,
        "S" => ControlSize::Small,
        "L" => ControlSize::Large,
        "XL" => ControlSize::ExtraLarge,
        _ => ControlSize::Medium,
    }
}

fn size_code(value: &str) -> &'static str {
    match value {
        "XS" => "ControlSize::ExtraSmall",
        "S" => "ControlSize::Small",
        "L" => "ControlSize::Large",
        "XL" => "ControlSize::ExtraLarge",
        _ => "ControlSize::Medium",
    }
}

fn parse_button_tone(value: &str) -> SemanticTone {
    match value {
        "Success" => SemanticTone::Success,
        "Warning" => SemanticTone::Warning,
        "Danger" => SemanticTone::Danger,
        "Info" => SemanticTone::Info,
        _ => SemanticTone::Default,
    }
}

fn parse_semantic_tone(value: &str) -> SemanticTone {
    match value {
        "Success" => SemanticTone::Success,
        "Warning" => SemanticTone::Warning,
        "Danger" => SemanticTone::Danger,
        "Info" => SemanticTone::Info,
        _ => SemanticTone::Default,
    }
}

fn parse_fab_size(value: &str) -> FabSize {
    match value {
        "Small" => FabSize::Small,
        "Large" => FabSize::Large,
        _ => FabSize::Medium,
    }
}

fn parse_fab_color(value: &str) -> FabColor {
    match value {
        "Secondary" => FabColor::Secondary,
        "Tertiary" => FabColor::Tertiary,
        _ => FabColor::Primary,
    }
}

fn parse_icon(value: &str) -> IconName {
    match value {
        "Edit" => IconName::Edit,
        "Search" => IconName::Search,
        "Favorite" => IconName::Favorite,
        _ => IconName::Add,
    }
}

// --- Button ---
pub static BUTTON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Tonal + Icon",
            values: &[
                ("variant", Set::Option("Filled tonal")),
                ("icon", Set::Bool(true)),
                ("label", Set::Text("Save draft")),
            ],
        },
        Scenario {
            name: "Danger",
            values: &[
                ("tone", Set::Option("Danger")),
                ("label", Set::Text("Delete project")),
            ],
        },
        Scenario {
            name: "Loading",
            values: &[
                ("loading", Set::Bool(true)),
                ("label", Set::Text("Publishing")),
            ],
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
                    options: &["Filled", "Filled tonal", "Elevated", "Outlined", "Text"],
                    default: 0,
                },
                Control::Choice {
                    id: "tone",
                    label: "Tone",
                    options: &["Default", "Info", "Success", "Warning", "Danger"],
                    default: 0,
                },
                Control::Choice {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 2,
                },
                Control::Choice {
                    id: "shape",
                    label: "Shape",
                    options: &["Round", "Square"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content & state",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Publish",
                },
                Control::Switch {
                    id: "icon",
                    label: "Leading icon",
                    default: false,
                },
                Control::Switch {
                    id: "loading",
                    label: "Loading",
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
    preview: |cfg, ctx, _, cx| {
        let variant = match cfg.option("variant") {
            "Filled tonal" => ButtonVariant::FilledTonal,
            "Elevated" => ButtonVariant::Elevated,
            "Outlined" => ButtonVariant::Outlined,
            "Text" => ButtonVariant::Text,
            _ => ButtonVariant::Filled,
        };
        let shape = if cfg.option("shape") == "Square" {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        };
        let on_status = ctx.on_status(cx);
        let label = cfg.text("label").to_owned();
        let mut btn = Button::new("pg-button", label.clone())
            .variant(variant)
            .tone(parse_button_tone(cfg.option("tone")))
            .size(parse_size(cfg.option("size")))
            .shape(shape)
            .loading(cfg.bool("loading"))
            .disabled(cfg.bool("disabled"))
            .on_activate(move |_, cx| on_status(format!("Activated \"{label}\""), cx));
        if cfg.bool("icon") {
            btn = btn.leading_icon(Icon::new(IconName::Add));
        }
        btn.into_any_element()
    },
    code: |cfg| {
        let variant = match cfg.option("variant") {
            "Filled tonal" => "ButtonVariant::FilledTonal",
            "Elevated" => "ButtonVariant::Elevated",
            "Outlined" => "ButtonVariant::Outlined",
            "Text" => "ButtonVariant::Text",
            _ => "ButtonVariant::Filled",
        };
        let mut s = format!(
            "Button::new(\"publish\", {:?})\n    .variant({variant})\n    .size({})",
            cfg.text("label"),
            size_code(cfg.option("size"))
        );
        if cfg.option("tone") != "Default" {
            s.push_str(&format!(
                "\n    .tone(SemanticTone::{})",
                cfg.option("tone")
            ));
        }
        if cfg.option("shape") == "Square" {
            s.push_str("\n    .shape(ButtonShape::Square)");
        }
        if cfg.bool("icon") {
            s.push_str("\n    .leading_icon(Icon::new(IconName::Add))");
        }
        if cfg.bool("loading") {
            s.push_str("\n    .loading(true)");
        }
        if cfg.bool("disabled") {
            s.push_str("\n    .disabled(true)");
        }
        s.push_str("\n    .on_activate(|_, _| {})");
        s
    },
};

// --- ToggleButton ---
pub static TOGGLE_BUTTON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Selected + Icon",
            values: &[
                ("selected", Set::Bool(true)),
                ("icon", Set::Bool(true)),
                ("label", Set::Text("Pinned")),
            ],
        },
        Scenario {
            name: "Square outlined",
            values: &[
                ("variant", Set::Option("Outlined")),
                ("shape", Set::Option("Square")),
            ],
        },
        Scenario {
            name: "Loading",
            values: &[("loading", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "variant",
                    label: "Variant",
                    options: &["Filled", "Filled tonal", "Elevated", "Outlined"],
                    default: 1,
                },
                Control::Choice {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 2,
                },
                Control::Choice {
                    id: "shape",
                    label: "Shape",
                    options: &["Round", "Square"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content & state",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Favorite",
                },
                Control::Switch {
                    id: "icon",
                    label: "Leading icon",
                    default: true,
                },
                Control::Switch {
                    id: "selected",
                    label: "Selected",
                    default: false,
                },
                Control::Switch {
                    id: "loading",
                    label: "Loading",
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
    preview: |cfg, ctx, _, cx| {
        let variant = match cfg.option("variant") {
            "Filled" => ToggleButtonVariant::Filled,
            "Elevated" => ToggleButtonVariant::Elevated,
            "Outlined" => ToggleButtonVariant::Outlined,
            _ => ToggleButtonVariant::FilledTonal,
        };
        let shape = if cfg.option("shape") == "Square" {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        };
        let selected = cfg.bool("selected");
        let on_change = ctx.on_bool("selected", cx);
        let mut btn = ToggleButton::new("pg-toggle-button", cfg.text("label").to_owned(), selected)
            .variant(variant)
            .size(parse_size(cfg.option("size")))
            .shape(shape)
            .loading(cfg.bool("loading"))
            .disabled(cfg.bool("disabled"))
            .on_change(on_change);
        if cfg.bool("icon") {
            btn = btn.leading_icon(Icon::new(IconName::Favorite).filled(selected));
        }
        btn.into_any_element()
    },
    code: |cfg| {
        let variant = match cfg.option("variant") {
            "Filled" => "ToggleButtonVariant::Filled",
            "Elevated" => "ToggleButtonVariant::Elevated",
            "Outlined" => "ToggleButtonVariant::Outlined",
            _ => "ToggleButtonVariant::FilledTonal",
        };
        let mut s = format!(
            "ToggleButton::new(\"toggle\", {:?}, {})\n    .variant({variant})\n    .size({})",
            cfg.text("label"),
            cfg.bool("selected"),
            size_code(cfg.option("size"))
        );
        if cfg.option("shape") == "Square" {
            s.push_str("\n    .shape(ButtonShape::Square)");
        }
        if cfg.bool("icon") {
            s.push_str("\n    .leading_icon(Icon::new(IconName::Favorite))");
        }
        if cfg.bool("loading") {
            s.push_str("\n    .loading(true)");
        }
        if cfg.bool("disabled") {
            s.push_str("\n    .disabled(true)");
        }
        s.push_str("\n    .on_change(|next, _, _| {})");
        s
    },
};

// --- IconButton ---
pub static ICON_BUTTON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Filled toggle",
            values: &[
                ("variant", Set::Option("Filled")),
                ("toggleable", Set::Bool(true)),
                ("selected", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Wide outlined",
            values: &[
                ("variant", Set::Option("Outlined")),
                ("width", Set::Option("Wide")),
            ],
        },
        Scenario {
            name: "Narrow square",
            values: &[
                ("width", Set::Option("Narrow")),
                ("shape", Set::Option("Square")),
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
                    options: &["Standard", "Filled", "Filled tonal", "Outlined"],
                    default: 2,
                },
                Control::Choice {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 2,
                },
                Control::Choice {
                    id: "width",
                    label: "Width",
                    options: &["Narrow", "Uniform", "Wide"],
                    default: 1,
                },
                Control::Choice {
                    id: "shape",
                    label: "Shape",
                    options: &["Round", "Square"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "toggleable",
                    label: "Toggle button",
                    default: false,
                },
                Control::Switch {
                    id: "selected",
                    label: "Selected",
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
    preview: |cfg, ctx, _, cx| {
        let variant = match cfg.option("variant") {
            "Standard" => IconButtonVariant::Standard,
            "Filled" => IconButtonVariant::Filled,
            "Outlined" => IconButtonVariant::Outlined,
            _ => IconButtonVariant::FilledTonal,
        };
        let width = match cfg.option("width") {
            "Narrow" => IconButtonWidth::Narrow,
            "Wide" => IconButtonWidth::Wide,
            _ => IconButtonWidth::Uniform,
        };
        let shape = if cfg.option("shape") == "Square" {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        };
        let size = parse_size(cfg.option("size"));
        if cfg.bool("toggleable") {
            let selected = cfg.bool("selected");
            let on_change = ctx.on_bool("selected", cx);
            IconToggleButton::new(
                "pg-icon-toggle",
                "Favorite",
                Icon::new(IconName::Favorite).filled(selected),
                selected,
            )
            .variant(variant)
            .size(size)
            .width(width)
            .shape(shape)
            .disabled(cfg.bool("disabled"))
            .on_change(on_change)
            .into_any_element()
        } else {
            let on_status = ctx.on_status(cx);
            IconButton::new("pg-icon-button", "Edit", Icon::new(IconName::Edit))
                .variant(variant)
                .size(size)
                .width(width)
                .shape(shape)
                .disabled(cfg.bool("disabled"))
                .on_activate(move |_, cx| on_status("Icon button clicked".into(), cx))
                .into_any_element()
        }
    },
    code: |cfg| {
        let variant = match cfg.option("variant") {
            "Standard" => "IconButtonVariant::Standard",
            "Filled" => "IconButtonVariant::Filled",
            "Outlined" => "IconButtonVariant::Outlined",
            _ => "IconButtonVariant::FilledTonal",
        };
        if cfg.bool("toggleable") {
            format!(
                "IconToggleButton::new(\"fav\", \"Favorite\", Icon::new(IconName::Favorite), {})\n    .variant({variant})\n    .size({})\n    .width(IconButtonWidth::{})\n    .shape(ButtonShape::{})\n    .disabled({})\n    .on_change(|next, _, _| {{}})",
                cfg.bool("selected"),
                size_code(cfg.option("size")),
                cfg.option("width"),
                cfg.option("shape"),
                cfg.bool("disabled")
            )
        } else {
            format!(
                "IconButton::new(\"edit\", \"Edit\", Icon::new(IconName::Edit))\n    .variant({variant})\n    .size({})\n    .width(IconButtonWidth::{})\n    .shape(ButtonShape::{})\n    .disabled({})\n    .on_activate(|_, _| {{}})",
                size_code(cfg.option("size")),
                cfg.option("width"),
                cfg.option("shape"),
                cfg.bool("disabled")
            )
        }
    },
};

// --- ButtonGroup ---
pub static BUTTON_GROUP: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Connected tonal",
            values: &[
                ("connected", Set::Bool(true)),
                ("variant", Set::Option("Filled tonal")),
            ],
        },
        Scenario {
            name: "Square outlined",
            values: &[
                ("connected", Set::Bool(true)),
                ("variant", Set::Option("Outlined")),
                ("shape", Set::Option("Square")),
            ],
        },
        Scenario {
            name: "Standard row",
            values: &[("connected", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Layout & style",
        controls: &[
            Control::Switch {
                id: "connected",
                label: "Connected",
                default: true,
            },
            Control::Choice {
                id: "variant",
                label: "Appearance",
                options: &["Filled tonal", "Outlined", "Filled"],
                default: 0,
            },
            Control::Choice {
                id: "shape",
                label: "Shape",
                options: &["Round", "Square"],
                default: 0,
            },
            Control::Choice {
                id: "selected",
                label: "Selected",
                options: &["Day", "Week", "Month"],
                default: 1,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let appearance = match cfg.option("variant") {
            "Outlined" => ButtonVariant::Outlined,
            "Filled" => ButtonVariant::Filled,
            _ => ButtonVariant::FilledTonal,
        };
        let shape = if cfg.option("shape") == "Square" {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        };
        let selected: SharedString = cfg.option("selected").to_ascii_lowercase().into();
        let on_choice = ctx.on_choice("selected", cx);
        let items = [
            ButtonGroupItem::new("day", "Day"),
            ButtonGroupItem::new("week", "Week"),
            ButtonGroupItem::new("month", "Month"),
        ];
        let group = if cfg.bool("connected") {
            ButtonGroup::connected("pg-btn-group", "Period", items)
        } else {
            ButtonGroup::new("pg-btn-group", "Period", items)
        };
        group
            .appearance(appearance)
            .shape(shape)
            .single_select(selected, move |next, window, cx| {
                let ix = match next.as_ref() {
                    "day" => 0,
                    "month" => 2,
                    _ => 1,
                };
                on_choice(ix, window, cx);
            })
            .into_any_element()
    },
    code: |cfg| {
        let ctor = if cfg.bool("connected") {
            "ButtonGroup::connected"
        } else {
            "ButtonGroup::new"
        };
        format!(
            "{ctor}(\"period\", \"Period\", [\n    ButtonGroupItem::new(\"day\", \"Day\"),\n    ButtonGroupItem::new(\"week\", \"Week\"),\n    ButtonGroupItem::new(\"month\", \"Month\"),\n])\n.shape(ButtonShape::{})\n.single_select({:?}, |next, _, _| {{}})",
            cfg.option("shape"),
            cfg.option("selected").to_ascii_lowercase()
        )
    },
};

// --- SplitButton ---
pub static SPLIT_BUTTON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Tonal + Icon",
            values: &[
                ("variant", Set::Option("Filled tonal")),
                ("icon", Set::Bool(true)),
                ("label", Set::Text("Export")),
            ],
        },
        Scenario {
            name: "Leading disabled",
            values: &[
                ("disabled", Set::Bool(true)),
                ("trailing_disabled", Set::Bool(false)),
            ],
        },
        Scenario {
            name: "Both disabled",
            values: &[
                ("disabled", Set::Bool(true)),
                ("trailing_disabled", Set::Bool(true)),
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
                    options: &["Filled", "Filled tonal", "Elevated", "Outlined"],
                    default: 0,
                },
                Control::Choice {
                    id: "tone",
                    label: "Tone",
                    options: &["Default", "Info", "Success", "Warning", "Danger"],
                    default: 0,
                },
                Control::Choice {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 2,
                },
            ],
        },
        Section {
            title: "Content & state",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Primary label",
                    default: "Save",
                },
                Control::Switch {
                    id: "icon",
                    label: "Leading icon",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Leading disabled",
                    default: false,
                },
                Control::Switch {
                    id: "trailing_disabled",
                    label: "Trailing disabled",
                    default: false,
                },
            ],
        },
    ],
    preview: |cfg, ctx, _, cx| {
        let variant = match cfg.option("variant") {
            "Filled tonal" => ButtonVariant::FilledTonal,
            "Elevated" => ButtonVariant::Elevated,
            "Outlined" => ButtonVariant::Outlined,
            _ => ButtonVariant::Filled,
        };
        let s1 = ctx.on_status(cx);
        let s2 = ctx.on_status(cx);
        let s3 = ctx.on_status(cx);
        let mut split = SplitButton::new(
            "pg-split-button",
            cfg.text("label").to_owned(),
            "More options",
            [
                MenuItem::new("save-copy", "Save a copy")
                    .on_activate(move |_, cx| s2("Menu: Save a copy".into(), cx)),
                MenuItem::new("export-pdf", "Export PDF")
                    .on_activate(move |_, cx| s3("Menu: Export PDF".into(), cx)),
            ],
        )
        .variant(variant)
        .tone(parse_semantic_tone(cfg.option("tone")))
        .size(parse_size(cfg.option("size")))
        .disabled(cfg.bool("disabled"))
        .trailing_disabled(cfg.bool("trailing_disabled"))
        .on_activate(move |_, cx| s1("Primary split action triggered".into(), cx));
        if cfg.bool("icon") {
            split = split.leading_icon(Icon::new(IconName::Edit));
        }
        split.into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "SplitButton::new(\"save\", {:?}, \"More options\", [\n    MenuItem::new(\"copy\", \"Save a copy\"),\n    MenuItem::new(\"pdf\", \"Export PDF\"),\n])\n.size({})\n.disabled({})\n.trailing_disabled({})",
            cfg.text("label"),
            size_code(cfg.option("size")),
            cfg.bool("disabled"),
            cfg.bool("trailing_disabled")
        );
        if cfg.option("tone") != "Default" {
            s.push_str(&format!("\n.tone(SemanticTone::{})", cfg.option("tone")));
        }
        if cfg.bool("icon") {
            s.push_str("\n.leading_icon(Icon::new(IconName::Edit))");
        }
        s.push_str("\n.on_activate(|_, _| {})");
        s
    },
};

// --- Fab ---
pub static FAB: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Large tertiary",
            values: &[
                ("size", Set::Option("Large")),
                ("color", Set::Option("Tertiary")),
                ("icon", Set::Option("Edit")),
            ],
        },
        Scenario {
            name: "Small secondary",
            values: &[
                ("size", Set::Option("Small")),
                ("color", Set::Option("Secondary")),
                ("icon", Set::Option("Search")),
            ],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "size",
                label: "Size",
                options: &["Small", "Medium", "Large"],
                default: 1,
            },
            Control::Choice {
                id: "color",
                label: "Color",
                options: &["Primary", "Secondary", "Tertiary"],
                default: 0,
            },
            Control::Choice {
                id: "icon",
                label: "Icon",
                options: &["Add", "Edit", "Search", "Favorite"],
                default: 0,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let on_status = ctx.on_status(cx);
        let icon_name = cfg.option("icon");
        Fab::new("pg-fab", "Create", Icon::new(parse_icon(icon_name)))
            .size(parse_fab_size(cfg.option("size")))
            .color(parse_fab_color(cfg.option("color")))
            .on_activate(move |_, cx| on_status(format!("FAB ({icon_name}) activated"), cx))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Fab::new(\"create\", \"Create\", Icon::new(IconName::{}))\n    .size(FabSize::{})\n    .color(FabColor::{})\n    .on_activate(|_, _| {{}})",
            cfg.option("icon"),
            cfg.option("size"),
            cfg.option("color")
        )
    },
};

// --- FabMenu ---
pub static FAB_MENU: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Expanded",
            values: &[("expanded", Set::Bool(true))],
        },
        Scenario {
            name: "Large secondary",
            values: &[
                ("size", Set::Option("Large")),
                ("color", Set::Option("Secondary")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "size",
                    label: "Size",
                    options: &["Small", "Medium", "Large"],
                    default: 1,
                },
                Control::Choice {
                    id: "color",
                    label: "Color",
                    options: &["Primary", "Secondary", "Tertiary"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "State",
            controls: &[Control::Switch {
                id: "expanded",
                label: "Expanded",
                default: false,
            }],
        },
    ],
    preview: |cfg, ctx, _, cx| {
        let on_expanded = ctx.on_bool("expanded", cx);
        let s1 = ctx.on_status(cx);
        let s2 = ctx.on_status(cx);
        let s3 = ctx.on_status(cx);
        FabMenu::new(
            "pg-fab-menu",
            "Create menu",
            IconName::Add,
            [
                FabMenuItem::new("note", "New note", IconName::Edit)
                    .on_activate(move |_, cx| s1("Created new note".into(), cx)),
                FabMenuItem::new("doc", "New document", IconName::Description)
                    .on_activate(move |_, cx| s2("Created new document".into(), cx)),
                FabMenuItem::new("inbox", "Quick capture", IconName::Inbox)
                    .on_activate(move |_, cx| s3("Opened quick capture".into(), cx)),
            ],
        )
        .size(parse_fab_size(cfg.option("size")))
        .color(parse_fab_color(cfg.option("color")))
        .expanded(cfg.bool("expanded"))
        .on_expanded_change(on_expanded)
        .into_any_element()
    },
    code: |cfg| {
        format!(
            "FabMenu::new(\"create\", \"Create menu\", IconName::Add, [\n    FabMenuItem::new(\"note\", \"New note\", IconName::Edit),\n    FabMenuItem::new(\"doc\", \"New document\", IconName::Description),\n    FabMenuItem::new(\"inbox\", \"Quick capture\", IconName::Inbox),\n])\n.size(FabSize::{})\n.color(FabColor::{})\n.expanded({})",
            cfg.option("size"),
            cfg.option("color"),
            cfg.bool("expanded")
        )
    },
};

// --- ExtendedFab ---
pub static EXTENDED_FAB: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Collapsed",
            values: &[("expanded", Set::Bool(false))],
        },
        Scenario {
            name: "Secondary",
            values: &[
                ("color", Set::Option("Secondary")),
                ("icon", Set::Option("Add")),
                ("label", Set::Text("New message")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Choice {
                    id: "color",
                    label: "Color",
                    options: &["Primary", "Secondary", "Tertiary"],
                    default: 0,
                },
                Control::Choice {
                    id: "icon",
                    label: "Icon",
                    options: &["Edit", "Add", "Search"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Compose",
                },
            ],
        },
        Section {
            title: "State",
            controls: &[Control::Switch {
                id: "expanded",
                label: "Expanded",
                default: true,
            }],
        },
    ],
    preview: |cfg, ctx, _, cx| {
        let on_status = ctx.on_status(cx);
        let label = cfg.text("label").to_owned();
        ExtendedFab::new(
            "pg-extended-fab",
            label.clone(),
            Icon::new(parse_icon(cfg.option("icon"))),
        )
        .color(parse_fab_color(cfg.option("color")))
        .expanded(cfg.bool("expanded"))
        .on_activate(move |_, cx| on_status(format!("Extended FAB \"{label}\" clicked"), cx))
        .into_any_element()
    },
    code: |cfg| {
        format!(
            "ExtendedFab::new(\"compose\", {:?}, Icon::new(IconName::{}))\n    .color(FabColor::{})\n    .expanded({})\n    .on_activate(|_, _| {{}})",
            cfg.text("label"),
            cfg.option("icon"),
            cfg.option("color"),
            cfg.bool("expanded")
        )
    },
};
