use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, div, px};
use gpui_material::{
    Button, ButtonVariant, Card, CardVariant, Carousel, CarouselItem, CarouselVariant, ControlSize,
    Dialog, DialogState, Divider, DropTarget, Icon, IconName, List, ListItem, SemanticTone, Sheet,
    SheetEdge, Surface, Text, VerticalDragHandle,
    data_views::{
        ColumnAlignment, DataTable, DataTableCell, DataTableColumn, DataTableRow, SortDirection,
    },
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerRole, Direction, Elevation, TypeRole},
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

// --- Card ---
pub static CARD: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Elevated + Media",
            values: &[
                ("variant", Set::Option("Elevated")),
                ("media", Set::Bool(true)),
                ("actions", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Outlined minimal",
            values: &[
                ("variant", Set::Option("Outlined")),
                ("media", Set::Bool(false)),
            ],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Filled", "Elevated", "Outlined"],
                default: 0,
            },
            Control::Text {
                id: "title",
                label: "Headline",
                default: "Release checklist",
            },
            Control::Switch {
                id: "media",
                label: "Show cover header",
                default: true,
            },
            Control::Switch {
                id: "actions",
                label: "Show actions",
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
        let variant = match cfg.option("variant") {
            "Elevated" => CardVariant::Elevated,
            "Outlined" => CardVariant::Outlined,
            _ => CardVariant::Filled,
        };
        let on_status = ctx.on_status(cx);
        let on_action = ctx.on_status(cx);
        let title = cfg.text("title").to_owned();
        let mut card = Card::new("pg-card")
            .variant(variant)
            .disabled(cfg.bool("disabled"))
            .accessible_label("Open checklist")
            .on_activate(move |_, cx| on_status(format!("Opened card \"{title}\""), cx));
        if cfg.bool("media") {
            card = card.child(
                div()
                    .w_full()
                    .h(px(88.))
                    .rounded(px(12.))
                    .bg(color(theme.colors.color(ColorRole::PrimaryContainer)))
                    .p_3()
                    .flex()
                    .items_end()
                    .justify_between()
                    .child(
                        Icon::new(IconName::Description)
                            .size(px(28.))
                            .color(ColorRole::OnPrimaryContainer),
                    )
                    .child(
                        div()
                            .px_2()
                            .py(px(2.))
                            .rounded_full()
                            .bg(color(theme.colors.color(ColorRole::Primary)))
                            .text_xs()
                            .text_color(color(theme.colors.color(ColorRole::OnPrimary)))
                            .child("v0.3"),
                    ),
            );
        }
        card = card
            .child(
                Text::new(cfg.text("title").to_owned())
                    .id("pg-card-title")
                    .role(TypeRole::TitleMedium),
            )
            .child(
                Text::new("Verify desktop layouts, keyboard focus, and release artifacts.")
                    .id("pg-card-body")
                    .role(TypeRole::BodySmall)
                    .color(ColorRole::OnSurfaceVariant)
                    .wrap(),
            );
        if cfg.bool("actions") {
            card = card.child(
                div().pt_2().flex().justify_end().gap_2().child(
                    Button::new("pg-card-open", "Inspect")
                        .variant(ButtonVariant::FilledTonal)
                        .size(ControlSize::ExtraSmall)
                        .disabled(cfg.bool("disabled"))
                        .on_activate(move |_, cx| on_action("Card action: Inspect".into(), cx)),
                ),
            );
        }
        div().w(px(320.)).child(card).into_any_element()
    },
    code: |cfg| {
        format!(
            "Card::new(\"checklist\")\n    .variant(CardVariant::{})\n    .disabled({})\n    .child(Text::new({:?}).role(TypeRole::TitleMedium))",
            cfg.option("variant"),
            cfg.bool("disabled"),
            cfg.text("title")
        )
    },
};

