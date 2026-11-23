use gpui::{AppContext, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use gpui_material::{
    AppBar, AppBarVariant, BottomAppBar, BreadcrumbItem, Breadcrumbs, Fab, Icon, IconButton,
    IconName, NavigationBar, NavigationDrawer, NavigationItem, NavigationItems, NavigationRail,
    RailMode, SearchBar, SearchState, TabVariant, Tabs, Toolbar, ToolbarAction, ToolbarColor,
    ToolbarPlacement,
    foundation::{color, window_material_theme},
    theme::ColorRole,
};

use crate::playground::{Control, Playground, Scenario, Section, Set};

fn app_frame(window: &Window, cx: &gpui::App) -> gpui::Div {
    let theme = window_material_theme(window, cx);
    div()
        .w_full()
        .max_w(px(520.))
        .rounded(px(16.))
        .border_1()
        .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
        .bg(color(theme.colors.color(ColorRole::SurfaceContainerLowest)))
        .overflow_hidden()
        .flex()
        .flex_col()
}

fn mock_body(title: &str, subtitle: &str, window: &Window, cx: &gpui::App) -> gpui::Div {
    let theme = window_material_theme(window, cx);
    let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
    let muted = color(theme.colors.color(ColorRole::OnSurfaceVariant));
    div()
        .flex_1()
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(title.to_owned()),
        )
        .child(div().text_xs().text_color(muted).child(subtitle.to_owned()))
        .child(
            div()
                .mt_1()
                .h(px(36.))
                .w_full()
                .rounded(px(10.))
                .bg(card_bg),
        )
        .child(div().h(px(36.)).w_full().rounded(px(10.)).bg(card_bg))
}

// --- AppBar ---
pub static APP_BAR: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Medium flexible",
            values: &[
                ("variant", Set::Option("Medium")),
                ("subtitle", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Large flexible",
            values: &[
                ("variant", Set::Option("Large")),
                ("subtitle", Set::Bool(true)),
            ],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Small", "Medium", "Large"],
                default: 0,
            },
            Control::Text {
                id: "title",
                label: "Title",
                default: "Projects",
            },
            Control::Switch {
                id: "subtitle",
                label: "Show subtitle",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let variant = match cfg.option("variant") {
            "Medium" => AppBarVariant::MediumFlexible,
            "Large" => AppBarVariant::LargeFlexible,
            _ => AppBarVariant::Small,
        };
        let on_status = ctx.on_status(cx);
        let mut bar = AppBar::new("pg-app-bar", cfg.text("title").to_owned())
            .variant(variant)
            .actions(
                IconButton::new("pg-appbar-search", "Search", Icon::new(IconName::Search))
                    .on_activate(move |_, cx| on_status("App bar search clicked".into(), cx)),
            );
        if cfg.bool("subtitle") {
            bar = bar.subtitle("Workspace overview");
        }
        app_frame(window, cx)
            .child(bar)
            .child(mock_body(
                "Active deliverables",
                "Scrollable workspace content beneath the top app bar.",
                window,
                cx,
            ))
            .into_any_element()
    },
    code: |cfg| {
        let variant = match cfg.option("variant") {
            "Medium" => "AppBarVariant::MediumFlexible",
            "Large" => "AppBarVariant::LargeFlexible",
            _ => "AppBarVariant::Small",
        };
        let mut s = format!(
            "AppBar::new(\"app-bar\", {:?})\n    .variant({variant})",
            cfg.text("title")
        );
        if cfg.bool("subtitle") {
            s.push_str("\n    .subtitle(\"Workspace overview\")");
        }
        s
    },
};

