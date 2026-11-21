use gpui::InteractiveElement;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui::{FocusHandle, SharedString};
use gpui_base::TestSupportExt as _;
use gpui_material::{
    Button, Dialog, DialogColors, DialogProperties, DialogState, DialogVariant, Icon, IconName,
    Sheet, SheetEdge, SheetMode, SheetPlacement, Snackbar, SnackbarColors, SnackbarHost, Surface,
    TextField, TextFieldState, TextFieldVariant,
    theme::{ColorRole, CornerRole, Elevation},
};
use gpui_material::{
    InteractionSource, Label, Menu, MenuColor, MenuItem, Switch, Text, Tooltip, TooltipCaret,
    TooltipPlacement, TooltipState,
};

use crate::story::{Story, section, stack};

mod bottom_sheet_examples;
#[cfg(all(test, feature = "test-support"))]
mod bottom_sheet_tests;

#[cfg(all(test, feature = "test-support"))]
mod dialog_tests;
#[cfg(all(test, feature = "test-support"))]
mod label_tests;
#[cfg(all(test, feature = "test-support"))]
mod menu_tests;
#[cfg(all(test, feature = "test-support"))]
mod side_sheet_tests;
#[cfg(all(test, feature = "test-support"))]
mod snackbar_tests;
#[cfg(all(test, feature = "test-support"))]
mod tooltip_tests;

struct DialogDemoState {
    custom: Entity<DialogState>,
    native: Entity<DialogState>,
    name: Entity<TextFieldState>,
}

struct SheetDemoState {
    standard: Entity<DialogState>,
    detached_side: Entity<DialogState>,
    standard_side: Entity<DialogState>,
    bottom_examples: Entity<bottom_sheet_examples::BottomSheetExamples>,
    standard_side_visible: bool,
    reviewing_side_details: bool,
    expanded: bool,
}

pub(crate) struct ModalExamples {
    story: Story,
    primary: Entity<DialogState>,
    secondary: Entity<DialogState>,
    dialog: Option<DialogDemoState>,
    sheet: Option<SheetDemoState>,
}

