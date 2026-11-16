use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShape, ButtonShapes,
    ButtonVariant, ControlSize, ExtendedFab, Fab, FabAlignment, FabColor, FabSize,
    FabVisibilityAnimation, Icon, IconButton, IconButtonVariant, IconButtonWidth, IconName,
    IconToggleButton, InteractionSource, Text, ToggleButton, ToggleButtonVariant,
    theme::{ColorRole, CornerShape, Elevation, TypeRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod fab_tests;

#[derive(Default)]
pub(crate) struct ActionExamples {
    story: Option<Story>,
    count: usize,
    toggles: [bool; 5],
    icon_toggles: [bool; 5],
    fab_hidden: bool,
    collapsed: bool,
    interactions: Option<Entity<InteractionSource>>,
    observed_toggle: bool,
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

    fn interaction_sample(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> crate::story::StorySection {
        let source = self.interactions.get_or_insert_with(|| {
            let source = cx.new(|_| InteractionSource::new());
            cx.observe(&source, |_, _, cx| cx.notify()).detach();
            source
        });
        let window_id = window.window_handle().window_id();
        let state = |id: &'static str| {
            let snapshot = source
                .read(cx)
                .state(window_id, &id.into())
                .unwrap_or_default();
            match snapshot {
                state if state.pressed => "Pressed",
                state if state.focus_visible => "Keyboard focus",
                state if state.hovered => "Hovered",
                _ => "Idle",
            }
        };
        let toggle = cx.entity();
        section("Shared interaction source")
            .description("Hover, press, or focus each control to inspect its interaction state.")
            .child(
                row()
                    .child(labeled_sample_at(
                        "observed-action-status",
                        state("observed-action-example"),
                        Button::new("observed-action-example", "Action")
                            .interaction_source(source)
                            .on_activate(|_, _| {}),
                    ))
                    .child(labeled_sample_at(
                        "observed-toggle-status",
                        state("observed-toggle-example"),
                        ToggleButton::new(
                            "observed-toggle-example",
                            "Toggle",
                            self.observed_toggle,
                        )
                        .interaction_source(source)
                        .on_change(move |selected, _, cx| {
                            toggle.update(cx, |this, cx| {
                                this.observed_toggle = selected;
                                cx.notify();
                            });
                        }),
                    )),
            )
    }
}

impl Render for ActionExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let interaction_sample = (self.shows(Story::Button) || self.shows(Story::ToggleButton))
            .then(|| self.interaction_sample(window, cx));
        let button_variants = [
            ("Elevated", ButtonVariant::Elevated),
            ("Filled", ButtonVariant::Filled),
            ("Filled tonal", ButtonVariant::FilledTonal),
            ("Outlined", ButtonVariant::Outlined),
            ("Text", ButtonVariant::Text),
        ];
        let sizes = [
            ControlSize::ExtraSmall,
            ControlSize::Small,
            ControlSize::Medium,
            ControlSize::Large,
            ControlSize::ExtraLarge,
        ];
        let toggle_variants = [
            ("Elevated", ToggleButtonVariant::Elevated),
            ("Filled", ToggleButtonVariant::Filled),
            ("Filled tonal", ToggleButtonVariant::FilledTonal),
            ("Outlined", ToggleButtonVariant::Outlined),
        ];
        let icon_variants = [
            ("Standard", IconButtonVariant::Standard),
            ("Filled", IconButtonVariant::Filled),
            ("Filled tonal", IconButtonVariant::FilledTonal),
            ("Outlined", IconButtonVariant::Outlined),
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

        let size_names = ["Extra small", "Small", "Medium", "Large", "Extra large"];

        stack()
            .children(self.shows(Story::Button).then(|| {
                let count = format!("Activations: {}", self.count);
                section("Variants")
                    .description("Press any button to update the activation count.")
                    .child(row().children(button_variants.into_iter().enumerate().map(
                        |(ix, (label, variant))| {
                            let view = cx.entity();
                            Button::new(("action-variant", ix), label)
                                .variant(variant)
                                .on_activate(move |_, cx| {
                                    view.update(cx, |this, cx| {
                                        this.count += 1;
                                        cx.notify();
                                    })
                                })
                        },
                    )))
                    .child(
                        div()
                            .id("action-count")
                            .role(Role::Status)
                            .aria_label(count.clone())
                            .test_support()
                            .child(Text::new(count).role(TypeRole::BodySmall)),
                    )
            }))
            .children(self.shows(Story::Button).then(|| {
                section("Sizes and icon slots").child(
                    row().children(sizes.into_iter().zip(size_names).enumerate().map(
                        |(ix, (size, label))| {
                            labeled_sample(
                                label,
                                Button::new(("action-size", ix), "Add")
                                    .size(size)
                                    .shape(ButtonShape::Square)
                                    .leading_icon(Icon::new(IconName::Add))
                                    .trailing_icon(
                                        Icon::new(IconName::ChevronRight).mirror_in_rtl(true),
                                    ),
                            )
                        },
                    )),
                )
            }))
            .children(self.shows(Story::ToggleButton).then(|| {
                section("Selectable buttons").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        let view = cx.entity();
                        let (variant_name, variant) = toggle_variants[ix % toggle_variants.len()];
                        let button =
                            ToggleButton::new(("toggle-variant", ix), "Pin", self.toggles[ix])
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
                                });
                        labeled_sample(format!("{variant_name} · {}", size_names[ix]), button)
                    }),
                ))
            }))
            .children(self.shows(Story::IconButton).then(|| {
                section("Icon button variants").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        let (variant_name, variant) = icon_variants[ix % icon_variants.len()];
                        let label = size_names[ix];
                        let button = IconButton::new(
                            ("icon-variant", ix),
                            format!("Add ({label})"),
                            Icon::new(IconName::Add),
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
                        );
                        labeled_sample(format!("{variant_name} · {label}"), button)
                    }),
                ))
            }))
            .children(self.shows(Story::IconButton).then(|| {
                section("Toggle and selected icons").child(row().children(
                    sizes.into_iter().enumerate().map(|(ix, size)| {
                        let view = cx.entity();
                        let (variant_name, variant) = icon_variants[ix % icon_variants.len()];
                        let label = size_names[ix];
                        let button = IconToggleButton::new(
                            ("icon-toggle-variant", ix),
                            format!("Favorite ({label})"),
                            Icon::new(IconName::Favorite),
                            self.icon_toggles[ix],
                        )
                        .selected_icon(Icon::new(IconName::Favorite).filled(true))
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
                        });
                        labeled_sample(format!("{variant_name} · {label}"), button)
                    }),
                ))
            }))
            .children(self.shows(Story::Fab).then(|| {
                section("Colors and sizes").child(
                    row().children(
                        fab_colors
                            .into_iter()
                            .zip([
                                "Primary",
                                "Secondary",
                                "Tertiary",
                                "Strong primary",
                                "Strong secondary",
                                "Strong tertiary",
                            ])
                            .enumerate()
                            .map(|(ix, (color, label))| {
                                let size = fab_sizes[ix % fab_sizes.len()];
                                let button = Fab::new(
                                    ("fab-size", ix),
                                    format!("Create ({label})"),
                                    Icon::new(IconName::Add),
                                )
                                .size(size)
                                .color(color);
                                labeled_sample(label, button)
                            }),
                    ),
                )
            }))
            .children(self.shows(Story::Fab).then(|| {
                let view = cx.entity();
                let toggle = view.clone();
                section("Visibility")
                    .description(
                        "Use the toggle to show or hide the FAB. Activate the FAB to hide it.",
                    )
                    .child(
                        row()
                            .child(
                                ToggleButton::new(
                                    "show-fab",
                                    "Show primary action",
                                    !self.fab_hidden,
                                )
                                .on_change(
                                    move |visible, _, cx| {
                                        toggle.update(cx, |this, cx| {
                                            this.fab_hidden = !visible;
                                            cx.notify();
                                        })
                                    },
                                ),
                            )
                            .child(
                                Fab::new("visible-fab", "Create item", Icon::new(IconName::Add))
                                    .size(FabSize::Medium)
                                    .color(FabColor::Tertiary)
                                    .shape(CornerShape::Radii([12., 24., 24., 12.]))
                                    .elevations(ButtonElevations {
                                        resting: Some(Elevation::Level2),
                                        pressed: Some(Elevation::Level1),
                                        ..Default::default()
                                    })
                                    .visible(!self.fab_hidden)
                                    .visibility_animation(FabVisibilityAnimation {
                                        alignment: FabAlignment::BottomEnd,
                                        ..Default::default()
                                    })
                                    .on_activate(move |_, cx| {
                                        view.update(cx, |this, cx| {
                                            this.fab_hidden = true;
                                            cx.notify();
                                        })
                                    }),
                            ),
                    )
            }))
            .children(self.shows(Story::ExtendedFab).then(|| {
                section("Sizes and expansion")
                    .description("Activate any button to expand or collapse the labels.")
                    .child(
                        row().children(fab_sizes.into_iter().enumerate().map(|(ix, size)| {
                            let view = cx.entity();
                            ExtendedFab::new(
                                ("extended-fab-size", ix),
                                "New document",
                                Icon::new(IconName::Add),
                            )
                            .size(size)
                            .expanded(!self.collapsed)
                            .on_activate(move |_, cx| {
                                view.update(cx, |this, cx| {
                                    this.collapsed = !this.collapsed;
                                    cx.notify();
                                })
                            })
                        })),
                    )
            }))
            .children(self.shows(Story::Button).then(|| {
                section("Disabled").child(
                    row().child(Button::new("action-disabled", "Unavailable").disabled(true)),
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
                            IconButton::new(
                                "icon-disabled",
                                "Unavailable action",
                                Icon::new(IconName::Add),
                            )
                            .disabled(true),
                        )
                        .child(
                            IconToggleButton::new(
                                "icon-toggle-disabled",
                                "Unavailable favorite",
                                Icon::new(IconName::Favorite).filled(true),
                                true,
                            )
                            .disabled(true),
                        ),
                )
            }))
            .children(interaction_sample)
            .children(self.shows(Story::Button).then(|| {
                let view = cx.entity();
                section("Custom content and appearance").child(
                    row()
                        .child(custom_action().on_activate(move |_, cx| {
                            view.update(cx, |this, cx| {
                                this.count += 1;
                                cx.notify();
                            });
                        }))
                        .child(Button::new("default-action-neighbor", "Default")),
                )
            }))
            .children(self.shows(Story::ToggleButton).then(|| {
                let view = cx.entity();
                section("Custom checked appearance").child(
                    custom_toggle(self.toggles[4]).on_change(move |selected, _, cx| {
                        view.update(cx, |model, cx| {
                            model.toggles[4] = selected;
                            cx.notify();
                        });
                    }),
                )
            }))
    }
}

