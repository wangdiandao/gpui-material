use gpui::InteractiveElement;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui::{FocusHandle, SharedString};
use gpui_base::TestSupportExt as _;
use gpui_material::{
    Button, Dialog, DialogState, DialogVariant, Sheet, SheetEdge, SheetMode, Snackbar,
    SnackbarHost, TextField, TextFieldState, TextFieldVariant,
};
use gpui_material::{Menu, MenuItem, Text, Tooltip};

use crate::story::{Story, stack};

pub(crate) struct ModalExamples {
    story: Story,
    primary: Entity<DialogState>,
    secondary: Entity<DialogState>,
    standard: Entity<DialogState>,
    name: Entity<TextFieldState>,
    expanded: bool,
}

impl ModalExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let primary = cx.new(DialogState::new);
        let secondary = cx.new(DialogState::new);
        let standard = cx.new(DialogState::new);
        cx.observe(&primary, |_, _, cx| cx.notify()).detach();
        cx.observe(&secondary, |_, _, cx| cx.notify()).detach();
        Self {
            story,
            primary,
            secondary,
            standard,
            name: cx.new(|cx| TextFieldState::new(window, cx).placeholder("名称")),
            expanded: false,
        }
    }

    fn trigger(id: &'static str, label: &'static str, state: &Entity<DialogState>) -> Button {
        let state = state.clone();
        Button::new(id, label).on_activate(move |window, cx| {
            state.update(cx, |state, cx| state.open(window, cx));
        })
    }

    fn dialogs(&self) -> impl IntoElement {
        let editor = self.name.clone();
        stack()
            .items_start()
            .child(Self::trigger("rename", "Rename project…", &self.primary))
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
    }

    fn sheets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        stack()
            .child(
                div()
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
                    )),
            )
            .child(
                Sheet::new(
                    "standard",
                    &self.standard,
                    "Project notes",
                    "The project name is used in the activity log.",
                )
                .extent(px(160.))
                .expandable(self.expanded, move |expanded, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.expanded = expanded;
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

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn bottom_sheet_cancels_drag_and_accepts_next_gesture(cx: &mut gpui::TestAppContext) {
    use gpui::{
        InputEvent as _, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, WindowOptions,
        point,
    };
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let drag = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let start_height = find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height;
            let origin = find(window, &[], &"drag-handle".into())
                .unwrap()
                .bounds()
                .center();
            let handle = find(window, &[], &"drag-handle".into()).unwrap().bounds();
            let expand = find(window, &[], &"expand".into()).unwrap().bounds();
            let panel = find(window, &[], &"standard".into()).unwrap().bounds();
            assert_eq!(handle.size.height, px(48.));
            assert!(expand.top() >= handle.bottom());
            assert!(
                expand.bottom() <= panel.bottom(),
                "collapsed sheet keeps its action visible"
            );
            let destination = origin - point(px(0.), px(36.));
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: origin,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseMoveEvent {
                    position: destination,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            assert!(cx.has_active_drag());
            window.dispatch_event(
                MouseMoveEvent {
                    position: destination,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            let height = find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height;
            assert!(height > start_height && height < px(160.));
            (destination, height)
        })
        .unwrap()
    };
    let release = |cx: &mut gpui::TestAppContext, position| {
        cx.update_window(window.into(), |_, window, cx| {
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
    };
    let (destination, dragged_height) = drag(cx);
    cx.simulate_keystrokes(window.into(), "escape");
    release(cx, destination);
    cx.update(|cx| {
        assert!(!cx.has_active_drag());
        assert!(!view.read(cx).expanded);
    });
    let height = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height
        })
        .unwrap()
    };
    assert_eq!(height(cx), dragged_height);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    assert_eq!(height(cx), px(120.));

    let (destination, _) = drag(cx);
    release(cx, destination);
    cx.update(|cx| assert!(view.read(cx).expanded));
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn escape_closes_nested_dialogs_and_restores_each_trigger(cx: &mut gpui::TestAppContext) {
    use gpui::WindowOptions;
    use gpui_material::MaterialRoot;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Dialog, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let (first, second) = cx.update(|cx| {
        let view = view.read(cx);
        (view.primary.clone(), view.secondary.clone())
    });
    let trigger = cx
        .update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.focus_next(cx);
            let trigger = window.focused(cx).unwrap();
            first.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            second.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            trigger
        })
        .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| {
        assert!(first.read(cx).is_open());
        assert!(!second.read(cx).is_open());
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update_window(window.into(), |_, window, cx| {
        assert!(!first.read(cx).is_open());
        assert!(trigger.is_focused(window));
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dialog_body_stays_with_the_panel_during_dismissal(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, px};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let (window, dialog) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut dialog = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = ModalExamples::new(Story::Dialog, window, cx);
                dialog = Some(view.primary.clone());
                view
            })
        })
        .unwrap();
        (window, dialog.unwrap())
    });
    cx.update_window(window.into(), |_, window, cx| {
        dialog.update(cx, |dialog, cx| dialog.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(220));
    let body = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"dialog-content".into()).map(|body| body.bounds())
        })
        .unwrap()
    };
    let opened = body(cx).unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    let first_exit = body(cx).unwrap();
    assert_eq!(first_exit.size, opened.size);
    assert!((first_exit.top() - opened.top()).abs() <= px(1.));
    cx.executor().advance_clock(Duration::from_millis(60));
    let exiting = body(cx).unwrap();
    assert_eq!(opened.size, exiting.size);
    assert!(exiting.top() > opened.top() && exiting.top() < opened.top() + px(8.));
    cx.executor().advance_clock(Duration::from_millis(250));
    assert!(body(cx).is_none());
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn bottom_sheets_center_with_the_compose_width_limit(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, size};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;

    let (handle, examples) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut examples = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let view = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            examples = Some(view.clone());
            view
        })
        .unwrap();
        (handle, examples.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1024.), px(720.)));
    cx.update_window(handle.into(), |_, window, cx| {
        let state = examples.read(cx).primary.clone();
        state.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        for id in ["standard", "activity-sheet"] {
            let bounds = find(window, &[], &id.into()).unwrap().bounds();
            assert_eq!(bounds.size.width, px(640.));
            assert_eq!(bounds.center().x, window.viewport_size().width / 2.);
        }
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn modal_side_sheet_mirrors_and_fits_a_compact_scaled_window(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, size};
    use gpui_base::test_support::find;
    use gpui_material::{MaterialRoot, material_theme, set_material_theme, theme::Direction};

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(408.), px(420.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    let assert_side = |cx: &mut gpui::TestAppContext, side, reduced| {
        cx.update_window(window.into(), |_, window, cx| {
            cx.set_reduce_motion(reduced);
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = side;
            set_material_theme(theme, cx);
            let state = view.read(cx).secondary.clone();
            state.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            let bounds = find(window, &[], &"details-sheet".into()).unwrap().bounds();
            match (side, reduced) {
                (Direction::Ltr, false) => assert!(bounds.right() > window.viewport_size().width),
                (Direction::Rtl, false) => assert!(bounds.left() < px(0.)),
                (Direction::Ltr, true) => assert_eq!(bounds.right(), window.viewport_size().width),
                (Direction::Rtl, true) => assert_eq!(bounds.left(), px(0.)),
            }
        })
        .unwrap();
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(60));
        if !reduced {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let bounds = find(window, &[], &"details-sheet".into()).unwrap().bounds();
                match side {
                    Direction::Ltr => assert!(
                        bounds.left() < window.viewport_size().width
                            && bounds.right() > window.viewport_size().width
                    ),
                    Direction::Rtl => assert!(bounds.right() > px(0.) && bounds.left() < px(0.)),
                }
            })
            .unwrap();
        }
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(250));
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let sheet = find(window, &[], &"details-sheet".into()).unwrap();
            assert_eq!(sheet.role(), Some(gpui::Role::Dialog));
            let bounds = sheet.bounds();
            let viewport = window.viewport_size();
            assert!(bounds.size.width > px(0.) && bounds.size.width < viewport.width);
            assert_eq!(bounds.bottom(), viewport.height);
            match side {
                Direction::Ltr => assert_eq!(bounds.right(), viewport.width),
                Direction::Rtl => assert_eq!(bounds.left(), px(0.)),
            }
            let state = view.read(cx).secondary.clone();
            state.update(cx, |state, cx| state.close(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"sheet-content".into()).is_none(),
                reduced
            );
        })
        .unwrap();
        if !reduced {
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(60));
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let body = find(window, &[], &"sheet-content".into()).unwrap().bounds();
                match side {
                    Direction::Ltr => assert!(body.right() > window.viewport_size().width),
                    Direction::Rtl => assert!(body.left() < px(0.)),
                }
            })
            .unwrap();
        }
    };
    assert_side(cx, Direction::Ltr, false);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(250));
    assert_side(cx, Direction::Rtl, true);
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
            .items_start()
            .child(div().id("save-region").test_support().child(
                Button::new("save-message", "Save changes").on_activate(move |window, cx| {
                    timed.update(cx, |host, cx| {
                        host.push(Snackbar::new("Changes saved"), window, cx);
                    });
                }),
            ))
            .child(div().id("archive-region").test_support().child(
                Button::new("archive-message", "Archive draft").on_activate(move |window, cx| {
                    let host = persistent.downgrade();
                    persistent.update(cx, |queue, cx| {
                        queue.push(
                            Snackbar::new("Draft archived").action("Undo", move |window, cx| {
                                let _ = host.update(cx, |host, cx| {
                                    host.push(Snackbar::new("Draft restored"), window, cx);
                                });
                            }),
                            window,
                            cx,
                        );
                    });
                }),
            ))
            .child(self.host.clone())
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn snackbar_advances_after_timeout_and_accepts_undo(cx: &mut gpui::TestAppContext) {
    use gpui::{ElementId, InputEvent as _, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(SnackbarExamples::new)
        })
        .unwrap()
    });
    let click = |cx: &mut gpui::TestAppContext, id: ElementId| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let position = find(window, &[], &id).unwrap().bounds().center();
            window.dispatch_event(
                gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    click_count: 1,
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
                    click_count: 1,
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        cx.run_until_parked();
    };
    let message = |cx: &mut gpui::TestAppContext, id| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &("snackbar-message", id).into())
                .and_then(|message| message.label().map(str::to_owned))
        })
        .unwrap()
    };
    click(cx, "save-region".into());
    let snackbar_bounds = |cx: &mut gpui::TestAppContext, state, id| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &(state, id).into()).map(|surface| surface.bounds())
        })
        .unwrap()
    };
    let collapsed = snackbar_bounds(cx, "snackbar", 1usize).unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    let expanded = snackbar_bounds(cx, "snackbar", 1usize).unwrap();
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert!((collapsed.center().x - expanded.center().x).abs() <= px(1.));
    click(cx, "archive-region".into());
    assert_eq!(message(cx, 1usize).as_deref(), Some("Changes saved"));
    assert_eq!(message(cx, 2usize), None);
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(message(cx, 2usize).as_deref(), Some("Draft archived"));
    click(cx, "snackbar-action-region".into());
    assert_eq!(message(cx, 3usize).as_deref(), Some("Draft restored"));
}