// --- Carousel ---
pub static CAROUSEL: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Uncontained",
            values: &[("variant", Set::Option("Uncontained"))],
        },
        Scenario {
            name: "Hero",
            values: &[("variant", Set::Option("Hero"))],
        },
        Scenario {
            name: "Centered hero",
            values: &[("variant", Set::Option("Centered hero"))],
        },
    ],
    sections: &[Section {
        title: "Layout",
        controls: &[Control::Choice {
            id: "variant",
            label: "Variant",
            options: &["Multi-browse", "Uncontained", "Hero", "Centered hero"],
            default: 0,
        }],
    }],
    preview: |cfg, _, window, cx| {
        let theme = window_material_theme(window, cx);
        let variant = match cfg.option("variant") {
            "Uncontained" => CarouselVariant::Uncontained,
            "Hero" => CarouselVariant::Hero,
            "Centered hero" => CarouselVariant::CenteredHero,
            _ => CarouselVariant::MultiBrowse,
        };
        let roles = [
            (
                "Kyoto",
                "Autumn temples",
                ColorRole::PrimaryContainer,
                ColorRole::OnPrimaryContainer,
            ),
            (
                "Zurich",
                "Alpine lakes",
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
            (
                "Reykjavik",
                "Northern lights",
                ColorRole::TertiaryContainer,
                ColorRole::OnTertiaryContainer,
            ),
            (
                "Lisbon",
                "Coastal trams",
                ColorRole::PrimaryContainer,
                ColorRole::OnPrimaryContainer,
            ),
            (
                "Oslo",
                "Fjord harbors",
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
        ];
        let items = roles.into_iter().map(|(name, sub, bg, fg)| {
            CarouselItem::new(
                name,
                name,
                div()
                    .size_full()
                    .bg(color(theme.colors.color(bg)))
                    .p_4()
                    .flex()
                    .flex_col()
                    .justify_end()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(color(theme.colors.color(fg)))
                            .child(name),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(color(theme.colors.color(fg)))
                            .child(sub),
                    ),
            )
        });
        div()
            .w_full()
            .h(px(196.))
            .child(Carousel::new("pg-carousel", "Featured destinations", items).variant(variant))
            .into_any_element()
    },
    code: |cfg| {
        let variant = match cfg.option("variant") {
            "Uncontained" => "CarouselVariant::Uncontained",
            "Hero" => "CarouselVariant::Hero",
            "Centered hero" => "CarouselVariant::CenteredHero",
            _ => "CarouselVariant::MultiBrowse",
        };
        format!(
            "Carousel::new(\"places\", \"Featured destinations\", items)\n    .variant({variant})"
        )
    },
};

