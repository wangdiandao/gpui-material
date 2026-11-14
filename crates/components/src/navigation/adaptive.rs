use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::window_material_theme;
use gpui_material_theme::{Direction, WidthClass};

use super::{
    NavigationBar, NavigationBarLayout, NavigationItems, NavigationRail, RailMode, Scaffold,
};

/// Material 3 navigation suite. Compact and medium widths use a short bar;
/// expanded widths use a collapsed rail. Create the items with the same
/// collection and destination IDs on every render.
#[derive(IntoElement)]
pub struct AdaptiveNavigation {
    scaffold: Scaffold,
    items: NavigationItems,
    available_width: Pixels,
}

impl AdaptiveNavigation {
    /// Use the available width of the scaffold in logical units.
    pub fn new(scaffold: Scaffold, items: NavigationItems, available_width: Pixels) -> Self {
        Self {
            scaffold,
            items,
            available_width,
        }
    }
}

impl RenderOnce for AdaptiveNavigation {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = self.items.retain(window, cx);
        match WidthClass::from_width(self.available_width.into()) {
            WidthClass::Compact => self.scaffold.bottom_bar(NavigationBar::new(items)),
            WidthClass::Medium => self
                .scaffold
                .bottom_bar(NavigationBar::new(items).layout(NavigationBarLayout::Horizontal)),
            _ => self
                .scaffold
                .rail(NavigationRail::new("adaptive-rail", items).mode(RailMode::Collapsed)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListDetailPane {
    #[default]
    List,
    Detail,
}

/// Shows a list or detail at compact and medium widths, and both at expanded
/// widths. The owner retains pane entities and scroll handles, selects the
/// narrow pane and supplies the back command.
#[derive(IntoElement)]
pub struct ListDetail {
    id: ElementId,
    list: Option<AnyElement>,
    detail: AnyElement,
    width: Pixels,
    active: ListDetailPane,
    list_width: Pixels,
}

impl ListDetail {
    pub fn new(
        id: impl Into<ElementId>,
        width: Pixels,
        list: impl IntoElement,
        detail: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            list: Some(list.into_any_element()),
            detail: detail.into_any_element(),
            width,
            active: ListDetailPane::List,
            list_width: px(360.),
        }
    }

    /// Shows detail full-width when the app has hidden its list.
    pub fn detail_only(mut self, detail_only: bool) -> Self {
        if detail_only {
            self.list = None;
            self.active = ListDetailPane::Detail;
        }
        self
    }

    pub fn active_pane(mut self, pane: ListDetailPane) -> Self {
        self.active = pane;
        self
    }

    pub fn list_width(mut self, width: Pixels) -> Self {
        self.list_width = width.max(px(0.));
        self
    }

    pub fn is_single_pane(width: Pixels) -> bool {
        matches!(
            WidthClass::from_width(width.into()),
            WidthClass::Compact | WidthClass::Medium
        )
    }
}

impl RenderOnce for ListDetail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let single = Self::is_single_pane(self.width);
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let show_detail = !single || self.active == ListDetailPane::Detail || self.list.is_none();
        let list = self.list.filter(|_| !single || !show_detail);
        div()
            .id(self.id)
            .test_support()
            .size_full()
            .flex()
            .gap(px(if single { 0. } else { 24. }))
            .when(rtl, Styled::flex_row_reverse)
            .children(list.map(|list| {
                div()
                    .id("list-pane")
                    .test_support()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .when_else(single, Styled::flex_1, |pane| {
                        pane.w(self.list_width.min(self.width / 2.)).flex_none()
                    })
                    .child(list)
            }))
            .children(show_detail.then(|| {
                div()
                    .id("detail-pane")
                    .test_support()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(self.detail)
            }))
    }
}

/// A main workspace with an optional trailing support pane. In single-pane
/// mode the caller can show the supporting pane as a full-width view.
#[derive(IntoElement)]
pub struct SupportingPane {
    id: ElementId,
    primary: AnyElement,
    supporting: AnyElement,
    width: Pixels,
    show_support: bool,
}

impl SupportingPane {
    pub fn new(
        id: impl Into<ElementId>,
        width: Pixels,
        primary: impl IntoElement,
        supporting: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            primary: primary.into_any_element(),
            supporting: supporting.into_any_element(),
            width,
            show_support: false,
        }
    }

    pub fn show_support(mut self, show_support: bool) -> Self {
        self.show_support = show_support;
        self
    }
}

impl RenderOnce for SupportingPane {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let single = ListDetail::is_single_pane(self.width);
        div()
            .id(self.id)
            .test_support()
            .size_full()
            .flex()
            .gap(px(if single { 0. } else { 24. }))
            .when(rtl, Styled::flex_row_reverse)
            .children((!single || !self.show_support).then(|| {
                div()
                    .id("primary-pane")
                    .test_support()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(self.primary)
            }))
            .children((!single || self.show_support).then(|| {
                div()
                    .id("supporting-pane")
                    .test_support()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .when_else(single, Styled::flex_1, |pane| {
                        pane.w((self.width / 3.).min(px(360.))).flex_none()
                    })
                    .child(self.supporting)
            }))
    }
}

/// An ordered, vertically scrolling feed. Supply the same scroll handle when
/// the number of columns changes. Cards are placed in reading order.
#[derive(IntoElement)]
pub struct Feed {
    id: ElementId,
    width: Pixels,
    scroll: gpui::ScrollHandle,
    cards: Vec<AnyElement>,
}

impl Feed {
    pub fn new(id: impl Into<ElementId>, width: Pixels, scroll: &gpui::ScrollHandle) -> Self {
        Self {
            id: id.into(),
            width,
            scroll: scroll.clone(),
            cards: Vec::new(),
        }
    }

    pub fn columns(width: Pixels) -> usize {
        match WidthClass::from_width(width.into()) {
            WidthClass::Compact => 1,
            WidthClass::Medium => 2,
            WidthClass::Expanded => 3,
            WidthClass::Large | WidthClass::ExtraLarge => 4,
        }
    }
}

impl ParentElement for Feed {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.cards.extend(elements);
    }
}

impl RenderOnce for Feed {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let columns = Self::columns(self.width);
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let state = window.use_keyed_state((self.id.clone(), "feed-columns"), cx, |_, _| columns);
        state.update(cx, |previous, _| {
            if *previous != columns && !self.cards.is_empty() {
                let first_card = (self.scroll.top_item() * *previous).min(self.cards.len() - 1);
                self.scroll.scroll_to_top_of_item(first_card / columns);
            }
            *previous = columns;
        });
        let mut cards = self.cards.into_iter();
        let rows = std::iter::from_fn(move || {
            let card = cards.next()?;
            let row = std::iter::once(card).chain(cards.by_ref().take(columns - 1));
            let mut len = 0;
            let mut line = div()
                .flex()
                .flex_none()
                .gap(px(16.))
                .when(rtl, Styled::flex_row_reverse)
                .children(row.map(|card| {
                    len += 1;
                    div().flex_1().min_w_0().child(card)
                }));
            line.extend((len..columns).map(|_| div().flex_1().min_w_0().into_any_element()));
            Some(line)
        });
        gpui_material_foundation::isolated_scroll(self.id, &self.scroll)
            .test_support()
            .flex_1()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(16.))
            .children(rows)
    }
}
