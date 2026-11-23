use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, div, px};
use gpui_material::{
    Avatar, AvatarGroup, AvatarSize, Badge, Banner, Button, ButtonVariant, CircularProgress,
    EmptyState, Icon, IconName, LinearProgress, LoadingIndicator, MaterialShape, ProgressShape,
    SemanticTone, Skeleton, Snackbar, SnackbarHost, Text, Tooltip,
    theme::{ColorRole, CornerRole, TypeRole},
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

// --- Avatar ---
pub static AVATAR: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Avatar group",
            values: &[
                ("mode", Set::Option("Group")),
                ("size", Set::Option("Medium")),
            ],
        },
        Scenario {
            name: "Large icon",
            values: &[
                ("mode", Set::Option("Icon")),
                ("size", Set::Option("Large")),
            ],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "mode",
                label: "Mode",
                options: &["Initials", "Icon", "Group"],
                default: 0,
            },
            Control::Choice {
                id: "size",
                label: "Size",
                options: &["Small", "Medium", "Large", "ExtraLarge"],
                default: 1,
            },
            Control::Text {
                id: "initials",
                label: "Name / initials",
                default: "Ada Lovelace",
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let size = match cfg.option("size") {
            "Small" => AvatarSize::Small,
            "Large" => AvatarSize::Large,
            "ExtraLarge" => AvatarSize::ExtraLarge,
            _ => AvatarSize::Medium,
        };
        match cfg.option("mode") {
            "Icon" => Avatar::icon(Icon::new(IconName::Favorite))
                .size(size)
                .into_any_element(),
            "Group" => AvatarGroup::new(
                "pg-avatar-group",
                [
                    Avatar::initials("Ada Lovelace"),
                    Avatar::initials("Grace Hopper"),
                    Avatar::initials("Alan Turing"),
                    Avatar::initials("Margaret Hamilton"),
                ],
            )
            .size(size)
            .max_visible(3)
            .into_any_element(),
            _ => Avatar::initials(cfg.text("initials").to_owned())
                .size(size)
                .into_any_element(),
        }
    },
    code: |cfg| match cfg.option("mode") {
        "Icon" => format!(
            "Avatar::icon(Icon::new(IconName::Favorite))\n    .size(AvatarSize::{})",
            cfg.option("size")
        ),
        "Group" => format!(
            "AvatarGroup::new(\"team\", [\n    Avatar::initials(\"Ada Lovelace\"),\n    Avatar::initials(\"Grace Hopper\"),\n    Avatar::initials(\"Alan Turing\"),\n])\n.size(AvatarSize::{})\n.max_visible(3)",
            cfg.option("size")
        ),
        _ => format!(
            "Avatar::initials({:?})\n    .size(AvatarSize::{})",
            cfg.text("initials"),
            cfg.option("size")
        ),
    },
};