// --- Dialog ---
pub static DIALOG: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Destructive",
            values: &[
                ("title", Set::Text("Delete project?")),
                ("confirm", Set::Text("Delete")),
                ("destructive", Set::Bool(true)),
                ("icon", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "With icon",
            values: &[("icon", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Content & tone",
        controls: &[
            Control::Text {
                id: "title",
                label: "Title",
                default: "Rename workspace",
            },
            Control::Text {
                id: "confirm",
                label: "Confirm label",
                default: "Confirm",
            },
            Control::Switch {
                id: "icon",
                label: "Show header icon",
                default: false,
            },
            Control::Switch {
                id: "destructive",
                label: "Danger confirm tone",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let state = ctx
            .local::<Entity<DialogState>>("dialog-state", || cx.new(DialogState::new))
            .clone();
        let open_state = state.clone();
        let on_confirm = ctx.on_status(cx);
        let mut dialog = Dialog::new("pg-dialog", &state, cfg.text("title").to_owned())
            .description("Changes apply immediately to all team members.")
            .confirm_tone(if cfg.bool("destructive") {
                SemanticTone::Danger
            } else {
                SemanticTone::Default
            })
            .confirm(cfg.text("confirm").to_owned(), move |_, cx| {
                on_confirm("Dialog confirmed".into(), cx);
                true
            });
        if cfg.bool("icon") {
            dialog = dialog.icon(Icon::new(IconName::Info).size(px(24.)));
        }
        div()
            .child(
                Button::new("pg-open-dialog", "Open dialog")
                    .variant(ButtonVariant::Filled)
                    .on_activate(move |window, cx| {
                        open_state.update(cx, |s, cx| s.open(window, cx));
                    }),
            )
            .child(dialog)
            .into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "Dialog::new(\"workspace\", &state, {:?})\n    .description(\"Changes apply immediately.\")\n    .confirm({:?}, |_, _| true)",
            cfg.text("title"),
            cfg.text("confirm")
        );
        if cfg.bool("icon") {
            s.push_str("\n    .icon(Icon::new(IconName::Info).size(px(24.)))");
        }
        if cfg.bool("destructive") {
            s.push_str("\n    .confirm_tone(SemanticTone::Danger)");
        }
        s
    },
};

// --- Divider ---
pub static DIVIDER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Inset horizontal",
            values: &[
                ("orientation", Set::Option("Horizontal")),
                ("inset", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Vertical",
            values: &[("orientation", Set::Option("Vertical"))],
        },
    ],
    sections: &[Section {
        title: "Style",
        controls: &[
            Control::Choice {
                id: "orientation",
                label: "Orientation",
                options: &["Horizontal", "Vertical"],
                default: 0,
            },
            Control::Switch {
                id: "inset",
                label: "Inset (16 dp)",
                default: false,
            },
        ],
    }],
    preview: |cfg, _, window, cx| {
        let theme = window_material_theme(window, cx);
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let inset = cfg.bool("inset");
        if cfg.option("orientation") == "Vertical" {
            let mut div_el = Divider::vertical();
            if inset {
                div_el = div_el.inset(px(16.), px(16.));
            }
            return div()
                .h(px(96.))
                .px_4()
                .rounded(px(12.))
                .border_1()
                .border_color(outline)
                .bg(card_bg)
                .flex()
                .items_center()
                .gap_4()
                .child(div().text_sm().child("Leading pane"))
                .child(div_el)
                .child(div().text_sm().child("Trailing pane"))
                .into_any_element();
        }
        let make = || {
            let d = Divider::horizontal();
            if inset { d.inset(px(16.), px(16.)) } else { d }
        };
        div()
            .w(px(320.))
            .rounded(px(12.))
            .border_1()
            .border_color(outline)
            .bg(card_bg)
            .flex()
            .flex_col()
            .child(div().px_4().py_3().text_sm().child("Profile & account"))
            .child(make())
            .child(div().px_4().py_3().text_sm().child("Notifications"))
            .child(make())
            .child(div().px_4().py_3().text_sm().child("Security & sessions"))
            .into_any_element()
    },
    code: |cfg| {
        let ctor = if cfg.option("orientation") == "Vertical" {
            "Divider::vertical()"
        } else {
            "Divider::horizontal()"
        };
        if cfg.bool("inset") {
            format!("{ctor}\n    .inset(px(16.), px(16.))")
        } else {
            ctor.into()
        }
    },
};

// --- List ---
pub static LIST: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Compact (no supporting)",
            values: &[("supporting", Set::Bool(false))],
        },
        Scenario {
            name: "With trailing metadata",
            values: &[("trailing", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Content",
        controls: &[
            Control::Choice {
                id: "selected",
                label: "Selected row",
                options: &["Inbox", "Drafts", "Archive"],
                default: 0,
            },
            Control::Switch {
                id: "supporting",
                label: "Supporting text",
                default: true,
            },
            Control::Switch {
                id: "trailing",
                label: "Trailing count",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let theme = window_material_theme(window, cx);
        let selected = cfg.option("selected");
        let supporting = cfg.bool("supporting");
        let trailing = cfg.bool("trailing");
        let rows = [
            ("Inbox", "12 unread messages", "12", IconName::Inbox),
            ("Drafts", "3 unsent drafts", "3", IconName::Edit),
            ("Archive", "Stored conversations", "84", IconName::Archive),
        ];
        let mut list = List::new("pg-list");
        for (ix, (title, sub, count, icon)) in rows.into_iter().enumerate() {
            let on_choice = ctx.on_choice("selected", cx);
            let mut item = ListItem::new(("pg-list-item", ix), title)
                .leading(Icon::new(icon))
                .selected(selected == title)
                .on_activate(move |window, cx| on_choice(ix, window, cx));
            if supporting {
                item = item.supporting(sub);
            }
            if trailing {
                item = item.trailing(
                    div()
                        .text_xs()
                        .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                        .child(count),
                );
            }
            list = list.item(item);
        }
        div().w(px(340.)).child(list).into_any_element()
    },
    code: |cfg| {
        let mut s = String::from(
            "List::new(\"folders\").item(\n    ListItem::new(\"inbox\", \"Inbox\")\n        .leading(Icon::new(IconName::Inbox))",
        );
        if cfg.bool("supporting") {
            s.push_str("\n        .supporting(\"12 unread messages\")");
        }
        s.push_str("\n)");
        s
    },
};

// --- Sheets ---
pub static SHEETS: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Side sheet",
            values: &[("placement", Set::Option("Side"))],
        },
        Scenario {
            name: "No header",
            values: &[("header", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Presentation",
        controls: &[
            Control::Choice {
                id: "placement",
                label: "Placement",
                options: &["Bottom", "Side"],
                default: 0,
            },
            Control::Switch {
                id: "header",
                label: "Show header",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let state = ctx
            .local::<Entity<DialogState>>("sheet-state", || cx.new(DialogState::new))
            .clone();
        let open_state = state.clone();
        let edge = if cfg.option("placement") == "Side" {
            SheetEdge::End
        } else {
            SheetEdge::Bottom
        };
        div()
            .child(
                Button::new("pg-open-sheet", "Open sheet")
                    .variant(ButtonVariant::FilledTonal)
                    .on_activate(move |window, cx| {
                        open_state.update(cx, |s, cx| s.open(window, cx));
                    }),
            )
            .child(
                Sheet::new(
                    "pg-sheet",
                    &state,
                    "Inspector",
                    "Supplementary details presented without leaving the current screen.",
                )
                .edge(edge)
                .show_header(cfg.bool("header")),
            )
            .into_any_element()
    },
    code: |cfg| {
        let edge = if cfg.option("placement") == "Side" {
            "SheetEdge::End"
        } else {
            "SheetEdge::Bottom"
        };
        format!(
            "Sheet::new(\"inspector\", &state, \"Inspector\", \"Supplementary details.\")\n    .edge({edge})\n    .show_header({})",
            cfg.bool("header")
        )
    },
};

// --- Surface ---
pub static SURFACE: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Elevated card",
            values: &[
                ("role", Set::Option("Surface high")),
                ("elevation", Set::Option("Level 3")),
            ],
        },
        Scenario {
            name: "Primary container",
            values: &[
                ("role", Set::Option("Primary container")),
                ("corner", Set::Option("Extra large")),
            ],
        },
    ],
    sections: &[Section {
        title: "Tokens",
        controls: &[
            Control::Choice {
                id: "role",
                label: "Color role",
                options: &[
                    "Surface low",
                    "Surface high",
                    "Primary container",
                    "Tertiary container",
                ],
                default: 0,
            },
            Control::Choice {
                id: "elevation",
                label: "Elevation",
                options: &["Level 0", "Level 1", "Level 2", "Level 3", "Level 4"],
                default: 1,
            },
            Control::Choice {
                id: "corner",
                label: "Corner radius",
                options: &["Small", "Medium", "Large", "Extra large"],
                default: 2,
            },
        ],
    }],
    preview: |cfg, _, window, cx| {
        let role = match cfg.option("role") {
            "Surface high" => ColorRole::SurfaceContainerHigh,
            "Primary container" => ColorRole::PrimaryContainer,
            "Tertiary container" => ColorRole::TertiaryContainer,
            _ => ColorRole::SurfaceContainerLow,
        };
        let elevation = match cfg.option("elevation") {
            "Level 0" => Elevation::Level0,
            "Level 2" => Elevation::Level2,
            "Level 3" => Elevation::Level3,
            "Level 4" => Elevation::Level4,
            _ => Elevation::Level1,
        };
        let fg_role = role.on_role().unwrap_or(ColorRole::OnSurface);
        let corner = match cfg.option("corner") {
            "Small" => CornerRole::Small,
            "Medium" => CornerRole::Medium,
            "Extra large" => CornerRole::ExtraLarge,
            _ => CornerRole::Large,
        };
        let radius = corner.shape().radii(260., 132., Direction::Ltr)[0];
        let theme = window_material_theme(window, cx);
        Surface::new(role)
            .tonal_elevation(elevation)
            .w(px(280.))
            .h(px(136.))
            .rounded(px(radius))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_1()
            .child(
                Text::new(cfg.option("role"))
                    .id("pg-surface-label")
                    .role(TypeRole::TitleMedium)
                    .color(fg_role),
            )
            .child(
                Text::new(format!(
                    "{} · {}",
                    cfg.option("elevation"),
                    cfg.option("corner")
                ))
                .id("pg-surface-meta")
                .role(TypeRole::LabelSmall)
                .color(fg_role),
            )
            .into_any_element()
    },
    code: |cfg| {
        let role = match cfg.option("role") {
            "Surface high" => "ColorRole::SurfaceContainerHigh",
            "Primary container" => "ColorRole::PrimaryContainer",
            "Tertiary container" => "ColorRole::TertiaryContainer",
            _ => "ColorRole::SurfaceContainerLow",
        };
        let elev = match cfg.option("elevation") {
            "Level 0" => "Elevation::Level0",
            "Level 2" => "Elevation::Level2",
            "Level 3" => "Elevation::Level3",
            "Level 4" => "Elevation::Level4",
            _ => "Elevation::Level1",
        };
        format!(
            "Surface::new({role})\n    .tonal_elevation({elev})\n    .p_4()\n    .child(Text::new(\"Content\"))"
        )
    },
};

// --- VerticalDragHandle ---
pub static VERTICAL_DRAG_HANDLE: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Wider left pane",
            values: &[("split", Set::Number(220.))],
        },
        Scenario {
            name: "Narrow left pane",
            values: &[("split", Set::Number(110.))],
        },
    ],
    sections: &[Section {
        title: "Split width",
        controls: &[Control::Slider {
            id: "split",
            label: "Leading pane (px)",
            min: 90.,
            max: 250.,
            step: 10.,
            default: 160.,
        }],
    }],
    preview: |cfg, _, window, cx| {
        let theme = window_material_theme(window, cx);
        let left_w = cfg.number("split");
        let bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let muted = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        div()
            .w(px(420.))
            .h(px(164.))
            .rounded(px(14.))
            .border_1()
            .border_color(outline)
            .p_2()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(px(left_w))
                    .h_full()
                    .rounded(px(10.))
                    .bg(bg)
                    .p_3()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Navigation"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(format!("{left_w:.0} px")),
                    ),
            )
            .child(VerticalDragHandle::new("pg-drag-handle"))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .rounded(px(10.))
                    .bg(bg)
                    .p_3()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("Detail pane"),
                    )
                    .child(div().text_xs().text_color(muted).child("Flexible width")),
            )
            .into_any_element()
    },
    code: |_| "VerticalDragHandle::new(\"pane-splitter\")".into(),
};

