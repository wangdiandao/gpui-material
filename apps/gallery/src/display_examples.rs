use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_material_components::{
    Badge, Button, ButtonVariant, Card, CardVariant, CircularProgress, Divider, Icon,
    LinearProgress, List, ListItem, ListVariant, LoadingIndicator, ProgressShape, Text,
};
use gpui_material_theme::TypeRole;

pub(crate) struct DisplayExamples {
    progress: f32,
    loading: bool,
    selected: bool,
    opened: bool,
    child_count: usize,
}

impl Default for DisplayExamples {
    fn default() -> Self {
        Self {
            progress: 0.35,
            loading: false,
            selected: false,
            opened: false,
            child_count: 0,
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
            .child(LinearProgress::new("linear-progress", "Transfer progress").value(self.progress))
            .child(
                LinearProgress::new("linear-wave", "Wavy transfer progress")
                    .value(self.progress)
                    .shape(ProgressShape::Wavy),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        CircularProgress::new("circular-progress", "Sync progress")
                            .value(self.progress),
                    )
                    .child(
                        CircularProgress::new("circular-wave", "Wavy sync progress")
                            .shape(ProgressShape::Wavy)
                            .value(self.progress),
                    )
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
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, size,
    };
    use gpui_base::test_support::find;
    use gpui_material_foundation::MaterialRoot;

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
