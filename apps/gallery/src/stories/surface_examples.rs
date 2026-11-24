use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_material::{
    Button, ButtonVariant, Icon, IconName, InteractionSource, MaterialShape, ShapeMorph, Surface,
    SurfaceBorder, SurfaceFeedback, Text,
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerShape, Elevation, TypeRole},
};

use crate::story::{section, stack};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(crate) struct SurfaceExamples {
    opened: bool,
    weekly: bool,
    summary: bool,
    morph_from: MaterialShape,
    morph_to: MaterialShape,
    morph_progress: f32,
    shape_clicks: usize,
    source: Entity<InteractionSource>,
}

impl SurfaceExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            opened: false,
            weekly: false,
            summary: false,
            morph_from: MaterialShape::SoftBurst,
            morph_to: MaterialShape::Heart,
            morph_progress: 0.5,
            shape_clicks: 0,
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
            .child(self.render_material_shapes(tint, outline, cx))
    }
}

impl SurfaceExamples {
    fn render_material_shapes(
        &self,
        tint: impl Fn(ColorRole) -> gpui::Hsla,
        outline: SurfaceBorder,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let card_backdrop = tint(ColorRole::SurfaceContainerLow);
        let morphed =
            ShapeMorph::new(self.morph_from, self.morph_to).sample_polygon(self.morph_progress);
        let click_owner = cx.weak_entity();
        let progress_controls = [0.0_f32, 0.25, 0.5, 0.75, 1.0].map(|step| {
            let owner = cx.weak_entity();
            let selected = (self.morph_progress - step).abs() < 0.01;
            Button::new(
                format!("morph-step-{}", (step * 100.) as u32),
                format!("{}%", (step * 100.) as u32),
            )
            .variant(if selected {
                ButtonVariant::Filled
            } else {
                ButtonVariant::Outlined
            })
            .on_activate(move |_, cx| {
                let _ = owner.update(cx, |view, cx| {
                    view.morph_progress = step;
                    cx.notify();
                });
            })
        });
        let catalog = div()
            .id("material-shape-catalog")
            .role(Role::ListBox)
            .aria_label("Expressive Material shapes")
            .flex()
            .flex_wrap()
            .gap_2()
            .children(MaterialShape::ALL.into_iter().map(|shape| {
                let selected = shape == self.morph_to;
                let owner = cx.weak_entity();
                Surface::selectable(
                    format!("shape-{}", shape.name()),
                    shape.name(),
                    if selected {
                        ColorRole::PrimaryContainer
                    } else {
                        ColorRole::SurfaceContainerHigh
                    },
                    selected,
                )
                .polygon_shape(shape)
                .border(SurfaceBorder {
                    width: px(if selected { 2. } else { 1. }),
                    color: tint(if selected {
                        ColorRole::Primary
                    } else {
                        ColorRole::OutlineVariant
                    }),
                })
                .size(px(44.))
                .on_activate(move |_, cx| {
                    let _ = owner.update(cx, |view, cx| {
                        view.morph_from = view.morph_to;
                        view.morph_to = shape;
                        cx.notify();
                    });
                })
            }));
        section("Material shapes and morphing")
            .description(
                "All 35 Expressive shapes with live morphing, backdrop clipping, and contour hit testing.",
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .flex_wrap()
                    .child(
                        Surface::clickable(
                            "surface-morph-preview",
                            "Activate morphed shape",
                            ColorRole::PrimaryContainer,
                        )
                        .polygon_shape(morphed)
                        .shape_clip_backdrop(card_backdrop)
                        .border(outline)
                        .shadow_elevation(Elevation::Level1)
                        .interaction_source(&self.source)
                        .size(px(96.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_activate(move |_, cx| {
                            let _ = click_owner.update(cx, |view, cx| {
                                view.shape_clicks += 1;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .w(px(120.))
                                .h(px(28.))
                                .bg(tint(ColorRole::TertiaryContainer))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Text::new(format!("{}", self.shape_clicks))
                                        .id("surface-morph-clicks")
                                        .role(TypeRole::LabelLarge),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Text::new(format!(
                                    "{} -> {} ({:.0}%)",
                                    self.morph_from.name(),
                                    self.morph_to.name(),
                                    self.morph_progress * 100.
                                ))
                                .id("surface-morph-label")
                                .role(TypeRole::LabelLarge),
                            )
                            .child(div().flex().gap_2().flex_wrap().children(progress_controls)),
                    ),
            )
            .child(catalog)
    }
}
