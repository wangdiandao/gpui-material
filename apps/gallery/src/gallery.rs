use gpui::{
    AnyView, App, AppContext, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, Role, ScrollHandle,
    StatefulInteractiveElement, Styled, Subscription, Window, div, point, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonVariant, ControlSize, Fab, FloatingAction, Icon, IconButton, IconName, List,
    ListItem, ListVariant, Text, TextField, TextFieldEvent, TextFieldState, TextFieldVariant,
    ToggleButton, ToggleButtonVariant,
    foundation::{
        Surface, color, set_material_theme, set_window_material_theme, window_material_theme,
    },
    theme::{Appearance, ColorRole, CornerRole, Direction, Elevation, TypeRole, WidthClass},
};

use crate::{
    drawing, input_examples,
    story::{Story, section, stack},
};

gpui::actions!(gallery, [CloseCatalog]);

const COMPACT_MARGIN: Pixels = px(16.);
const PANE_SPACING: Pixels = px(24.);
const CATALOG_WIDTH: Pixels = px(360.);

pub(crate) struct Gallery {
    local_theme: bool,
    catalog_collapsed: bool,
    single_pane_catalog_open: bool,
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
            catalog_collapsed: false,
            single_pane_catalog_open: false,
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
            .on_action(cx.listener(move |this, _: &CloseCatalog, window, cx| {
                if single_pane {
                    this.toggle_catalog(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .when_else(
                single_pane,
                |panel| panel.flex_1().min_w_0(),
                |panel| panel.w(CATALOG_WIDTH).flex_none(),
            )
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
            .test_support()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .rounded(radius)
            .bg(color(theme.colors.color(ColorRole::Surface)))
            .child(
                stack()
                    .flex_none()
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
                            .child(Text::new(self.current.name()).role(TypeRole::HeadlineSmall)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                            .when(rtl, Styled::text_right)
                            .child(self.current.description()),
                    ),
            )
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
                            .px(inset)
                            .py_4()
                            .when(self.current == Story::Fab, |content| content.pb_24())
                            .child(content),
                    ),
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
        let appearance = theme.colors.appearance();
        let rtl = theme.layout.direction == Direction::Rtl;
        let local_theme = self.local_theme;
        let label = match (single_pane, show_catalog) {
            (true, true) => "Back",
            (false, true) => "Hide components",
            (_, false) => "Components",
        };
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
                            if appearance == Appearance::Dark {
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
                            theme.colors = gpui_material::theme::ColorScheme::baseline(appearance);
                            if local_theme {
                                set_window_material_theme(Some(theme), window, cx);
                            } else {
                                set_material_theme(theme, cx);
                            }
                        }),
                    )
                    .child(
                        ToggleButton::new("toggle-motion", "Reduced motion", cx.reduce_motion())
                            .variant(ToggleButtonVariant::Outlined)
                            .size(ControlSize::ExtraSmall)
                            .on_change(|next, _, cx| cx.set_reduce_motion(next)),
                    ),
            )
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
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        let width_class = Self::width_class(window);
        let single_pane = Self::single_pane(width_class);
        let show_catalog = if single_pane {
            self.single_pane_catalog_open
        } else {
            !self.catalog_collapsed
        };
        let show_story = !single_pane || !show_catalog;
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
            .child(self.render_app_bar(single_pane, show_catalog, margin, window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .gap(PANE_SPACING)
                    .when(!single_pane, |body| body.px(margin))
                    .when(rtl, Styled::flex_row_reverse)
                    .children(
                        show_catalog.then(|| {
                            self.render_catalog(single_pane, inset, pane_radius, window, cx)
                        }),
                    )
                    .children(
                        show_story.then(|| self.render_story(inset, pane_radius, window, cx)),
                    ),
            )
            .child(self.render_status_bar(margin, window, cx))
            .children((show_story && self.current == Story::Fab).then(|| {
                FloatingAction::new(
                    Fab::new(
                        "floating-motion",
                        "Toggle reduced motion",
                        Icon::new(IconName::MotionPhotosOn),
                    )
                    .on_activate(|_, cx| cx.set_reduce_motion(!cx.reduce_motion())),
                )
                .margin(px(52.))
            }))
    }
}
