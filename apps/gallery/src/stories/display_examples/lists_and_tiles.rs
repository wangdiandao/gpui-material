use gpui::{
    AppContext, Axis, Context, InteractiveElement, ParentElement, StatefulInteractiveElement,
    Styled, Window, div, px,
};
use gpui_material::foundation::{color, isolated_scroll, window_material_theme};
use gpui_material::theme::ColorRole;
use gpui_material::{
    Badge, Button, ButtonVariant, CheckboxTile, ChoiceOption, ChoiceTile, DisplayScrollbar,
    DisplayScrollbarFade, DisplayScrollbarStyle, Icon, IconName, List, ListItem, ListSection,
    ListVariant, RadioTile, SliderState, SliderTile, SwitchTile,
};

use super::DisplayExamples;
use crate::story::{Story, StorySection, section};

impl DisplayExamples {
    pub(super) fn segmented_list_section(&self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn standard_list_section(&self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn settings_tiles_section(&mut self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn horizontal_scroll_section(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> StorySection {
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
