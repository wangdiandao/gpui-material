use gpui::{
    AnyView, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, Role, ScrollHandle,
    StatefulInteractiveElement, Styled, Subscription, Window, div, point, prelude::FluentBuilder,
    px, svg,
};
use gpui_base::{TestSupportExt, TextView, TextViewStyle};
use gpui_material::{
    ControlSize, Fab, FabColor, FloatingAction, Icon, IconButton, IconName, List, ListDetail,
    ListDetailPane, ListItem, ListVariant, Text, TextField, TextFieldEvent, TextFieldState,
    TextFieldVariant, ToggleButton, ToggleButtonVariant, Tooltip,
    foundation::{
        Surface, color, set_material_theme, set_window_material_theme, text_style,
        window_material_theme,
    },
    theme::{
        Appearance, ColorRole, CornerRole, Direction, TypeRole, TypographyEmphasis, WidthClass,
    },
};

use crate::story::{Story, stack};

gpui::actions!(gallery, [CloseCatalog]);

const COMPACT_MARGIN: Pixels = px(16.);
const PANE_SPACING: Pixels = px(24.);
const STORY_MAX_WIDTH: Pixels = px(1120.);
const README_MAX_WIDTH: Pixels = px(840.);
pub(crate) const REPOSITORY_URL: &str = "https://github.com/wangdiandao/gpui-material";
const README: &str = include_str!("../../../README.md");

pub(crate) struct Gallery {
    local_theme: bool,
    catalog_collapsed: bool,
    single_pane_catalog_open: bool,
    current: Story,
    search: Entity<TextFieldState>,
    catalog_scroll: ScrollHandle,
    catalog_focus: FocusHandle,
    content_scroll: ScrollHandle,
    content_focus: FocusHandle,
    stories: Vec<(Story, AnyView)>,
    _search: Subscription,
    _bounds: Subscription,
}

impl Gallery {
    pub(crate) fn new(
        local_theme: bool,
        initial_story: Story,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| TextFieldState::new(window, cx));
        let catalog_scroll = ScrollHandle::new();
        let observe_search =
            cx.subscribe_in(
                &search,
                window,
                move |this, _, event, window, cx| match event {
                    TextFieldEvent::Change => {
                        this.catalog_scroll.set_offset(point(px(0.), px(0.)));
                        cx.notify();
                    }
                    TextFieldEvent::PressEnter {
                        secondary: false,
                        shift: false,
                    } => this.open_search_result(window, cx),
                    _ => {}
                },
            );
        let stories = Story::ALL
            .iter()
            .filter_map(|&story| story.new_view(window, cx).map(|view| (story, view)))
            .collect();
        let bounds = cx.observe_window_bounds(window, |this, window, cx| {
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
            cx.notify();
        });
        Self {
            local_theme,
            catalog_collapsed: false,
            single_pane_catalog_open: false,
            current: initial_story,
            search,
            catalog_scroll,
            catalog_focus: cx.focus_handle(),
            content_scroll: ScrollHandle::new(),
            content_focus: cx.focus_handle(),
            stories,
            _search: observe_search,
            _bounds: bounds,
        }
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
        if self.current != story {
            self.current = story;
            self.content_scroll.set_offset(point(px(0.), px(0.)));
        }
        self.single_pane_catalog_open = false;
        if Self::single_pane(Self::width_class(window)) {
            self.content_focus.focus(window, cx);
        }
        cx.notify();
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
        let query = self.query(cx);
        let results: Vec<_> = Story::ALL
            .iter()
            .copied()
            .filter(|story| story.matches(&query))
            .collect();
        let no_results = results.is_empty();
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let surface = color(theme.colors.color(ColorRole::Surface));
        let owner = cx.weak_entity();
        let mut search = TextField::new("gallery-search", "Search components", &self.search)
            .variant(TextFieldVariant::Outlined);
        if !self.search.read(cx).value().is_empty() {
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
                    .track_scroll(&self.catalog_scroll)
                    .overflow_y_scroll()
                    .min_h_0()
                    .flex_1()
                    .child(
                        div()
                            .px(inset)
                            .pb_4()
                            .child(
                                List::new("component-catalog")
                                    .variant(ListVariant::Segmented)
                                    .items(results.into_iter().map(|story| {
                                        let owner = owner.clone();
                                        ListItem::new(
                                            (ElementId::from("gallery-story"), story.id()),
                                            story.name(),
                                        )
                                        .selected(self.current == story)
                                        .on_activate(
                                            move |window, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    this.navigate(story, window, cx);
                                                });
                                            },
                                        )
                                    })),
                            )
                            .when(no_results, |panel| {
                                panel.child(
                                    div()
                                        .id("no-components")
                                        .role(Role::Status)
                                        .aria_label("No matching components")
                                        .test_support()
                                        .p_2()
                                        .child(Text::new("No matching components")),
                                )
                            }),
                    ),
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
        &self,
        inset: Pixels,
        radius: Pixels,
        window: &Window,
        cx: &App,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let content = match self.current {
            Story::Introduction => self.render_introduction(window, cx).into_any_element(),
            _ => self
                .stories
                .iter()
                .find(|(story, _)| *story == self.current)
                .map(|(_, view)| view.clone().into_any_element())
                .expect("registered story"),
        };

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
