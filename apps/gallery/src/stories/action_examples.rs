use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonColors, ButtonShape, ButtonVariant, ControlBorder, ControlElevations,
    ControlShapes, ControlSize, ExtendedFab, Fab, FabAlignment, FabColor, FabSize,
    FabVisibilityAnimation, Icon, IconButton, IconButtonVariant, IconButtonWidth, IconName,
    IconToggleButton, InteractionSource, LogicalEdges, SemanticTone, Text, ToggleButton,
    ToggleButtonVariant,
    theme::{ColorRole, CornerShape, Elevation, TypeRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod button_tests;
#[cfg(all(test, feature = "test-support"))]
mod extended_fab_tests;
#[cfg(all(test, feature = "test-support"))]
mod fab_tests;

const CONTROL_SIZES: [ControlSize; 5] = [
    ControlSize::ExtraSmall,
    ControlSize::Small,
    ControlSize::Medium,
    ControlSize::Large,
    ControlSize::ExtraLarge,
];

const SIZE_NAMES: [&str; 5] = ["Extra small", "Small", "Medium", "Large", "Extra large"];

const ICON_VARIANTS: [(&str, IconButtonVariant); 4] = [
    ("Standard", IconButtonVariant::Standard),
    ("Filled", IconButtonVariant::Filled),
    ("Filled tonal", IconButtonVariant::FilledTonal),
    ("Outlined", IconButtonVariant::Outlined),
];

const FAB_SIZES: [FabSize; 3] = [FabSize::Small, FabSize::Medium, FabSize::Large];

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
    custom_icon_selected: bool,
    submitting: bool,
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

    fn button_variants_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let button_variants = [
            ("Elevated", ButtonVariant::Elevated),
            ("Filled", ButtonVariant::Filled),
            ("Filled tonal", ButtonVariant::FilledTonal),
            ("Outlined", ButtonVariant::Outlined),
            ("Text", ButtonVariant::Text),
        ];
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
    }

    fn button_loading_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let view = cx.entity();
        section("Loading state")
            .description(
                "Controlled `.loading(bool)` suppresses activation, preserves resting width, and renders an inline circular spinner.",
            )
            .child(
                row()
                    .child(
                        ToggleButton::new("toggle-loading", "Simulate loading", self.submitting)
                            .variant(ToggleButtonVariant::Outlined)
                            .on_change(move |next, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.submitting = next;
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        Button::filled("loading-save", "Save changes")
                            .loading(self.submitting)
                            .on_activate(|_, _| {}),
                    )
                    .child(
                        Button::tonal("loading-sync", "Sync workspace")
                            .leading_icon(Icon::new(IconName::Add))
                            .loading(self.submitting)
                            .on_activate(|_, _| {}),
                    )
                    .child(
                        Button::outlined("loading-export", "Export report")
                            .loading(self.submitting)
                            .on_activate(|_, _| {}),
                    )
                    .child(
                        IconButton::filled("loading-icon", "Refresh", Icon::new(IconName::Add))
                            .loading(self.submitting)
                            .on_activate(|_, _| {}),
                    ),
            )
    }

    fn button_tones_section(&self) -> crate::story::StorySection {
        let tones = [
            ("Default", SemanticTone::Default),
            ("Danger", SemanticTone::Danger),
            ("Success", SemanticTone::Success),
            ("Warning", SemanticTone::Warning),
            ("Info", SemanticTone::Info),
        ];
        section("Semantic tones")
            .description(
                "Apply Material 3 semantic color roles across button variants with `.tone(SemanticTone)`.",
            )
            .child(
                row().children(tones.into_iter().enumerate().map(|(ix, (label, tone))| {
                    Button::new(("tone-filled", ix), label)
                        .variant(ButtonVariant::Filled)
                        .tone(tone)
                        .on_activate(|_, _| {})
                })),
            )
            .child(
                row().children(tones.into_iter().enumerate().map(|(ix, (label, tone))| {
                    Button::new(("tone-tonal", ix), label)
                        .variant(ButtonVariant::FilledTonal)
                        .tone(tone)
                        .on_activate(|_, _| {})
                })),
            )
            .child(
                row().children(tones.into_iter().enumerate().map(|(ix, (label, tone))| {
                    Button::new(("tone-outlined", ix), label)
                        .variant(ButtonVariant::Outlined)
                        .tone(tone)
                        .on_activate(|_, _| {})
                })),
            )
    }

    fn button_sizes_section(&self) -> crate::story::StorySection {
        section("Sizes and icon slots").child(
            row().children(CONTROL_SIZES.into_iter().zip(SIZE_NAMES).enumerate().map(
                |(ix, (size, label))| {
                    labeled_sample(
                        label,
                        Button::new(("action-size", ix), "Add")
                            .size(size)
                            .shape(ButtonShape::Square)
                            .leading_icon(Icon::new(IconName::Add))
                            .trailing_icon(Icon::new(IconName::ChevronRight).mirror_in_rtl(true)),
                    )
                },
            )),
        )
    }

    fn toggle_buttons_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let toggle_variants = [
            ("Elevated", ToggleButtonVariant::Elevated),
            ("Filled", ToggleButtonVariant::Filled),
            ("Filled tonal", ToggleButtonVariant::FilledTonal),
            ("Outlined", ToggleButtonVariant::Outlined),
        ];
        section("Selectable buttons").child(row().children(
            CONTROL_SIZES.into_iter().enumerate().map(|(ix, size)| {
                let view = cx.entity();
                let (variant_name, variant) = toggle_variants[ix % toggle_variants.len()];
                let button = ToggleButton::new(("toggle-variant", ix), "Pin", self.toggles[ix])
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
                labeled_sample(format!("{variant_name} · {}", SIZE_NAMES[ix]), button)
            }),
        ))
    }

    fn icon_button_variants_section(&self) -> crate::story::StorySection {
        section("Icon button variants").child(row().children(
            CONTROL_SIZES.into_iter().enumerate().map(|(ix, size)| {
                let (variant_name, variant) = ICON_VARIANTS[ix % ICON_VARIANTS.len()];
                let label = SIZE_NAMES[ix];
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
    }

    fn icon_toggle_buttons_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        section("Toggle and selected icons").child(row().children(
            CONTROL_SIZES.into_iter().enumerate().map(|(ix, size)| {
                let view = cx.entity();
                let (variant_name, variant) = ICON_VARIANTS[ix % ICON_VARIANTS.len()];
                let label = SIZE_NAMES[ix];
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
    }

    fn custom_icon_buttons_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let action = cx.entity();
        let toggle = cx.entity();
        section("Custom icon buttons").child(
            row()
                .child(labeled_sample(
                    format!("Activations: {}", self.count),
                    IconButton::new("custom-icon-action", "Add item", Icon::new(IconName::Add))
                        .variant(IconButtonVariant::Outlined)
                        .shapes(ControlShapes {
                            resting: Some(CornerShape::Radii([5., 15., 5., 15.])),
                            pressed: Some(CornerShape::Radii([4.; 4])),
                            ..Default::default()
                        })
                        .border(ControlBorder {
                            width: gpui::px(2.),
                            color: custom_border_color(),
                        })
                        .on_activate(move |_, cx| {
                            action.update(cx, |view, cx| {
                                view.count += 1;
                                cx.notify();
                            });
                        }),
                ))
                .child(
                    IconToggleButton::new(
                        "custom-icon-toggle",
                        "Favorite item",
                        Icon::new(IconName::Favorite),
                        self.custom_icon_selected,
                    )
                    .selected_icon(Icon::new(IconName::Favorite).filled(true))
                    .variant(IconButtonVariant::FilledTonal)
                    .shapes(ControlShapes {
                        checked: Some(CornerShape::Radii([6.; 4])),
                        ..Default::default()
                    })
                    .colors(ButtonColors {
                        checked_container: Some(custom_toggle_color()),
                        checked_content: Some(gpui::white()),
                        ..Default::default()
                    })
                    .on_change(move |selected, _, cx| {
                        toggle.update(cx, |view, cx| {
                            view.custom_icon_selected = selected;
                            cx.notify();
                        });
                    }),
                ),
        )
    }

    fn fab_variants_section(&self) -> crate::story::StorySection {
        let fab_colors = [
            FabColor::Primary,
            FabColor::Secondary,
            FabColor::Tertiary,
            FabColor::PrimaryStrong,
            FabColor::SecondaryStrong,
            FabColor::TertiaryStrong,
        ];
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
                        let size = FAB_SIZES[ix % FAB_SIZES.len()];
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
    }

    fn fab_visibility_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let view = cx.entity();
        let toggle = view.clone();
        section("Visibility")
            .description("Use the toggle to show or hide the FAB. Activate the FAB to hide it.")
            .child(
                row()
                    .child(
                        ToggleButton::new("show-fab", "Show primary action", !self.fab_hidden)
                            .on_change(move |visible, _, cx| {
                                toggle.update(cx, |this, cx| {
                                    this.fab_hidden = !visible;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(
                        Fab::new("visible-fab", "Create item", Icon::new(IconName::Add))
                            .size(FabSize::Medium)
                            .color(FabColor::Tertiary)
                            .shape(CornerShape::Radii([12., 24., 24., 12.]))
                            .elevations(ControlElevations {
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
    }

    fn extended_fab_sizes_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        section("Sizes and expansion")
            .description("Activate any button to expand or collapse the labels.")
            .child(
                row().children(FAB_SIZES.into_iter().enumerate().map(|(ix, size)| {
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
    }

    fn extended_fab_custom_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let view = cx.entity();
        section("Custom content and visibility").child(
            row()
                .child(
                    ExtendedFab::text("extended-text", "Compose email")
                        .expanded(false)
                        .color(FabColor::Secondary),
                )
                .child(
                    ExtendedFab::new(
                        "extended-custom",
                        "Print document",
                        Icon::new(IconName::Add),
                    )
                    .content(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child("Print")
                            .child("A4"),
                    )
                    .color(FabColor::Tertiary)
                    .shape(CornerShape::Radii([12., 24., 24., 12.]))
                    .elevations(ControlElevations {
                        resting: Some(Elevation::Level2),
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
                )
                .child({
                    let view = cx.entity();
                    ToggleButton::new("extended-shown", "Show print action", !self.fab_hidden)
                        .on_change(move |shown, _, cx| {
                            view.update(cx, |this, cx| {
                                this.fab_hidden = !shown;
                                cx.notify();
                            })
                        })
                }),
        )
    }

    fn disabled_icon_buttons_section(&self) -> crate::story::StorySection {
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
    }

    fn custom_button_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
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
    }

    fn custom_toggle_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let view = cx.entity();
        section("Custom checked appearance").child(custom_toggle(self.toggles[4]).on_change(
            move |selected, _, cx| {
                view.update(cx, |model, cx| {
                    model.toggles[4] = selected;
                    cx.notify();
                });
            },
        ))
    }
}

impl Render for ActionExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let interaction_sample = (self.shows(Story::Button) || self.shows(Story::ToggleButton))
            .then(|| self.interaction_sample(window, cx));

        stack()
            .children(
                self.shows(Story::Button)
                    .then(|| self.button_variants_section(cx)),
            )
            .children(
                self.shows(Story::Button)
                    .then(|| self.button_sizes_section()),
            )
            .children(
                self.shows(Story::ToggleButton)
                    .then(|| self.toggle_buttons_section(cx)),
            )
            .children(
                self.shows(Story::IconButton)
                    .then(|| self.icon_button_variants_section()),
            )
            .children(
                self.shows(Story::IconButton)
                    .then(|| self.icon_toggle_buttons_section(cx)),
            )
            .children(
                self.shows(Story::IconButton)
                    .then(|| self.custom_icon_buttons_section(cx)),
            )
            .children(self.shows(Story::Fab).then(|| self.fab_variants_section()))
            .children(
                self.shows(Story::Fab)
                    .then(|| self.fab_visibility_section(cx)),
            )
            .children(
                self.shows(Story::ExtendedFab)
                    .then(|| self.extended_fab_sizes_section(cx)),
            )
            .children(
                (self.story == Some(Story::ExtendedFab))
                    .then(|| self.extended_fab_custom_section(cx)),
            )
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
            .children(
                self.shows(Story::IconButton)
                    .then(|| self.disabled_icon_buttons_section()),
            )
            .children(interaction_sample)
            .children(
                self.shows(Story::Button)
                    .then(|| self.button_loading_section(cx)),
            )
            .children(
                self.shows(Story::Button)
                    .then(|| self.button_tones_section()),
            )
            .children(
                self.shows(Story::Button)
                    .then(|| self.custom_button_section(cx)),
            )
            .children(
                self.shows(Story::ToggleButton)
                    .then(|| self.custom_toggle_section(cx)),
            )
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
        .shapes(ControlShapes {
            resting: Some(CornerShape::Radii([3., 7., 11., 13.])),
            pressed: Some(CornerShape::Radii([2.; 4])),
            ..ControlShapes::default()
        })
        .colors(ButtonColors {
            container: Some(custom_action_color()),
            content: Some(gpui::white()),
            ..ButtonColors::default()
        })
        .border(ControlBorder {
            width: gpui::px(2.),
            color: custom_border_color(),
        })
        .content_padding(LogicalEdges {
            start: 22.,
            end: 10.,
            top: 6.,
            bottom: 6.,
        })
        .elevations(ControlElevations {
            resting: Some(Elevation::Level2),
            pressed: Some(Elevation::Level1),
            ..ControlElevations::default()
        })
}

fn custom_toggle(selected: bool) -> ToggleButton {
    ToggleButton::new("custom-toggle", "Pin sample", selected)
        .content(div().child(if selected { "Pinned" } else { "Pin" }))
        .shapes(ControlShapes {
            checked: Some(CornerShape::Radii([6.; 4])),
            ..ControlShapes::default()
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
