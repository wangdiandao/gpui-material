use gpui::InteractiveElement;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_base::TestSupportExt as _;
use gpui_base::input::{Input, InputState};
use gpui_material::{
    Button, Dialog, DialogState, DialogVariant, Sheet, SheetEdge, SheetMode, Snackbar, SnackbarHost,
};

use crate::story::{Story, stack};

pub(crate) struct ModalExamples {
    story: Story,
    primary: Entity<DialogState>,
    secondary: Entity<DialogState>,
    standard: Entity<DialogState>,
    name: Entity<InputState>,
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
            name: cx.new(|cx| InputState::new(window, cx).placeholder("名称")),
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
                                div()
                                    .rounded_sm()
                                    .border_1()
                                    .p_2()
                                    .child(Input::new(&editor)),
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
    click(cx, "archive-region".into());
    assert_eq!(message(cx, 1usize).as_deref(), Some("Changes saved"));
    assert_eq!(message(cx, 2usize), None);
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(message(cx, 2usize).as_deref(), Some("Draft archived"));
    click(cx, "snackbar-action-region".into());
    assert_eq!(message(cx, 3usize).as_deref(), Some("Draft restored"));
}
