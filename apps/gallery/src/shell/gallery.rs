mod chrome;
mod navigation;
mod sidebar;
mod story_pane;

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyView, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ListAlignment,
    ListState, ParentElement, Pixels, Render, ScrollHandle, Size, Styled, Subscription, Window,
    div, prelude::FluentBuilder, px,
};
use gpui_material::{
    ListDetail, ListDetailPane, TextFieldEvent, TextFieldState,
    foundation::Surface,
    theme::{ColorRole, CornerRole, Direction, ThemePreset, WidthClass},
};

use super::{
    catalog::{CATALOG, CatalogRow, GalleryPage, StylePage, filter_catalog_rows},
    overview::OverviewView,
};
use crate::story::Story;

#[cfg(all(test, feature = "desktop", feature = "test-support"))]
use super::catalog::entry_index;

gpui::actions!(gallery, [CloseCatalog]);

const COMPACT_MARGIN: Pixels = px(16.);
const PANE_SPACING: Pixels = px(24.);
const STORY_MAX_WIDTH: Pixels = px(1120.);
const REPOSITORY_URL: &str = "https://github.com/wangdiandao/gpui-material";

/// The full Gallery view with grouped navigation, top-level sections, Overview home,
/// Styles reference pages, and retained example state.
pub struct Gallery {
    local_theme: bool,
    preset: ThemePreset,
    catalog_collapsed: bool,
    single_pane_catalog_open: bool,
    current: GalleryPage,
    search: Entity<TextFieldState>,
    search_has_value: bool,
    catalog_list: ListState,
    catalog_focus_handles: Rc<[FocusHandle]>,
    filtered_rows: Rc<[CatalogRow]>,
    catalog_focus: FocusHandle,
    content_scroll: ScrollHandle,
    content_focus: FocusHandle,
    last_viewport: Size<Pixels>,
    available_content_width: Rc<Cell<Pixels>>,
    overview: Option<Entity<OverviewView>>,
    stories: Vec<(Story, AnyView)>,
    styles: Vec<(StylePage, AnyView)>,
    _search: Subscription,
    _bounds: Subscription,
}

impl Gallery {
    /// Creates a Gallery in an existing GPUI window, starting at `initial_story`.
    /// The theme toggle updates the application's Material theme by default.
    pub fn new(initial_story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_at_page(GalleryPage::Story(initial_story), window, cx)
    }

