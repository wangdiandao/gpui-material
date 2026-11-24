use gpui::{
    AppContext, Axis, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, div, point,
    prelude::FluentBuilder, px,
};
use gpui_material::foundation::{color, isolated_scroll, window_material_theme};
use gpui_material::theme::{
    ColorRole, CornerRole, CornerShape, Elevation, MaterialShape, TypeRole,
};
use gpui_material::{
    Avatar, AvatarGroup, AvatarShape, AvatarSize, Badge, Banner, Button, ButtonVariant, Card,
    CardColors, CardVariant, CheckboxState, CheckboxTile, ChoiceOption, ChoiceTile,
    CircularProgress, ControlBorder, ControlElevations, DisplayScrollbar, DisplayScrollbarFade,
    DisplayScrollbarStyle, Divider, DropTarget, EmptyView, Icon, IconName, LinearProgress, List,
    ListItem, ListSection, ListVariant, LoadingIndicator, LoadingIndicatorShape,
    LoadingIndicatorShapes, ProgressCap, ProgressShape, ProgressStroke, RadioTile, SemanticTone,
    Skeleton, SliderState, SliderTile, SwitchTile, Text,
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod avatar_tests;
#[cfg(all(test, feature = "test-support"))]
mod badge_tests;
#[cfg(all(test, feature = "test-support"))]
mod banner_tests;
#[cfg(all(test, feature = "test-support"))]
mod card_tests;
#[cfg(all(test, feature = "test-support"))]
mod display_scrollbar_tests;
#[cfg(all(test, feature = "test-support"))]
mod divider_tests;
#[cfg(all(test, feature = "test-support"))]
mod drop_target_tests;
#[cfg(all(test, feature = "test-support"))]
mod empty_state_tests;
#[cfg(all(test, feature = "test-support"))]
mod icon_tests;
mod list_examples;
#[cfg(all(test, feature = "test-support"))]
mod loading_tests;
#[cfg(all(test, feature = "test-support"))]
mod progress_tests;
mod pull_to_refresh_demo;
#[cfg(all(test, feature = "test-support"))]
mod pull_to_refresh_tests;
#[cfg(all(test, feature = "test-support"))]
mod settings_tiles_tests;
#[cfg(all(test, feature = "test-support"))]
mod skeleton_tests;
mod text_examples;

pub(crate) struct DisplayExamples {
    story: Option<Story>,
    progress: f32,
    loading: bool,
    loading_progress: f32,
    loading_shapes: Option<LoadingIndicatorShapes>,
    pull_to_refresh: Option<Entity<pull_to_refresh_demo::RefreshExample>>,
    selected: bool,
    opened: bool,
    child_count: usize,
    indeterminate: bool,
    selected_file: Option<usize>,
    pinned_file: Option<usize>,
    files_scroll: ScrollHandle,
    list_examples: Option<Entity<list_examples::ListExamples>>,
    wide_scroll: ScrollHandle,
    drop_status: SharedString,
    empty_state_status: SharedString,
    banner_visible: bool,
    banner_status: SharedString,
    skeleton_animated: bool,
    auto_sync: bool,
    telemetry_opt_in: CheckboxState,
    release_channel: SharedString,
    notification_volume: Option<Entity<SliderState>>,
    choice_density: SharedString,
}

impl Default for DisplayExamples {
    fn default() -> Self {
        Self {
            story: None,
            progress: 0.35,
            loading: false,
            loading_progress: 0.,
            loading_shapes: None,
            pull_to_refresh: None,
            selected: false,
            opened: false,
            child_count: 0,
            indeterminate: false,
            selected_file: None,
            pinned_file: None,
            files_scroll: ScrollHandle::new(),
            list_examples: None,
            wide_scroll: ScrollHandle::new(),
            drop_status: "Drop files or click the drop zone".into(),
            empty_state_status: "No project action triggered yet".into(),
            banner_visible: true,
            banner_status: "Ready".into(),
            skeleton_animated: true,
            auto_sync: true,
            telemetry_opt_in: CheckboxState::Indeterminate,
            release_channel: "stable".into(),
            notification_volume: None,
            choice_density: "comfortable".into(),
        }
    }
}

impl DisplayExamples {
    pub(crate) fn for_story(story: Story) -> Self {
        Self {
            story: Some(story),
            loading: story == Story::LoadingIndicator,
            ..Self::default()
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }

    fn icon_symbols_section(&self) -> crate::story::StorySection {
        section("Outlined and filled symbols").child(
            div().flex().flex_wrap().gap_4().items_center().children(
                [20., 24., 40., 48.].into_iter().map(|side| {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::Favorite).size(px(side)))
                        .child(
                            Icon::new(IconName::Favorite)
                                .filled(true)
                                .size(px(side))
                                .color(ColorRole::Primary),
                        )
                }),
            ),
        )
    }

    fn icon_inherited_section(&self) -> crate::story::StorySection {
        section("Shared size and color").child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(
                    Button::new("icon-inherited", "Add document")
                        .leading_icon(Icon::new(IconName::Add)),
                )
                .child(
                    Button::new("icon-inherited-disabled", "Unavailable")
                        .leading_icon(Icon::new(IconName::Add))
                        .disabled(true),
                ),
        )
    }

    fn icon_directional_section(&self) -> crate::story::StorySection {
        section("Directional and decorative SVGs")
            .description(
                "The first arrow mirrors with right-to-left layout. The second retains its direction.",
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .h_10()
                    .items_center()
                    .child(
                        Icon::new(IconName::ArrowForward)
                            .size(px(24.))
                            .mirror_in_rtl(true)
                            .label("forward", "Forward"),
                    )
                    .child(Divider::vertical().inset(px(4.), px(8.)))
                    .child(Icon::new(IconName::ArrowForward).size(px(24.))),
            )
    }

    fn card_section(&self, window: &Window, cx: &mut Context<Self>) -> crate::story::StorySection {
        let card = cx.entity();
        let child = cx.entity();
        let theme = window_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        section("Elevated, filled and outlined")
            .description("Open the project card or activate its Pin button independently.")
            .child(
                Card::new("elevated-card")
                    .variant(CardVariant::Elevated)
                    .shape(CornerRole::Large.shape())
                    .child(Text::new("Elevated card")),
            )
            .child(
                Card::new("filled-card")
                    .colors(CardColors {
                        container: Some(tone(ColorRole::SecondaryContainer)),
                        content: Some(tone(ColorRole::OnSecondaryContainer)),
                        ..Default::default()
                    })
                    .elevations(ControlElevations {
                        hovered: Some(Elevation::Level2),
                        focused: Some(Elevation::Level1),
                        ..Default::default()
                    })
                    .accessible_label("Open project")
                    .on_activate(move |_, cx| {
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
                    .border(ControlBorder {
                        width: px(2.),
                        color: tone(ColorRole::OutlineVariant),
                    })
                    .disabled(true)
                    .accessible_label("Unavailable card")
                    .on_activate(|_, _| {})
                    .child(Text::new("Unavailable card")),
            )
    }

    fn divider_section(&self) -> crate::story::StorySection {
        section("Horizontal and vertical dividers")
            .child(
                Text::new("Workspace")
                    .id("divider-from")
                    .role(TypeRole::LabelLarge),
            )
            .child(Divider::horizontal().inset(px(16.), px(0.)))
            .child(
                Text::new("Documents")
                    .id("divider-to")
                    .role(TypeRole::BodyMedium),
            )
            .child(
                Divider::horizontal()
                    .thickness(px(3.))
                    .color(ColorRole::Primary),
            )
            .child(Divider::horizontal().thickness(px(0.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .h_10()
                    .child(Text::new("Before").id("divider-before"))
                    .child(Divider::vertical().inset(px(4.), px(8.)))
                    .child(Text::new("After").id("divider-after"))
                    .child(
                        Divider::vertical()
                            .thickness(px(3.))
                            .color(ColorRole::Primary),
                    )
                    .child(Divider::vertical().thickness(px(0.))),
            )
    }

    fn badge_section(&self) -> impl IntoElement {
        stack()
            .child(
                section("Anchored badges")
                    .description("The archive badge caps the displayed count at 99+.")
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .gap_5()
                            .p_2()
                            .child(Badge::dot(
                                Icon::new(IconName::Notifications)
                                    .size(px(24.))
                                    .label("new-activity", "New activity"),
                            ))
                            .child(Badge::count(
                                Icon::new(IconName::Inbox)
                                    .size(px(24.))
                                    .label("inbox", "Inbox, 8 unread"),
                                8,
                                99,
                            ))
                            .child(Badge::label(
                                Icon::new(IconName::Description)
                                    .size(px(24.))
                                    .label("new-file", "Files"),
                                "New",
                            ))
                            .child(Badge::count(
                                Icon::new(IconName::Archive)
                                    .size(px(24.))
                                    .label("archive-overflow", "More than 99 archived items"),
                                125,
                                99,
                            ))
                            .child(
                                Badge::dot(
                                    Icon::new(IconName::Favorite)
                                        .size(px(24.))
                                        .label("favorite-status", "Favorites, synced"),
                                )
                                .content("synced-marker", "✓")
                                .container_color(gpui::rgb(0x21644a))
                                .content_color(gpui::rgb(0xffffff)),
                            ),
                    ),
            )
            .child(
                section("Standalone badges").child(
                    div()
                        .flex()
                        .items_center()
                        .gap_5()
                        .p_2()
                        .child(Badge::standalone())
                        .child(
                            Badge::standalone()
                                .content(
                                    "priority-marker",
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(div().size(px(4.)).rounded_full().bg(gpui::white()))
                                        .child("P1"),
                                )
                                .container_color(gpui::rgb(0x334e68))
                                .content_color(gpui::rgb(0xffffff))
                                .accessible_label("priority-badge", "Priority 1"),
                        ),
                ),
            )
    }

    fn segmented_list_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let list = cx.entity();
        section("Segmented list")
            .description("Select Documents to see the selected row treatment.")
            .child(
                List::new("segmented-list")
                    .variant(ListVariant::Segmented)
                    .items([
                        ListItem::new("documents", "Documents")
                            .selected(self.selected)
                            .supporting("12 items")
                            .leading(Icon::new(IconName::Description).size(px(24.)))
                            .on_activate(move |_, cx| {
                                list.update(cx, |this, cx| {
                                    this.selected = !this.selected;
                                    cx.notify();
                                })
                            }),
                        ListItem::new("archives", "Archives")
                            .overline("Storage")
                            .supporting("Last updated today")
                            .trailing(Badge::count(
                                Icon::new(IconName::Archive)
                                    .size(px(24.))
                                    .label("archive-status", "More than 99 archived items"),
                                125,
                                99,
                            )),
                        ListItem::new("shared", "Shared").trailing(Badge::dot(
                            Icon::new(IconName::Notifications)
                                .size(px(24.))
                                .label("shared-status", "New shared activity"),
                        )),
                    ]),
            )
    }

    fn progress_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let progress = cx.entity();
        let mode = cx.entity();
        let show_progress_customization = self.story == Some(Story::Progress);
        let linear = |id, name, shape| {
            let progress = LinearProgress::new(id, name).shape(shape);
            let progress = if show_progress_customization && shape == ProgressShape::Wavy {
                progress
                    .active_stroke(ProgressStroke {
                        width: Some(px(6.)),
                        ..Default::default()
                    })
                    .track_stroke(ProgressStroke {
                        width: Some(px(3.)),
                        ..Default::default()
                    })
                    .gap(px(6.))
                    .wavelength(px(32.))
                    .wave_speed(px(20.))
                    .stop_size(px(6.))
                    .draw_stop_indicator(|stop, window, _| {
                        let bounds = stop.bounds;
                        let center = bounds.center();
                        let mut diamond = gpui::PathBuilder::fill();
                        diamond.move_to(point(center.x, bounds.top()));
                        diamond.line_to(point(bounds.right(), center.y));
                        diamond.line_to(point(center.x, bounds.bottom()));
                        diamond.line_to(point(bounds.left(), center.y));
                        diamond.close();
                        if let Ok(path) = diamond.build() {
                            window.paint_path(path, stop.color);
                        }
                    })
            } else {
                progress
            };
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };
        let circular = |id, name, shape| {
            let progress = CircularProgress::new(id, name).shape(shape);
            let progress = if show_progress_customization && shape == ProgressShape::Wavy {
                progress
                    .active_stroke(ProgressStroke {
                        width: Some(px(5.)),
                        cap: ProgressCap::Square,
                        ..Default::default()
                    })
                    .track_stroke(ProgressStroke {
                        width: Some(px(2.)),
                        ..Default::default()
                    })
                    .wave_amplitude(0.7)
                    .wavelength(px(25.))
                    .wave_speed(px(18.))
            } else {
                progress
            };
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };

        section("Linear and circular progress")
            .description(
                "Toggle between a measured transfer and one with no known completion time.",
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
                        Button::new("advance-progress", "Advance")
                            .disabled(self.indeterminate)
                            .on_activate(move |_, cx| {
                                progress.update(cx, |this, cx| {
                                    this.progress = (this.progress + 0.25) % 1.;
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
    }

    fn indeterminate_loading_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let loading = cx.entity();
        section("Indeterminate").child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(LoadingIndicator::new("loading", "Loading documents").active(self.loading))
                .child(
                    LoadingIndicator::new("contained-loading", "Loading archives")
                        .contained(true)
                        .active(self.loading),
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

    fn determinate_loading_section(
        &mut self,
        cx: &mut Context<Self>,
    ) -> crate::story::StorySection {
        let shapes = self.loading_shapes.get_or_insert_with(|| {
            LoadingIndicatorShapes::new([
                LoadingIndicatorShape::rounded_polygon(
                    &[
                        point(-1., -1.),
                        point(1., -1.),
                        point(1., 1.),
                        point(-1., 1.),
                    ],
                    0.25,
                )
                .expect("valid square"),
                LoadingIndicatorShape::circle(),
                LoadingIndicatorShape::rounded_polygon(
                    &[point(0., -1.), point(1., 1.), point(-1., 1.)],
                    0.2,
                )
                .expect("valid triangle"),
            ])
            .expect("valid Gallery shape sequence")
        });
        section("Determinate")
            .description("Set progress to compare the default and custom shape sequences.")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        LoadingIndicator::new("measured-loading", "Document loading progress")
                            .value(self.loading_progress),
                    )
                    .child(
                        LoadingIndicator::new(
                            "measured-contained-loading",
                            "Archive loading progress",
                        )
                        .value(self.loading_progress)
                        .contained(true)
                        .shapes(shapes)
                        .indicator_color(gpui::rgb(0x263a62))
                        .container_color(gpui::rgb(0xd7e4ff))
                        .container_shape(CornerShape::Radii([8., 16., 8., 16.])),
                    ),
            )
            .child(
                div().flex().flex_wrap().gap_2().children(
                    [(0., "Reset"), (0.5, "Halfway"), (1., "Complete")]
                        .into_iter()
                        .enumerate()
                        .map(|(index, (value, label))| {
                            let owner = cx.entity();
                            Button::new(("loading-progress", index), label)
                                .variant(ButtonVariant::Outlined)
                                .on_activate(move |_, cx| {
                                    owner.update(cx, |this, cx| {
                                        this.loading_progress = value;
                                        cx.notify();
                                    });
                                })
                        }),
                ),
            )
    }

    fn standard_list_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        section("Standard list")
            .description(
                "Scroll for more files. Pinning a file leaves the row selection as it was.",
            )
            .child(
                div()
                    .relative()
                    .child(
                        isolated_scroll("files-scroll", &self.files_scroll)
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
                                        .on_activate(
                                            move |_, cx| {
                                                pin.update(cx, |this, cx| {
                                                    this.pinned_file = (!pinned).then_some(file);
                                                    cx.notify();
                                                })
                                            },
                                        ),
                                    )
                            }))),
                    )
                    .children((self.story == Some(Story::List)).then(|| {
                        DisplayScrollbar::new("files-indicator", Axis::Vertical, &self.files_scroll)
                    })),
            )
    }

    fn avatar_section(&self) -> impl IntoElement {
        stack()
            .child(
                section("Sizes, shapes, and content")
                    .description(
                        "Avatars scale typography with size and support initials, icons, images with fallbacks, and rounded shapes.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_4()
                            .child(Avatar::initials("Ada Lovelace").size(AvatarSize::ExtraSmall))
                            .child(Avatar::initials("Grace Hopper").size(AvatarSize::Small))
                            .child(
                                Avatar::initials("Alan Turing")
                                    .size(AvatarSize::Medium)
                                    .accessible_label("avatar-at", "Alan Turing"),
                            )
                            .child(
                                Avatar::icon(Icon::new(IconName::Favorite))
                                    .size(AvatarSize::Large)
                                    .color_role(
                                        ColorRole::SecondaryContainer,
                                        ColorRole::OnSecondaryContainer,
                                    ),
                            )
                            .child(
                                Avatar::initials("Katherine Johnson")
                                    .size(AvatarSize::ExtraLarge)
                                    .shape(AvatarShape::Rounded(CornerRole::Large.shape()))
                                    .color_role(
                                        ColorRole::TertiaryContainer,
                                        ColorRole::OnTertiaryContainer,
                                    )
                                    .border(px(2.)),
                            ),
                    ),
            )
            .child(
                section("Avatar group")
                    .description(
                        "AvatarGroup overlaps members with surface-matched cutout rings, RTL mirroring, and +N overflow indicators.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_6()
                            .child(
                                AvatarGroup::new(
                                    "gallery-team-avatars",
                                    [
                                        Avatar::initials("Ada Lovelace"),
                                        Avatar::initials("Grace Hopper").color_role(
                                            ColorRole::SecondaryContainer,
                                            ColorRole::OnSecondaryContainer,
                                        ),
                                        Avatar::initials("Alan Turing").color_role(
                                            ColorRole::TertiaryContainer,
                                            ColorRole::OnTertiaryContainer,
                                        ),
                                        Avatar::initials("Katherine Johnson"),
                                        Avatar::initials("Margaret Hamilton"),
                                    ],
                                )
                                .size(AvatarSize::Medium)
                                .max_visible(3)
                                .ring_role(ColorRole::SurfaceContainerLow)
                                .accessible_label("5 team members"),
                            )
                            .child(
                                AvatarGroup::new(
                                    "gallery-compact-avatars",
                                    [
                                        Avatar::icon(Icon::new(IconName::Favorite)),
                                        Avatar::initials("Design").color_role(
                                            ColorRole::SecondaryContainer,
                                            ColorRole::OnSecondaryContainer,
                                        ),
                                        Avatar::initials("Engineering"),
                                        Avatar::initials("Research"),
                                    ],
                                )
                                .size(AvatarSize::Small)
                                .overlap(px(10.))
                                .ring_width(px(2.))
                                .ring_role(ColorRole::SurfaceContainerLow)
                                .max_visible(2)
                                .accessible_label("4 contributors"),
                            ),
                    ),
            )
    }

    fn empty_state_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let create_view = cx.entity();
        let import_view = cx.entity();
        stack()
            .child(
                section("Expressive icon backdrop and actions")
                    .description(
                        "EmptyView is a purely presentational M3E layout shell with a decorative shape backdrop, copy, and caller-owned actions.",
                    )
                    .child(
                        EmptyView::new("gallery-empty-projects", "No projects yet")
                            .icon(Icon::new(IconName::Inbox))
                            .supporting(
                                "Create a workspace project or import an existing folder to get started.",
                            )
                            .primary_action(
                                Button::new("empty-create-project", "Create project").on_activate(
                                    move |_, cx| {
                                        create_view.update(cx, |this, cx| {
                                            this.empty_state_status =
                                                "Create project requested".into();
                                            cx.notify();
                                        });
                                    },
                                ),
                            )
                            .secondary_action(
                                Button::new("empty-import-project", "Import folder")
                                    .variant(ButtonVariant::Outlined)
                                    .on_activate(move |_, cx| {
                                        import_view.update(cx, |this, cx| {
                                            this.empty_state_status =
                                                "Import folder requested".into();
                                            cx.notify();
                                        });
                                    }),
                            ),
                    )
                    .child(
                        Text::new(self.empty_state_status.clone())
                            .id("empty-state-status")
                            .role(TypeRole::BodySmall)
                            .color(ColorRole::OnSurfaceVariant),
                    ),
            )
            .child(
                section("Custom shape and visual")
                    .description(
                        "Swap the icon backdrop to any MaterialShape or CornerShape, or supply a custom visual element.",
                    )
                    .child(
                        EmptyView::new("gallery-empty-search", "No matching items")
                            .icon(Icon::new(IconName::Search))
                            .icon_container_shape(MaterialShape::SoftBurst)
                            .icon_container_color_role(
                                ColorRole::TertiaryContainer,
                                ColorRole::OnTertiaryContainer,
                            )
                            .title_role(TypeRole::HeadlineSmall)
                            .max_width(px(400.))
                            .supporting("Try broadening your search terms or clearing active filters."),
                    ),
            )
    }

    fn drop_target_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let on_drop_view = cx.entity();
        let on_activate_view = cx.entity();
        section("File drop target")
            .description("Drop external files onto the zone or activate it with click or keyboard.")
            .child(
                DropTarget::files("gallery-file-drop")
                    .title("Drop files to upload")
                    .supporting(self.drop_status.clone())
                    .on_drop(move |paths, _, cx| {
                        let count = paths.paths().len();
                        on_drop_view.update(cx, |this, cx| {
                            this.drop_status = format!("Dropped {count} file(s)").into();
                            cx.notify();
                        });
                    })
                    .on_activate(move |_, cx| {
                        on_activate_view.update(cx, |this, cx| {
                            this.drop_status = "Browse dialog requested".into();
                            cx.notify();
                        });
                    }),
            )
            .child(
                DropTarget::files("gallery-disabled-drop")
                    .title("Uploads paused")
                    .supporting("Disabled drop targets ignore drags and clicks.")
                    .disabled(true),
            )
    }

    fn settings_tiles_section(&mut self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let toggle_view = cx.entity();
        let checkbox_view = cx.entity();
        let stable_view = cx.entity();
        let beta_view = cx.entity();
        let choice_view = cx.entity();
        let volume = self
            .notification_volume
            .get_or_insert_with(|| {
                let state = cx.new(|_| {
                    SliderState::new()
                        .min(0.)
                        .max(100.)
                        .step(5.)
                        .default_value(65.)
                });
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                state
            })
            .clone();
        section("Settings tiles and ListSection")
            .description(
                "Grouped M3E Segmented settings section mixing SwitchTile, CheckboxTile, RadioTile, SliderTile, and ChoiceTile with unified roving focus.",
            )
            .child(
                ListSection::segmented("gallery-settings-section")
                    .title("Workspace preferences")
                    .item(
                        SwitchTile::new("tile-auto-sync", "Automatic sync", self.auto_sync)
                            .overline("Cloud")
                            .supporting("Keep workspace preferences synchronized across devices.")
                            .leading(Icon::new(IconName::Notifications).size(px(24.)))
                            .on_change(move |checked, _, cx| {
                                toggle_view.update(cx, |this, cx| {
                                    this.auto_sync = checked;
                                    cx.notify();
                                });
                            }),
                    )
                    .item(
                        CheckboxTile::new(
                            "tile-diagnostics",
                            "Diagnostics & crash reports",
                            self.telemetry_opt_in,
                        )
                        .supporting("Supports checked, unchecked, and indeterminate states.")
                        .leading(Icon::new(IconName::Info).size(px(24.)))
                        .on_change(move |next, _, cx| {
                            checkbox_view.update(cx, |this, cx| {
                                this.telemetry_opt_in = next;
                                cx.notify();
                            });
                        }),
                    )
                    .item(
                        RadioTile::new(
                            "tile-channel-stable",
                            "Stable release channel",
                            self.release_channel == "stable",
                        )
                        .supporting("Verified desktop builds.")
                        .on_select(move |_, cx| {
                            stable_view.update(cx, |this, cx| {
                                this.release_channel = "stable".into();
                                cx.notify();
                            });
                        }),
                    )
                    .item(
                        RadioTile::new(
                            "tile-channel-beta",
                            "Beta release channel",
                            self.release_channel == "beta",
                        )
                        .supporting("Preview upcoming Material 3 features early.")
                        .on_select(move |_, cx| {
                            beta_view.update(cx, |this, cx| {
                                this.release_channel = "beta".into();
                                cx.notify();
                            });
                        }),
                    )
                    .item(
                        SliderTile::new("tile-volume", "Notification volume", &volume)
                            .supporting("Adjust desktop alert level.")
                            .format_value(|v| format!("{}%", v.round() as i32).into())
                            .slider(|slider| slider.tick_intervals(5)),
                    )
                    .item(
                        ChoiceTile::new(
                            "tile-density",
                            "Layout density",
                            [
                                ChoiceOption::new("compact", "Compact")
                                    .supporting("Tighter row height for dense desktop views"),
                                ChoiceOption::new("comfortable", "Comfortable")
                                    .supporting("Balanced Material 3 spacing"),
                                ChoiceOption::new("spacious", "Spacious")
                                    .supporting("Extra breathing room between rows"),
                            ],
                            self.choice_density.clone(),
                        )
                        .overline("Appearance")
                        .supporting("Choose the default density for tables and lists.")
                        .leading(Icon::new(IconName::Contrast).size(px(24.)))
                        .on_select(move |id, _, cx| {
                            choice_view.update(cx, |this, cx| {
                                this.choice_density = id;
                                cx.notify();
                            });
                        }),
                    ),
            )
    }

    fn banner_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let on_retry = cx.entity();
        let on_dismiss = cx.entity();
        let on_restore = cx.entity();
        section("Banners")
            .description("In-flow Material 3 banners with semantic tones, actions, and caller-owned visibility.")
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(Text::new(self.banner_status.clone()).muted())
                    .when(self.banner_visible, |col| {
                        col.child(
                            Banner::new(
                                "gallery-banner-warning",
                                "Workspace sync paused while offline. Pending changes remain local.",
                            )
                            .title("Offline mode")
                            .tone(SemanticTone::Warning)
                            .icon(Icon::new(IconName::Info))
                            .action("Retry now", move |_, cx| {
                                on_retry.update(cx, |this, cx| {
                                    this.banner_status = "Retry requested".into();
                                    cx.notify();
                                });
                            })
                            .secondary_action("Details", |_, _| {})
                            .on_dismiss(move |_, cx| {
                                on_dismiss.update(cx, |this, cx| {
                                    this.banner_visible = false;
                                    this.banner_status = "Warning banner dismissed".into();
                                    cx.notify();
                                });
                            }),
                        )
                    })
                    .when(!self.banner_visible, |col| {
                        col.child(
                            Button::outlined("restore-banner", "Show warning banner").on_activate(
                                move |_, cx| {
                                    on_restore.update(cx, |this, cx| {
                                        this.banner_visible = true;
                                        this.banner_status = "Warning banner restored".into();
                                        cx.notify();
                                    });
                                },
                            ),
                        )
                    })
                    .child(
                        Banner::new(
                            "gallery-banner-default",
                            "New keyboard shortcuts are available in the workspace command palette.",
                        )
                        .title("Tips")
                        .icon(Icon::new(IconName::Notifications)),
                    )
                    .child(
                        Banner::new(
                            "gallery-banner-danger",
                            "Failed to save changes to remote storage.",
                        )
                        .title("Save failed")
                        .tone(SemanticTone::Danger)
                        .icon(Icon::new(IconName::Error))
                        .action("Try again", |_, _| {}),
                    )
                    .child(
                        Banner::new(
                            "gallery-banner-success",
                            "All 12 background checks completed without warnings.",
                        )
                        .tone(SemanticTone::Success)
                        .icon(Icon::new(IconName::Check)),
                    ),
            )
    }

    fn skeleton_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let on_toggle = cx.entity();
        let animated = self.skeleton_animated;
        section("Skeleton placeholders")
            .description(
                "Material 3 SurfaceContainerHigh/Highest pulse placeholders with zero offscreen frames.",
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        Button::outlined(
                            "toggle-skeleton-animation",
                            if animated {
                                "Pause pulse"
                            } else {
                                "Resume pulse"
                            },
                        )
                        .on_activate(move |_, cx| {
                            on_toggle.update(cx, |this, cx| {
                                this.skeleton_animated = !this.skeleton_animated;
                                cx.notify();
                            });
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Skeleton::circle("skel-avatar", px(48.)).animated(animated))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Skeleton::text_line("skel-title")
                                            .width(px(180.))
                                            .role(TypeRole::TitleMedium)
                                            .animated(animated),
                                    )
                                    .child(
                                        Skeleton::text_line("skel-subtitle").animated(animated),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Skeleton::rect("skel-card", px(160.), px(72.))
                                    .corner_role(CornerRole::Medium)
                                    .animated(animated),
                            )
                            .child(
                                Skeleton::shape(
                                    "skel-cookie",
                                    px(64.),
                                    px(64.),
                                    MaterialShape::Cookie7Sided,
                                )
                                .animated(animated),
                            ),
                    ),
            )
    }

    fn horizontal_scroll_section(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> crate::story::StorySection {
        let colors = &window_material_theme(window, cx).colors;
        section("Horizontal scrolling").child(
            div()
                .relative()
                .child(
                    div()
                        .id("wide-files-scroll")
                        .overflow_x_scroll()
                        .track_scroll(&self.wide_scroll)
                        .child(div().flex().w(px(1008.)).h(px(52.)).children(
                            (1..=12).map(|file| div().flex_1().p_2().child(format!("File {file}"))),
                        )),
                )
                .child(
                    DisplayScrollbar::new(
                        "wide-files-indicator",
                        Axis::Horizontal,
                        &self.wide_scroll,
                    )
                    .style(DisplayScrollbarStyle {
                        thumb_color: Some(color(colors.color(ColorRole::Primary))),
                        track_color: Some(color(colors.color(ColorRole::SecondaryContainer))),
                        thickness: px(5.),
                        main_axis_track_inset: px(8.),
                        cross_axis_track_inset: px(3.),
                        ..Default::default()
                    })
                    .fade(DisplayScrollbarFade {
                        enabled: false,
                        ..Default::default()
                    }),
                ),
        )
    }
}

