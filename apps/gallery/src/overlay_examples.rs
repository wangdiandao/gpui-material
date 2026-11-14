use gpui::{AppContext, Entity, InteractiveElement, div, px};
use gpui::{
    Context, FocusHandle, IntoElement, ParentElement, Render, SharedString, Styled, Window,
};
use gpui_base::TestSupportExt as _;
use gpui_material::{Button, ButtonVariant, Menu, MenuItem, Snackbar, SnackbarHost, Text, Tooltip};

use crate::story::{Story, stack};

pub(crate) struct MenuTooltipExamples {
    story: Story,
    trigger: FocusHandle,
    rich_trigger: FocusHandle,
    labels: bool,
    output: SharedString,
}

impl MenuTooltipExamples {
    pub(crate) fn new(story: Story, cx: &mut Context<Self>) -> Self {
        Self {
            story,
            trigger: cx.focus_handle(),
            rich_trigger: cx.focus_handle(),
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
                        Button::new("dismiss-policy", "Show retention period")
                            .variant(ButtonVariant::Text)
                            .on_activate(move |_, cx| {
                                let _ = tooltip_owner.update(cx, |state, cx| {
                                    state.output = "30 days".into();
                                    cx.notify();
                                });
                            }),
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
    cx.update_window(window.into(), |_, window, _| {
        assert!(
            gpui_base::test_support::find(window, &[], &"menu".into()).is_some(),
            "menu opened"
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
