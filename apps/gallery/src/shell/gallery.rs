use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use gpui::{
    AnyView, App, AppContext, Axis, Context, ElementId, Entity, FocusHandle, Focusable, Image,
    ImageFormat, InteractiveElement, IntoElement, ListAlignment, ListOffset, ListState, ObjectFit,
    ParentElement, Pixels, Render, Role, ScrollHandle, Size, StatefulInteractiveElement, Styled,
    StyledImage, Subscription, Window, div, img, list, point, prelude::FluentBuilder, px, svg,
    transparent_black,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    ControlSize, DisplayScrollbar, Icon, IconButton, IconName, ListDetail, ListDetailPane,
    ListItem, ListItemShapes, NavigationItem, NavigationItems, TabVariant, Tabs, TabsColors, Text,
    TextField, TextFieldEvent, TextFieldState, TextFieldVariant, ThemeScope, ToggleButton,
    ToggleButtonVariant, Tooltip,
    foundation::{
        Surface, color, set_material_theme, set_window_material_theme, window_material_theme,
    },
    theme::{
        Appearance, ColorRole, CornerRole, CornerShape, Density, Direction, ThemePreset, TypeRole,
        WidthClass,
    },
};

use super::{
    catalog::{
        CATALOG, CatalogRow, GalleryPage, StylePage, TopSection, entry_index, filter_catalog_rows,
        first_of, section_of,
    },
    overview::OverviewView,
    styles_examples::StylesExamples,
};
use crate::{
    playground::ComponentPageView,
    playgrounds::playground_for,
    story::{Story, stack},
};

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

