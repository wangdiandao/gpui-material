use gpui::{Context, IntoElement, ParentElement, Styled, div, point, prelude::FluentBuilder, px};
use gpui_material::theme::{ColorRole, CornerRole, CornerShape, MaterialShape, TypeRole};
use gpui_material::{
    Banner, Button, ButtonVariant, CircularProgress, DropTarget, EmptyView, Icon, IconName,
    LinearProgress, LoadingIndicator, LoadingIndicatorShape, LoadingIndicatorShapes, ProgressCap,
    ProgressShape, ProgressStroke, SemanticTone, Skeleton, Text,
};

use super::DisplayExamples;
use crate::story::{Story, StorySection, section, stack};

impl DisplayExamples {
    pub(super) fn progress_section(&self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn indeterminate_loading_section(&self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn determinate_loading_section(&mut self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn empty_state_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
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

    pub(super) fn drop_target_section(&self, cx: &mut Context<Self>) -> StorySection {
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

    pub(super) fn banner_section(&self, cx: &Context<Self>) -> StorySection {
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

    pub(super) fn skeleton_section(&self, cx: &Context<Self>) -> StorySection {
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
}