// --- Badge ---
pub static BADGE: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Dot",
            values: &[("kind", Set::Option("Dot"))],
        },
        Scenario {
            name: "Overflow count",
            values: &[("kind", Set::Option("Count")), ("count", Set::Number(120.))],
        },
        Scenario {
            name: "Label",
            values: &[("kind", Set::Option("Label"))],
        },
    ],
    sections: &[Section {
        title: "Configuration",
        controls: &[
            Control::Choice {
                id: "kind",
                label: "Kind",
                options: &["Count", "Dot", "Label"],
                default: 0,
            },
            Control::Slider {
                id: "count",
                label: "Count",
                min: 1.,
                max: 150.,
                step: 1.,
                default: 7.,
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let icon = Icon::new(IconName::Notifications).size(px(28.));
        match cfg.option("kind") {
            "Dot" => Badge::dot(icon).into_any_element(),
            "Label" => Badge::label(icon, "NEW").into_any_element(),
            _ => Badge::count(icon, cfg.number("count") as usize, 99).into_any_element(),
        }
    },
    code: |cfg| match cfg.option("kind") {
        "Dot" => "Badge::dot(Icon::new(IconName::Notifications).size(px(28.)))".into(),
        "Label" => "Badge::label(Icon::new(IconName::Notifications).size(px(28.)), \"NEW\")".into(),
        _ => format!(
            "Badge::count(Icon::new(IconName::Notifications).size(px(28.)), {}, 99)",
            cfg.number("count") as usize
        ),
    },
};

// --- Banner ---
pub static BANNER: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Warning + Action",
            values: &[
                ("tone", Set::Option("Warning")),
                ("action", Set::Bool(true)),
                ("title", Set::Text("Offline mode")),
            ],
        },
        Scenario {
            name: "Success",
            values: &[
                ("tone", Set::Option("Success")),
                ("title", Set::Text("Checks passed")),
            ],
        },
        Scenario {
            name: "Danger",
            values: &[
                ("tone", Set::Option("Danger")),
                ("title", Set::Text("Sync failed")),
            ],
        },
    ],
    sections: &[Section {
        title: "Appearance & actions",
        controls: &[
            Control::Choice {
                id: "tone",
                label: "Tone",
                options: &["Default", "Info", "Success", "Warning", "Danger"],
                default: 1,
            },
            Control::Text {
                id: "title",
                label: "Title",
                default: "Sync status",
            },
            Control::Switch {
                id: "action",
                label: "Show action button",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let tone = match cfg.option("tone") {
            "Info" => SemanticTone::Info,
            "Success" => SemanticTone::Success,
            "Warning" => SemanticTone::Warning,
            "Danger" => SemanticTone::Danger,
            _ => SemanticTone::Default,
        };
        let on_status = ctx.on_status(cx);
        let mut banner = Banner::new(
            "pg-banner",
            "Workspace preferences are synchronized across desktop sessions.",
        )
        .title(cfg.text("title").to_owned())
        .icon(Icon::new(IconName::Info))
        .tone(tone);
        if cfg.bool("action") {
            banner = banner.action("Retry", move |_, cx| {
                on_status("Banner action: Retry".into(), cx)
            });
        }
        div().w_full().child(banner).into_any_element()
    },
    code: |cfg| {
        let mut s = format!(
            "Banner::new(\"sync\", \"Workspace preferences are synchronized.\")\n    .title({:?})\n    .icon(Icon::new(IconName::Info))\n    .tone(SemanticTone::{})",
            cfg.text("title"),
            cfg.option("tone")
        );
        if cfg.bool("action") {
            s.push_str("\n    .action(\"Retry\", |_, _| {})");
        }
        s
    },
};

// --- EmptyState ---
pub static EMPTY_STATE: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Search empty",
            values: &[
                ("title", Set::Text("No matching results")),
                ("shape", Set::Option("Cookie")),
            ],
        },
        Scenario {
            name: "Minimal (no CTA)",
            values: &[("action", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Content & backdrop",
        controls: &[
            Control::Choice {
                id: "shape",
                label: "Backdrop shape",
                options: &["SoftBurst", "Cookie", "Clover", "Arch"],
                default: 0,
            },
            Control::Text {
                id: "title",
                label: "Headline",
                default: "No items yet",
            },
            Control::Switch {
                id: "action",
                label: "Show primary action",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let shape = match cfg.option("shape") {
            "Cookie" => MaterialShape::Cookie9Sided,
            "Clover" => MaterialShape::Clover4Leaf,
            "Arch" => MaterialShape::Arch,
            _ => MaterialShape::SoftBurst,
        };
        let on_status = ctx.on_status(cx);
        let mut empty = EmptyState::new("pg-empty-state", cfg.text("title").to_owned())
            .supporting("Create your first item or import an existing workspace to get started.")
            .icon(Icon::new(IconName::Inbox))
            .icon_container_shape(shape);
        if cfg.bool("action") {
            empty = empty.primary_action(
                Button::new("pg-empty-cta", "Create item")
                    .variant(ButtonVariant::Filled)
                    .on_activate(move |_, cx| on_status("Empty state action clicked".into(), cx)),
            );
        }
        empty.into_any_element()
    },
    code: |cfg| {
        let shape = match cfg.option("shape") {
            "Cookie" => "MaterialShape::Cookie9Sided",
            "Clover" => "MaterialShape::Clover4Leaf",
            "Arch" => "MaterialShape::Arch",
            _ => "MaterialShape::SoftBurst",
        };
        let mut s = format!(
            "EmptyState::new(\"empty\", {:?})\n    .supporting(\"Create your first item to get started.\")\n    .icon(Icon::new(IconName::Inbox))\n    .icon_container_shape({shape})",
            cfg.text("title")
        );
        if cfg.bool("action") {
            s.push_str("\n    .primary_action(Button::new(\"create\", \"Create item\"))");
        }
        s
    },
};

// --- Icon ---
pub static ICON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Filled primary",
            values: &[
                ("filled", Set::Bool(true)),
                ("size", Set::Number(40.)),
                ("color", Set::Option("Primary")),
            ],
        },
        Scenario {
            name: "Error alert",
            values: &[
                ("symbol", Set::Option("Notifications")),
                ("color", Set::Option("Error")),
                ("size", Set::Number(36.)),
            ],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "symbol",
                label: "Symbol",
                options: &["Favorite", "Search", "Notifications", "Inbox", "Edit"],
                default: 0,
            },
            Control::Choice {
                id: "color",
                label: "Color role",
                options: &["Primary", "Secondary", "Tertiary", "Error", "OnSurface"],
                default: 0,
            },
            Control::Slider {
                id: "size",
                label: "Size (dp)",
                min: 16.,
                max: 64.,
                step: 4.,
                default: 32.,
            },
            Control::Switch {
                id: "filled",
                label: "Filled",
                default: false,
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let name = match cfg.option("symbol") {
            "Search" => IconName::Search,
            "Notifications" => IconName::Notifications,
            "Inbox" => IconName::Inbox,
            "Edit" => IconName::Edit,
            _ => IconName::Favorite,
        };
        let role = match cfg.option("color") {
            "Secondary" => ColorRole::Secondary,
            "Tertiary" => ColorRole::Tertiary,
            "Error" => ColorRole::Error,
            "OnSurface" => ColorRole::OnSurface,
            _ => ColorRole::Primary,
        };
        Icon::new(name)
            .size(px(cfg.number("size")))
            .filled(cfg.bool("filled"))
            .color(role)
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Icon::new(IconName::{})\n    .size(px({:.0}.))\n    .filled({})\n    .color(ColorRole::{})",
            cfg.option("symbol"),
            cfg.number("size"),
            cfg.bool("filled"),
            cfg.option("color")
        )
    },
};

// --- LoadingIndicator ---
pub static LOADING_INDICATOR: Playground = Playground {
    scenarios: &[Scenario {
        name: "Contained",
        values: &[("contained", Set::Bool(true))],
    }],
    sections: &[Section {
        title: "Appearance",
        controls: &[Control::Switch {
            id: "contained",
            label: "Contained",
            default: false,
        }],
    }],
    preview: |cfg, _, _, _| {
        LoadingIndicator::new("pg-loading", "Loading workspace")
            .contained(cfg.bool("contained"))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "LoadingIndicator::new(\"loading\", \"Loading workspace\")\n    .contained({})",
            cfg.bool("contained")
        )
    },
};