    pub(crate) fn new_at_page(
        initial_page: GalleryPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| TextFieldState::new(window, cx));
        let filtered_rows: Rc<[CatalogRow]> = filter_catalog_rows("").into();
        let catalog_list = ListState::new(filtered_rows.len(), ListAlignment::Top, px(200.));
        let catalog_focus_handles: Rc<[FocusHandle]> = CATALOG
            .iter()
            .map(|_| cx.focus_handle())
            .collect::<Vec<_>>()
            .into();
        let observe_search =
            cx.subscribe_in(
                &search,
                window,
                move |this, _, event, window, cx| match event {
                    TextFieldEvent::Change => {
                        this.refresh_filtered_rows(cx);
                    }
                    TextFieldEvent::PressEnter {
                        secondary: false,
                        shift: false,
                    } => this.open_search_result(window, cx),
                    _ => {}
                },
            );
        let available_content_width = Rc::new(Cell::new(Self::compute_content_width(
            window.viewport_size().width,
            Self::single_pane(Self::width_class(window)),
            !Self::single_pane(Self::width_class(window)),
            if Self::width_class(window) == WidthClass::Compact {
                COMPACT_MARGIN
            } else {
                PANE_SPACING
            },
            if Self::single_pane(Self::width_class(window)) {
                if Self::width_class(window) == WidthClass::Compact {
                    COMPACT_MARGIN
                } else {
                    PANE_SPACING
                }
            } else {
                COMPACT_MARGIN
            },
        )));
        let mut stories = Vec::new();
        if let GalleryPage::Story(story) = initial_page
            && let Some(view) =
                Self::build_story_view(story, available_content_width.clone(), window, cx)
        {
            stories.push((story, view));
        }
        let bounds = cx.observe_window_bounds(window, |this, window, cx| {
            let viewport = window.viewport_size();
            let viewport_changed = this.last_viewport != viewport;
            this.last_viewport = viewport;
            let prev_open = this.single_pane_catalog_open;
            if Self::single_pane(Self::width_class(window)) {
                let catalog_focused = this
                    .search
                    .read(cx)
                    .focus_handle(cx)
                    .contains_focused(window, cx)
                    || this.catalog_focus.contains_focused(window, cx);
                let content_focused = this.content_focus.contains_focused(window, cx);
                if catalog_focused || content_focused {
                    this.single_pane_catalog_open = catalog_focused;
                }
            }
            if viewport_changed || this.single_pane_catalog_open != prev_open {
                cx.notify();
            }
        });
        Self {
            local_theme: false,
            preset: ThemePreset::Baseline,
            catalog_collapsed: false,
            single_pane_catalog_open: false,
            current: initial_page,
            search,
            search_has_value: false,
            catalog_list,
            catalog_focus_handles,
            filtered_rows,
            catalog_focus: cx.focus_handle(),
            content_scroll: ScrollHandle::new(),
            content_focus: cx.focus_handle(),
            last_viewport: window.viewport_size(),
            available_content_width,
            overview: None,
            stories,
            styles: Vec::new(),
            _search: observe_search,
            _bounds: bounds,
        }
    }

    /// Scopes changes from the theme controls to the current window when enabled.
    pub fn with_window_theme_toggle(mut self, enabled: bool) -> Self {
        self.local_theme = enabled;
        self
    }

    #[cfg(all(test, feature = "desktop", feature = "test-support"))]
    pub(crate) fn loaded_story_count(&self) -> usize {
        self.stories.len()
    }

    #[cfg(all(test, feature = "desktop", feature = "test-support"))]
    pub(crate) fn story_view_id(&self, story: Story) -> Option<gpui::EntityId> {
        self.stories
            .iter()
            .find(|(entry, _)| *entry == story)
            .map(|(_, view)| view.entity_id())
    }

    #[cfg(all(test, feature = "desktop", feature = "test-support"))]
    pub(crate) fn catalog_story_focus(&self, story: Story) -> Option<FocusHandle> {
        entry_index(GalleryPage::Story(story)).map(|ix| self.catalog_focus_handles[ix].clone())
    }

    #[cfg(all(test, feature = "desktop", feature = "test-support"))]
    pub(crate) fn navigate_story(
        &mut self,
        story: Story,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(GalleryPage::Story(story), window, cx);
    }

    fn compute_content_width(
        viewport_width: Pixels,
        single_pane: bool,
        show_catalog: bool,
        margin: Pixels,
        inset: Pixels,
    ) -> Pixels {
        let detail_width = if single_pane {
            viewport_width
        } else {
            let inner = (viewport_width - margin * 2.).max(px(0.));
            if show_catalog {
                let list_width = px(360.).min(viewport_width / 2.);
                (inner - list_width - PANE_SPACING).max(px(0.))
            } else {
                inner
            }
        };
        (detail_width.min(STORY_MAX_WIDTH) - inset * 2.).max(px(0.))
    }

    fn width_class(window: &Window) -> WidthClass {
        WidthClass::from_width(window.viewport_size().width.into())
    }

    fn single_pane(width_class: WidthClass) -> bool {
        matches!(width_class, WidthClass::Compact | WidthClass::Medium)
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width_class = Self::width_class(window);
        let single_pane = Self::single_pane(width_class);
        let show_catalog = if single_pane {
            self.single_pane_catalog_open
        } else {
            !self.catalog_collapsed
        };
        let margin = if width_class == WidthClass::Compact {
            COMPACT_MARGIN
        } else {
            PANE_SPACING
        };
        let inset = if single_pane { margin } else { COMPACT_MARGIN };
        self.available_content_width
            .set(Self::compute_content_width(
                window.viewport_size().width,
                single_pane,
                show_catalog,
                margin,
                inset,
            ));
        let pane_radius = if single_pane {
            px(0.)
        } else {
            let size = window.viewport_size();
            px(CornerRole::ExtraLarge.shape().radii(
                size.width.into(),
                size.height.into(),
                Direction::Ltr,
            )[0])
        };

        Surface::new(ColorRole::SurfaceContainer)
            .size_full()
            .flex()
            .flex_col()
            .child(self.render_app_bar(single_pane, show_catalog, margin, window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .when(!single_pane, |body| body.px(margin))
                    .child({
                        let catalog = show_catalog.then(|| {
                            self.render_catalog(single_pane, inset, pane_radius, window, cx)
                                .into_any_element()
                        });
                        let story = self.render_story(inset, pane_radius, window, cx);
                        ListDetail::new(
                            "gallery-panes",
                            window.viewport_size().width,
                            div().flex().flex_col().size_full().children(catalog),
                            story,
                        )
                        .active_pane(if show_catalog {
                            ListDetailPane::List
                        } else {
                            ListDetailPane::Detail
                        })
                        .detail_only(!show_catalog)
                    }),
            )
            .child(self.render_status_bar(margin, window, cx))
    }
}
