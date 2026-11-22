use std::rc::Rc;

use gpui::{
    AnyView, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ListAlignment, ListOffset, ListState, ParentElement, Pixels,
    Render, Role, ScrollHandle, Size, StatefulInteractiveElement, Styled, Subscription, Window,
    div, list, point, prelude::FluentBuilder, px, svg,
};
use gpui_base::{TestSupportExt, TextView, TextViewStyle};
use gpui_material::{
    ControlSize, Fab, FabColor, FloatingAction, Icon, IconButton, IconName, ListDetail,
    ListDetailPane, ListItem, ListItemShapes, Text, TextField, TextFieldEvent, TextFieldState,
    TextFieldVariant, ToggleButton, ToggleButtonVariant, Tooltip,
    foundation::{
        Surface, color, set_material_theme, set_window_material_theme, text_style,
        window_material_theme,
    },
    theme::{
        Appearance, ColorRole, CornerRole, CornerShape, Direction, TypeRole, TypographyEmphasis,
        WidthClass,
    },
};

use crate::story::{Story, stack};

gpui::actions!(gallery, [CloseCatalog]);

const COMPACT_MARGIN: Pixels = px(16.);
const PANE_SPACING: Pixels = px(24.);
const STORY_MAX_WIDTH: Pixels = px(1120.);
const README_MAX_WIDTH: Pixels = px(840.);
const REPOSITORY_URL: &str = "https://github.com/wangdiandao/gpui-material";
const README: &str = include_str!("../../../README.md");

/// The full Gallery view with a searchable catalog and retained example state.
pub struct Gallery {
    local_theme: bool,
    catalog_collapsed: bool,
    single_pane_catalog_open: bool,
    current: Story,
    search: Entity<TextFieldState>,
    search_has_value: bool,
    catalog_list: ListState,
    catalog_focus_handles: Rc<[FocusHandle]>,
    filtered_stories: Rc<[usize]>,
    catalog_focus: FocusHandle,
    content_scroll: ScrollHandle,
    content_focus: FocusHandle,
    last_viewport: Size<Pixels>,
    stories: Vec<(Story, AnyView)>,
    _search: Subscription,
    _bounds: Subscription,
}