// --- Progress ---
pub static PROGRESS: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Circular",
            values: &[
                ("kind", Set::Option("Circular")),
                ("value", Set::Number(75.)),
            ],
        },
        Scenario {
            name: "Wavy linear",
            values: &[("wavy", Set::Bool(true))],
        },
        Scenario {
            name: "Indeterminate",
            values: &[("indeterminate", Set::Bool(true))],
        },
    ],
    sections: &[Section {
        title: "Configuration",
        controls: &[
            Control::Choice {
                id: "kind",
                label: "Kind",
                options: &["Linear", "Circular"],
                default: 0,
            },
            Control::Switch {
                id: "indeterminate",
                label: "Indeterminate",
                default: false,
            },
            Control::Slider {
                id: "value",
                label: "Progress (%)",
                min: 0.,
                max: 100.,
                step: 5.,
                default: 60.,
            },
            Control::Switch {
                id: "wavy",
                label: "Wavy expressive",
                default: false,
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let fraction = cfg.number("value") / 100.;
        let indeterminate = cfg.bool("indeterminate");
        let shape = if cfg.bool("wavy") {
            ProgressShape::Wavy
        } else {
            ProgressShape::default()
        };
        if cfg.option("kind") == "Circular" {
            let mut p =
                CircularProgress::new("pg-circular-progress", "Upload progress").shape(shape);
            if !indeterminate {
                p = p.value(fraction);
            }
            p.into_any_element()
        } else {
            let mut p = LinearProgress::new("pg-linear-progress", "Upload progress").shape(shape);
            if !indeterminate {
                p = p.value(fraction);
            }
            div().w(px(300.)).child(p).into_any_element()
        }
    },
    code: |cfg| {
        let fraction = cfg.number("value") / 100.;
        let shape = if cfg.bool("wavy") {
            "ProgressShape::Wavy"
        } else {
            "ProgressShape::Flat"
        };
        let ctor = if cfg.option("kind") == "Circular" {
            "CircularProgress::new(\"progress\", \"Upload progress\")"
        } else {
            "LinearProgress::new(\"progress\", \"Upload progress\")"
        };
        if cfg.bool("indeterminate") {
            format!("{ctor}\n    .shape({shape})")
        } else {
            format!("{ctor}\n    .value({fraction:.2})\n    .shape({shape})")
        }
    },
};

// --- Skeleton ---
pub static SKELETON: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Media card",
            values: &[("layout", Set::Option("Media card"))],
        },
        Scenario {
            name: "Expressive shapes",
            values: &[("layout", Set::Option("Shapes"))],
        },
        Scenario {
            name: "Static",
            values: &[("animated", Set::Bool(false))],
        },
    ],
    sections: &[Section {
        title: "Layout & motion",
        controls: &[
            Control::Choice {
                id: "layout",
                label: "Preset layout",
                options: &["List item", "Media card", "Shapes"],
                default: 0,
            },
            Control::Switch {
                id: "animated",
                label: "Shimmer animation",
                default: true,
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let animated = cfg.bool("animated");
        match cfg.option("layout") {
            "Media card" => div()
                .w(px(280.))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    Skeleton::rect("pg-skel-media", px(280.), px(112.))
                        .corner_role(CornerRole::Large)
                        .animated(animated),
                )
                .child(
                    Skeleton::text_line("pg-skel-mtitle")
                        .width(px(180.))
                        .role(TypeRole::TitleMedium)
                        .animated(animated),
                )
                .child(Skeleton::text_line("pg-skel-msub").animated(animated))
                .into_any_element(),
            "Shapes" => div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    Skeleton::shape(
                        "pg-skel-cookie",
                        px(64.),
                        px(64.),
                        MaterialShape::Cookie7Sided,
                    )
                    .animated(animated),
                )
                .child(
                    Skeleton::shape(
                        "pg-skel-clover",
                        px(64.),
                        px(64.),
                        MaterialShape::Clover4Leaf,
                    )
                    .animated(animated),
                )
                .child(
                    Skeleton::shape("pg-skel-burst", px(64.), px(64.), MaterialShape::SoftBurst)
                        .animated(animated),
                )
                .into_any_element(),
            _ => div()
                .w(px(300.))
                .flex()
                .items_center()
                .gap_3()
                .child(Skeleton::circle("pg-skel-avatar", px(48.)).animated(animated))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            Skeleton::text_line("pg-skel-title")
                                .width(px(160.))
                                .role(TypeRole::TitleMedium)
                                .animated(animated),
                        )
                        .child(Skeleton::text_line("pg-skel-sub").animated(animated)),
                )
                .into_any_element(),
        }
    },
    code: |cfg| {
        let animated = cfg.bool("animated");
        match cfg.option("layout") {
            "Media card" => format!(
                "Skeleton::rect(\"media\", px(280.), px(112.))\n    .corner_role(CornerRole::Large)\n    .animated({animated})"
            ),
            "Shapes" => format!(
                "Skeleton::shape(\"cookie\", px(64.), px(64.), MaterialShape::Cookie7Sided)\n    .animated({animated})"
            ),
            _ => format!(
                "Skeleton::circle(\"avatar\", px(48.)).animated({animated})\nSkeleton::text_line(\"title\").width(px(160.)).animated({animated})"
            ),
        }
    },
};

