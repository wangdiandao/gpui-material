use gpui::{
    Axis, Context, ElementId, FocusHandle, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_material::foundation::window_material_theme;
use gpui_material::theme::Direction;
use gpui_material::{Checkbox, CheckboxState, Chip, ChipVariant, Radio, RadioGroup, Switch};

pub(crate) struct SelectionExamples {
    colors: [CheckboxState; 2],
    alignment: ElementId,
    notifications: bool,
    plain_switch: bool,
    filter: bool,
    chip_selected: bool,
    chip_visible: bool,
    restore: FocusHandle,
}

impl SelectionExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            colors: [CheckboxState::Checked, CheckboxState::Unchecked],
            alignment: "start".into(),
            notifications: false,
            plain_switch: false,
            filter: false,
            chip_selected: false,
            chip_visible: true,
            restore: cx.focus_handle(),
        }
    }
}

impl Render for SelectionExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_lg().child("Selection"))
            .child(
                Checkbox::new(
                    "palette-all",
                    "All colors",
                    Checkbox::aggregate(self.colors),
                )
                .on_change(move |next, _, cx| {
                    owner.update(cx, |this, cx| {
                        this.colors.fill(next);
                        cx.notify();
                    })
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .children(
                        [("warm", "Warm"), ("cool", "Cool")]
                            .into_iter()
                            .enumerate()
                            .map(|(ix, (id, label))| {
                                let owner = cx.entity();
                                Checkbox::new(id, label, self.colors[ix]).on_change(
                                    move |next, _, cx| {
                                        owner.update(cx, |this, cx| {
                                            this.colors[ix] = next;
                                            cx.notify();
                                        });
                                    },
                                )
                            }),
                    ),
            )
            .child(
                Checkbox::new(
                    "invalid-option",
                    "Required option",
                    CheckboxState::Unchecked,
                )
                .error(true),
            )
            .child(
                Checkbox::new("disabled-option", "Unavailable", CheckboxState::Checked)
                    .disabled(true),
            )
            .child({
                let owner = cx.entity();
                RadioGroup::new("alignment", "Alignment")
                    .axis(Axis::Horizontal)
                    .selected(self.alignment.clone())
                    .options([
                        Radio::new("start", "Start"),
                        Radio::new("center", "Center").disabled(true),
                        Radio::new("end", "End"),
                    ])
                    .on_select(move |id, _, cx| {
                        owner.update(cx, |this, cx| {
                            this.alignment = id.clone();
                            cx.notify();
                        });
                    })
            })
            .child({
                let owner = cx.entity();
                Switch::new("notifications", "Notifications", self.notifications)
                    .checked_icon("✓")
                    .unchecked_icon("−")
                    .on_change(move |next, _, cx| {
                        owner.update(cx, |this, cx| {
                            this.notifications = next;
                            cx.notify();
                        });
                    })
            })
            .child({
                let owner = cx.entity();
                Switch::new("sync-switch", "Sync automatically", self.plain_switch).on_change(
                    move |next, _, cx| {
                        owner.update(cx, |this, cx| {
                            this.plain_switch = next;
                            cx.notify();
                        })
                    },
                )
            })
            .child(Switch::new("disabled-switch", "Sync locked on", true).disabled(true))
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .child({
                        let owner = cx.entity();
                        gpui_material::Button::new("restore-chip", "Restore tag")
                            .variant(gpui_material::ButtonVariant::Text)
                            .track_focus(&self.restore)
                            .on_activate(move |_, cx| {
                                owner.update(cx, |this, cx| {
                                    this.chip_visible = true;
                                    cx.notify();
                                })
                            })
                    })
                    .child(
                        Chip::new("assist", "Browse")
                            .leading_icon("+")
                            .elevated(true),
                    )
                    .child({
                        let owner = cx.entity();
                        Chip::new("filter", "Recent")
                            .variant(ChipVariant::Filter)
                            .selected(self.filter)
                            .on_change(move |value, _, cx| {
                                owner.update(cx, |this, cx| {
                                    this.filter = value;
                                    cx.notify();
                                })
                            })
                    })
                    .children(self.chip_visible.then(|| {
                        let remove = cx.entity();
                        let select = cx.entity();
                        Chip::new("tag", "Project")
                            .variant(ChipVariant::Input)
                            .selected(self.chip_selected)
                            .on_change(move |value, _, cx| {
                                select.update(cx, |this, cx| {
                                    this.chip_selected = value;
                                    cx.notify();
                                })
                            })
                            .on_remove(
                                "Remove Project tag",
                                Some(self.restore.clone()),
                                move |_, cx| {
                                    remove.update(cx, |this, cx| {
                                        this.chip_visible = false;
                                        cx.notify();
                                    });
                                },
                            )
                    }))
                    .child(Chip::new("suggestion", "Suggested").variant(ChipVariant::Suggestion)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .child(
                        Chip::new("disabled-tag", "Locked tag")
                            .variant(ChipVariant::Input)
                            .selected(true)
                            .disabled(true)
                            .on_remove("Remove locked tag", None, |_, _| {}),
                    )
                    .child(
                        Chip::new("elevated-suggestion", "Nearby")
                            .variant(ChipVariant::Suggestion)
                            .elevated(true),
                    ),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, px, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn controlled_selection_and_chip_removal(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let selection = cx.new(SelectionExamples::new);
                view = Some(selection.clone());
                selection
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(850.), px(640.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            let assert_switch_centered = |window: &Window| {
                let id = ElementId::from("notifications");
                let track = find(window, &[], &(id.clone(), "switch-track").into()).unwrap();
                let thumb = find(window, &[], &(id, "switch-thumb").into()).unwrap();
                assert_eq!(track.bounds().center().y, thumb.bounds().center().y);
            };
            draw(window, cx);
            assert_switch_centered(window);
            for (scope, id) in [
                (vec![], "palette-all".into()),
                (vec![], "notifications".into()),
                (vec![], ElementId::from((ElementId::from("tag"), "remove"))),
            ] {
                let position = find(window, &scope, &id)
                    .unwrap_or_else(|| {
                        panic!(
                            "missing {id:?} in {scope:?}, observed {:?}",
                            gpui_base::test_support::snapshots(window)
                                .iter()
                                .map(|snapshot| snapshot.label())
                                .collect::<Vec<_>>()
                        )
                    })
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
                draw(window, cx);
                assert_switch_centered(window);
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
                draw(window, cx);
                assert_switch_centered(window);
            }
            let state = view.read(cx);
            assert_eq!(state.colors, [CheckboxState::Checked; 2]);
            assert!(state.notifications);
            assert!(!state.chip_visible);
            assert!(!state.chip_selected);
            assert!(state.restore.is_focused(window));
        })
        .unwrap();
    }
}