impl ModalExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let primary = cx.new(DialogState::new);
        let secondary = cx.new(DialogState::new);
        cx.observe(&primary, |_, _, cx| cx.notify()).detach();
        cx.observe(&secondary, |_, _, cx| cx.notify()).detach();
        let dialog = (story == Story::Dialog).then(|| {
            let custom = cx.new(DialogState::new);
            cx.observe(&custom, |_, _, cx| cx.notify()).detach();
            DialogDemoState {
                custom,
                native: cx.new(DialogState::new),
                name: cx.new(|cx| TextFieldState::new(window, cx).placeholder("名称")),
            }
        });
        let sheet = (story == Story::Sheets).then(|| {
            let detached_side = cx.new(DialogState::new);
            cx.observe(&detached_side, |_, _, cx| cx.notify()).detach();
            SheetDemoState {
                standard: cx.new(DialogState::new),
                detached_side,
                standard_side: cx.new(DialogState::new),
                bottom_examples: cx.new(bottom_sheet_examples::BottomSheetExamples::new),
                standard_side_visible: true,
                reviewing_side_details: false,
                expanded: false,
            }
        });
        Self {
            story,
            primary,
            secondary,
            dialog,
            sheet,
        }
    }

    fn trigger(id: &'static str, label: &'static str, state: &Entity<DialogState>) -> Button {
        let state = state.clone();
        Button::new(id, label).on_activate(move |window, cx| {
            state.update(cx, |state, cx| state.open(window, cx));
        })
    }

    fn dialogs(&self) -> impl IntoElement {
        let dialog = self
            .dialog
            .as_ref()
            .expect("dialog demo state initialized for Story::Dialog");
        let editor = dialog.name.clone();
        let native = dialog.native.clone();
        stack()
            .child(
                section("Rename a project")
                    .description(
                        "Enter a project name or open Naming guidelines from inside the dialog.",
                    )
                    .child(div().child(Self::trigger("rename", "Rename project…", &self.primary))),
            )
            .child(
                Dialog::new("project-dialog", &self.primary, "Rename project")
                    .description("Use a name people can recognize in the project list.")
                    .body(
                        stack()
                            .child(
                                TextField::new("project-name", "Project name", &editor)
                                    .variant(TextFieldVariant::Outlined),
                            )
                            .child(Self::trigger(
                                "guidelines",
                                "Naming guidelines…",
                                &self.secondary,
                            )),
                    )
                    .confirm("Rename", move |_, cx| {
                        !editor.read(cx).value().trim().is_empty()
                    }),
            )
            .child(
                Dialog::new("guidelines-dialog", &self.secondary, "Naming guidelines")
                    .layer(1)
                    .variant(DialogVariant::FullScreen)
                    .description("Use the project name as it appears in your team’s roadmap.")
                    .cancel_label("Close"),
            )
            .child(
                section("Customize a modal")
                    .child(Self::trigger(
                        "review-export",
                        "Review export…",
                        &dialog.custom,
                    ))
                    .child(
                        Button::new("open-native-dialog", "Open native dialog…").on_activate(
                            move |parent, cx| {
                                let result =
                                    Dialog::open_window(
                                        &native,
                                        parent,
                                        WindowOptions {
                                            titlebar: Some(TitlebarOptions {
                                                title: Some("Native project dialog".into()),
                                                ..TitlebarOptions::default()
                                            }),
                                            window_bounds: Some(WindowBounds::centered(
                                                size(px(460.), px(320.)),
                                                cx,
                                            )),
                                            ..WindowOptions::default()
                                        },
                                        cx,
                                        |state, _, _| {
                                            Dialog::unstructured(
                                                "native-project-dialog",
                                                state,
                                                "Project information",
                                            )
                                            .width(px(360.))
                                            .body_content(|_, dismiss| {
                                                Surface::new(ColorRole::SecondaryContainer)
                                                    .shape(CornerRole::Large.shape())
                                                    .p(px(24.))
                                                    .child(
                                                        stack()
                                                            .child(Text::new("Project information"))
                                                            .child(
                                                                Button::new(
                                                                    "done-native-dialog",
                                                                    "Done",
                                                                )
                                                                .on_activate(move |window, cx| {
                                                                    dismiss.activate(window, cx)
                                                                }),
                                                            ),
                                                    )
                                            })
                                        },
                                    );
                                if let Err(error) = result {
                                    eprintln!("Could not open native project dialog: {error}");
                                }
                            },
                        ),
                    ),
            )
            .child(
                Dialog::new("export-dialog", &dialog.custom, "Review export")
                    .icon(Icon::new(IconName::Info).size(px(24.)))
                    .title_content(div().child("Review your export"))
                    .description("The export includes the project name and task history.")
                    .properties(DialogProperties {
                        dismiss_on_outside_click: false,
                        ..DialogProperties::default()
                    })
                    .dismiss_content(|dismiss| {
                        Button::new("keep-editing", "Keep editing")
                            .on_activate(move |window, cx| dismiss.activate(window, cx))
                    })
                    .confirm_content(|confirm| {
                        Button::new("download-export", "Download")
                            .on_activate(move |window, cx| confirm.activate(window, cx))
                    })
                    .shape(CornerRole::Large.shape())
                    .colors(DialogColors {
                        icon: Some(gpui::rgb(0x6750a4).into()),
                        ..DialogColors::default()
                    })
                    .shadow_elevation(Elevation::Level2)
                    .scrim_opacity(0.28),
            )
    }

    fn sheets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self
            .sheet
            .as_ref()
            .expect("sheet demo state initialized for Story::Sheets");
        let owner = cx.entity().downgrade();
        stack()
            .child(
                section("Modal sheets")
                    .description("Project activity opens from the bottom. Project details opens at the trailing edge.")
                    .child(div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(Self::trigger(
                        "activity",
                        "Project activity…",
                        &self.primary,
                    ))
                    .child(Self::trigger(
                        "details",
                        "Project details…",
                        &self.secondary,
                    ))),
            )
            .child(
                Sheet::new(
                    "standard",
                    &sheet.standard,
                    "Project notes",
                    "The project name is used in the activity log.",
                )
                .extent(px(160.))
                .expandable(sheet.expanded, move |expanded, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        if let Some(sheet) = &mut this.sheet {
                            sheet.expanded = expanded;
                        }
                        cx.notify();
                    });
                }),
            )
            .child(
                Sheet::new(
                    "activity-sheet",
                    &self.primary,
                    "Project activity",
                    stack()
                        .children((1..=20).map(|entry| div().child(format!("Revision {entry}")))),
                )
                .mode(SheetMode::Modal)
                .extent(px(300.)),
            )
            .child(
                Sheet::new(
                    "details-sheet",
                    &self.secondary,
                    "Project details",
                    "Visible to project members",
                )
                .edge(SheetEdge::End)
                .mode(SheetMode::Modal)
                .extent(px(360.)),
            )
            .child(sheet.bottom_examples.clone())
            .child(self.side_sheets(sheet, cx))
    }

    fn side_sheets(&self, sheet: &SheetDemoState, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let restore = owner.clone();
        let close_standard = owner.clone();
        let back = owner.clone();
        let detached = sheet.detached_side.clone();
        section("Side-sheet layout and actions")
            .child(Self::trigger(
                "filters",
                "Project filters…",
                &sheet.detached_side,
            ))
            .child(
                div()
                    .flex()
                    .h(px(250.))
                    .gap_3()
                    .child(div().flex_1().min_w_0().child(
                        Button::new("restore-inspector", "Show project inspector").on_activate(
                            move |_, cx| {
                                let _ = restore.update(cx, |view, cx| {
                                    if let Some(sheet) = &mut view.sheet {
                                        sheet.standard_side_visible = true;
                                    }
                                    cx.notify();
                                });
                            },
                        ),
                    ))
                    .children(sheet.standard_side_visible.then(|| {
                        Sheet::new(
                            "project-inspector",
                            &sheet.standard_side,
                            "Project inspector",
                            "Inspect the active project while its tasks stay available.",
                        )
                        .edge(SheetEdge::End)
                        .extent(px(260.))
                        .edge_divider(true)
                        .close_label("Close inspector")
                        .on_close(move |_, _, cx| {
                            let _ = close_standard.update(cx, |view, cx| {
                                if let Some(sheet) = &mut view.sheet {
                                    sheet.standard_side_visible = false;
                                }
                                cx.notify();
                            });
                        })
                    })),
            )
            .child(
                Sheet::new(
                    "filter-sheet",
                    &sheet.detached_side,
                    "Project filters",
                    stack()
                        .child(Button::new("filter-details", "Filter details").on_activate(
                            move |_, cx| {
                                let _ = owner.update(cx, |view, cx| {
                                    if let Some(sheet) = &mut view.sheet {
                                        sheet.reviewing_side_details = true;
                                    }
                                    cx.notify();
                                });
                            },
                        ))
                        .children(
                            (1..=15).map(|index| div().child(format!("Project option {index}"))),
                        ),
                )
                .edge(SheetEdge::Start)
                .mode(SheetMode::Modal)
                .placement(SheetPlacement::Detached)
                .header_content(
                    Text::new(if sheet.reviewing_side_details {
                        "Filter details"
                    } else {
                        "Project filters"
                    })
                    .id("filters-heading")
                    .role(gpui_material::theme::TypeRole::TitleMedium),
                )
                .back("Back to filters", move |_, cx| {
                    let _ = back.update(cx, |view, cx| {
                        if let Some(sheet) = &mut view.sheet {
                            sheet.reviewing_side_details = false;
                        }
                        cx.notify();
                    });
                })
                .close_content(|close| {
                    Button::new("close-filters", "Close filters")
                        .variant(gpui_material::ButtonVariant::Text)
                        .on_activate(move |window, cx| close.activate(window, cx))
                })
                .footer(Button::new("apply-filters", "Apply filters").on_activate(
                    move |window, cx| {
                        detached.update(cx, |state, cx| state.close(window, cx));
                    },
                ))
                .footer_divider(true),
            )
    }
}

