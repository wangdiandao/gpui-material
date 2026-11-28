use gpui::{IntoElement, SharedString};
use gpui_material::{Chip, ChipGroup, ChipGroupItem, ChipVariant, Icon, IconName};

use crate::playground::{Control, Playground, Scenario, Section, Set};

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