// --- BottomAppBar ---
pub static BOTTOM_APP_BAR: Playground = Playground {
    scenarios: &[Scenario {
        name: "Without FAB",
        values: &[("fab", Set::Bool(false))],
    }],
    sections: &[Section {
        title: "Slots",
        controls: &[Control::Switch {
            id: "fab",
            label: "Show trailing FAB",
            default: true,
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let s1 = ctx.on_status(cx);
        let s2 = ctx.on_status(cx);
        let s3 = ctx.on_status(cx);
        let s4 = ctx.on_status(cx);
        let mut bar = BottomAppBar::new("pg-bottom-app-bar")
            .action(
                IconButton::new("pg-bab-search", "Search", Icon::new(IconName::Search))
                    .on_activate(move |_, cx| s1("Bottom bar: Search".into(), cx)),
            )
            .action(
                IconButton::new("pg-bab-archive", "Archive", Icon::new(IconName::Archive))
                    .on_activate(move |_, cx| s2("Bottom bar: Archive".into(), cx)),
            )
            .action(
                IconButton::new("pg-bab-fav", "Favorite", Icon::new(IconName::Favorite))
                    .on_activate(move |_, cx| s4("Bottom bar: Favorite".into(), cx)),
            );
        if cfg.bool("fab") {
            bar = bar.fab(
                Fab::new("pg-bab-fab", "Add", Icon::new(IconName::Add))
                    .on_activate(move |_, cx| s3("Bottom bar FAB clicked".into(), cx)),
            );
        }
        app_frame(window, cx)
            .child(mock_body(
                "Notes feed",
                "Primary actions stay reachable at the bottom edge.",
                window,
                cx,
            ))
            .child(bar)
            .into_any_element()
    },
    code: |cfg| {
        let mut s = String::from(
            "BottomAppBar::new(\"bottom-bar\")\n    .action(IconButton::new(\"search\", \"Search\", Icon::new(IconName::Search)))\n    .action(IconButton::new(\"archive\", \"Archive\", Icon::new(IconName::Archive)))",
        );
        if cfg.bool("fab") {
            s.push_str("\n    .fab(Fab::new(\"add\", \"Add\", Icon::new(IconName::Add)))");
        }
        s
    },
};

// --- NavigationBar ---
pub static NAVIGATION_BAR: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Alerts selected",
            values: &[("selected", Set::Option("Alerts"))],
        },
        Scenario {
            name: "Archive selected",
            values: &[("selected", Set::Option("Archive"))],
        },
    ],
    sections: &[Section {
        title: "Destination",
        controls: &[Control::Choice {
            id: "selected",
            label: "Selected",
            options: &["Inbox", "Archive", "Alerts"],
            default: 0,
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let (selected, subtitle) = match cfg.option("selected") {
            "Archive" => ("archive", "Stored threads and completed tasks."),
            "Alerts" => ("alerts", "Recent mentions and build notifications."),
            _ => ("inbox", "12 unread messages awaiting triage."),
        };
        let on_choice = ctx.on_choice("selected", cx);
        let items = NavigationItems::new(
            "pg-nav-bar-items",
            "Primary navigation",
            [
                NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
                NavigationItem::new("archive", "Archive").icon(Icon::new(IconName::Archive)),
                NavigationItem::new("alerts", "Alerts").icon(Icon::new(IconName::Notifications)),
            ],
        )
        .selected(selected)
        .on_select(move |next, window, cx| {
            let ix = match next.as_ref() {
                "archive" => 1,
                "alerts" => 2,
                _ => 0,
            };
            on_choice(ix, window, cx);
        });
        app_frame(window, cx)
            .child(mock_body(cfg.option("selected"), subtitle, window, cx))
            .child(NavigationBar::new(items))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "NavigationBar::new(\n    NavigationItems::new(\"nav\", \"Primary\", [\n        NavigationItem::new(\"inbox\", \"Inbox\").icon(Icon::new(IconName::Inbox)),\n        NavigationItem::new(\"archive\", \"Archive\").icon(Icon::new(IconName::Archive)),\n        NavigationItem::new(\"alerts\", \"Alerts\").icon(Icon::new(IconName::Notifications)),\n    ]).selected({:?})\n)",
            cfg.option("selected").to_ascii_lowercase()
        )
    },
};

// --- NavigationRail ---
pub static NAVIGATION_RAIL: Playground = Playground {
    scenarios: &[Scenario {
        name: "Expanded",
        values: &[("expanded", Set::Bool(true))],
    }],
    sections: &[Section {
        title: "State",
        controls: &[
            Control::Choice {
                id: "selected",
                label: "Selected",
                options: &["Inbox", "Archive", "Alerts"],
                default: 0,
            },
            Control::Switch {
                id: "expanded",
                label: "Expanded",
                default: false,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let (selected, subtitle) = match cfg.option("selected") {
            "Archive" => ("archive", "Archived records synced across devices."),
            "Alerts" => ("alerts", "Priority alerts and team mentions."),
            _ => ("inbox", "Primary workspace inbox with live rail switching."),
        };
        let on_choice = ctx.on_choice("selected", cx);
        let items = NavigationItems::new(
            "pg-nav-rail-items",
            "Rail navigation",
            [
                NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
                NavigationItem::new("archive", "Archive").icon(Icon::new(IconName::Archive)),
                NavigationItem::new("alerts", "Alerts").icon(Icon::new(IconName::Notifications)),
            ],
        )
        .selected(selected)
        .on_select(move |next, window, cx| {
            let ix = match next.as_ref() {
                "archive" => 1,
                "alerts" => 2,
                _ => 0,
            };
            on_choice(ix, window, cx);
        });
        let theme = window_material_theme(window, cx);
        div()
            .w_full()
            .max_w(px(520.))
            .h(px(260.))
            .rounded(px(16.))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLowest)))
            .overflow_hidden()
            .flex()
            .child(
                NavigationRail::new("pg-nav-rail", items).mode(if cfg.bool("expanded") {
                    RailMode::Expanded
                } else {
                    RailMode::Collapsed
                }),
            )
            .child(mock_body(cfg.option("selected"), subtitle, window, cx))
            .into_any_element()
    },
    code: |cfg| {
        let mode = if cfg.bool("expanded") {
            "RailMode::Expanded"
        } else {
            "RailMode::Collapsed"
        };
        format!("NavigationRail::new(\"rail\", items)\n    .mode({mode})")
    },
};

// --- NavigationDrawer ---
pub static NAVIGATION_DRAWER: Playground = Playground {
    scenarios: &[Scenario {
        name: "Archive selected",
        values: &[("selected", Set::Option("Archive"))],
    }],
    sections: &[Section {
        title: "Destination",
        controls: &[Control::Choice {
            id: "selected",
            label: "Selected",
            options: &["Inbox", "Archive", "Alerts"],
            default: 0,
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let (selected, subtitle) = match cfg.option("selected") {
            "Archive" => ("archive", "Previously completed items."),
            "Alerts" => ("alerts", "Unread notifications and updates."),
            _ => (
                "inbox",
                "Docked 360 dp navigation drawer beside workspace content.",
            ),
        };
        let on_choice = ctx.on_choice("selected", cx);
        let items = NavigationItems::new(
            "pg-nav-drawer-items",
            "Drawer destinations",
            [
                NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
                NavigationItem::new("archive", "Archive").icon(Icon::new(IconName::Archive)),
                NavigationItem::new("alerts", "Alerts").icon(Icon::new(IconName::Notifications)),
            ],
        )
        .selected(selected)
        .on_select(move |next, window, cx| {
            let ix = match next.as_ref() {
                "archive" => 1,
                "alerts" => 2,
                _ => 0,
            };
            on_choice(ix, window, cx);
        });
        let theme = window_material_theme(window, cx);
        div()
            .w_full()
            .max_w(px(560.))
            .h(px(260.))
            .rounded(px(16.))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLowest)))
            .overflow_hidden()
            .flex()
            .child(NavigationDrawer::new("pg-nav-drawer", items))
            .child(mock_body(cfg.option("selected"), subtitle, window, cx))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "NavigationDrawer::new(\"drawer\", items.selected({:?}))",
            cfg.option("selected").to_ascii_lowercase()
        )
    },
};

// --- Tabs ---
pub static TABS: Playground = Playground {
    scenarios: &[Scenario {
        name: "Secondary",
        values: &[("variant", Set::Option("Secondary"))],
    }],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "variant",
                label: "Variant",
                options: &["Primary", "Secondary"],
                default: 0,
            },
            Control::Choice {
                id: "selected",
                label: "Active tab",
                options: &["Overview", "Activity", "Settings"],
                default: 0,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let variant = if cfg.option("variant") == "Secondary" {
            TabVariant::Secondary
        } else {
            TabVariant::Primary
        };
        let (selected, subtitle) = match cfg.option("selected") {
            "Activity" => ("activity", "Live audit log and recent workspace commits."),
            "Settings" => ("settings", "Permissions, notifications, and billing rules."),
            _ => (
                "overview",
                "Project summary, status health, and milestones.",
            ),
        };
        let on_choice = ctx.on_choice("selected", cx);
        let items = NavigationItems::new(
            "pg-tabs",
            "Workspace views",
            [
                NavigationItem::new("overview", "Overview"),
                NavigationItem::new("activity", "Activity"),
                NavigationItem::new("settings", "Settings"),
            ],
        )
        .selected(selected)
        .on_select(move |next, window, cx| {
            let ix = match next.as_ref() {
                "activity" => 1,
                "settings" => 2,
                _ => 0,
            };
            on_choice(ix, window, cx);
        });
        app_frame(window, cx)
            .child(Tabs::new(items).variant(variant))
            .child(mock_body(cfg.option("selected"), subtitle, window, cx))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Tabs::new(\n    NavigationItems::new(\"views\", \"Workspace views\", [\n        NavigationItem::new(\"overview\", \"Overview\"),\n        NavigationItem::new(\"activity\", \"Activity\"),\n        NavigationItem::new(\"settings\", \"Settings\"),\n    ]).selected({:?})\n).variant(TabVariant::{})",
            cfg.option("selected").to_ascii_lowercase(),
            cfg.option("variant")
        )
    },
};

// --- Search ---
pub static SEARCH: Playground = Playground {
    scenarios: &[Scenario {
        name: "Custom placeholder",
        values: &[("placeholder", Set::Text("Search files or symbols"))],
    }],
    sections: &[Section {
        title: "Content",
        controls: &[Control::Text {
            id: "placeholder",
            label: "Placeholder",
            default: "Search documents",
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let state = ctx
            .local::<Entity<SearchState>>("search-state", || {
                cx.new(|cx| SearchState::new(window, cx))
            })
            .clone();
        app_frame(window, cx)
            .child(
                div().p_3().flex().justify_center().child(
                    SearchBar::new("pg-search-bar", cfg.text("placeholder").to_owned(), &state)
                        .widths(px(260.), px(340.)),
                ),
            )
            .child(mock_body(
                "Quick search",
                "Activate the search bar above to expand the docked search view.",
                window,
                cx,
            ))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "let state = cx.new(|cx| SearchState::new(window, cx));\nSearchBar::new(\"search\", {:?}, &state)\n    .widths(px(260.), px(340.))",
            cfg.text("placeholder")
        )
    },
};

// --- Toolbars ---
pub static TOOLBARS: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Vibrant floating",
            values: &[
                ("placement", Set::Option("Floating")),
                ("color", Set::Option("Vibrant")),
            ],
        },
        Scenario {
            name: "Docked",
            values: &[("placement", Set::Option("Docked"))],
        },
    ],
    sections: &[Section {
        title: "Appearance",
        controls: &[
            Control::Choice {
                id: "placement",
                label: "Placement",
                options: &["Floating", "Docked"],
                default: 0,
            },
            Control::Choice {
                id: "color",
                label: "Color",
                options: &["Standard", "Vibrant"],
                default: 0,
            },
        ],
    }],
    preview: |cfg, ctx, window, cx| {
        let placement = if cfg.option("placement") == "Docked" {
            ToolbarPlacement::Docked
        } else {
            ToolbarPlacement::Floating
        };
        let color_style = if cfg.option("color") == "Vibrant" {
            ToolbarColor::Vibrant
        } else {
            ToolbarColor::Standard
        };
        let s1 = ctx.on_status(cx);
        let s2 = ctx.on_status(cx);
        let s3 = ctx.on_status(cx);
        let s4 = ctx.on_status(cx);
        let toolbar = Toolbar::new(
            "pg-toolbar",
            "Document actions",
            [
                ToolbarAction::new("edit", "Edit", IconName::Edit)
                    .on_activate(move |_, cx| s1("Toolbar: Edit".into(), cx)),
                ToolbarAction::new("fav", "Favorite", IconName::Favorite)
                    .on_activate(move |_, cx| s2("Toolbar: Favorite".into(), cx)),
                ToolbarAction::new("archive", "Archive", IconName::Archive)
                    .on_activate(move |_, cx| s3("Toolbar: Archive".into(), cx)),
                ToolbarAction::new("alerts", "Notifications", IconName::Notifications)
                    .on_activate(move |_, cx| s4("Toolbar: Notifications".into(), cx)),
            ],
        )
        .placement(placement)
        .color(color_style);
        app_frame(window, cx)
            .child(mock_body(
                "Rich document editor",
                "Floating or docked editing commands aligned to the workspace edge.",
                window,
                cx,
            ))
            .child(div().w_full().p_3().flex().justify_center().child(toolbar))
            .into_any_element()
    },
    code: |cfg| {
        format!(
            "Toolbar::new(\"actions\", \"Document actions\", [\n    ToolbarAction::new(\"edit\", \"Edit\", IconName::Edit),\n    ToolbarAction::new(\"fav\", \"Favorite\", IconName::Favorite),\n    ToolbarAction::new(\"archive\", \"Archive\", IconName::Archive),\n])\n.placement(ToolbarPlacement::{})\n.color(ToolbarColor::{})",
            cfg.option("placement"),
            cfg.option("color")
        )
    },
};

