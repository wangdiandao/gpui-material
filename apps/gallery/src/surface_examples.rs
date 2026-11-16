use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_material::{
    Icon, IconName, InteractionSource, Surface, SurfaceBorder, SurfaceFeedback, Text,
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerShape, Elevation},
};

use crate::story::{section, stack};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(crate) struct SurfaceExamples {
    opened: bool,
    weekly: bool,
    summary: bool,
    source: Entity<InteractionSource>,
}

impl SurfaceExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            opened: false,
            weekly: false,
            summary: false,
            source: cx.new(|_| InteractionSource::new()),
        }
    }
}

impl Render for SurfaceExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let tint = |role| color(theme.colors.color(role));
        let outline = SurfaceBorder {
            width: px(1.),
            color: tint(ColorRole::OutlineVariant),
        };
        let shape = CornerShape::Radii([16., 16., 8., 8.]);
        let opener = cx.weak_entity();
        let summary = cx.weak_entity();
        stack()
            .child(
                section("Passive and nested").child(
                    Surface::new(ColorRole::PrimaryContainer)
                        .shape(shape)
                        .p_3()
                        .child(
                            Surface::new(ColorRole::PrimaryContainer)
                                .container_color(tint(ColorRole::SurfaceContainerHigh))
                                .shape(CornerShape::Radii([8.; 4]))
                                .p_3()
                                .child(
                                    Text::new("Inherits the surrounding foreground")
                                        .id("surface-inherited-text"),
                                ),
                        ),
                ),
            )
            .child(
                section("Interactive surfaces")
                    .child(
                        Surface::clickable(
                            "surface-open",
                            "Open project details",
                            ColorRole::PrimaryContainer,
                        )
                        .shape(shape)
                        .border(outline)
                        .shadow_elevation(Elevation::Level1)
                        .interaction_source(&self.source)
                        .p_3()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::Archive))
                        .child(Text::new("Project details").id("surface-open-text"))
                        .on_activate(move |_, cx| {
                            let _ = opener.update(cx, |view, cx| {
                                view.opened = true;
                                cx.notify();
                            });
                        }),
                    )
                    .children(
                        self.opened
                            .then(|| Text::new("Opened project details").id("surface-open-status")),
                    )
                    .child(
                        div()
                            .id("surface-cadence")
                            .role(Role::ListBox)
                            .aria_label("Delivery cadence")
                            .flex()
                            .gap_2()
                            .children([(false, "Daily"), (true, "Weekly")].map(
                                |(weekly, label)| {
                                    let selected = weekly == self.weekly;
                                    let owner = cx.weak_entity();
                                    Surface::selectable(
                                        label,
                                        label,
                                        if selected {
                                            ColorRole::SecondaryContainer
                                        } else {
                                            ColorRole::SurfaceContainerLow
                                        },
                                        selected,
                                    )
                                    .shape(CornerShape::Full)
                                    .p_3()
                                    .child(Text::new(label).id("surface-cadence-label"))
                                    .on_activate(
                                        move |_, cx| {
                                            let _ = owner.update(cx, |view, cx| {
                                                view.weekly = weekly;
                                                cx.notify();
                                            });
                                        },
                                    )
                                },
                            )),
                    )
                    .child(
                        Surface::toggleable(
                            "surface-summary",
                            "Enable summary",
                            ColorRole::Surface,
                            self.summary,
                        )
                        .shape(shape)
                        .border(outline)
                        .tonal_elevation(Elevation::Level2)
                        .interaction_source(&self.source)
                        .feedback(SurfaceFeedback {
                            ripple: false,
                            ..Default::default()
                        })
                        .p_3()
                        .child(
                            Text::new(if self.summary {
                                "Summary enabled"
                            } else {
                                "Summary disabled"
                            })
                            .id("surface-summary-text"),
                        )
                        .on_checked_change(move |value, _, cx| {
                            let _ = summary.update(cx, |view, cx| {
                                view.summary = value;
                                cx.notify();
                            });
                        }),
                    )
                    .child(
                        Surface::clickable(
                            "surface-locked",
                            "Managed notification",
                            ColorRole::SurfaceContainerLow,
                        )
                        .disabled(true)
                        .interaction_source(&self.source)
                        .shape(shape)
                        .p_3()
                        .child(
                            Text::new("Managed by your organization").id("surface-disabled-text"),
                        ),
                    ),
            )
    }
}