// --- DropTarget ---
pub static DROP_TARGET: Playground = Playground {
    scenarios: &[Scenario {
        name: "Disabled",
        values: &[("disabled", Set::Bool(true))],
    }],
    sections: &[Section {
        title: "Content & state",
        controls: &[
            Control::Text {
                id: "label",
                label: "Headline",
                default: "Drop release artifacts here",
            },
            Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let on_status = ctx.on_status(cx);
        div()
            .w(px(340.))
            .child(
                DropTarget::<()>::new("pg-drop-target")
                    .title(cfg.text("label").to_owned())
                    .supporting("Supports .zip, .tar.gz, and .json files")
                    .disabled(cfg.bool("disabled"))
                    .on_activate(move |_, cx| on_status("Drop target browse triggered".into(), cx)),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "DropTarget::<()>::new(\"uploads\")\n    .title({:?})\n    .supporting(\"Supports .zip, .tar.gz, and .json files\")\n    .disabled({})",
            cfg.text("label"),
            cfg.bool("disabled")
        )
    },
};

// --- DataViews ---
pub static DATA_VIEWS: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Striped descending",
            values: &[
                ("striped", Set::Bool(true)),
                ("sort", Set::Option("Descending")),
            ],
        },
        Scenario {
            name: "No row dividers",
            values: &[("dividers", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Table presentation",
        controls: &[
            Control::Choice {
                id: "sort",
                label: "Sort direction",
                options: &["Ascending", "Descending"],
                default: 0,
            },
            Control::Switch {
                id: "striped",
                label: "Striped rows",
                default: false,
            },
            Control::Switch {
                id: "dividers",
                label: "Row dividers",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let sort_dir = match cfg.option("sort") {
            "Descending" => SortDirection::Descending,
            _ => SortDirection::Ascending,
        };
        let on_sort = ctx.on_status(cx);
        let on_select = ctx.on_status(cx);
        let mut rows = [
            ("alpha", "API Gateway", "18 ms", false),
            ("beta", "Search Indexer", "41 ms", false),
            ("gamma", "Cold Archive", "120 ms", true),
        ];
        if sort_dir == SortDirection::Descending {
            rows.reverse();
        }
        div()
            .w(px(420.))
            .child(
                DataTable::new("pg-data-table")
                    .accessible_label("Preview services")
                    .striped(cfg.bool("striped"))
                    .show_row_dividers(cfg.bool("dividers"))
                    .columns([
                        DataTableColumn::new("name", "Service")
                            .weight(2.0)
                            .sort_direction(Some(sort_dir)),
                        DataTableColumn::new("latency", "Latency")
                            .width(px(110.))
                            .alignment(ColumnAlignment::End),
                    ])
                    .rows(rows.into_iter().map(|(id, name, latency, disabled)| {
                        DataTableRow::new(
                            id,
                            [DataTableCell::text(name), DataTableCell::text(latency)],
                        )
                        .selected(id == "alpha")
                        .disabled(disabled)
                    }))
                    .on_sort(move |col, next, _, cx| {
                        on_sort(format!("Sort {col} -> {next:?}"), cx);
                    })
                    .on_row_select(move |id, _, cx| {
                        on_select(format!("Selected row {id}"), cx);
                    }),
            )
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "DataTable::new(\"services\")\n    .striped({})\n    .show_row_dividers({})\n    .columns([\n        DataTableColumn::new(\"name\", \"Service\").sort_direction(Some(SortDirection::{})),\n        DataTableColumn::new(\"latency\", \"Latency\").alignment(ColumnAlignment::End),\n    ])",
            cfg.bool("striped"),
            cfg.bool("dividers"),
            cfg.option("sort")
        )
    },
};
