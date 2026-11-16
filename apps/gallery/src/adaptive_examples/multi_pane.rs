use std::{cell::Cell, rc::Rc};

use gpui::{
    AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Pixels, Render, Size,
    Styled, Window, canvas, div, px, size,
};
use gpui_material::{
    AdaptivePane, AdaptivePaneState, Button, ListDetail, PaneAdaptStrategy, PaneBackBehavior,
    PaneDestination, PaneExpansionAnchor, PaneMargins, PaneOverlayAlignment, PaneRole,
    PaneSeparator, Scrim, Text,
    foundation::window_material_theme,
    theme::{ColorRole, TypeRole},
};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(super) struct ReviewExample {
    viewport: Rc<Cell<Size<Pixels>>>,
    state: Entity<AdaptivePaneState>,
    list: FocusHandle,
    detail: FocusHandle,
    comments: FocusHandle,
}

impl ReviewExample {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            viewport: Rc::new(Cell::new(size(px(0.), px(0.)))),
            state: cx.new(|_| AdaptivePaneState::new(PaneDestination::new(PaneRole::Secondary))),
            list: cx.focus_handle(),
            detail: cx.focus_handle(),
            comments: cx.focus_handle(),
        }
    }
}

impl Render for ReviewExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = gpui_material::foundation::color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::SurfaceContainer),
        );
        let open = self.state.clone();
        let comment = self.state.clone();
        let back = self.state.clone();
        let list = AdaptivePane::new(
            "review-list",
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .bg(background)
                .child(Text::new("Reports").role(TypeRole::TitleLarge))
                .child(
                    Button::new("review-open-report", "Open budget report")
                        .track_focus(&self.list)
                        .on_activate(move |_, cx| {
                            open.update(cx, |state, cx| {
                                state.navigate_to(
                                    PaneDestination::new(PaneRole::Primary).content_key("budget"),
                                    cx,
                                )
                            });
                        }),
                ),
        )
        .preferred_width(px(280.))
        .minimum_width(px(190.))
        .entry_focus(&self.list);
        let detail = AdaptivePane::new(
            "review-detail",
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(Text::new("Budget report").role(TypeRole::HeadlineSmall))
                .child(
                    Button::new("review-open-comments", "Open report comments")
                        .track_focus(&self.detail)
                        .on_activate(move |_, cx| {
                            comment.update(cx, |state, cx| {
                                state.navigate_to(
                                    PaneDestination::new(PaneRole::Tertiary)
                                        .content_key("budget-comments"),
                                    cx,
                                )
                            });
                        }),
                ),
        )
        .minimum_width(px(200.))
        .entry_focus(&self.detail);
        let comments = AdaptivePane::new(
            "review-comments",
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .bg(background)
                .rounded_lg()
                .child(
                    Text::new("Report comments")
                        .id("comments-heading")
                        .role(TypeRole::TitleLarge),
                )
                .child(
                    Text::new("Review budget allocation with the finance team.")
                        .id("comments-body")
                        .wrap(),
                )
                .child(
                    Button::new("review-close-comments", "Back from comments")
                        .track_focus(&self.comments)
                        .on_activate(move |_, cx| {
                            back.update(cx, |state, cx| {
                                state.navigate_back(PaneBackBehavior::PopLatest, cx);
                            });
                        }),
                ),
        )
        .preferred_width(px(340.))
        .preferred_height(px(245.))
        .minimum_width(px(180.))
        .margins(PaneMargins {
            bottom: px(12.),
            ..Default::default()
        })
        .adapt_with(|viewport, directive| {
            if directive.horizontal_partitions < 3 && viewport.height < px(900.) {
                PaneAdaptStrategy::Levitate(PaneOverlayAlignment::BottomCenter)
            } else {
                PaneAdaptStrategy::Reflow(PaneRole::Primary)
            }
        })
        .levitate_scrim(Scrim::new("review-scrim"))
        .entry_focus(&self.comments);
        let pane = ListDetail::adaptive(
            "report-review",
            self.viewport.get(),
            &self.state,
            list,
            detail,
        )
        .pane(PaneRole::Tertiary, comments)
        .separator(
            PaneRole::Secondary,
            PaneRole::Primary,
            PaneSeparator::new("Resize reports list and detail").anchors([
                PaneExpansionAnchor::Proportion(0.25),
                PaneExpansionAnchor::Proportion(0.4),
            ]),
        )
        .separator(
            PaneRole::Primary,
            PaneRole::Tertiary,
            PaneSeparator::new("Resize detail and comments").anchors([
                PaneExpansionAnchor::FromEnd(px(260.)),
                PaneExpansionAnchor::FromEnd(px(340.)),
            ]),
        );
        let viewport = self.viewport.clone();
        let owner = cx.weak_entity();
        div().size_full().relative().child(pane).child(
            canvas(
                move |bounds, window, cx| {
                    if viewport.replace(bounds.size) != bounds.size {
                        window.defer(cx, move |_, cx| {
                            let _ = owner.update(cx, |_, cx| cx.notify());
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
    }
}