pub(crate) struct MenuTooltipExamples {
    story: Story,
    trigger: FocusHandle,
    rich_trigger: FocusHandle,
    rich_action: FocusHandle,
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
            labels: false,
            output: "Sort by name".into(),
        }
    }
}

impl Render for MenuTooltipExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let tooltip_owner = owner.clone();
        let update = move |value: &'static str| {
            let owner = owner.clone();
            move |_: &mut Window, cx: &mut gpui::App| {
                let _ = owner.update(cx, |state, cx| {
                    state.output = value.into();
                    cx.notify();
                });
            }
        };
        let label_owner = cx.entity().downgrade();
        if self.story == Story::Menu {
            stack()
                .items_start()
                .child(Menu::new(
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
                            .on_activate(move |_, cx| {
                                let _ = label_owner.update(cx, |state, cx| {
                                    state.labels = !state.labels;
                                    cx.notify();
                                });
                            }),
                        MenuItem::new("columns", "Configure columns").disabled(true),
                        MenuItem::new("sort", "Sort by").submenu([
                            MenuItem::new("name", "Name")
                                .selected(self.output.as_ref() == "Sort by name")
                                .on_activate(update("Sort by name")),
                            MenuItem::new("date", "Date modified")
                                .selected(self.output.as_ref() == "Sort by date modified")
                                .on_activate(update("Sort by date modified")),
                        ]),
                    ],
                ))
                .child(
                    Menu::new(
                        "context-actions",
                        gpui_base::Button::new("open-context")
                            .accessibility_label("Roadmap context actions")
                            .px_3()
                            .py_2()
                            .rounded_sm()
                            .border_1()
                            .child("Roadmap"),
                        [
                            MenuItem::new("pin", "Pin roadmap")
                                .on_activate(update("Roadmap pinned")),
                        ],
                    )
                    .context(),
                )
                .child(Text::new(self.output.clone()))
                .into_any_element()
        } else {
            stack()
                .items_start()
                .child(
                    Tooltip::plain(
                        "pin-tooltip",
                        Button::new("pin", "Pin roadmap")
                            .track_focus(&self.trigger)
                            .on_activate(update("Roadmap pinned")),
                        "Keep this roadmap at the top of the project list",
                    )
                    .track_focus(&self.trigger),
                )
                .child(
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
                        move |_, cx| {
                            let _ = tooltip_owner.update(cx, |state, cx| {
                                state.output = "30 days".into();
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(Text::new(if self.output.as_ref() == "Sort by name" {
                    SharedString::default()
                } else {
                    self.output.clone()
                }))
                .into_any_element()
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn menu_skips_disabled_items_and_restores_trigger(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, WindowOptions};
    use gpui_material::MaterialRoot;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Menu, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.simulate_keystrokes(window.into(), "enter");
    frame(cx);
    frame(cx);
    let bounds = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, _| {
            gpui_base::test_support::find(window, &[], &"menu".into())
                .unwrap()
                .bounds()
        })
        .unwrap()
    };
    let collapsed = bounds(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(250));
    frame(cx);
    let expanded = bounds(cx);
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert_eq!(collapsed.top(), expanded.top());
    cx.update_window(window.into(), |_, window, _| {
        let row = gpui_base::test_support::find(window, &[], &"labels".into()).unwrap();
        assert_eq!(
            row.bounds().size.height,
            px(48.),
            "Compose MenuListItemContainerHeight"
        );
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "down");
    frame(cx);
    cx.simulate_keystrokes(window.into(), "right");
    frame(cx);
    frame(cx);
    cx.update_window(window.into(), |_, window, _| {
        assert!(
            gpui_base::test_support::find(window, &[], &"submenu-content".into()).is_some(),
            "submenu opened"
        );
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "down enter");
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let view = view.read(cx);
        assert_eq!(view.output.as_ref(), "Sort by date modified");
        assert!(view.trigger.is_focused(window));
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn submenu_accepts_pointer_selection_before_dismissing_its_parent(cx: &mut gpui::TestAppContext) {
    use gpui::{InputEvent as _, MouseButton, MouseDownEvent, MouseUpEvent, WindowOptions, point};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;
    let submenu_exit = (gpui::ElementId::from("options"), "submenu-exit-1").into();

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Menu, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "enter");
    frame(cx);
    frame(cx);
    cx.simulate_keystrokes(window.into(), "down right");
    frame(cx);
    frame(cx);
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let submenu = find(window, &[], &"submenu-content".into())
            .unwrap()
            .bounds();
        let position = point(submenu.center().x, submenu.top() + px(68.));
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &"submenu-content".into()).is_some());
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(view.read(cx).output.as_ref(), "Sort by date modified");
        assert!(find(window, &[], &"menu".into()).is_none());
        assert!(find(window, &[], &submenu_exit).is_some());
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    cx.update_window(window.into(), |_, window, _| {
        assert!(find(window, &[], &submenu_exit).is_none());
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn rich_tooltip_keeps_its_action_available_while_focused(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Tooltip, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"tooltip".into()).is_some()
        })
        .unwrap()
    };
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).rich_trigger.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(10));
    cx.run_until_parked();
    assert!(frame(cx));
    let bounds = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, _| {
            find(window, &[], &"tooltip".into()).unwrap().bounds()
        })
        .unwrap()
    };
    let collapsed = bounds(cx);
    cx.simulate_keystrokes(window.into(), "tab");
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        assert!(
            view.read(cx).rich_action.is_focused(window),
            "tooltip action focused"
        );
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    cx.run_until_parked();
    assert!(frame(cx));
    let expanded = bounds(cx);
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert!((collapsed.center().x - expanded.center().x).abs() <= px(1.));
    cx.simulate_keystrokes(window.into(), "enter");
    cx.update(|cx| assert_eq!(view.read(cx).output.as_ref(), "30 days"));
    cx.simulate_keystrokes(window.into(), "escape");
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, cx| {
        assert!(view.read(cx).rich_trigger.is_focused(window));
        assert!(find(window, &[], &"tooltip-exit".into()).is_some());
        assert!(find(window, &[], &"tooltip-action".into()).is_none());
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(200));
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, _| {
        assert!(find(window, &[], &"tooltip-exit".into()).is_none());
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn task_dialog_adapts_at_compact_breakpoint_without_losing_state(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, size};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let (window, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut state = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = ModalExamples::new(Story::Dialog, window, cx);
                state = Some(view.secondary.clone());
                view
            })
        })
        .unwrap();
        (window, state.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(400.), px(600.)));
    cx.update_window(window.into(), |_, window, cx| {
        state.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let panel = find(window, &[], &"guidelines-dialog".into())
            .unwrap()
            .bounds();
        let bar = find(window, &[], &"dialog-app-bar".into())
            .unwrap()
            .bounds();
        let body = find(window, &[], &"dialog-content".into())
            .unwrap()
            .bounds();
        assert_eq!(panel.size, window.viewport_size());
        assert_eq!(bar.size.height, px(56.));
        assert!(body.top() >= bar.bottom());
        assert!(body.bottom() <= panel.bottom());
    })
    .unwrap();
    let focus = cx
        .update_window(window.into(), |_, window, cx| {
            window.focus_next(cx);
            window.focused(cx).unwrap()
        })
        .unwrap();
    cx.simulate_window_resize(window.into(), size(px(800.), px(600.)));
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(state.read(cx).is_open());
        assert!(
            focus.is_focused(window),
            "resizing preserves the keyboard focus handle"
        );
        assert!(find(window, &[], &"dialog-app-bar".into()).is_none());
        let panel = find(window, &[], &"guidelines-dialog".into())
            .unwrap()
            .bounds();
        assert_eq!(panel.size.width, px(560.));
        assert!(panel.size.height < window.viewport_size().height);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| assert!(!state.read(cx).is_open()));
}
