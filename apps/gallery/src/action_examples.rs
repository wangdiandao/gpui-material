use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonShape, ButtonVariant, ControlSize, ExtendedFab, Fab, FabColor, FabSize,
    IconButton, IconButtonVariant, IconButtonWidth, IconToggleButton, ToggleButton,
    ToggleButtonVariant,
};

use crate::story::{Story, section};

#[derive(Default)]
pub(crate) struct ActionExamples {
    story: Option<Story>,
    count: usize,
    toggles: [bool; 5],
    icon_toggles: [bool; 5],
    collapsed: bool,
}

impl ActionExamples {
    pub(crate) fn for_story(story: Story) -> Self {
        Self {
            story: Some(story),
            ..Self::default()
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }
}

impl Render for ActionExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button_variants = [
            ButtonVariant::Elevated,
            ButtonVariant::Filled,
            ButtonVariant::FilledTonal,
            ButtonVariant::Outlined,
            ButtonVariant::Text,
        ];
        let sizes = [
            ControlSize::ExtraSmall,
            ControlSize::Small,
            ControlSize::Medium,
            ControlSize::Large,
            ControlSize::ExtraLarge,
        ];
        let toggle_variants = [
            ToggleButtonVariant::Elevated,
            ToggleButtonVariant::Filled,
            ToggleButtonVariant::FilledTonal,
            ToggleButtonVariant::Outlined,
        ];
        let icon_variants = [
            IconButtonVariant::Standard,
            IconButtonVariant::Filled,
            IconButtonVariant::FilledTonal,
            IconButtonVariant::Outlined,
        ];
        let fab_sizes = [FabSize::Small, FabSize::Medium, FabSize::Large];
        let fab_colors = [
            FabColor::Primary,
            FabColor::Secondary,
            FabColor::Tertiary,
            FabColor::PrimaryStrong,
            FabColor::SecondaryStrong,
            FabColor::TertiaryStrong,
        ];

        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(self.shows(Story::Button).then(|| {
                section("Variants").child(
                    row().children(
                        button_variants
                            .into_iter()
                            .enumerate()
                            .map(|(ix, variant)| {
                                let view = cx.entity();
                                Button::new(("action-variant", ix), format!("{variant:?}"))
                                    .variant(variant)
                                    .on_activate(move |_, cx| {
                                        view.update(cx, |this, cx| {
                                            this.count += 1;
                                            cx.notify();
                                        })
                                    })
                            }),
                    ),
                )
            }))
            .children(self.shows(Story::Button).then(|| {
                section("Sizes and icon slots").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        Button::new(("action-size", ix), format!("{size:?}"))
                            .size(size)
                            .shape(ButtonShape::Square)
                            .leading_icon("+")
                            .trailing_icon("›")
                    }),
                ))
            }))
            .children(self.shows(Story::ToggleButton).then(|| {
                section("Toggle variants").child(row().children(sizes.into_iter().enumerate().map(
                    |(ix, size)| {
                        let view = cx.entity();
                        let variant = toggle_variants[ix % toggle_variants.len()];
                        ToggleButton::new(
                            ("toggle-variant", ix),
                            format!("{variant:?} {size:?}"),
                            self.toggles[ix],
                        )
                        .variant(variant)
                        .size(size)
                        .shape(if ix % 2 == 0 {
                            ButtonShape::Round
                        } else {
                            ButtonShape::Square
                        })
                        .on_change(move |value, _, cx| {
                            view.update(cx, |this, cx| {
                                this.toggles[ix] = value;
                                cx.notify();
                            })
                        })
                    },
                )))
            }))
            .children(self.shows(Story::IconButton).then(|| {
                section("Icon button variants").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        let variant = icon_variants[ix % icon_variants.len()];
                        IconButton::new(
                            ("icon-variant", ix),
                            format!("{variant:?} {size:?} action"),
                            "+",
                        )
                        .variant(variant)
                        .size(size)
                        .shape(if ix % 2 == 0 {
                            ButtonShape::Round
                        } else {
                            ButtonShape::Square
                        })
                        .width(
                            [
                                IconButtonWidth::Narrow,
                                IconButtonWidth::Uniform,
                                IconButtonWidth::Wide,
                                IconButtonWidth::Uniform,
                                IconButtonWidth::Narrow,
                            ][ix],
                        )
                    }),
                ))
            }))
            .children(self.shows(Story::IconButton).then(|| {
                section("Toggle and selected icons").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        let view = cx.entity();
                        let variant = icon_variants[ix % icon_variants.len()];
                        IconToggleButton::new(
                            ("icon-toggle-variant", ix),
                            format!("{variant:?} {size:?} favorite"),
                            "♡",
                            self.icon_toggles[ix],
                        )
                        .selected_icon("♥")
                        .variant(variant)
                        .size(size)
                        .shape(if ix % 2 == 0 {
                            ButtonShape::Square
                        } else {
                            ButtonShape::Round
                        })
                        .width(
                            [
                                IconButtonWidth::Narrow,
                                IconButtonWidth::Uniform,
                                IconButtonWidth::Wide,
                            ][ix % 3],
                        )
                        .on_change(move |value, _, cx| {
                            view.update(cx, |this, cx| {
                                this.icon_toggles[ix] = value;
                                cx.notify();
                            })
                        })
                    }),
                ))
            }))
            .children(self.shows(Story::Fab).then(|| {
                section("Colors and sizes").child(row().children(
                    fab_colors.into_iter().enumerate().map(|(ix, color)| {
                        let size = fab_sizes[ix % fab_sizes.len()];
                        Fab::new(
                            ("fab-size", ix),
                            format!("Create {color:?} ({size:?})"),
                            "+",
                        )
                        .size(size)
                        .color(color)
                    }),
                ))
            }))
            .children(self.shows(Story::Fab).then(|| {
                section("Extended FAB").child(row().children(
                    fab_sizes.into_iter().enumerate().map(|(ix, size)| {
                        let view = cx.entity();
                        ExtendedFab::new(
                            ("extended-fab-size", ix),
                            "Create a new document 新建文档",
                            "+",
                        )
                        .size(size)
                        .expanded(!self.collapsed)
                        .on_activate(move |_, cx| {
                            view.update(cx, |this, cx| {
                                this.collapsed = !this.collapsed;
                                cx.notify();
                            })
                        })
                    }),
                ))
            }))
            .children(self.shows(Story::Button).then(|| {
                let label = format!("Activations: {}", self.count);
                section("Disabled and activation").child(
                    row()
                        .child(Button::new("action-disabled", "Unavailable").disabled(true))
                        .child(
                            div()
                                .id("action-count")
                                .role(Role::Status)
                                .aria_label(label.clone())
                                .test_support()
                                .child(label),
                        ),
                )
            }))
            .children(self.shows(Story::ToggleButton).then(|| {
                section("Disabled").child(row().child(
                    ToggleButton::new("toggle-disabled", "Unavailable toggle", true).disabled(true),
                ))
            }))
            .children(self.shows(Story::IconButton).then(|| {
                section("Disabled").child(
                    row()
                        .child(
                            IconButton::new("icon-disabled", "Unavailable action", "+")
                                .disabled(true),
                        )
                        .child(
                            IconToggleButton::new(
                                "icon-toggle-disabled",
                                "Unavailable favorite",
                                "♥",
                                true,
                            )
                            .disabled(true),
                        ),
                )
            }))
    }
}