fn custom_action_color() -> gpui::Hsla {
    gpui::hsla(0.59, 0.55, 0.31, 1.)
}

fn custom_border_color() -> gpui::Hsla {
    gpui::hsla(0.58, 0.5, 0.59, 1.)
}

fn custom_toggle_color() -> gpui::Hsla {
    gpui::hsla(0.37, 0.48, 0.24, 1.)
}

fn custom_action() -> Button {
    Button::new("custom-action", "Run the sample action")
        .content(
            div()
                .id("custom-action-content")
                .test_support()
                .min_w(gpui::px(84.))
                .h(gpui::px(24.))
                .child("Run sample"),
        )
        .shapes(ButtonShapes {
            resting: Some(CornerShape::Radii([3., 7., 11., 13.])),
            pressed: Some(CornerShape::Radii([2.; 4])),
            ..ButtonShapes::default()
        })
        .colors(ButtonColors {
            container: Some(custom_action_color()),
            content: Some(gpui::white()),
            ..ButtonColors::default()
        })
        .border(ButtonBorder {
            width: gpui::px(2.),
            color: custom_border_color(),
        })
        .content_padding(ButtonPadding {
            start: gpui::px(22.),
            end: gpui::px(10.),
            top: gpui::px(6.),
            bottom: gpui::px(6.),
        })
        .elevations(ButtonElevations {
            resting: Some(Elevation::Level2),
            pressed: Some(Elevation::Level1),
            ..ButtonElevations::default()
        })
}