impl Gallery {
    /// Creates a Gallery in an existing GPUI window, starting at `initial_story`.
    /// The theme toggle updates the application's Material theme by default.
    pub fn new(initial_story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextFieldState::new(window, cx));
        let catalog_list = ListState::new(Story::ALL.len(), ListAlignment::Top, px(200.));
        let catalog_focus_handles: Rc<[FocusHandle]> = Story::ALL
            .iter()
            .map(|_| cx.focus_handle())
            .collect::<Vec<_>>()
            .into();
        let filtered_stories: Rc<[usize]> = (0..Story::ALL.len()).collect::<Vec<_>>().into();
        let observe_search =
            cx.subscribe_in(
                &search,
                window,
                move |this, _, event, window, cx| match event {
                    TextFieldEvent::Change => {
                        this.refresh_filtered_stories(cx);
                    }
                    TextFieldEvent::PressEnter {
                        secondary: false,
                        shift: false,
                    } => this.open_search_result(window, cx),
                    _ => {}
                },
            );
        let mut stories = Vec::new();
        if let Some(view) = initial_story.new_view(window, cx) {
            stories.push((initial_story, view));
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
            catalog_collapsed: false,
            single_pane_catalog_open: false,
            current: initial_story,
            search,
            search_has_value: false,
            catalog_list,
            catalog_focus_handles,
            filtered_stories,
            catalog_focus: cx.focus_handle(),
            content_scroll: ScrollHandle::new(),
            content_focus: cx.focus_handle(),
            last_viewport: window.viewport_size(),
            stories,
            _search: observe_search,
            _bounds: bounds,
        }
    }

    /// Scopes changes from the theme toggle to the current window when enabled.
    pub fn with_window_theme_toggle(mut self, enabled: bool) -> Self {
        self.local_theme = enabled;
        self
    }

    #[cfg(test)]
    pub(crate) fn loaded_story_count(&self) -> usize {
        self.stories.len()
    }

    #[cfg(test)]
    pub(crate) fn story_view_id(&self, story: Story) -> Option<gpui::EntityId> {
        self.stories
            .iter()
            .find(|(entry, _)| *entry == story)
            .map(|(_, view)| view.entity_id())
    }

    #[cfg(test)]
    pub(crate) fn catalog_story_focus(&self, story: Story) -> Option<FocusHandle> {
        Story::ALL
            .iter()
            .position(|entry| *entry == story)
            .map(|ix| self.catalog_focus_handles[ix].clone())
    }

    fn story_view(&mut self, story: Story, window: &mut Window, cx: &mut App) -> Option<AnyView> {
        if let Some((_, view)) = self.stories.iter().find(|(entry, _)| *entry == story) {
            return Some(view.clone());
        }
        let view = story.new_view(window, cx)?;
        self.stories.push((story, view.clone()));
        Some(view)
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

    fn refresh_filtered_stories(&mut self, cx: &mut Context<Self>) {
        let has_value = !self.search.read(cx).value().is_empty();
        let query = self.query(cx);
        let next: Vec<usize> = Story::ALL
            .iter()
            .enumerate()
            .filter_map(|(ix, story)| story.matches(&query).then_some(ix))
            .collect();
        let list_changed = self.filtered_stories.as_ref() != next.as_slice();
        let clear_button_changed = self.search_has_value != has_value;
        self.search_has_value = has_value;
        if list_changed {
            self.filtered_stories = next.into();
            self.catalog_list.reset(self.filtered_stories.len());
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
        let result = Story::find(&query).or_else(|| {
            Story::ALL
                .iter()
                .copied()
                .find(|story| story.matches(&query))
        });
        if let Some(story) = result {
            self.navigate(story, window, cx);
            self.content_focus.focus(window, cx);
        }
    }

    fn navigate(&mut self, story: Story, window: &mut Window, cx: &mut Context<Self>) {
        let story_changed = self.current != story;
        if story_changed {
            self.current = story;
            self.content_scroll.set_offset(point(px(0.), px(0.)));
            self.story_view(story, window, cx);
        }
        let closed_catalog = std::mem::take(&mut self.single_pane_catalog_open);
        if Self::single_pane(Self::width_class(window)) {
            self.content_focus.focus(window, cx);
        }
        if story_changed || closed_catalog {
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

    fn render_catalog(
        &self,
        single_pane: bool,
        inset: Pixels,
        radius: Pixels,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let results = self.filtered_stories.clone();
        let handles = self.catalog_focus_handles.clone();
        let no_results = results.is_empty();
        let current = self.current;
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let surface = color(theme.colors.color(ColorRole::Surface));
        let owner = cx.weak_entity();
        let mut search = TextField::new("gallery-search", "Search components", &self.search)
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

        let tab_stop = results
            .iter()
            .position(|&story_ix| handles[story_ix].is_focused(window))
            .or_else(|| {
                results
                    .iter()
                    .position(|&story_ix| Story::ALL[story_ix] == current)
            })
            .or_else(|| (!results.is_empty()).then_some(0));

        let nav_results = results.clone();
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
                    .child(Text::new("Components").role(TypeRole::TitleMedium))
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
                        let last = results.len() - 1;
                        let render_results = results;
                        let render_handles = handles;
                        div()
                            .id("component-catalog")
                            .role(Role::List)
                            .test_support()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .on_key_down(move |event, window, cx| {
                                let Some(active) = nav_results
                                    .iter()
                                    .position(|&story_ix| nav_handles[story_ix].is_focused(window))
                                else {
                                    return;
                                };
                                let count = nav_results.len();
                                let next = match event.keystroke.key.as_str() {
                                    "up" => Some((active + count - 1) % count),
                                    "down" => Some((active + 1) % count),
                                    "home" => Some(0),
                                    "end" => Some(count - 1),
                                    _ => None,
                                };
                                if let Some(next) = next {
                                    nav_handles[nav_results[next]].focus(window, cx);
                                    if nav_list.bounds_for_item(next).is_some() {
                                        nav_list.scroll_to_reveal_item(next);
                                    } else {
                                        nav_list.scroll_to(ListOffset {
                                            item_ix: next,
                                            offset_in_item: px(0.),
                                        });
                                    }
                                    cx.stop_propagation();
                                    window.refresh();
                                }
                            })
                            .child(
                                list(self.catalog_list.clone(), move |ix, _, _| {
                                    let story_ix = render_results[ix];
                                    let story = Story::ALL[story_ix];
                                    let owner = owner.clone();
                                    let top = if ix == 0 { 16. } else { 4. };
                                    let bottom = if ix == last { 16. } else { 4. };
                                    let shapes = ListItemShapes {
                                        resting: Some(CornerShape::Radii([
                                            top, top, bottom, bottom,
                                        ])),
                                        selected: Some(CornerRole::Large.shape()),
                                        ..ListItemShapes::default()
                                    };
                                    div()
                                        .w_full()
                                        .px(inset)
                                        .pb(if ix == last { px(16.) } else { px(2.) })
                                        .child(
                                            ListItem::new(
                                                (ElementId::from("gallery-story"), story.id()),
                                                story.name(),
                                            )
                                            .shapes(shapes)
                                            .selected(current == story)
                                            .track_focus(
                                                &render_handles[story_ix],
                                                Some(ix) == tab_stop,
                                            )
                                            .on_activate(move |window, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    this.navigate(story, window, cx);
                                                });
                                            }),
                                        )
                                        .into_any_element()
                                })
                                .size_full(),
                            )
                            .into_any_element()
                    }),
            )
    }

    fn render_introduction(&self, window: &Window, cx: &App) -> impl IntoElement {
        let scheme = &window_material_theme(window, cx).colors;
        let tone = |role| color(scheme.color(role));
        let style = TextViewStyle::default()
            .with_dark(scheme.appearance() == Appearance::Dark)
            .with_foreground(tone(ColorRole::OnSurface))
            .with_muted_foreground(tone(ColorRole::OnSurfaceVariant))
            .with_link(tone(ColorRole::Primary))
            .with_border(tone(ColorRole::OutlineVariant))
            .with_code_background(tone(ColorRole::SurfaceContainer))
            .with_inline_code(gpui::HighlightStyle {
                background_color: Some(tone(ColorRole::SurfaceContainer)),
                ..Default::default()
            })
            .with_selection(tone(ColorRole::SecondaryContainer));
        text_style(
            div().w_full().max_w(README_MAX_WIDTH).mx_auto(),
            TypeRole::BodyLarge.style(TypographyEmphasis::Baseline),
            1.,
            window,
            cx,
        )
        .child(
            TextView::markdown("gallery-readme", README)
                .style(style)
                .selectable(true)
                .on_link_click(|url, event, _, cx| {
                    if event.is_right_click() {
                        return;
                    }
                    if url.starts_with("https://") || url.starts_with("http://") {
                        cx.open_url(url);
                    } else {
                        cx.open_url(&format!("{REPOSITORY_URL}/blob/main/{url}"));
                    }
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
            Story::Introduction => self.render_introduction(window, cx).into_any_element(),
            story => self
                .story_view(story, window, cx)
                .expect("registered story")
                .into_any_element(),
        };
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;

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
            .children((self.current != Story::Introduction).then(|| {
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
                                    .id("story-title")
                                    .role(Role::Heading)
                                    .aria_label(self.current.name())
                                    .test_support()
                                    .child(
                                        Text::new(self.current.name())
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
                    .id("gallery-scroll")
                    .tab_group()
                    .track_scroll(&self.content_scroll)
                    .overflow_y_scroll()
                    .flex_1()
                    .min_h_0()
                    .child(if self.current == Story::AdaptiveLayout {
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
    }

    fn render_app_bar(
        &self,
        show_catalog: bool,
        theme_in_app_bar: bool,
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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .children(theme_in_app_bar.then(|| self.render_theme_toggle(false, window, cx)))
                    .child(Tooltip::plain(
                        "repository-tooltip",
                        IconButton::new(
                            "open-repository",
                            "Open GitHub repository",
                            svg()
                                .data(include_bytes!("../assets/mark-github-24.svg"))
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

    fn render_theme_toggle(&self, floating: bool, window: &Window, cx: &App) -> Tooltip {
        let (label, icon, appearance) = match window_material_theme(window, cx).colors.appearance()
        {
            Appearance::Light => ("Switch to dark theme", IconName::DarkMode, Appearance::Dark),
            Appearance::Dark => (
                "Switch to light theme",
                IconName::LightMode,
                Appearance::Light,
            ),
        };
        let local_theme = self.local_theme;
        let activate = move |window: &mut Window, cx: &mut App| {
            let mut theme = window_material_theme(window, cx).clone();
            theme.colors = gpui_material::theme::ColorScheme::baseline(appearance);
            if local_theme {
                set_window_material_theme(Some(theme), window, cx);
            } else {
                set_material_theme(theme, cx);
            }
        };
        let icon = Icon::new(icon);
        let button = if floating {
            Fab::new("toggle-theme", label, icon)
                .color(FabColor::Secondary)
                .on_activate(activate)
                .into_any_element()
        } else {
            IconButton::new("toggle-theme", label, icon)
                .on_activate(activate)
                .into_any_element()
        };
        Tooltip::plain("theme-tooltip", button, label)
    }

    fn render_floating_actions(
        &self,
        show_story: bool,
        window: &Window,
        cx: &App,
    ) -> FloatingAction {
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        FloatingAction::new(
            div()
                .flex()
                .gap_2()
                .when(rtl, Styled::flex_row_reverse)
                .children((show_story && self.current == Story::Fab).then(|| {
                    Fab::new(
                        "floating-motion",
                        "Toggle reduced motion",
                        Icon::new(IconName::MotionPhotosOn),
                    )
                    .on_activate(|_, cx| cx.set_reduce_motion(!cx.reduce_motion()))
                }))
                .child(self.render_theme_toggle(true, window, cx)),
        )
        .margin(px(52.))
    }

    fn render_status_bar(&self, margin: Pixels, window: &Window, cx: &App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let appearance = theme.colors.appearance();
        let status = format!("Display scale {:.0}%", window.scale_factor() * 100.);
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
                    .child(Text::new(theme_status).role(TypeRole::LabelMedium)),
            )
            .child(
                div()
                    .id("display-scale")
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .test_support()
                    .child(Text::new(status).role(TypeRole::LabelMedium)),
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
        let show_story = !single_pane || !show_catalog;
        // Keep the document and the adaptive example unobstructed.
        let theme_in_app_bar = matches!(self.current, Story::Introduction | Story::AdaptiveLayout);
        let margin = if width_class == WidthClass::Compact {
            COMPACT_MARGIN
        } else {
            PANE_SPACING
        };
        let inset = if single_pane { margin } else { COMPACT_MARGIN };
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
            .child(self.render_app_bar(show_catalog, theme_in_app_bar, margin, window, cx))
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
            .children(
                (!theme_in_app_bar).then(|| self.render_floating_actions(show_story, window, cx)),
            )
    }
}
