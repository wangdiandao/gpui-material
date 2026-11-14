use gpui::{
    AnyView, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Role, ScrollHandle,
    StatefulInteractiveElement, Styled, Subscription, Window, div, point, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonVariant, ControlSize, Divider, Fab, FloatingAction, IconButton, List, ListItem,
    ListVariant, Text, TextField, TextFieldEvent, TextFieldState, TextFieldVariant, ToggleButton,
    ToggleButtonVariant,
    foundation::{
        Surface, color, set_material_theme, set_window_material_theme, window_material_theme,
    },
    theme::{Appearance, ColorRole, Direction, Elevation, TypeRole},
};

use crate::{
    drawing, input_examples,
    story::{Story, section, stack},
};

gpui::actions!(gallery, [CloseCatalog]);

pub(crate) struct Gallery {
    local_theme: bool,
    sidebar_collapsed: bool,
    compact_catalog_open: bool,
    current: Story,
    search: Entity<TextFieldState>,
    catalog_scroll: ScrollHandle,
    content_scroll: ScrollHandle,
    content_focus: FocusHandle,
    stories: Vec<(Story, AnyView)>,
    drawing: Entity<drawing::Drawing>,
    input: Entity<input_examples::InputExamples>,
    _search: Subscription,
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
        Self {
            local_theme,
            sidebar_collapsed: false,
            compact_catalog_open: false,
            current: initial_story,
            search,
            catalog_scroll,
            content_scroll: ScrollHandle::new(),
            content_focus: cx.focus_handle(),
            stories,
            drawing: cx.new(drawing::Drawing::new),
            input: cx.new(|cx| input_examples::InputExamples::new(window, cx)),
            _search: observe_search,
        }
    }

    fn compact(window: &Window) -> bool {
        window.viewport_size().width < px(700.)
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
        if Self::compact(window) {
            self.compact_catalog_open = false;
            self.content_focus.focus(window, cx);
        }
        cx.notify();
    }

    fn toggle_catalog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = if Self::compact(window) {
            self.compact_catalog_open = !self.compact_catalog_open;
            self.compact_catalog_open
        } else {
            self.sidebar_collapsed = !self.sidebar_collapsed;
            !self.sidebar_collapsed
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
        compact: bool,
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
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let owner = cx.weak_entity();
        let mut search = TextField::new("gallery-search", "Search components", &self.search)
            .variant(TextFieldVariant::Outlined);
        if !self.search.read(cx).value().is_empty() {
            let state = self.search.downgrade();
            search = search.trailing(
                IconButton::new("clear-search", "Clear search", "×")
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
            .on_action(cx.listener(move |this, _: &CloseCatalog, window, cx| {
                if compact {
                    this.toggle_catalog(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .when_else(
                compact,
                |panel| panel.flex_1().min_w_0(),
                |panel| panel.w_64().flex_none(),
            )
            .min_h_0()
            .child(
                Surface::new(ColorRole::SurfaceContainerLow)
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(
                        stack()
                            .gap_3()
                            .flex_none()
                            .p_4()
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
                            .px_3()
                            .pb_3()
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
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        stack()
            .when(rtl, Styled::text_right)
            .child(div().text_sm().child(
                "Search or browse the component catalog, then select a story to try its variants and interactions.",
            ))
    }

    fn render_foundation(&self) -> impl IntoElement {
        stack()
            .child(
                section("Window and typography").child(
                    Surface::new(ColorRole::Surface)
                        .tonal_elevation(Elevation::Level3)
                        .shadow_elevation(Elevation::Level2)
                        .rounded_lg()
                        .p_4()
                        .child(
                            Text::new("Material 3 · 中文排版")
                                .id("typography")
                                .role(TypeRole::HeadlineSmall),
                        ),
                ),
            )
            .child(section("Shape and interaction").child(self.drawing.clone()))
            .child(section("Input and overlays").child(self.input.clone()))
    }

    fn render_story(&self, compact: bool, window: &Window, cx: &App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let local_theme = self.local_theme;
        let content = match self.current {
            Story::Introduction => self.render_introduction(window, cx).into_any_element(),
            Story::Foundation => self.render_foundation().into_any_element(),
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
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .justify_between()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(
                        stack()
                            .gap_1()
                            .flex_1()
                            .min_w_48()
                            .when(rtl, Styled::items_end)
                            .child(
                                div()
                                    .id("story-title")
                                    .role(Role::Heading)
                                    .aria_label(self.current.name())
                                    .child(Text::new(self.current.name()).role(if compact {
                                        TypeRole::TitleLarge
                                    } else {
                                        TypeRole::HeadlineSmall
                                    })),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(color(
                                        theme.colors.color(ColorRole::OnSurfaceVariant),
                                    ))
                                    .when(rtl, Styled::text_right)
                                    .child(self.current.description()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_1()
                            .when(rtl, Styled::flex_row_reverse)
                            .child(
                                Button::new(
                                    "toggle-theme",
                                    if theme.colors.appearance() == Appearance::Dark {
                                        "Light theme"
                                    } else {
                                        "Dark theme"
                                    },
                                )
                                .variant(ButtonVariant::Text)
                                .size(ControlSize::ExtraSmall)
                                .on_activate(move |window, cx| {
                                    let mut theme = window_material_theme(window, cx).clone();
                                    let appearance = match theme.colors.appearance() {
                                        Appearance::Light => Appearance::Dark,
                                        Appearance::Dark => Appearance::Light,
                                    };
                                    theme.colors =
                                        gpui_material::theme::ColorScheme::baseline(appearance);
                                    if local_theme {
                                        set_window_material_theme(Some(theme), window, cx);
                                    } else {
                                        set_material_theme(theme, cx);
                                    }
                                }),
                            )
                            .child(
                                ToggleButton::new(
                                    "toggle-motion",
                                    "Reduced motion",
                                    cx.reduce_motion(),
                                )
                                .variant(ToggleButtonVariant::Outlined)
                                .size(ControlSize::ExtraSmall)
                                .on_change(|next, _, cx| cx.set_reduce_motion(next)),
                            ),
                    ),
            )
            .child(Divider::horizontal())
            .child(
                div()
                    .id("gallery-scroll")
                    .tab_group()
                    .track_scroll(&self.content_scroll)
                    .overflow_y_scroll()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .p_4()
                            .when(self.current == Story::Fab, |content| content.pb_24())
                            .child(content),
                    ),
            )
    }

    fn render_status_bar(
        &self,
        compact: bool,
        show_catalog: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let appearance = theme.colors.appearance();
        let rtl = theme.layout.direction == Direction::Rtl;
        let status = format!("Display scale {:.0}%", window.scale_factor() * 100.);
        let theme_status = format!("Window theme {appearance:?}");
        let label = match (compact, show_catalog) {
            (true, true) => "Back",
            (false, true) => "Hide sidebar",
            (false, false) => "Show sidebar",
            (true, false) => "Components",
        };
        Surface::new(ColorRole::SurfaceContainerLow)
            .w_full()
            .flex_none()
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_3()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                Button::new("browse-components", label)
                    .variant(ButtonVariant::Text)
                    .size(ControlSize::ExtraSmall)
                    .on_activate({
                        let owner = cx.weak_entity();
                        move |window, cx| {
                            let _ = owner.update(cx, |this, cx| this.toggle_catalog(window, cx));
                        }
                    }),
            )
            .child(div().flex_1())
            .when(!compact, |bar| {
                bar.child(
                    div()
                        .id("window-theme")
                        .role(Role::Status)
                        .aria_label(theme_status.clone())
                        .test_support()
                        .child(Text::new(theme_status).role(TypeRole::LabelMedium)),
                )
            })
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
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let compact = Self::compact(window);
        let show_catalog = if compact {
            self.compact_catalog_open
        } else {
            !self.sidebar_collapsed
        };
        let show_story = !compact || !show_catalog;

        Surface::new(ColorRole::Surface)
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .children(show_catalog.then(|| self.render_catalog(compact, window, cx)))
                    .when(show_catalog && show_story, |body| {
                        body.child(Divider::vertical())
                    })
                    .children(show_story.then(|| self.render_story(compact, window, cx))),
            )
            .child(Divider::horizontal())
            .child(self.render_status_bar(compact, show_catalog, window, cx))
            .children((show_story && self.current == Story::Fab).then(|| {
                FloatingAction::new(
                    Fab::new("floating-motion", "Toggle reduced motion", "◐")
                        .on_activate(|_, cx| cx.set_reduce_motion(!cx.reduce_motion())),
                )
                .margin(px(52.))
            }))
    }
}
