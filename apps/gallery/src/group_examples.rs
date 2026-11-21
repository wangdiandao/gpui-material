use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FocusHandle, ParentElement, Radians, Render, SharedString, Styled, Window, div, px,
};
use gpui_material::{
    ButtonGroup, ButtonGroupItem, ButtonShape, ButtonVariant, ControlSize, Icon, IconName,
    MenuItem, SplitButton, Text,
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod button_group_tests;
#[cfg(all(test, feature = "test-support"))]
mod split_button_tests;

pub(crate) struct GroupExamples {
    story: Story,
    selected: SharedString,
    optional_filter: Option<SharedString>,
    message: SharedString,
    split_focus: (FocusHandle, FocusHandle),
    keep_export: bool,
    keep_archive: bool,
}

impl GroupExamples {
    pub(crate) fn new(story: Story, cx: &Context<Self>) -> Self {
        Self {
            story,
            selected: "week".into(),
            optional_filter: Some("active".into()),
            message: "No action yet".into(),
            split_focus: (cx.focus_handle(), cx.focus_handle()),
            keep_export: false,
            keep_archive: false,
        }
    }

    fn command(
        message: &'static str,
        cx: &Context<Self>,
    ) -> impl Fn(&mut Window, &mut gpui::App) + use<> {
        let view = cx.weak_entity();
        move |_, cx| {
            let _ = view.update(cx, |this, cx| {
                this.message = message.into();
                cx.notify();
            });
        }
    }
}

impl Render for GroupExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let mut body = stack();
        if self.story == Story::ButtonGroup {
            let archive = cx.weak_entity();
            let keep = self.keep_archive;
            body = body.child(
                section("Standard commands and toggle").child(ButtonGroup::new(
                    "file-actions",
                    "File actions",
                    [
                        ButtonGroupItem::new("copy", "Copy")
                            .leading_icon(|| Icon::new(IconName::Description))
                            .action(Self::command("Copied", cx)),
                        ButtonGroupItem::new("rename", "Rename")
                            .on_activate(Self::command("Renamed", cx)),
                        ButtonGroupItem::new("archive", "Archive").disabled(true),
                        ButtonGroupItem::new("keep-archive", "Keep archive")
                            .icon_only(move || Icon::new(IconName::Favorite).filled(keep))
                            .toggle(keep, move |keep, _, cx| {
                                let _ = archive.update(cx, |this, cx| {
                                    this.keep_archive = keep;
                                    this.message = if keep {
                                        "Keeping archive"
                                    } else {
                                        "Archive unpinned"
                                    }
                                    .into();
                                    cx.notify();
                                });
                            }),
                    ],
                )),
            );
            body = body.child(
                section("Connected single selection")
                    .description("Select a time period. The group keeps one option selected.")
                    .child(
                        div().max_w(px(290.)).child(
                            ButtonGroup::connected(
                                "period",
                                "Period",
                                [
                                    ButtonGroupItem::new("day", "Day").weight(1.),
                                    ButtonGroupItem::new("week", "Week").weight(2.),
                                    ButtonGroupItem::new("month", "Month"),
                                    ButtonGroupItem::new("quarter", "Quarter"),
                                    ButtonGroupItem::new("year", "Year"),
                                ],
                            )
                            .shape(ButtonShape::Square)
                            .appearance(ButtonVariant::FilledTonal)
                            .expanded_ratio(0.2)
                            .overflow_trigger(px(52.), |open| {
                                Icon::new(IconName::ArrowDropDown)
                                    .size(px(24.))
                                    .rotate(Radians(if open { std::f32::consts::PI } else { 0. }))
                            })
                            .single_select(self.selected.clone(), {
                                let view = cx.weak_entity();
                                move |next, _, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        this.message = format!("Period: {next}").into();
                                        this.selected = next;
                                        cx.notify();
                                    });
                                }
                            }),
                        ),
                    ),
            );
            body = body.child(
                section("Optional single selection")
                    .description(
                        "Click an active option again to clear the filter via single_optional_select.",
                    )
                    .child(
                        ButtonGroup::connected(
                            "status-filter",
                            "Status filter",
                            [
                                ButtonGroupItem::new("active", "Active"),
                                ButtonGroupItem::new("archived", "Archived"),
                                ButtonGroupItem::new("draft", "Draft"),
                            ],
                        )
                        .appearance(ButtonVariant::Outlined)
                        .single_optional_select(self.optional_filter.clone(), {
                            let view = cx.weak_entity();
                            move |next, _, cx| {
                                let _ = view.update(cx, |this, cx| {
                                    this.message = match &next {
                                        Some(id) => format!("Filter: {id}").into(),
                                        None => "Filter cleared".into(),
                                    };
                                    this.optional_filter = next;
                                    cx.notify();
                                });
                            }
                        }),
                    ),
            );
        } else {
            let owner = cx.weak_entity();
            body = body.child(
                section("Command and toggle")
                    .child(
                        SplitButton::new("export-split", "Export report", "Keep latest export", [])
                            .variant(ButtonVariant::FilledTonal)
                            .leading_icon(Icon::new(IconName::Description))
                            .trailing_icon(Icon::new(IconName::Favorite).filled(self.keep_export))
                            .spacing(px(4.))
                            .on_activate(Self::command("Exported report", cx))
                            .trailing_toggle(self.keep_export, move |selected, _, cx| {
                                let _ = owner.update(cx, |this, cx| {
                                    this.keep_export = selected;
                                    this.message = if selected {
                                        "Keeping latest export"
                                    } else {
                                        "Export unpinned"
                                    }
                                    .into();
                                    cx.notify();
                                });
                            }),
                    )
                    .child(Text::new(self.message.clone()).id("split-action-result")),
            );
            let sizes = [
                (
                    "Extra small",
                    ControlSize::ExtraSmall,
                    ButtonVariant::Elevated,
                ),
                ("Small", ControlSize::Small, ButtonVariant::Filled),
                ("Medium", ControlSize::Medium, ButtonVariant::FilledTonal),
                ("Large", ControlSize::Large, ButtonVariant::Outlined),
                (
                    "Extra large",
                    ControlSize::ExtraLarge,
                    ButtonVariant::Filled,
                ),
            ];
            for (title, size, variant) in sizes {
                body = body.child(
                    section(title).child(
                        SplitButton::new(
                            title,
                            "Save",
                            "More save options",
                            [
                                MenuItem::new("copy", "Save a copy")
                                    .on_activate(Self::command("Saved a copy", cx)),
                                MenuItem::new("template", "Save as template")
                                    .on_activate(Self::command("Saved as template", cx)),
                            ],
                        )
                        .size(size)
                        .variant(variant)
                        .when(size == ControlSize::ExtraSmall, |button| {
                            button.track_focus(&self.split_focus.0, &self.split_focus.1)
                        })
                        .on_activate(Self::command("Saved", cx)),
                    ),
                );
            }
        }
        div()
            .w_full()
            .child(body)
            .children((self.story == Story::ButtonGroup).then(|| Text::new(self.message.clone())))
    }
}
#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn split_button_keeps_primary_activation_and_menu_focus_separate(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, InputEvent as _};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| GroupExamples::new(Story::SplitButton, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };

    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        view.read(cx).split_focus.0.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(view.read(cx).split_focus.0.is_focused(window));
        let keystroke = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
    })
    .unwrap();
    frame(cx);
    cx.update(|cx| assert_eq!(view.read(cx).message, "Saved"));

    cx.update_window(handle.into(), |_, window, cx| {
        view.read(cx).split_focus.1.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.simulate_keystrokes(handle.into(), "enter");
    frame(cx);
    frame(cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"copy".into()).is_some());
    })
    .unwrap();

    cx.simulate_keystrokes(handle.into(), "escape");
    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(view.read(cx).split_focus.1.is_focused(window));
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "space");
    frame(cx);
    frame(cx);
    cx.simulate_keystrokes(handle.into(), "down enter");
    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(view.read(cx).message, "Saved as template");
        assert!(view.read(cx).split_focus.1.is_focused(window));
    })
    .unwrap();
}
