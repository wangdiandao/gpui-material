use gpui::{
    AppContext, Axis, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, Window, div, point, px,
};
use gpui_material::foundation::{color, isolated_scroll, window_material_theme};
use gpui_material::theme::{ColorRole, CornerRole, CornerShape, Elevation, TypeRole};
use gpui_material::{
    Badge, Button, ButtonVariant, Card, CardBorder, CardColors, CardElevations, CardVariant,
    CircularProgress, DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, Divider, Icon,
    IconName, LinearProgress, List, ListItem, ListVariant, LoadingIndicator, LoadingIndicatorShape,
    LoadingIndicatorShapes, ProgressCap, ProgressShape, ProgressStroke, Text,
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod badge_tests;
#[cfg(all(test, feature = "test-support"))]
mod card_tests;
#[cfg(all(test, feature = "test-support"))]
mod display_scrollbar_tests;
#[cfg(all(test, feature = "test-support"))]
mod divider_tests;
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
                    .elevations(CardElevations {
                        hovered: Some(Elevation::Level2),
                        focused: Some(Elevation::Level1),
                        ..Default::default()
                    })
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
                    .border(CardBorder {
                        width: px(2.),
                        color: tone(ColorRole::OutlineVariant),
                    })
                    .disabled(true)
                    .on_activate("Unavailable card", |_, _| {})
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
                            .supporting_text("12 items")
                            .leading(Icon::new(IconName::Description).size(px(24.)))
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
            .children(self.shows(Story::Divider).then(|| self.divider_section()))
            .children(self.shows(Story::Badge).then(|| self.badge_section()))
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