// --- Snackbar ---
pub static SNACKBAR: Playground = Playground {
    scenarios: &[Scenario {
        name: "Without action",
        values: &[("action", Set::Bool(false))],
    }],
    sections: &[Section {
        title: "Content",
        controls: &[
            Control::Text {
                id: "message",
                label: "Message",
                default: "Draft archived",
            },
            Control::Switch {
                id: "action",
                label: "Show Undo action",
                default: true,
            },
        ],
    }],
    preview: |cfg, ctx, _, cx| {
        let host = ctx
            .local::<Entity<SnackbarHost>>("snackbar-host", || cx.new(|_| SnackbarHost::default()))
            .clone();
        let show_host = host.clone();
        let message = cfg.text("message").to_owned();
        let with_action = cfg.bool("action");
        let on_undo = ctx.on_status(cx);
        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(
                Button::new("pg-show-snackbar", "Show snackbar")
                    .variant(ButtonVariant::FilledTonal)
                    .on_activate(move |window, cx| {
                        let on_undo = on_undo.clone();
                        let msg = message.clone();
                        show_host.update(cx, |host, cx| {
                            let mut bar = Snackbar::new(msg);
                            if with_action {
                                bar = bar.action("Undo", move |_, cx| {
                                    on_undo("Snackbar Undo clicked".into(), cx);
                                });
                            }
                            host.enqueue(bar, window, cx);
                        });
                    }),
            )
            .child(host)
            .into_any_element()
    },
    code: |cfg| {
        let mut s = format!("let bar = Snackbar::new({:?})", cfg.text("message"));
        if cfg.bool("action") {
            s.push_str("\n    .action(\"Undo\", |_, _| {})");
        }
        s.push_str(";\nhost.update(cx, |host, cx| host.enqueue(bar, window, cx));");
        s
    },
};

