use gpui::{
    Axis, Context, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, Role,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    DisplayScrollbar, Text,
    foundation::{color, window_material_theme},
    theme::{ColorRole, Direction, TypeRole},
};

use super::{Gallery, STORY_MAX_WIDTH};
use crate::{
    shell::catalog::{CATALOG, GalleryPage, entry_index},
    story::{Story, stack},
};

impl Gallery {
    pub(super) fn render_story(
        &mut self,
        inset: Pixels,
        radius: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let content = match self.current {
            GalleryPage::Story(Story::Introduction) => self.overview_view(cx).into_any_element(),
            GalleryPage::Story(story) => self
                .story_view(story, window, cx)
                .expect("registered story")
                .into_any_element(),
            GalleryPage::Style(style) => self.style_view(style, cx).into_any_element(),
        };
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let show_header = !self.current.is_overview();
        let is_adaptive = self.current == GalleryPage::Story(Story::AdaptiveLayout);

        div()
            .id((ElementId::from("gallery-story-pane"), self.current.id()))
            .track_focus(&self.content_focus)
            .test_support()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .rounded(radius)
            .bg(color(theme.colors.color(ColorRole::Surface)))
            .children(show_header.then(|| {
                let group_title = entry_index(self.current)
                    .map(|ix| CATALOG[ix].group.title())
                    .unwrap_or("Components");
                let width_badge = is_adaptive.then(|| {
                    let w: f32 = window.viewport_size().width.into();
                    let class = Self::width_class(window);
                    format!("{class:?} · {w:.0} px")
                });
                div()
                    .flex_none()
                    .border_b_1()
                    .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
                    .child(
                        stack()
                            .w_full()
                            .max_w(STORY_MAX_WIDTH)
                            .mx_auto()
                            .gap_1()
                            .px(inset)
                            .py_4()
                            .when(rtl, Styled::items_end)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .when(rtl, Styled::flex_row_reverse)
                                    .child(
                                        div()
                                            .px_2()
                                            .py(px(2.))
                                            .rounded_full()
                                            .bg(color(
                                                theme.colors.color(ColorRole::SecondaryContainer),
                                            ))
                                            .text_xs()
                                            .text_color(color(
                                                theme.colors.color(ColorRole::OnSecondaryContainer),
                                            ))
                                            .child(group_title),
                                    )
                                    .children(width_badge.map(|badge| {
                                        div()
                                            .px_2()
                                            .py(px(2.))
                                            .rounded_full()
                                            .bg(color(
                                                theme.colors.color(ColorRole::TertiaryContainer),
                                            ))
                                            .text_xs()
                                            .text_color(color(
                                                theme.colors.color(ColorRole::OnTertiaryContainer),
                                            ))
                                            .child(badge)
                                    })),
                            )
                            .child(
                                div()
                                    .id("story-title")
                                    .role(Role::Heading)
                                    .aria_label(self.current.name())
                                    .test_support()
                                    .child(
                                        Text::new(self.current.name())
                                            .id("story-title-text")
                                            .role(TypeRole::HeadlineSmall)
                                            .emphasized(true),
                                    ),
                            )
                            .child(
                                Text::new(self.current.description())
                                    .id("story-description")
                                    .role(TypeRole::BodyMedium)
                                    .color(ColorRole::OnSurfaceVariant)
                                    .wrap(),
                            ),
                    )
            }))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("gallery-scroll")
                            .tab_group()
                            .track_scroll(&self.content_scroll)
                            .overflow_y_scroll()
                            .flex_1()
                            .min_h_0()
                            .child(if is_adaptive {
                                div().size_full().min_h_0().overflow_hidden().child(content)
                            } else {
                                div()
                                    .w_full()
                                    .max_w(STORY_MAX_WIDTH)
                                    .mx_auto()
                                    .px(inset)
                                    .pt_4()
                                    .pb_24()
                                    .child(content)
                            }),
                    )
                    .child(DisplayScrollbar::new(
                        "gallery-scrollbar",
                        Axis::Vertical,
                        &self.content_scroll,
                    )),
            )
    }
}