impl Render for DisplayExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        stack()
            .children(self.shows(Story::Text).then(text_examples::examples))
            .children(self.shows(Story::Icon).then(|| self.icon_symbols_section()))
            .children(
                self.shows(Story::Icon)
                    .then(|| self.icon_inherited_section()),
            )
            .children(
                self.shows(Story::Icon)
                    .then(|| self.icon_directional_section()),
            )
            .children(
                self.shows(Story::Card)
                    .then(|| self.card_section(window, cx)),
            )
            .children(self.shows(Story::Avatar).then(|| self.avatar_section()))
            .children(self.shows(Story::Divider).then(|| self.divider_section()))
            .children(
                self.shows(Story::DropTarget)
                    .then(|| self.drop_target_section(cx)),
            )
            .children(
                self.shows(Story::EmptyState)
                    .then(|| self.empty_state_section(cx)),
            )
            .children(self.shows(Story::Banner).then(|| self.banner_section(cx)))
            .children(
                self.shows(Story::Skeleton)
                    .then(|| self.skeleton_section(cx)),
            )
            .children(self.shows(Story::Badge).then(|| self.badge_section()))
            .children(
                self.shows(Story::List)
                    .then(|| self.settings_tiles_section(cx)),
            )
            .children((self.story == Some(Story::List)).then(|| {
                self.list_examples
                    .get_or_insert_with(|| cx.new(list_examples::ListExamples::new))
                    .clone()
            }))
            .children(
                self.shows(Story::List)
                    .then(|| self.segmented_list_section(cx)),
            )
            .children(
                self.shows(Story::Progress)
                    .then(|| self.progress_section(cx)),
            )
            .children(self.story.eq(&Some(Story::LoadingIndicator)).then(|| {
                let refresh = self
                    .pull_to_refresh
                    .get_or_insert_with(|| cx.new(pull_to_refresh_demo::RefreshExample::new));
                section("Pull to refresh")
                    .description("Pull down at the top of the list or use Refresh files.")
                    .child(refresh.clone())
            }))
            .children(
                self.shows(Story::LoadingIndicator)
                    .then(|| self.indeterminate_loading_section(cx)),
            )
            .children(
                self.shows(Story::LoadingIndicator)
                    .then(|| self.determinate_loading_section(cx)),
            )
            .children(
                self.shows(Story::List)
                    .then(|| self.standard_list_section(cx)),
            )
            .children(
                (self.story == Some(Story::List))
                    .then(|| self.horizontal_scroll_section(window, cx)),
            )
    }
}
