use gpui::{
    AppContext, Axis, Context, ElementId, Entity, FocusHandle, IntoElement, ParentElement, Render,
    StrokeOptions, Styled, Window, div, prelude::FluentBuilder, px, rgb,
};
use gpui_material::foundation::{color, window_material_theme};
use gpui_material::theme::{ColorRole, Direction};
use gpui_material::{
    Checkbox, CheckboxColors, CheckboxState, CheckboxStateColors, Chip, ChipVariant, Icon,
    IconName, InteractionSource, Radio, RadioColors, RadioGroup, Switch, Text,
};

use crate::story::{Story, section};

#[cfg(all(test, feature = "test-support"))]
mod checkbox_tests;
#[cfg(all(test, feature = "test-support"))]
mod radio_tests;

pub(crate) struct SelectionExamples {
    story: Option<Story>,
    colors: [CheckboxState; 2],
    alignment: ElementId,
    density: ElementId,
    notifications: bool,
    plain_switch: bool,
    filter: bool,
    filter_removed: bool,
    chip_selected: bool,
    chip_visible: bool,
    restore: FocusHandle,
    styled_checkbox: CheckboxState,
    checkbox_source: Option<Entity<InteractionSource>>,
}

impl SelectionExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            story: None,
            colors: [CheckboxState::Checked, CheckboxState::Unchecked],
            alignment: "start".into(),
            density: "standard".into(),
            notifications: false,
            plain_switch: false,
            filter: false,
            filter_removed: false,
            chip_selected: false,
            chip_visible: true,
            restore: cx.focus_handle(),
            styled_checkbox: CheckboxState::Indeterminate,
            checkbox_source: None,
        }
    }
    pub(crate) fn for_story(story: Story, cx: &mut Context<Self>) -> Self {
        Self {
            story: Some(story),
            ..Self::new(cx)
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }

    fn custom_checkboxes(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> crate::story::StorySection {
        let source = self.checkbox_source.get_or_insert_with(|| {
            let source = cx.new(|_| InteractionSource::new());
            cx.observe(&source, |_, _, cx| cx.notify()).detach();
            source
        });
        let window_id = window.window_handle().window_id();
        let interaction = source.read(cx).state(window_id, &"styled-mixed".into());
        let status = match interaction {
            Some(state) if state.pressed => "Pressed",
            Some(state) if state.focus_visible => "Keyboard focus",
            Some(state) if state.hovered => "Hovered",
            _ => "Ready",
        };
        let colors = custom_checkbox_colors();
        let stroke = StrokeOptions::default().with_line_width(3.5);
        let owner = cx.entity();
        let examples = [
            (
                "styled-unchecked",
                "Unchecked",
                CheckboxState::Unchecked,
                false,
                false,
            ),
            (
                "styled-checked",
                "Checked",
                CheckboxState::Checked,
                false,
                false,
            ),
            (
                "styled-mixed",
                "Change selection",
                self.styled_checkbox,
                false,
                false,
            ),
            (
                "styled-error",
                "Error",
                CheckboxState::Indeterminate,
                true,
                false,
            ),
            (
                "styled-disabled",
                "Disabled",
                CheckboxState::Checked,
                false,
                true,
            ),
        ];
        section("Custom appearance")
            .child(div().flex().flex_wrap().gap_2().children(examples.map(
                |(id, label, state, error, disabled)| {
                    let control = Checkbox::new(id, label, state)
                        .colors(colors)
                        .checkmark_stroke(stroke)
                        .outline_stroke(px(1.))
                        .error(error)
                        .disabled(disabled)
                        .interaction_source(source);
                    if id == "styled-mixed" {
                        let owner = owner.clone();
                        control.on_change(move |next, _, cx| {
                            owner.update(cx, |this, cx| {
                                this.styled_checkbox = next;
                                cx.notify();
                            });
                        })
                    } else {
                        control
                    }
                },
            )))
            .child(Text::new(format!("Change selection · {status}")))
    }
}

fn custom_checkbox_colors() -> CheckboxColors {
    let paint = |box_color, border, mark| CheckboxStateColors {
        box_color: Some(rgb(box_color).into()),
        border: Some(rgb(border).into()),
        checkmark: Some(rgb(mark).into()),
    };
    CheckboxColors {
        unchecked: paint(0xe8ecf6, 0x4b536b, 0x29364f),
        checked: paint(0x285844, 0x182f29, 0xffeed6),
        indeterminate: paint(0x415380, 0x29364f, 0xffeed6),
        error_indeterminate: paint(0x8b3a31, 0x682c32, 0xffffff),
        disabled_checked: paint(0xbbb7be, 0x76717a, 0xf6f1f5),
        ..Default::default()
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
            .children(self.shows(Story::Checkbox).then(|| {
                section("Group selection and validation")
                    .description(
                        "Select Warm or Cool to see how All colors reflects a partial selection.",
                    )
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
            }))
            .children(
                (self.story == Some(Story::Checkbox)).then(|| self.custom_checkboxes(window, cx)),
            )
            .children(self.shows(Story::Radio).then(|| {
                let theme = window_material_theme(window, cx);
                let tone = |role| color(theme.colors.color(role));
                let alignment_colors = RadioColors {
                    selected: Some(tone(ColorRole::Secondary)),
                    unselected: Some(tone(ColorRole::Outline)),
                    ..Default::default()
                };
                let density_colors = RadioColors {
                    selected: Some(tone(ColorRole::Tertiary)),
                    unselected: Some(tone(ColorRole::OnSurfaceVariant)),
                    ..Default::default()
                };
                let radio_story = self.story == Some(Story::Radio);
                section(if radio_story {
                    "Alignment and density"
                } else {
                    "Horizontal group"
                })
                .description("Use the arrow keys to navigate each group.")
                .child({
                    let owner = cx.entity();
                    RadioGroup::new("alignment", "Alignment")
                        .axis(Axis::Horizontal)
                        .selected(self.alignment.clone())
                        .options(
                            [
                                Radio::new("start", "Start"),
                                Radio::new("center", "Center").disabled(true),
                                Radio::new("end", "End"),
                            ]
                            .map(|radio| radio.colors(alignment_colors)),
                        )
                        .on_select(move |id, _, cx| {
                            owner.update(cx, |this, cx| {
                                this.alignment = id.clone();
                                cx.notify();
                            });
                        })
                })
                .children(radio_story.then(|| {
                    let owner = cx.entity();
                    RadioGroup::new("density", "Density")
                        .selected(self.density.clone())
                        .options(
                            [
                                Radio::new("compact", "Compact"),
                                Radio::new("standard", "Standard"),
                                Radio::new("comfortable", "Comfortable"),
                            ]
                            .map(|radio| radio.colors(density_colors)),
                        )
                        .on_select(move |id, _, cx| {
                            owner.update(cx, |this, cx| {
                                this.density = id.clone();
                                cx.notify();
                            });
                        })
                }))
            }))
            .children(self.shows(Story::Switch).then(|| {
                section("With icons, without icons and disabled")
                    .child({
                        let owner = cx.entity();
                        Switch::new("notifications", "Notifications", self.notifications)
                            .checked_icon(Icon::new(IconName::Check))
                            .unchecked_icon(Icon::new(IconName::Remove))
                            .on_change(move |next, _, cx| {
                                owner.update(cx, |this, cx| {
                                    this.notifications = next;
                                    cx.notify();
                                });
                            })
                    })
                    .child({
                        let owner = cx.entity();
                        Switch::new("sync-switch", "Sync automatically", self.plain_switch)
                            .on_change(move |next, _, cx| {
                                owner.update(cx, |this, cx| {
                                    this.plain_switch = next;
                                    cx.notify();
                                })
                            })
                    })
                    .child(Switch::new("disabled-switch", "Sync locked on", true).disabled(true))
            }))
            .children(self.shows(Story::Chip).then(|| {
                section("Assist, filter, input and suggestion")
                    .description(
                        "Toggle Recent, or remove the tag and use Restore tag to bring it back.",
                    )
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
                                    .leading_icon(Icon::new(IconName::Add))
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
                            .children((!self.filter_removed).then(|| {
                                let owner = cx.entity();
                                let remove = owner.clone();
                                Chip::new("removable-filter", "Downloads")
                                    .variant(ChipVariant::Filter)
                                    .selected(self.filter)
                                    .on_change(move |value, _, cx| {
                                        owner.update(cx, |this, cx| {
                                            this.filter = value;
                                            cx.notify();
                                        })
                                    })
                                    .on_remove(
                                        "Remove downloads filter",
                                        Some(self.restore.clone()),
                                        move |_, cx| {
                                            remove.update(cx, |this, cx| {
                                                this.filter_removed = true;
                                                cx.notify();
                                            })
                                        },
                                    )
                            }))
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
                            .child(
                                Chip::new("suggestion", "Suggested")
                                    .variant(ChipVariant::Suggestion),
                            ),
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
            }))
            .children((self.story == Some(Story::Chip)).then(chip_content_examples))
    }
}

fn chip_content_examples() -> impl IntoElement {
    use gpui::{px, rgb};
    use gpui_material::{ButtonPadding, ChipColors, ChipStateColors};

    section("Custom chip content").child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                Chip::new("chip-avatar-example", "Workspace attendee")
                    .variant(ChipVariant::Input)
                    .tonal(true)
                    .avatar(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(0xd6e4ff))
                            .text_color(rgb(0x153a65))
                            .text_size(px(12.))
                            .child("JD"),
                    )
                    .content(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Jordan"),
                    ),
            )
            .child(
                Chip::new("chip-trailing-example", "Pending items")
                    .variant(ChipVariant::Assist)
                    .trailing(
                        div()
                            .rounded_full()
                            .px_2()
                            .bg(rgb(0xd6eee1))
                            .text_size(px(12.))
                            .child("3"),
                    )
                    .colors(ChipColors {
                        enabled: ChipStateColors {
                            trailing: Some(rgb(0x204e3d).into()),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .content_padding(ButtonPadding {
                        start: px(12.),
                        end: px(4.),
                        ..Default::default()
                    }),
            )
            .child(
                Chip::new("chip-tonal-example", "Selected label")
                    .variant(ChipVariant::Filter)
                    .selected(true)
                    .tonal(true),
            ),
    )
}

#[cfg(all(test, feature = "test-support"))]
mod chip_tests;

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, px, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn filter_removal_has_its_own_keyboard_action_and_restores_focus(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let selection = cx.new(|cx| SelectionExamples::for_story(Story::Chip, cx));
                view = Some(selection.clone());
                selection
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.update_window(handle.into(), |_, window, cx| {
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            draw(window, cx);
            let remove = ElementId::from((ElementId::from("removable-filter"), "remove"));
            let target =
                find(window, &[], &remove).expect("Filter chip exposes removal separately");
            assert_eq!(target.label(), Some("Remove downloads filter"));
            assert!(target.bounds().size.width >= px(48.));
            for _ in 0..12 {
                if find(window, &[], &remove).unwrap().focused() == Some(true) {
                    break;
                }
                window.focus_next(cx);
                draw(window, cx);
            }
            assert_eq!(find(window, &[], &remove).unwrap().focused(), Some(true));
            let key = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(gpui::KeyUpEvent { keystroke: key }.to_platform_input(), cx);
            draw(window, cx);
            assert!(view.read(cx).filter_removed);
            assert!(
                !view.read(cx).filter,
                "remove must not invoke the selection callback"
            );
            assert!(view.read(cx).restore.is_focused(window));
            assert!(find(window, &[], &remove).is_none());
        })
        .unwrap();
    }

    #[gpui::test]
    fn keyboard_switch_press_uses_the_same_thumb_feedback_as_pointer(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let selection = cx.new(|cx| SelectionExamples::for_story(Story::Switch, cx));
                view = Some(selection.clone());
                selection
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.update_window(handle.into(), |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(window.is_window_active());
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            draw(window, cx);
            window.focus_next(cx);
            window.focus_next(cx);
            draw(window, cx);
            assert_eq!(
                find(window, &[], &"sync-switch".into()).unwrap().focused(),
                Some(true)
            );
            let width = |window: &Window| {
                find(
                    window,
                    &[],
                    &(ElementId::from("sync-switch"), "switch-thumb").into(),
                )
                .unwrap()
                .bounds()
                .size
                .width
            };
            assert_eq!(width(window), px(16.));
            let key = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert_eq!(
                width(window),
                px(28.),
                "keyboard press must expand the handle like pointer press"
            );
            window.dispatch_event(gpui::KeyUpEvent { keystroke: key }.to_platform_input(), cx);
            draw(window, cx);
            assert!(view.read(cx).plain_switch);
            assert_eq!(width(window), px(24.));
            let key = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert_eq!(width(window), px(28.));
            window.focus_prev(cx);
            draw(window, cx);
            assert_eq!(
                width(window),
                px(24.),
                "blur cancels a key press whose release goes elsewhere"
            );
            window.dispatch_event(gpui::KeyUpEvent { keystroke: key }.to_platform_input(), cx);
            draw(window, cx);
            assert!(
                view.read(cx).plain_switch,
                "cancelled press must not toggle the owner"
            );
        })
        .unwrap();
    }

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
        cx.simulate_window_resize(handle.into(), size(px(850.), px(960.)));
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
                assert!(thumb.bounds().left() >= track.bounds().left() + px(2.));
                assert!(thumb.bounds().right() <= track.bounds().right() - px(2.));
            };
            draw(window, cx);
            assert_switch_centered(window);
            for (scope, id) in [
                (vec![], "palette-all".into()),
                (vec![], "notifications".into()),
                (vec![], ElementId::from((ElementId::from("tag"), "remove"))),
            ] {
                let target = find(window, &scope, &id).unwrap_or_else(|| {
                    panic!(
                        "missing {id:?} in {scope:?}, observed {:?}",
                        gpui_base::test_support::snapshots(window)
                            .iter()
                            .map(|snapshot| snapshot.label())
                            .collect::<Vec<_>>()
                    )
                });
                assert!(target.visible(), "{id}: {:?}", target.bounds());
                let position = target.bounds().center();
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
    #[gpui::test]
    fn switch_interpolates_size_and_position_and_respects_reduced_motion(cx: &mut TestAppContext) {
        use std::time::Duration;

        let (window, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let window = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let selection = cx.new(SelectionExamples::new);
                view = Some(selection.clone());
                selection
            })
            .unwrap();
            (window, view.unwrap())
        });
        let sample = |cx: &mut TestAppContext| {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let thumb = find(
                    window,
                    &[],
                    &(ElementId::from("sync-switch"), "switch-thumb").into(),
                )
                .unwrap()
                .bounds();
                (thumb.center().x, thumb.size.width)
            })
            .unwrap()
        };
        let toggle = |cx: &mut TestAppContext, checked| {
            cx.update(|cx| {
                view.update(cx, |selection, cx| {
                    selection.plain_switch = checked;
                    cx.notify();
                })
            });
        };
        let start = sample(cx);
        toggle(cx, true);
        assert_eq!(sample(cx), start);
        cx.executor().advance_clock(Duration::from_millis(40));
        let middle = sample(cx);
        assert!(middle.0 > start.0);
        assert!(middle.1 > start.1 && middle.1 < px(24.));
        toggle(cx, false);
        assert_eq!(sample(cx), middle);
        cx.executor().advance_clock(Duration::from_secs(1));
        assert_eq!(sample(cx), start);
        cx.update(|cx| cx.set_reduce_motion(true));
        toggle(cx, true);
        let end = sample(cx);
        assert_eq!(end.1, px(24.));
        assert!(end.0 > middle.0);
    }
}
