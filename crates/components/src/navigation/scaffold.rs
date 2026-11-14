use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::window_material_theme;
use gpui_material_theme::Direction;

/// Arranges the top app bar, leading rail, content and bottom bar. The floating
/// action and snackbar are anchored within the content region above bottom
/// navigation. Give the scaffold a bounded size and give its content a scroll
/// viewport if needed.
#[derive(IntoElement)]
pub struct Scaffold {
    id: ElementId,
    content: AnyElement,
    app_bar: Option<AnyElement>,
    rail: Option<AnyElement>,
    bottom_bar: Option<AnyElement>,
    fab: Option<AnyElement>,
    snackbar: Option<AnyElement>,
}

impl Scaffold {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            app_bar: None,
            rail: None,
            bottom_bar: None,
            fab: None,
            snackbar: None,
        }
    }

    pub fn app_bar(mut self, element: impl IntoElement) -> Self {
        self.app_bar = Some(element.into_any_element());
        self
    }

    pub fn rail(mut self, element: impl IntoElement) -> Self {
        self.rail = Some(element.into_any_element());
        self
    }

    pub fn bottom_bar(mut self, element: impl IntoElement) -> Self {
        self.bottom_bar = Some(element.into_any_element());
        self
    }

    pub fn fab(mut self, element: impl IntoElement) -> Self {
        self.fab = Some(element.into_any_element());
        self
    }

    pub fn snackbar(mut self, element: impl IntoElement) -> Self {
        self.snackbar = Some(element.into_any_element());
        self
    }
}

impl RenderOnce for Scaffold {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let has_snackbar = self.snackbar.is_some();
        div()
            .id(self.id)
            .test_support()
            .size_full()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .children(self.app_bar.map(|bar| div().flex_none().child(bar)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .children(self.rail.map(|rail| div().flex_none().h_full().child(rail)))
                    .child(
                        div()
                            .id("scaffold-content")
                            .test_support()
                            .relative()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(self.content),
                            )
                            .when(self.fab.is_some() || has_snackbar, |content| {
                                content.child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .bottom_0()
                                        .min_w_0()
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.))
                                        .p(px(16.))
                                        .children(self.fab.map(|fab| {
                                            div()
                                                .flex()
                                                .when_else(
                                                    rtl,
                                                    Styled::justify_start,
                                                    Styled::justify_end,
                                                )
                                                .child(fab)
                                        }))
                                        .children(self.snackbar),
                                )
                            }),
                    ),
            )
            .children(self.bottom_bar.map(|bar| div().flex_none().child(bar)))
    }
}