static BRAND_LOGO: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        ImageFormat::Svg,
        include_bytes!("../../assets/logo.svg").to_vec(),
    ))
});

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

    fn apply_theme_colors(
        &mut self,
        preset: ThemePreset,
        appearance: Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preset = preset;
        let mut theme = window_material_theme(window, cx).clone();
        theme.colors = preset.colors(appearance);
        if self.local_theme {
            set_window_material_theme(Some(theme), window, cx);
        } else {
            set_material_theme(theme, cx);
        }
        cx.notify();
    }

    fn select_preset(&mut self, preset: ThemePreset, window: &mut Window, cx: &mut Context<Self>) {
        let appearance = window_material_theme(window, cx).colors.appearance();
        self.apply_theme_colors(preset, appearance, window, cx);
    }

    fn set_appearance(
        &mut self,
        appearance: Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let preset = self.preset;
        self.apply_theme_colors(preset, appearance, window, cx);
    }

    fn overview_view(&mut self, cx: &mut Context<Self>) -> AnyView {
        if let Some(view) = &self.overview {
            return view.clone().into();
        }
        let owner = cx.weak_entity();
        let on_navigate = {
            let owner = owner.clone();
            Rc::new(
                move |page: GalleryPage, window: &mut Window, cx: &mut App| {
                    let _ = owner.update(cx, |this, cx| this.navigate(page, window, cx));
                },
            )
        };
        let active_preset = {
            let owner = owner.clone();
            Rc::new(move |cx: &App| {
                owner
                    .upgrade()
                    .map(|gallery| gallery.read(cx).preset)
                    .unwrap_or_default()
            })
        };
        let on_select_preset = {
            let owner = owner.clone();
            Rc::new(
                move |preset: ThemePreset, window: &mut Window, cx: &mut App| {
                    let _ = owner.update(cx, |this, cx| this.select_preset(preset, window, cx));
                },
            )
        };
        let on_set_appearance = Rc::new(
            move |appearance: Appearance, window: &mut Window, cx: &mut App| {
                let _ = owner.update(cx, |this, cx| this.set_appearance(appearance, window, cx));
            },
        );
        let available_width = self.available_content_width.clone();
        let view = cx.new(|cx| {
            OverviewView::new(
                on_navigate,
                active_preset,
                on_select_preset,
                on_set_appearance,
                available_width,
                cx,
            )
        });
        self.overview = Some(view.clone());
        view.into()
    }

    fn style_view(&mut self, style: StylePage, cx: &mut Context<Self>) -> AnyView {
        if let Some((_, view)) = self.styles.iter().find(|(entry, _)| *entry == style) {
            return view.clone();
        }
        let owner = cx.weak_entity();
        let active_preset = {
            let owner = owner.clone();
            Rc::new(move |cx: &App| {
                owner
                    .upgrade()
                    .map(|gallery| gallery.read(cx).preset)
                    .unwrap_or_default()
            })
        };
        let on_select_preset = Rc::new(
            move |preset: ThemePreset, window: &mut Window, cx: &mut App| {
                let _ = owner.update(cx, |this, cx| this.select_preset(preset, window, cx));
            },
        );
        let view: AnyView = cx
            .new(|_| StylesExamples::new(style, active_preset, on_select_preset))
            .into();
        self.styles.push((style, view.clone()));
        view
    }

    fn build_story_view(
        story: Story,
        available_width: Rc<Cell<Pixels>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyView> {
        if let Some(playground) = playground_for(story) {
            return Some(
                cx.new(|cx| ComponentPageView::new(story, playground, available_width, window, cx))
                    .into(),
            );
        }
        story.new_view(window, cx)
    }

    fn story_view(&mut self, story: Story, window: &mut Window, cx: &mut App) -> Option<AnyView> {
        if let Some((_, view)) = self.stories.iter().find(|(entry, _)| *entry == story) {
            return Some(view.clone());
        }
        let view = Self::build_story_view(story, self.available_content_width.clone(), window, cx)?;
        self.stories.push((story, view.clone()));
        Some(view)
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

    fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().trim().to_ascii_lowercase()
    }

    fn refresh_filtered_rows(&mut self, cx: &mut Context<Self>) {
        let has_value = !self.search.read(cx).value().is_empty();
        let query = self.query(cx);
        let next = filter_catalog_rows(&query);
        let list_changed = self.filtered_rows.as_ref() != next.as_slice();
        let clear_button_changed = self.search_has_value != has_value;
        self.search_has_value = has_value;
        if list_changed {
            self.filtered_rows = next.into();
            self.catalog_list.reset(self.filtered_rows.len());
            self.catalog_list.scroll_to(ListOffset {
                item_ix: 0,
                offset_in_item: px(0.),
            });
        }
        if list_changed || clear_button_changed {
            cx.notify();
        }
    }

    fn open_search_result(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let query = self.query(cx);
        if query.is_empty() {
            return;
        }
        let result = GalleryPage::find(&query).or_else(|| {
            CATALOG
                .iter()
                .find(|entry| entry.matches(&query))
                .map(|entry| entry.page)
        });
        if let Some(page) = result {
            self.navigate(page, window, cx);
            self.content_focus.focus(window, cx);
        }
    }

    fn reveal_page_in_catalog(&self, page: GalleryPage) {
        let Some(target_entry) = entry_index(page) else {
            return;
        };
        let Some(row_ix) = self
            .filtered_rows
            .iter()
            .position(|row| *row == CatalogRow::Entry(target_entry))
        else {
            return;
        };
        if self.catalog_list.bounds_for_item(row_ix).is_some() {
            self.catalog_list.scroll_to_reveal_item(row_ix);
        } else {
            self.catalog_list.scroll_to(ListOffset {
                item_ix: row_ix.saturating_sub(1),
                offset_in_item: px(0.),
            });
        }
    }

    fn navigate(&mut self, page: GalleryPage, window: &mut Window, cx: &mut Context<Self>) {
        let page_changed = self.current != page;
        if page_changed {
            self.current = page;
            self.content_scroll.set_offset(point(px(0.), px(0.)));
            match page {
                GalleryPage::Story(Story::Introduction) => {
                    self.overview_view(cx);
                }
                GalleryPage::Story(story) => {
                    self.story_view(story, window, cx);
                }
                GalleryPage::Style(style) => {
                    self.style_view(style, cx);
                }
            }
            self.reveal_page_in_catalog(page);
        }
        let closed_catalog = std::mem::take(&mut self.single_pane_catalog_open);
        if Self::single_pane(Self::width_class(window)) {
            self.content_focus.focus(window, cx);
        }
        if page_changed || closed_catalog {
            cx.notify();
        }
    }

    fn toggle_catalog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = if Self::single_pane(Self::width_class(window)) {
            self.single_pane_catalog_open = !self.single_pane_catalog_open;
            self.single_pane_catalog_open
        } else {
            self.catalog_collapsed = !self.catalog_collapsed;
            !self.catalog_collapsed
        };
        if open {
            self.search.read(cx).focus_handle(cx).focus(window, cx);
        } else {
            self.content_focus.focus(window, cx);
        }
        cx.notify();
    }

    fn render_section_tabs(
        &self,
        id_prefix: &'static str,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let active_section = section_of(self.current);
        let owner = cx.weak_entity();
        let destinations = TopSection::ALL
            .into_iter()
            .map(|section| NavigationItem::new(section.id(), section.title()));
        let items = NavigationItems::new(id_prefix, "Sections", destinations)
            .selected(active_section.id())
            .on_select(move |id, window, cx| {
                let Some(section) = TopSection::ALL
                    .into_iter()
                    .find(|candidate| candidate.id() == id.as_ref())
                else {
                    return;
                };
                let _ = owner.update(cx, |this, cx| {
                    this.navigate(first_of(section), window, cx);
                });
            });
        let mut compact = window_material_theme(window, cx).clone();
        compact.layout.density = Density::Compact;
        let tabs_width = if f32::from(window.viewport_size().width) >= 920. {
            px(324.)
        } else {
            px(264.)
        };
        div().w(tabs_width).flex_none().child(
            ThemeScope::new(
                Tabs::new(items)
                    .variant(TabVariant::Secondary)
                    .colors(TabsColors {
                        container: Some(transparent_black()),
                        divider: Some(transparent_black()),
                        ..Default::default()
                    }),
            )
            .theme(compact),
        )
    }

    fn render_catalog(
        &self,
        single_pane: bool,
        inset: Pixels,
        radius: Pixels,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let rows = self.filtered_rows.clone();
        let handles = self.catalog_focus_handles.clone();
        let no_results = rows.is_empty();
        let current = self.current;
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let surface = color(theme.colors.color(ColorRole::Surface));
        let owner = cx.weak_entity();
        let mut search = TextField::new("gallery-search", "Search catalog", &self.search)
            .variant(TextFieldVariant::Outlined);
        if self.search_has_value {
            let state = self.search.downgrade();
            search = search.trailing(
                IconButton::new("clear-search", "Clear search", Icon::new(IconName::Close))
                    .size(ControlSize::ExtraSmall)
                    .on_activate(move |window, cx| {
                        let _ = state.update(cx, |search, cx| {
                            search.focus(window, cx);
                            search.replace_all("", window, cx);
                        });
                    }),
            );
        }

        let tab_stop = rows
            .iter()
            .enumerate()
            .find_map(|(row_ix, row)| match *row {
                CatalogRow::Entry(entry_ix) if handles[entry_ix].is_focused(window) => Some(row_ix),
                _ => None,
            })
            .or_else(|| {
                rows.iter()
                    .enumerate()
                    .find_map(|(row_ix, row)| match *row {
                        CatalogRow::Entry(entry_ix) if CATALOG[entry_ix].page == current => {
                            Some(row_ix)
                        }
                        _ => None,
                    })
            })
            .or_else(|| {
                rows.iter()
                    .position(|row| matches!(row, CatalogRow::Entry(_)))
            });

        let nav_rows = rows.clone();
        let nav_handles = handles.clone();
        let nav_list = self.catalog_list.clone();

        div()
            .id("gallery-catalog-pane")
            .key_context("GalleryCatalog")
            .test_support()
            .track_focus(&self.catalog_focus)
            .on_action(cx.listener(move |this, _: &CloseCatalog, window, cx| {
                if single_pane {
                    this.toggle_catalog(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(radius)
            .bg(surface)
            .child(
                stack()
                    .gap_3()
                    .flex_none()
                    .px(inset)
                    .py_4()
                    .when(rtl, Styled::items_end)
                    .when(single_pane, |header| {
                        header.child(self.render_section_tabs("drawer-section", window, cx))
                    })
                    .child(search),
            )
            .child(
                div()
                    .id("gallery-catalog")
                    .tab_group()
                    .min_h_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(if no_results {
                        div()
                            .px(inset)
                            .pb_4()
                            .child(
                                div()
                                    .id("no-components")
                                    .role(Role::Status)
                                    .aria_label("No matching components")
                                    .test_support()
                                    .p_2()
                                    .child(Text::new("No matching components")),
                            )
                            .into_any_element()
                    } else {
                        let render_rows = rows;
                        let render_handles = handles;
                        div()
                            .id("component-catalog")
                            .role(Role::List)
                            .test_support()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .on_key_down(move |event, window, cx| {
                                let entry_positions: Vec<(usize, usize)> = nav_rows
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(row_ix, row)| match *row {
                                        CatalogRow::Entry(entry_ix) => Some((row_ix, entry_ix)),
                                        CatalogRow::GroupHeader(_) => None,
                                    })
                                    .collect();
                                if entry_positions.is_empty() {
                                    return;
                                }
                                let Some(active) =
                                    entry_positions.iter().position(|&(_, entry_ix)| {
                                        nav_handles[entry_ix].is_focused(window)
                                    })
                                else {
                                    return;
                                };
                                let count = entry_positions.len();
                                let next = match event.keystroke.key.as_str() {
                                    "up" => Some((active + count - 1) % count),
                                    "down" => Some((active + 1) % count),
                                    "home" => Some(0),
                                    "end" => Some(count - 1),
                                    _ => None,
                                };
                                if let Some(next) = next {
                                    let (row_ix, entry_ix) = entry_positions[next];
                                    nav_handles[entry_ix].focus(window, cx);
                                    if nav_list.bounds_for_item(row_ix).is_some() {
                                        nav_list.scroll_to_reveal_item(row_ix);
                                    } else {
                                        nav_list.scroll_to(ListOffset {
                                            item_ix: row_ix,
                                            offset_in_item: px(0.),
                                        });
                                    }
                                    cx.stop_propagation();
                                    window.refresh();
                                }
                            })
                            .child(
                                list(self.catalog_list.clone(), move |row_ix, _, _| {
                                    match render_rows[row_ix] {
                                        CatalogRow::GroupHeader(group) => div()
                                            .id((ElementId::from("catalog-group"), group.id()))
                                            .w_full()
                                            .px(inset)
                                            .pt(if row_ix == 0 { px(4.) } else { px(14.) })
                                            .pb(px(6.))
                                            .flex()
                                            .when(rtl, Styled::justify_end)
                                            .child(
                                                Text::new(group.title())
                                                    .role(TypeRole::LabelMedium)
                                                    .color(ColorRole::OnSurfaceVariant),
                                            )
                                            .into_any_element(),
                                        CatalogRow::Entry(entry_ix) => {
                                            let entry = CATALOG[entry_ix];
                                            let owner = owner.clone();
                                            let is_first_in_group = row_ix == 0
                                                || matches!(
                                                    render_rows.get(row_ix.saturating_sub(1)),
                                                    Some(CatalogRow::GroupHeader(_))
                                                );
                                            let is_last_in_group = row_ix + 1 == render_rows.len()
                                                || matches!(
                                                    render_rows.get(row_ix + 1),
                                                    Some(CatalogRow::GroupHeader(_))
                                                );
                                            let top = if is_first_in_group { 16. } else { 4. };
                                            let bottom = if is_last_in_group { 16. } else { 4. };
                                            let shapes = ListItemShapes {
                                                resting: Some(CornerShape::Radii([
                                                    top, top, bottom, bottom,
                                                ])),
                                                selected: Some(CornerRole::Large.shape()),
                                                ..ListItemShapes::default()
                                            };
                                            let is_last_row = row_ix + 1 == render_rows.len();
                                            div()
                                                .w_full()
                                                .px(inset)
                                                .pb(if is_last_row { px(16.) } else { px(2.) })
                                                .child(
                                                    ListItem::new(
                                                        (
                                                            ElementId::from("gallery-story"),
                                                            entry.page.id(),
                                                        ),
                                                        entry.page.name(),
                                                    )
                                                    .shapes(shapes)
                                                    .selected(current == entry.page)
                                                    .track_focus(&render_handles[entry_ix])
                                                    .tab_stop(Some(row_ix) == tab_stop)
                                                    .on_activate(move |window, cx| {
                                                        let _ = owner.update(cx, |this, cx| {
                                                            this.navigate(entry.page, window, cx);
                                                        });
                                                    }),
                                                )
                                                .into_any_element()
                                        }
                                    }
                                })
                                .size_full(),
                            )
                            .into_any_element()
                    }),
            )
    }

    fn render_story(
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

    fn render_brand(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let owner = cx.weak_entity();
        Surface::clickable(
            "gallery-brand",
            "GPUI Material Overview",
            ColorRole::SurfaceContainer,
        )
        .shape(CornerShape::uniform(12.))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .when(rtl, Styled::flex_row_reverse)
        .on_activate(move |window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.navigate(GalleryPage::Story(Story::Introduction), window, cx);
            });
        })
        .child(
            img(BRAND_LOGO.clone())
                .id("brand-mark")
                .size(px(28.))
                .object_fit(ObjectFit::Contain),
        )
        .child(
            Text::new("GPUI Material")
                .id("brand-title")
                .role(TypeRole::TitleSmall)
                .emphasized(true),
        )
    }

    fn render_app_bar(
        &self,
        single_pane: bool,
        show_catalog: bool,
        margin: Pixels,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let (label, icon) = if show_catalog {
            ("Hide components", IconName::LeftPanelClose)
        } else {
            ("Show components", IconName::LeftPanelOpen)
        };
        let owner = cx.weak_entity();
        div()
            .w_full()
            .flex_none()
            .px(margin)
            .py_2()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_2()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(Tooltip::plain(
                        "browse-components-tooltip",
                        IconButton::new(
                            "browse-components",
                            label,
                            Icon::new(icon).mirror_in_rtl(true),
                        )
                        .on_activate(move |window, cx| {
                            let _ = owner.update(cx, |this, cx| this.toggle_catalog(window, cx));
                        }),
                        label,
                    ))
                    .when(!single_pane, |left| {
                        left.child(self.render_brand(window, cx))
                            .child(self.render_section_tabs("top-section", window, cx))
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(self.render_theme_toggle(window, cx))
                    .child(Tooltip::plain(
                        "repository-tooltip",
                        IconButton::new(
                            "open-repository",
                            "Open GitHub repository",
                            svg()
                                .data(include_bytes!("../../assets/mark-github-24.svg"))
                                .size_full()
                                .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant))),
                        )
                        .on_activate(|_, cx| cx.open_url(REPOSITORY_URL)),
                        "Open GitHub repository",
                    ))
                    .child(
                        ToggleButton::new("toggle-motion", "Reduced motion", cx.reduce_motion())
                            .variant(ToggleButtonVariant::Outlined)
                            .size(ControlSize::ExtraSmall)
                            .on_change(|next, _, cx| cx.set_reduce_motion(next)),
                    ),
            )
    }

    fn render_theme_toggle(&self, window: &Window, cx: &Context<Self>) -> Tooltip {
        let (label, icon, appearance) = match window_material_theme(window, cx).colors.appearance()
        {
            Appearance::Light => ("Switch to dark theme", IconName::DarkMode, Appearance::Dark),
            Appearance::Dark => (
                "Switch to light theme",
                IconName::LightMode,
                Appearance::Light,
            ),
        };
        let owner = cx.weak_entity();
        let button = IconButton::new("toggle-theme", label, Icon::new(icon)).on_activate(
            move |window, cx| {
                let _ = owner.update(cx, |this, cx| {
                    this.set_appearance(appearance, window, cx);
                });
            },
        );
        Tooltip::plain("theme-tooltip", button, label)
    }

    fn render_status_bar(&self, margin: Pixels, window: &Window, cx: &App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let appearance = theme.colors.appearance();
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let theme_status = format!("Window theme {appearance:?}");
        div()
            .w_full()
            .flex_none()
            .px(margin)
            .py_2()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap_3()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                div()
                    .id("window-theme")
                    .role(Role::Status)
                    .aria_label(theme_status.clone())
                    .test_support()
                    .child(
                        Text::new(theme_status)
                            .id("label")
                            .role(TypeRole::LabelMedium),
                    ),
            )
            .child(
                div()
                    .id("gallery-version")
                    .role(Role::Status)
                    .aria_label(version.clone())
                    .test_support()
                    .child(Text::new(version).id("label").role(TypeRole::LabelMedium)),
            )
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