// --- Text ---
pub static TEXT: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Display Small",
            values: &[
                ("role", Set::Option("Display Small")),
                ("emphasized", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Primary headline",
            values: &[
                ("role", Set::Option("Headline Medium")),
                ("color", Set::Option("Primary")),
            ],
        },
    ],
    sections: &[Section {
        title: "Typography",
        controls: &[
            Control::Choice {
                id: "role",
                label: "Type role",
                options: &[
                    "Display Small",
                    "Headline Medium",
                    "Title Large",
                    "Body Large",
                    "Label Large",
                ],
                default: 2,
            },
            Control::Choice {
                id: "color",
                label: "Color role",
                options: &["OnSurface", "OnSurfaceVariant", "Primary", "Error"],
                default: 0,
            },
            Control::Switch {
                id: "emphasized",
                label: "Emphasized",
                default: false,
            },
            Control::Text {
                id: "sample",
                label: "Text",
                default: "Material 3 Expressive typography",
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let role = match cfg.option("role") {
            "Display Small" => TypeRole::DisplaySmall,
            "Headline Medium" => TypeRole::HeadlineMedium,
            "Body Large" => TypeRole::BodyLarge,
            "Label Large" => TypeRole::LabelLarge,
            _ => TypeRole::TitleLarge,
        };
        let color_role = match cfg.option("color") {
            "OnSurfaceVariant" => ColorRole::OnSurfaceVariant,
            "Primary" => ColorRole::Primary,
            "Error" => ColorRole::Error,
            _ => ColorRole::OnSurface,
        };
        Text::new(cfg.text("sample").to_owned())
            .id("pg-text-sample")
            .role(role)
            .color(color_role)
            .emphasized(cfg.bool("emphasized"))
            .wrap()
            .into_any_element()
    },
    code: |cfg| {
        let role = match cfg.option("role") {
            "Display Small" => "TypeRole::DisplaySmall",
            "Headline Medium" => "TypeRole::HeadlineMedium",
            "Body Large" => "TypeRole::BodyLarge",
            "Label Large" => "TypeRole::LabelLarge",
            _ => "TypeRole::TitleLarge",
        };
        format!(
            "Text::new({:?})\n    .role({role})\n    .color(ColorRole::{})\n    .emphasized({})",
            cfg.text("sample"),
            cfg.option("color"),
            cfg.bool("emphasized")
        )
    },
};

// --- Tooltip ---
pub static TOOLTIP: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Rich tooltip",
            values: &[
                ("variant", Set::Option("Rich")),
                (
                    "tip",
                    Set::Text("Saves changes locally and syncs to your workspace."),
                ),
            ],
        },
        Scenario {
            name: "Shortcut tip",
            values: &[("tip", Set::Text("Keyboard shortcut: Ctrl+S"))],
        },
    ],
    sections: &[Section {
        title: "Content",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Plain", "Rich"],
                default: 0,
            },
            Control::Text {
                id: "tip",
                label: "Tooltip text",
                default: "Save changes to workspace",
            },
        ],
    }],
    preview: |cfg, _, _, _| {
        let trigger =
            Button::new("pg-tooltip-trigger", "Hover or focus me").variant(ButtonVariant::Outlined);
        if cfg.option("variant") == "Rich" {
            Tooltip::rich(
                "pg-tooltip",
                trigger,
                "Save workspace",
                cfg.text("tip").to_owned(),
            )
            .into_any_element()
        } else {
            Tooltip::plain("pg-tooltip", trigger, cfg.text("tip").to_owned()).into_any_element()
        }
    },
    code: |cfg| {
        if cfg.option("variant") == "Rich" {
            format!(
                "Tooltip::rich(\n    \"save-tip\",\n    Button::new(\"save\", \"Hover or focus me\").variant(ButtonVariant::Outlined),\n    \"Save workspace\",\n    {:?},\n)",
                cfg.text("tip")
            )
        } else {
            format!(
                "Tooltip::plain(\n    \"save-tip\",\n    Button::new(\"save\", \"Hover or focus me\").variant(ButtonVariant::Outlined),\n    {:?},\n)",
                cfg.text("tip")
            )
        }
    },
};