fn row() -> gpui::Div {
    div().flex().flex_wrap().items_center().gap_2()
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
    fn extended_fab_interpolates_to_its_square_extent(cx: &mut TestAppContext) {
        use std::time::Duration;

        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let actions = cx.new(|_| ActionExamples::for_story(Story::Fab));
                view = Some(actions.clone());
                actions
            })
            .unwrap();
            (handle, view.unwrap())
        });
        let widths = |cx: &mut TestAppContext| {
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                [0_usize, 1, 2].map(|idx| {
                    find(window, &[], &("extended-fab-size", idx).into())
                        .unwrap()
                        .bounds()
                        .size
                        .width
                })
            })
            .unwrap()
        };
        let start = widths(cx);
        cx.update(|cx| {
            view.update(cx, |actions, cx| {
                actions.collapsed = true;
                cx.notify();
            })
        });
        assert_eq!(widths(cx), start);
        cx.executor().advance_clock(Duration::from_millis(40));
        let middle = widths(cx);
        cx.executor().advance_clock(Duration::from_secs(1));
        let end = widths(cx);
        assert_eq!(end, [px(56.), px(80.), px(96.)]);
        for idx in 0..3 {
            assert!(end[idx] < middle[idx] && middle[idx] < start[idx]);
        }
    }
    #[gpui::test]
    fn material_buttons_activate_once_and_toggles_update_the_owner(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let actions = cx.new(|_| ActionExamples::default());
                view = Some(actions.clone());
                actions
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(1000.), px(1450.)));
        let draw = |window: &mut Window, cx: &mut gpui::App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            draw(window, cx);
            for (id, expected, destination) in [
                (gpui::ElementId::from(("action-variant", 0_usize)), 1, None),
                (
                    gpui::ElementId::from(("action-variant", 0_usize)),
                    1,
                    Some(gpui::point(px(999.), px(1449.))),
                ),
                (gpui::ElementId::from("action-disabled"), 1, None),
                (gpui::ElementId::from(("toggle-variant", 0_usize)), 1, None),
            ] {
                let target = find(window, &[], &id).unwrap();
                assert_eq!(target.role(), Some(Role::Button));
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
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position: destination.unwrap_or(position),
                        click_count: 1,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                draw(window, cx);
                assert_eq!(view.read(cx).count, expected);
            }
            assert!(view.read(cx).toggles[0]);
            let toggle = find(window, &[], &("toggle-variant", 0_usize).into()).unwrap();
            assert_eq!(toggle.focused(), Some(true));
        })
        .unwrap();
        cx.update_window(handle.into(), |_, window, cx| {
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
            draw(window, cx);
            window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
            draw(window, cx);
            assert!(!view.read(cx).toggles[0]);
            assert_eq!(view.read(cx).count, 1);
        })
        .unwrap();
    }
}