fn custom_toggle(selected: bool) -> ToggleButton {
    ToggleButton::new("custom-toggle", "Pin sample", selected)
        .content(div().child(if selected { "Pinned" } else { "Pin" }))
        .shapes(ButtonShapes {
            checked: Some(CornerShape::Radii([6.; 4])),
            ..ButtonShapes::default()
        })
        .colors(ButtonColors {
            checked_container: Some(custom_toggle_color()),
            checked_content: Some(gpui::white()),
            ..ButtonColors::default()
        })
}

fn row() -> gpui::Div {
    div().flex().flex_wrap().items_end().gap_3()
}

fn labeled_sample(label: impl Into<gpui::SharedString>, example: impl IntoElement) -> gpui::Div {
    let label = label.into();
    labeled_sample_at(
        (gpui::ElementId::from("sample-label"), label.clone()),
        label,
        example,
    )
}

fn labeled_sample_at(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    example: impl IntoElement,
) -> gpui::Div {
    stack()
        .items_start()
        .gap_1()
        .max_w_full()
        .child(
            Text::new(label)
                .id(id)
                .role(TypeRole::LabelMedium)
                .color(ColorRole::OnSurfaceVariant)
                .wrap(),
        )
        .child(example)
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
    fn action_and_selection_feedback_clear_after_pointer_cancellation(cx: &mut TestAppContext) {
        #[derive(Default)]
        struct Controls {
            disabled: bool,
            activations: usize,
        }
        impl Render for Controls {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let button = cx.entity();
                let switch = cx.entity();
                div()
                    .flex()
                    .gap_4()
                    .child(
                        Button::new("button", "Action")
                            .disabled(self.disabled)
                            .on_activate(move |_, cx| {
                                button.update(cx, |model, _| model.activations += 1)
                            }),
                    )
                    .child(
                        gpui_material::Switch::new("switch", "Selection", false)
                            .disabled(self.disabled)
                            .on_change(move |_, _, cx| {
                                switch.update(cx, |model, _| model.activations += 1)
                            }),
                    )
            }
        }
        let (handle, model) = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let mut model = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let controls = cx.new(|_| Controls::default());
                model = Some(controls.clone());
                controls
            })
            .unwrap();
            (handle, model.unwrap())
        });
        cx.update_window(handle.into(), |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            let feedback = |id: &'static str, window: &Window, cx: &gpui::App| {
                if id == "switch" {
                    f32::from(
                        find(
                            window,
                            &[],
                            &(gpui::ElementId::from(id), "switch-thumb").into(),
                        )
                        .unwrap()
                        .bounds()
                        .size
                        .width,
                    )
                } else {
                    let tint = gpui_material::foundation::color(
                        gpui_material::foundation::window_material_theme(window, cx)
                            .colors
                            .color(gpui_material::theme::ColorRole::Primary),
                    );
                    window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| quad.background == tint.into())
                        .unwrap()
                        .corner_radii
                        .top_left
                        .0
                        / window.scale_factor()
                }
            };
            draw(window, cx);
            for (id, resting, pressed) in [("button", 20., 8.), ("switch", 16., 28.)] {
                for disable in [false, true] {
                    let position = find(window, &[], &id.into()).unwrap().bounds().center();
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
                    assert_eq!(feedback(id, window, cx), pressed);
                    if disable {
                        model.update(cx, |model, cx| {
                            model.disabled = true;
                            cx.notify();
                        });
                        draw(window, cx);
                    }
                    window.dispatch_event(
                        MouseUpEvent {
                            button: MouseButton::Left,
                            position: gpui::point(px(700.), px(500.)),
                            click_count: 1,
                            modifiers: Default::default(),
                        }
                        .to_platform_input(),
                        cx,
                    );
                    model.update(cx, |model, cx| {
                        model.disabled = false;
                        cx.notify();
                    });
                    draw(window, cx);
                    assert_eq!(feedback(id, window, cx), resting);
                    assert_eq!(model.read(cx).activations, 0);
                }
            }
        })
        .unwrap();
    }

    #[gpui::test]
    fn button_keyboard_press_does_not_resume_after_focus_returns(cx: &mut TestAppContext) {
        struct Buttons;
        impl Render for Buttons {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .flex()
                    .gap_4()
                    .child(Button::new("first", "First").on_activate(|_, _| {}))
                    .child(Button::new("second", "Second").on_activate(|_, _| {}))
            }
        }
        let handle = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| Buttons)
            })
            .unwrap()
        });
        cx.update_window(handle.into(), |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            let radius = |window: &Window, cx: &gpui::App| {
                let tint = gpui_material::foundation::color(
                    gpui_material::foundation::window_material_theme(window, cx)
                        .colors
                        .color(gpui_material::theme::ColorRole::Primary),
                );
                let quad = window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.background == tint.into())
                    .expect("filled button paints its primary container");
                quad.corner_radii.top_left.0 / window.scale_factor()
            };
            draw(window, cx);
            window.focus_next(cx);
            draw(window, cx);
            assert_eq!(radius(window, cx), 20.);
            let key = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: key,
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert_eq!(radius(window, cx), 8.);
            window.focus_next(cx);
            draw(window, cx);
            window.focus_prev(cx);
            draw(window, cx);
            assert_eq!(
                find(window, &[], &"first".into()).unwrap().focused(),
                Some(true)
            );
            assert_eq!(
                radius(window, cx),
                20.,
                "a cancelled press must not resume when focus returns before key-up"
            );
        })
        .unwrap();
    }

    #[gpui::test]
    fn button_interaction_sources_deliver_short_presses_and_detach(cx: &mut TestAppContext) {
        use std::{cell::RefCell, rc::Rc};

        use gpui::{ElementId, Entity, KeyDownEvent, KeyUpEvent, Keystroke};
        use gpui_material::{InteractionSource, InteractionUpdate};

        struct Controls {
            sources: [Entity<InteractionSource>; 2],
            second_source: usize,
            second_parent: Option<&'static str>,
            second_disabled: bool,
            clicks: usize,
            requested: Vec<bool>,
        }

        impl Render for Controls {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let action = cx.entity();
                let toggle = cx.entity();
                div()
                    .flex()
                    .gap_4()
                    .child(
                        Button::new("observed-button", "Action")
                            .interaction_source(&self.sources[0])
                            .on_activate(move |_, cx| {
                                action.update(cx, |model, _| model.clicks += 1);
                            }),
                    )
                    .children(self.second_parent.map(|parent| {
                        div().id(parent).child(
                            ToggleButton::new("observed-toggle", "Selection", false)
                                .interaction_source(&self.sources[self.second_source])
                                .disabled(self.second_disabled)
                                .on_change(move |selected, _, cx| {
                                    toggle.update(cx, |model, _| model.requested.push(selected));
                                }),
                        )
                    }))
            }
        }

        let events = Rc::new(RefCell::new(Vec::<InteractionUpdate>::new()));
        let (handle, model, sources) = cx.update(|cx| {
            crate::init_gallery(cx);
            let sources = std::array::from_fn(|_| cx.new(|_| InteractionSource::new()));
            let model = cx.new(|cx: &mut Context<Controls>| {
                let events = events.clone();
                cx.subscribe(&sources[0], move |_, _, event, _| {
                    events.borrow_mut().push(event.clone());
                })
                .detach();
                Controls {
                    sources: sources.clone(),
                    second_source: 0,
                    second_parent: Some("original-parent"),
                    second_disabled: false,
                    clicks: 0,
                    requested: Vec::new(),
                }
            });
            let window =
                MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| model.clone())
                    .unwrap();
            (window, model, sources)
        });
        let first = ElementId::from("observed-button");
        let second = ElementId::from("observed-toggle");
        let window_id = cx
            .update_window(handle.into(), |_, window, _| {
                window.activate_window();
                window.window_handle().window_id()
            })
            .unwrap();
        let draw = |cx: &mut TestAppContext| {
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.run_until_parked();
        };
        let read = |source: usize, id: &ElementId, cx: &mut TestAppContext| {
            cx.update(|cx| sources[source].read(cx).state(window_id, id))
        };
        draw(cx);
        assert_eq!(read(0, &first, cx), Some(Default::default()));
        assert_eq!(read(0, &second, cx), Some(Default::default()));

        let press_edges = |id: &ElementId| {
            let mut pressed = false;
            events
                .borrow()
                .iter()
                .filter(|event| &event.id == id)
                .filter_map(|event| {
                    let current = event.state?.pressed;
                    (std::mem::replace(&mut pressed, current) != current).then_some(current)
                })
                .collect::<Vec<_>>()
        };
        events.borrow_mut().clear();
        cx.update_window(handle.into(), |_, window, cx| {
            let position = find(window, &[], &first).unwrap().bounds().center();
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
        })
        .unwrap();
        assert_eq!(press_edges(&first), [true, false]);
        assert_eq!(cx.update(|cx| model.read(cx).clicks), 1);

        cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
            .unwrap();
        draw(cx);
        events.borrow_mut().clear();
        cx.update_window(handle.into(), |_, window, cx| {
            let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
        })
        .unwrap();
        let edges = press_edges(&second);
        assert_eq!(edges, [true, false]);
        assert_eq!(cx.update(|cx| model.read(cx).requested.clone()), [true]);

        // Reuse the public ID in a new layout scope while the previous tracker
        // detaches. Its cleanup cannot clear the replacement's snapshot.
        cx.update(|cx| {
            model.update(cx, |model, cx| {
                model.second_parent = Some("replacement-parent");
                model.second_disabled = true;
                cx.notify();
            });
        });
        draw(cx);
        assert!(read(0, &second, cx).unwrap().disabled);

        cx.update(|cx| {
            model.update(cx, |model, cx| {
                model.second_source = 1;
                cx.notify();
            });
        });
        draw(cx);
        assert!(read(0, &second, cx).is_none());
        assert!(read(1, &second, cx).unwrap().disabled);

        cx.update(|cx| {
            model.update(cx, |model, cx| {
                model.second_parent = None;
                cx.notify();
            });
        });
        draw(cx);
        assert!(read(1, &second, cx).is_none());
        assert!(read(0, &first, cx).is_some());
    }

    #[gpui::test]
    fn custom_button_states_and_slots_keep_token_defaults_and_controlled_values(
        cx: &mut TestAppContext,
    ) {
        use gpui_material::{
            foundation::{color, set_window_material_theme, window_material_theme},
            theme::Direction,
        };

        #[derive(Default)]
        struct Controls {
            checks: Vec<bool>,
            checked: bool,
            disabled: bool,
            clicks: usize,
        }

        impl Render for Controls {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let toggle = cx.entity();
                let action = cx.entity();
                row()
                    .child(
                        custom_action()
                            .disabled(self.disabled)
                            .on_activate(move |_, cx| {
                                action.update(cx, |model, _| model.clicks += 1)
                            }),
                    )
                    .child(Button::new("plain-action", "Plain"))
                    .child(
                        custom_toggle(self.checked).on_change(move |selected, _, cx| {
                            toggle.update(cx, |model, _| model.checks.push(selected));
                        }),
                    )
            }
        }

        let (handle, model) = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let model = cx.new(|_| Controls::default());
            let window =
                MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| model.clone())
                    .unwrap();
            (window, model)
        });

        cx.update_window(handle.into(), |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            let quad = |tint: gpui::Hsla, window: &Window| {
                window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.background == tint.into())
                    .expect("button paints its instance or default container")
            };
            let check_padding = |start_on_left: bool, window: &Window| {
                let button = quad(custom_action_color(), window);
                let scale = window.scale_factor();
                let content = find(window, &[], &"custom-action-content".into())
                    .expect("custom button content is rendered")
                    .bounds();
                let left = f32::from(content.left()) - button.bounds.left().0 / scale;
                let right = button.bounds.right().0 / scale - f32::from(content.right());
                let (expected_left, expected_right) = if start_on_left {
                    (22., 10.)
                } else {
                    (10., 22.)
                };
                assert!((left - expected_left).abs() <= 1., "left={left}");
                assert!((right - expected_right).abs() <= 1., "right={right}");
            };
            let pointer = |id: &'static str, window: &Window| {
                find(window, &[], &id.into()).unwrap().bounds().center()
            };
            let down = |position, window: &mut Window, cx: &mut gpui::App| {
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
            };
            let up = |position, window: &mut Window, cx: &mut gpui::App| {
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
            };

            draw(window, cx);
            let node = find(window, &[], &"custom-action".into()).unwrap();
            assert_eq!(node.role(), Some(Role::Button));
            assert_eq!(node.label(), Some("Run the sample action"));
            let button = quad(custom_action_color(), window);
            let scale = window.scale_factor();
            assert_eq!(button.corner_radii.top_left.0 / scale, 3.);
            assert_eq!(button.border_widths.top.0 / scale, 2.);
            assert_eq!(button.border_color, custom_border_color());
            let primary = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::Primary),
            );
            assert_eq!(quad(primary, window).corner_radii.top_left.0 / scale, 20.);
            check_padding(true, window);

            let position = pointer("custom-action", window);
            down(position, window, cx);
            draw(window, cx);
            assert_eq!(
                quad(custom_action_color(), window).corner_radii.top_left.0 / scale,
                2.
            );
            up(position, window, cx);
            assert_eq!(model.read(cx).clicks, 1);

            let unchecked = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::SurfaceContainer),
            );
            let trigger = |window: &mut Window, cx: &mut gpui::App| {
                let position = pointer("custom-toggle", window);
                down(position, window, cx);
                up(position, window, cx);
            };
            trigger(window, cx);
            draw(window, cx);
            assert_eq!(model.read(cx).checks, [true]);
            assert_eq!(quad(unchecked, window).corner_radii.top_left.0 / scale, 20.);
            model.update(cx, |model, cx| {
                model.checked = true;
                cx.notify();
            });
            draw(window, cx);
            assert_eq!(
                quad(custom_toggle_color(), window).corner_radii.top_left.0 / scale,
                6.
            );

            // The toggle still owns keyboard focus after its pointer activation.
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
            assert_eq!(model.read(cx).checks, [true, false]);

            let mut theme = window_material_theme(window, cx).clone();
            theme.layout.direction = Direction::Rtl;
            set_window_material_theme(Some(theme), window, cx);
            draw(window, cx);
            check_padding(false, window);
            assert_eq!(
                quad(custom_action_color(), window).corner_radii.top_left.0 / scale,
                7.
            );

            model.update(cx, |model, cx| {
                model.disabled = true;
                cx.notify();
            });
            draw(window, cx);
            let disabled_container = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::OnSurface),
            )
            .opacity(0.10);
            assert_eq!(
                quad(disabled_container, window).border_color,
                custom_border_color()
            );
            let position = pointer("custom-action", window);
            down(position, window, cx);
            up(position, window, cx);
            assert_eq!(model.read(cx).clicks, 1);
        })
        .unwrap();
    }

    #[gpui::test]
    fn extended_fab_interpolates_to_its_square_extent(cx: &mut TestAppContext) {
        use std::time::Duration;

        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let actions = cx.new(|_| ActionExamples::for_story(Story::ExtendedFab));
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
        cx.simulate_window_resize(handle.into(), size(px(1000.), px(2000.)));
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
                    Some(gpui::point(px(999.), px(1999.))),
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
