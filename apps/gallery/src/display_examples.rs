use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_material::theme::TypeRole;
use gpui_material::{
    Badge, Button, ButtonVariant, Card, CardVariant, CircularProgress, Divider, Icon,
    LinearProgress, List, ListItem, ListVariant, LoadingIndicator, ProgressShape, Text,
};

pub(crate) struct DisplayExamples {
    progress: f32,
    loading: bool,
    selected: bool,
    opened: bool,
    child_count: usize,
    indeterminate: bool,
    selected_file: Option<usize>,
    pinned_file: Option<usize>,
    files_scroll: ScrollHandle,
}

impl Default for DisplayExamples {
    fn default() -> Self {
        Self {
            progress: 0.35,
            loading: false,
            selected: false,
            opened: false,
            child_count: 0,
            indeterminate: false,
            selected_file: None,
            pinned_file: None,
            files_scroll: ScrollHandle::new(),
        }
    }
}

impl Render for DisplayExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let card = cx.entity();
        let child = cx.entity();
        let list = cx.entity();
        let progress = cx.entity();
        let loading = cx.entity();
        let mode = cx.entity();
        let linear = |id, name, shape| {
            let progress = LinearProgress::new(id, name).shape(shape);
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };
        let circular = |id, name, shape| {
            let progress = CircularProgress::new(id, name).shape(shape);
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Text::new("Display")
                    .role(TypeRole::TitleLarge)
                    .id("display-heading"),
            )
            .child(
                Text::new("Material typography 中文混排")
                    .id("mixed-script-text")
                    .role(TypeRole::TitleMedium)
                    .emphasized(true)
                    .font_scale(1.25)
                    .truncate(px(350.)),
            )
            .child(Text::new("Material typography 中文混排").id("baseline-text"))
            .child(
                Card::new("elevated-card")
                    .variant(CardVariant::Elevated)
                    .child(Text::new("Elevated card")),
            )
            .child(
                Card::new("filled-card")
                    .on_activate("Open project", move |_, cx| {
                        card.update(cx, |this, cx| {
                            this.opened = !this.opened;
                            cx.notify();
                        });
                    })
                    .child(Text::new(if self.opened {
                        "Project open"
                    } else {
                        "Project"
                    }))
                    .child(
                        Button::new("card-child", format!("Pin ({})", self.child_count))
                            .variant(ButtonVariant::Outlined)
                            .on_activate(move |_, cx| {
                                child.update(cx, |this, cx| {
                                    this.child_count += 1;
                                    cx.notify();
                                })
                            }),
                    ),
            )
            .child(
                Card::new("outlined-card")
                    .variant(CardVariant::Outlined)
                    .disabled(true)
                    .child(Text::new("Unavailable card")),
            )
            .child(Divider::horizontal().inset(px(16.), px(0.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .h_10()
                    .child(
                        Icon::svg(crate::FORWARD_ICON)
                            .mirror_in_rtl(true)
                            .label("forward", "Forward"),
                    )
                    .child(Divider::vertical().inset(px(4.), px(8.)))
                    .child(Icon::svg(crate::FORWARD_ICON))
                    .child(Badge::count(
                        Icon::custom("▤").label("inbox", "Inbox"),
                        8,
                        99,
                    ))
                    .child(Badge::label(
                        Icon::custom("▤").label("new-file", "Files"),
                        "New",
                    )),
            )
            .child(
                List::new("segmented-list")
                    .variant(ListVariant::Segmented)
                    .items([
                        ListItem::new("documents", "Documents")
                            .selected(self.selected)
                            .supporting_text("12 items")
                            .leading(Icon::custom("▤"))
                            .on_activate(move |_, cx| {
                                list.update(cx, |this, cx| {
                                    this.selected = !this.selected;
                                    cx.notify();
                                })
                            }),
                        ListItem::new("archives", "Archives")
                            .overline("Storage")
                            .supporting_text("Last updated today")
                            .trailing(Badge::count(
                                Icon::custom("□")
                                    .label("archive-status", "More than 99 archived items"),
                                125,
                                99,
                            )),
                        ListItem::new("shared", "Shared").trailing(Badge::dot(
                            Icon::custom("◆").label("shared-status", "New shared activity"),
                        )),
                    ]),
            )
            .child(linear(
                "linear-progress",
                "Transfer progress",
                ProgressShape::Straight,
            ))
            .child(linear(
                "linear-wave",
                "Wavy transfer progress",
                ProgressShape::Wavy,
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(circular(
                        "circular-progress",
                        "Sync progress",
                        ProgressShape::Straight,
                    ))
                    .child(circular(
                        "circular-wave",
                        "Wavy sync progress",
                        ProgressShape::Wavy,
                    ))
                    .child(
                        LoadingIndicator::new("loading", "Loading documents").active(self.loading),
                    )
                    .child(
                        LoadingIndicator::new("contained-loading", "Loading archives")
                            .contained(true)
                            .active(self.loading),
                    )
                    .child(
                        Button::new("advance-progress", "Advance").on_activate(move |_, cx| {
                            progress.update(cx, |this, cx| {
                                this.progress = (this.progress + 0.25) % 1.;
                                cx.notify();
                            })
                        }),
                    )
                    .child(
                        Button::new(
                            "toggle-loading",
                            if self.loading {
                                "Stop loading"
                            } else {
                                "Start loading"
                            },
                        )
                        .variant(ButtonVariant::Outlined)
                        .on_activate(move |_, cx| {
                            loading.update(cx, |this, cx| {
                                this.loading = !this.loading;
                                cx.notify();
                            })
                        }),
                    ),
            )
            .child(
                Button::new(
                    "progress-mode",
                    if self.indeterminate {
                        "Use measured progress"
                    } else {
                        "Use indeterminate progress"
                    },
                )
                .on_activate(move |_, cx| {
                    mode.update(cx, |this, cx| {
                        this.indeterminate = !this.indeterminate;
                        cx.notify();
                    })
                }),
            )
            .child(
                Text::new("Files")
                    .id("files-title")
                    .role(TypeRole::TitleMedium),
            )
            .child(
                div()
                    .id("files-scroll")
                    .track_scroll(&self.files_scroll)
                    .overflow_y_scroll()
                    .h_48()
                    .child(List::new("standard-list").items((1..=24).map(|file| {
                        let select = cx.entity();
                        let pin = cx.entity();
                        let pinned = self.pinned_file == Some(file);
                        ListItem::new(("file", file), format!("File {file}"))
                            .selected(self.selected_file == Some(file))
                            .on_activate(move |_, cx| {
                                select.update(cx, |this, cx| {
                                    this.selected_file = Some(file);
                                    cx.notify();
                                })
                            })
                            .trailing(
                                Button::new(
                                    ("pin-file", file),
                                    if pinned {
                                        format!("Unpin file {file}")
                                    } else {
                                        format!("Pin file {file}")
                                    },
                                )
                                .variant(ButtonVariant::Text)
                                .size(gpui_material::ControlSize::ExtraSmall)
                                .on_activate(move |_, cx| {
                                    pin.update(cx, |this, cx| {
                                        this.pinned_file = (!pinned).then_some(file);
                                        cx.notify();
                                    })
                                }),
                            )
                    }))),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn nested_card_action_does_not_activate_its_parent(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let display = cx.new(|_| DisplayExamples::default());
                view = Some(display.clone());
                display
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(650.), px(870.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            let position = find(window, &[], &"card-child".into())
                .unwrap()
                .bounds()
                .center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(view.read(cx).child_count, 1);
            assert!(!view.read(cx).opened);
        })
        .unwrap();
    }
}