// --- Breadcrumbs ---
pub static BREADCRUMBS: Playground = Playground {
    scenarios: &[Scenario {
        name: "Deep path",
        values: &[("depth", Set::Option("5 levels"))],
    }],
    sections: &[Section {
        title: "Path depth",
        controls: &[Control::Choice {
            id: "depth",
            label: "Segments",
            options: &["3 levels", "5 levels"],
            default: 0,
        }],
    }],
    preview: |cfg, ctx, window, cx| {
        let on_status = ctx.on_status(cx);
        let items = if cfg.option("depth") == "5 levels" {
            vec![
                BreadcrumbItem::new("home", "Workspace").icon(Icon::new(IconName::Inbox)),
                BreadcrumbItem::new("org", "Engineering"),
                BreadcrumbItem::new("team", "Desktop UI"),
                BreadcrumbItem::new("crate", "Components"),
                BreadcrumbItem::new("file", "breadcrumbs.rs"),
            ]
        } else {
            vec![
                BreadcrumbItem::new("home", "Workspace").icon(Icon::new(IconName::Inbox)),
                BreadcrumbItem::new("projects", "Projects"),
                BreadcrumbItem::new("roadmap", "2026 Roadmap"),
            ]
        };
        let theme = window_material_theme(window, cx);
        app_frame(window, cx)
            .child(
                div()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
                    .child(Breadcrumbs::new("pg-breadcrumbs", items).on_select(
                        move |id, _, cx| on_status(format!("Navigated to segment \"{id}\""), cx),
                    )),
            )
            .child(mock_body(
                "Current directory",
                "Click any ancestor breadcrumb above to jump up the hierarchy.",
                window,
                cx,
            ))
            .into_any_element()
    },
    code: |cfg| {
        if cfg.option("depth") == "5 levels" {
            "Breadcrumbs::new(\"path\", [\n    BreadcrumbItem::new(\"home\", \"Workspace\"),\n    BreadcrumbItem::new(\"org\", \"Engineering\"),\n    BreadcrumbItem::new(\"team\", \"Desktop UI\"),\n    BreadcrumbItem::new(\"crate\", \"Components\"),\n    BreadcrumbItem::new(\"file\", \"breadcrumbs.rs\"),\n])\n.on_select(|id, _, _| {})".into()
        } else {
            "Breadcrumbs::new(\"path\", [\n    BreadcrumbItem::new(\"home\", \"Workspace\"),\n    BreadcrumbItem::new(\"projects\", \"Projects\"),\n    BreadcrumbItem::new(\"roadmap\", \"2026 Roadmap\"),\n])\n.on_select(|id, _, _| {})".into()
        }
    },
};
