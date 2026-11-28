use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px};
use gpui_material::{
    Accordion, AccordionItem, AccordionMode, AccordionVariant, Collapsible, Icon, IconName,
    Pagination, Text,
    theme::{ColorRole, TypeRole},
};

use crate::{
    Story,
    story::{section, stack},
};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(crate) struct DisclosureExamples {
    story: Story,
    collapsible_open: bool,
    disabled_open: bool,
    single_expanded: Vec<SharedString>,
    multi_expanded: Vec<SharedString>,
    page: usize,
    compact_page: usize,
}

impl DisclosureExamples {
    pub(crate) fn new(story: Story) -> Self {
        Self {
            story,
            collapsible_open: true,
            disabled_open: false,
            single_expanded: vec!["overview".into()],
            multi_expanded: vec!["shipping".into(), "billing".into()],
            page: 5,
            compact_page: 1,
        }
    }

    fn collapsible_view(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let on_toggle = cx.listener(|this, open: &bool, _, cx| {
            this.collapsible_open = *open;
            cx.notify();
        });
        let _ = on_toggle;
        let entity = cx.entity();

        stack()
            .child(
                section("Collapsible")
                    .description(
                        "Expand or collapse supplementary details with an animated disclosure chevron.",
                    )
                    .child(
                        Collapsible::new("story-collapsible", "Release notes", self.collapsible_open)
                            .supporting("Version 0.4.1 desktop highlights")
                            .leading(Icon::new(IconName::Description).size(px(20.)))
                            .content(
                                Text::new(
                                    "Includes Material 3 Expressive state layers, desktop pagination, and modal command palette support.",
                                )
                                .role(TypeRole::BodyMedium)
                                .color(ColorRole::OnSurfaceVariant)
                                .wrap(),
                            )
                            .on_open_change(move |next, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.collapsible_open = next;
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .child(
                section("Disabled collapsible")
                    .description("Disabled containers suppress pointer, keyboard, and accessible toggles.")
                    .child(
                        Collapsible::new("story-collapsible-disabled", "Locked diagnostics", self.disabled_open)
                            .supporting("Requires administrator permission")
                            .leading(Icon::new(IconName::Info).size(px(20.)))
                            .disabled(true)
                            .content(Text::new("Hidden diagnostic payload.")),
                    ),
            )
    }

    fn accordion_view(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let single_entity = cx.entity();
        let multi_entity = cx.entity();

        stack()
            .child(
                section("Standard single-expand accordion")
                    .description(
                        "Only one section expands at a time; use Up, Down, Home, and End to move focus between triggers.",
                    )
                    .child(
                        Accordion::new(
                            "story-accordion-single",
                            [
                                AccordionItem::new("overview", "Workspace overview")
                                    .supporting("Repository layout and crate boundaries")
                                    .leading(Icon::new(IconName::Inbox).size(px(20.)))
                                    .content(
                                        Text::new(
                                            "Theme tokens live in gpui-material-theme, surfaces in foundation, core controls in components, and opt-in families in extras.",
                                        )
                                        .role(TypeRole::BodyMedium)
                                        .color(ColorRole::OnSurfaceVariant)
                                        .wrap(),
                                    ),
                                AccordionItem::new("keyboard", "Keyboard navigation")
                                    .supporting("Roving Tab stop across enabled triggers")
                                    .leading(Icon::new(IconName::Edit).size(px(20.)))
                                    .content(
                                        Text::new(
                                            "Press Enter or Space to toggle the focused section, and Up/Down/Home/End to move across enabled triggers.",
                                        )
                                        .role(TypeRole::BodyMedium)
                                        .color(ColorRole::OnSurfaceVariant)
                                        .wrap(),
                                    ),
                                AccordionItem::new("locked", "Archived policy")
                                    .supporting("Disabled sections are skipped during keyboard navigation")
                                    .disabled(true)
                                    .content(Text::new("Unavailable content.")),
                            ],
                        )
                        .accessible_label("Documentation sections")
                        .mode(AccordionMode::Single)
                        .variant(AccordionVariant::Standard)
                        .expanded(self.single_expanded.clone())
                        .on_change(move |ids, _, cx| {
                            single_entity.update(cx, |this, cx| {
                                this.single_expanded = ids;
                                cx.notify();
                            });
                        }),
                    ),
            )
            .child(
                section("Segmented multi-expand accordion")
                    .description(
                        "Material 3 Expressive segmented corners with independent multi-section expansion.",
                    )
                    .child(
                        Accordion::new(
                            "story-accordion-multi",
                            [
                                AccordionItem::new("shipping", "Delivery schedule")
                                    .content(
                                        Text::new("Standard desktop builds publish with signed installers.")
                                            .role(TypeRole::BodyMedium)
                                            .color(ColorRole::OnSurfaceVariant)
                                            .wrap(),
                                    ),
                                AccordionItem::new("billing", "Licensing & notices")
                                    .content(
                                        Text::new("Original code and documentation are licensed under Apache-2.0.")
                                            .role(TypeRole::BodyMedium)
                                            .color(ColorRole::OnSurfaceVariant)
                                            .wrap(),
                                    ),
                                AccordionItem::new("support", "Platform targets")
                                    .content(
                                        Text::new("Supports Windows 11 x64, macOS ARM64, and Linux x64.")
                                            .role(TypeRole::BodyMedium)
                                            .color(ColorRole::OnSurfaceVariant)
                                            .wrap(),
                                    ),
                            ],
                        )
                        .mode(AccordionMode::Multiple)
                        .variant(AccordionVariant::Segmented)
                        .expanded(self.multi_expanded.clone())
                        .on_change(move |ids, _, cx| {
                            multi_entity.update(cx, |this, cx| {
                                this.multi_expanded = ids;
                                cx.notify();
                            });
                        }),
                    ),
            )
    }

    fn pagination_view(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_main = cx.entity();
        let entity_compact = cx.entity();

        stack()
            .child(
                section("Desktop pagination with ellipsis")
                    .description(
                        "Highlights the active page pill in PrimaryContainer and truncates long ranges with ellipses.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Pagination::new("story-pagination", self.page, 12)
                                    .accessible_label("Search results pages")
                                    .on_change(move |next, _, cx| {
                                        entity_main.update(cx, |this, cx| {
                                            this.page = next;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                Text::new(format!("Showing page {} of 12", self.page))
                                    .id("story-pagination-status")
                                    .role(TypeRole::BodySmall)
                                    .color(ColorRole::OnSurfaceVariant),
                            ),
                    ),
            )
            .child(
                section("Boundary and disabled states")
                    .description("Previous/next controls disable automatically at the first and last pages.")
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                Pagination::new("story-pagination-compact", self.compact_page, 4)
                                    .on_change(move |next, _, cx| {
                                        entity_compact.update(cx, |this, cx| {
                                            this.compact_page = next;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(Pagination::new("story-pagination-disabled", 2, 5).disabled(true)),
                    ),
            )
    }
}

impl Render for DisclosureExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.story {
            Story::Collapsible => self.collapsible_view(cx).into_any_element(),
            Story::Pagination => self.pagination_view(cx).into_any_element(),
            _ => self.accordion_view(cx).into_any_element(),
        }
    }
}