impl Render for ModalExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.story == Story::Dialog {
            self.dialogs().into_any_element()
        } else {
            self.sheets(cx).into_any_element()
        }
    }
}

pub(crate) struct SnackbarExamples {
    host: Entity<SnackbarHost>,
}

impl SnackbarExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            host: cx.new(|_| SnackbarHost::default()),
        }
    }
}

impl Render for SnackbarExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let timed = self.host.clone();
        let persistent = self.host.clone();
        stack()
            .max_w(px(480.))
            .child(
                section("Timed message")
                    .description("Save changes to show a message that dismisses itself.")
                    .child(div().id("save-region").test_support().child(
                        Button::new("save-message", "Save changes").on_activate(
                            move |window, cx| {
                                timed.update(cx, |host, cx| {
                                    host.push(Snackbar::new("Changes saved"), window, cx);
                                });
                            },
                        ),
                    )),
            )
            .child(
                section("Message with Undo")
                    .description("Archive a draft and use Undo to restore it.")
                    .child(div().id("archive-region").test_support().child(
                        Button::new("archive-message", "Archive draft").on_activate(
                            move |window, cx| {
                                let host = persistent.downgrade();
                                persistent.update(cx, |queue, cx| {
                                    queue.push(
                                        Snackbar::new("Draft archived").action(
                                            "Undo",
                                            move |window, cx| {
                                                let _ = host.update(cx, |host, cx| {
                                                    host.push(
                                                        Snackbar::new("Draft restored"),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            },
                                        )
                                        .message_content(|_, _| {
                                            div().flex().items_center().gap_2()
                                                .child(Icon::new(IconName::Archive).size(px(20.)))
                                                .child("Draft archived")
                                        })
                                        .action_content(|_, _| div().child("Undo archive"))
                                        .dismiss_content("Dismiss archive message", |_, _| {
                                            Icon::new(IconName::Close).size(px(20.))
                                        })
                                        .action_on_new_line(true)
                                        .shape(gpui_material::theme::CornerShape::Radii([12., 4., 12., 4.]))
                                        .colors(gpui_material::foundation::window_material_theme(window, cx).colors
                                            .pair(gpui_material::theme::ColorRole::PrimaryContainer)
                                            .map_or_else(SnackbarColors::default, |pair| {
                                                let tone = gpui_material::foundation::color;
                                                SnackbarColors {
                                                    container: Some(tone(pair.background)),
                                                    content: Some(tone(pair.foreground)),
                                                    action: Some(tone(pair.foreground)),
                                                    dismiss: Some(tone(pair.foreground)),
                                                }
                                            })),
                                        window,
                                        cx,
                                    );
                                });
                            },
                        ),
                    )),
            )
            .child(self.host.clone())
    }
}

pub(crate) struct MenuTooltipExamples {
    story: Story,
    trigger: FocusHandle,
    rich_trigger: FocusHandle,
    rich_action: FocusHandle,
    persistent: Entity<TooltipState>,
    persistent_trigger: FocusHandle,
    persistent_action: FocusHandle,
    label_source: Entity<InteractionSource>,
    keep_label: bool,
    menu_open: bool,
    labels: bool,
    output: SharedString,
}

impl MenuTooltipExamples {
    pub(crate) fn new(story: Story, cx: &mut Context<Self>) -> Self {
        Self {
            story,
            trigger: cx.focus_handle(),
            rich_trigger: cx.focus_handle(),
            rich_action: cx.focus_handle(),
            persistent: cx.new(|_| TooltipState::new()),
            persistent_trigger: cx.focus_handle(),
            persistent_action: cx.focus_handle(),
            label_source: cx.new(|_| InteractionSource::new()),
            keep_label: true,
            menu_open: false,
            labels: false,
            output: "Sort by name".into(),
        }
    }

    fn set_output(
        cx: &Context<Self>,
        value: &'static str,
    ) -> impl Fn(&mut Window, &mut gpui::App) + 'static {
        let owner = cx.entity().downgrade();
        move |_, cx| {
            let _ = owner.update(cx, |state, cx| {
                state.output = value.into();
                cx.notify();
            });
        }
    }

    fn command_menu_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let menu_owner = cx.entity().downgrade();
        let label_owner = cx.entity().downgrade();
        section("Command menu")
            .description("Open Sort by to choose how the results are ordered.")
            .child(
                div().child(
                    Menu::new(
                        "options",
                        gpui_base::Button::new("open-options")
                            .track_focus(&self.trigger)
                            .accessibility_label("View options")
                            .px_3()
                            .py_2()
                            .rounded_sm()
                            .border_1()
                            .child("View options"),
                        [
                            MenuItem::group("display", "Display"),
                            MenuItem::new("labels", "Show labels")
                                .checked(self.labels)
                                .supporting(|| "Show names next to each icon")
                                .on_checked_change(move |checked, _, cx| {
                                    let _ = label_owner.update(cx, |state, cx| {
                                        state.labels = checked;
                                        cx.notify();
                                    });
                                }),
                            MenuItem::new("columns", "Configure columns").disabled(true),
                            MenuItem::new("sort", "Sort by").submenu([
                                MenuItem::new("name", "Name")
                                    .selected(self.output.as_ref() == "Sort by name")
                                    .on_activate(Self::set_output(cx, "Sort by name")),
                                MenuItem::new("date", "Date modified")
                                    .selected(self.output.as_ref() == "Sort by date modified")
                                    .on_activate(Self::set_output(cx, "Sort by date modified")),
                            ]),
                        ],
                    )
                    .open(self.menu_open)
                    .on_open_change(move |open, _, cx| {
                        let _ = menu_owner.update(cx, |state, cx| {
                            state.menu_open = open;
                            cx.notify();
                        });
                    }),
                ),
            )
    }

    fn vibrant_menu_section(&self) -> crate::story::StorySection {
        section("Vibrant menu").child(
            div().child(
                Menu::new(
                    "vibrant-options",
                    gpui_base::Button::new("open-vibrant")
                        .px_3()
                        .py_2()
                        .rounded_sm()
                        .border_1()
                        .child("Share settings"),
                    [MenuItem::section(
                        "visibility",
                        "Visibility",
                        [
                            MenuItem::new("view", "View")
                                .selected(true)
                                .leading(|| Text::new("V"))
                                .trailing(|| Text::new("Default")),
                            MenuItem::new("edit", "Edit").supporting(|| Text::new("Allow changes")),
                        ],
                    )
                    .inactive_shape(CornerRole::Small.shape())],
                )
                .color(MenuColor::Vibrant)
                .offset(px(8.)),
            ),
        )
    }

    fn context_menu_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        section("Context menu")
            .description("Right-click Roadmap to see the actions for that item.")
            .child(
                div().child(
                    Menu::new(
                        "context-actions",
                        gpui_base::Button::new("open-context")
                            .accessibility_label("Roadmap context actions")
                            .px_3()
                            .py_2()
                            .rounded_sm()
                            .border_1()
                            .child("Roadmap"),
                        [MenuItem::new("pin", "Pin roadmap")
                            .on_activate(Self::set_output(cx, "Roadmap pinned"))],
                    )
                    .context(),
                ),
            )
    }

    fn render_menus(&self, cx: &Context<Self>) -> impl IntoElement {
        stack()
            .child(self.command_menu_section(cx))
            .child(self.vibrant_menu_section())
            .child(self.context_menu_section(cx))
            .child(Text::new(self.output.clone()))
    }

    fn plain_tooltip_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        section("Plain tooltip")
            .description("Hover over Pin roadmap or focus it with the keyboard.")
            .child(
                div().child(
                    Tooltip::plain(
                        "pin-tooltip",
                        Button::new("pin", "Pin roadmap")
                            .track_focus(&self.trigger)
                            .on_activate(Self::set_output(cx, "Roadmap pinned")),
                        "Keep this roadmap at the top of the project list",
                    )
                    .track_focus(&self.trigger),
                ),
            )
    }

    fn rich_tooltip_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        section("Rich tooltip")
            .description("Open the tooltip and activate Show retention period.")
            .child(
                div().child(
                    Tooltip::rich(
                        "archive-tooltip",
                        Button::new("retention", "Archive policy").track_focus(&self.rich_trigger),
                        "Archived drafts",
                        "Archived drafts stay available for 30 days.",
                    )
                    .track_focus(&self.rich_trigger)
                    .action(
                        "Show retention period",
                        &self.rich_action,
                        Self::set_output(cx, "30 days"),
                    ),
                ),
            )
    }

    fn persistent_tooltip_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let show = self.persistent.clone();
        let owner_show = self.persistent.clone();
        let dismiss = self.persistent.clone();
        section("Persistent tooltip").child(
            div()
                .flex()
                .gap_3()
                .flex_wrap()
                .child(
                    Tooltip::rich(
                        "sharing-tooltip",
                        Button::new("sharing-help", "Sharing help")
                            .track_focus(&self.persistent_trigger)
                            .on_activate(move |_, cx| show.update(cx, |state, cx| state.show(cx))),
                        "Sharing settings",
                        "Invite teammates from the Share menu.",
                    )
                    .persistent(&self.persistent)
                    .track_focus(&self.persistent_trigger)
                    .focus_target(&self.persistent_action)
                    .focusable(true)
                    .placement(TooltipPlacement::Below)
                    .offset(px(12.))
                    .caret(TooltipCaret::default())
                    .body_content(
                        Text::new("Invite teammates from the Share menu.")
                            .id("sharing-tooltip-body")
                            .wrap(),
                    )
                    .action_content(
                        Button::new("dismiss-sharing-help", "Dismiss help")
                            .variant(gpui_material::ButtonVariant::Text)
                            .track_focus(&self.persistent_action)
                            .on_activate(move |_, cx| {
                                dismiss.update(cx, |state, cx| state.dismiss(cx))
                            }),
                    )
                    .link(
                        "Sharing guide",
                        Self::set_output(cx, "Sharing guide requested"),
                    ),
                )
                .child(
                    Button::new("show-sharing-help", "Show help")
                        .variant(gpui_material::ButtonVariant::Outlined)
                        .on_activate(move |_, cx| {
                            owner_show.update(cx, |state, cx| state.show(cx))
                        }),
                ),
        )
    }

    fn control_labels_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let persistent_label_owner = cx.entity().downgrade();
        section("Control labels")
            .child(
                div().flex().child(
                    Label::new(
                        "retained-label",
                        Text::new("UTC").id("label-reference-value"),
                        "Coordinated Universal Time",
                    )
                    .content(
                        Text::new("Coordinated Universal Time")
                            .id("utc-label-content")
                            .role(gpui_material::theme::TypeRole::BodySmall),
                    )
                    .persistent(self.keep_label)
                    .placement(TooltipPlacement::End)
                    .offset(px(10.))
                    .caret(TooltipCaret::default()),
                ),
            )
            .child(
                div().flex().child(
                    Label::new(
                        "measure-label",
                        Button::new("measure", "Measure")
                            .interaction_source(&self.label_source)
                            .on_activate(Self::set_output(cx, "Measurement requested")),
                        "Measurement uses local time",
                    )
                    .interaction_source(&self.label_source, "measure")
                    .content(
                        Text::new("Measurement uses local time")
                            .id("measurement-label-content")
                            .wrap(),
                    ),
                ),
            )
            .child(
                Switch::new("keep-label", "Keep label visible", self.keep_label).on_change(
                    move |keep, _, cx| {
                        let _ = persistent_label_owner.update(cx, |view, cx| {
                            view.keep_label = keep;
                            cx.notify();
                        });
                    },
                ),
            )
    }

    fn render_tooltips(&self, cx: &Context<Self>) -> impl IntoElement {
        stack()
            .child(self.plain_tooltip_section(cx))
            .child(self.rich_tooltip_section(cx))
            .child(Text::new(if self.output.as_ref() == "Sort by name" {
                SharedString::default()
            } else {
                self.output.clone()
            }))
            .child(self.persistent_tooltip_section(cx))
            .child(self.control_labels_section(cx))
    }
}

impl Render for MenuTooltipExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.story == Story::Menu {
            self.render_menus(cx).into_any_element()
        } else {
            self.render_tooltips(cx).into_any_element()
        }
    }
}
