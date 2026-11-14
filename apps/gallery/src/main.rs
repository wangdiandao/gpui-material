mod action_examples;
mod display_examples;
mod drawing;
mod input_examples;
mod overlay_examples;
mod selection_examples;
mod slider_examples;
mod story;
mod text_field_examples;

use std::borrow::Cow;

use gpui::{
    AnyView, App, AppContext, AssetSource, Context, ElementId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, QuitMode, Render, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Subscription, TitlebarOptions, Window,
    WindowBounds, WindowHandle, WindowOptions, div, point, prelude::FluentBuilder, px, size,
};
use gpui_base::{
    Root, TestSupportExt,
    input::{Input, InputAccessibility, InputState},
};
#[cfg(all(test, feature = "test-support"))]
use gpui_material::foundation::shape_text;
use gpui_material::foundation::{
    MaterialRoot, Surface, color, material_theme, set_material_theme, set_window_material_theme,
    window_material_theme,
};
#[cfg(all(test, feature = "test-support"))]
use gpui_material::theme::TypographyEmphasis;
use gpui_material::theme::{Appearance, ColorRole, Direction, Elevation, MaterialTheme, TypeRole};
use gpui_material::{
    Button, ButtonVariant, ControlSize, Divider, Fab, FloatingAction, List, ListItem, ListVariant,
    Text, ToggleButton, ToggleButtonVariant,
};

use story::{Story, section, stack};

gpui::actions!(gallery, [BrowseComponents]);

const TITLE: &str = "Material Gallery";
const FORWARD_ICON: &str = "forward.svg";
const ARCHIVE_ICON: &str = "archive.svg";

struct GalleryAssets;

impl AssetSource for GalleryAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let svg: &[u8] = match path {
            FORWARD_ICON => br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 11h11l-4-5 1.5-1.4L20 12l-7.5 7.4L11 18l4-5H4z"/></svg>"#,
            ARCHIVE_ICON => br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill-rule="evenodd" d="M3 5h18v5h-1v10H4V10H3V5zm2 2v1h14V7H5zm1 3v8h12v-8H6zm3 2h6v2H9v-2z"/></svg>"#,
            _ => return Ok(None),
        };
        Ok(Some(Cow::Borrowed(svg)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(if path.is_empty() {
            vec![FORWARD_ICON.into(), ARCHIVE_ICON.into()]
        } else {
            vec![]
        })
    }
}

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let two_windows = arguments.iter().any(|arg| arg == "--two-windows");
    let rtl = arguments.iter().any(|arg| arg == "--rtl");
    let reduced_motion = arguments.iter().any(|arg| arg == "--reduce-motion");
    let dark = arguments.iter().any(|arg| arg == "--dark");
    gpui_platform::application()
        .with_assets(GalleryAssets)
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(move |cx| {
            init_gallery(cx);
            if rtl || dark {
                let mut theme = MaterialTheme::baseline(if dark {
                    Appearance::Dark
                } else {
                    Appearance::Light
                });
                if rtl {
                    theme.layout.direction = gpui_material::theme::Direction::Rtl;
                }
                set_material_theme(theme, cx);
            }
            if reduced_motion {
                cx.set_reduce_motion(true);
            }
            let result = open_gallery(cx, false).and_then(|_| {
                if two_windows {
                    open_gallery(cx, true)?;
                }
                Ok(())
            });
            if let Err(error) = result {
                eprintln!("Could not open Material Gallery: {error:#}");
                std::process::exit(1);
            }
        });
}

fn init_gallery(cx: &mut App) {
    MaterialRoot::init(MaterialTheme::default(), cx);
    cx.bind_keys([
        gpui::KeyBinding::new(
            "escape",
            input_examples::CancelDrag,
            Some("FoundationInput"),
        ),
        gpui::KeyBinding::new("escape", BrowseComponents, Some("GalleryCatalog")),
    ]);
}

fn open_gallery(cx: &mut App, local_dark: bool) -> gpui::Result<WindowHandle<Root>> {
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(
                if local_dark {
                    "Material Gallery · Dark preview"
                } else {
                    TITLE
                }
                .into(),
            ),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(1040.), px(744.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    MaterialRoot::open_window(options, cx, |window, cx| {
        if local_dark {
            let mut theme = material_theme(cx).clone();
            theme.colors = gpui_material::theme::ColorScheme::baseline(Appearance::Dark);
            set_window_material_theme(Some(theme), window, cx);
        }
        cx.new(|cx| Gallery::new(local_dark, window, cx))
    })
}

struct Gallery {
    local_theme: bool,
    catalog_open: bool,
    current: Story,
    search: Entity<InputState>,
    catalog_scroll: ScrollHandle,
    content_scroll: ScrollHandle,
    content_focus: FocusHandle,
    stories: Vec<(Story, AnyView)>,
    drawing: Entity<drawing::Drawing>,
    input: Entity<input_examples::InputExamples>,
    _search: Subscription,
}

impl Gallery {
    fn new(local_theme: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            let mut input = InputState::new(window, cx).placeholder("Search components");
            input.set_accessibility(
                InputAccessibility::new(Some(Role::TextInput))
                    .with_label(Some("Search components".into()))
                    .with_accessibility_id(Some("gallery-search".into())),
            );
            input
        });
        let catalog_scroll = ScrollHandle::new();
        let scroll = catalog_scroll.clone();
        let observe_search = cx.observe(&search, move |_, _, cx| {
            scroll.set_offset(point(px(0.), px(0.)));
            cx.notify();
        });
        let stories = Story::GROUPS
            .iter()
            .flat_map(|(_, stories)| stories.iter().copied())
            .filter(|story| *story != Story::Foundation)
            .map(|story| {
                let view: AnyView = match story {
                    Story::Button | Story::ToggleButton | Story::IconButton | Story::Fab => cx
                        .new(|_| action_examples::ActionExamples::for_story(story))
                        .into(),
                    Story::Checkbox | Story::Radio | Story::Switch | Story::Chip => cx
                        .new(|cx| selection_examples::SelectionExamples::for_story(story, cx))
                        .into(),
                    Story::Slider => cx.new(slider_examples::SliderExamples::new).into(),
                    Story::Menu | Story::Tooltip => cx
                        .new(|cx| overlay_examples::MenuTooltipExamples::new(story, cx))
                        .into(),
                    Story::TextField => cx
                        .new(|cx| text_field_examples::TextFieldExamples::new(window, cx))
                        .into(),
                    Story::Snackbar => cx.new(overlay_examples::SnackbarExamples::new).into(),
                    _ => cx
                        .new(|_| display_examples::DisplayExamples::for_story(story))
                        .into(),
                };
                (story, view)
            })
            .collect();
        Self {
            local_theme,
            catalog_open: false,
            current: Story::default(),
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

    fn navigate(
        &mut self,
        story: Story,
        compact: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.current != story {
            self.current = story;
            self.content_scroll.set_offset(point(px(0.), px(0.)));
        }
        if compact {
            self.catalog_open = false;
            self.content_focus.focus(window, cx);
        }
        cx.notify();
    }

    fn catalog(&self, compact: bool, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search.read(cx).value().trim().to_ascii_lowercase();
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(
            if self.search.read(cx).focus_handle(cx).is_focused(window) {
                ColorRole::Primary
            } else {
                ColorRole::OutlineVariant
            },
        ));
        let results: Vec<_> = Story::GROUPS
            .iter()
            .filter_map(|&(group, stories)| {
                let items: Vec<_> = stories
                    .iter()
                    .copied()
                    .filter(|story| story.matches(&query))
                    .collect();
                (!items.is_empty()).then_some((group, items))
            })
            .collect();
        let no_results = results.is_empty();
        let owner = cx.entity();
        div()
            .id("gallery-catalog-pane")
            .key_context("GalleryCatalog")
            .on_action(cx.listener(move |this, _: &BrowseComponents, window, cx| {
                if compact {
                    this.catalog_open = false;
                    this.content_focus.focus(window, cx);
                    cx.notify();
                }
            }))
            .when_else(
                compact,
                |panel| panel.flex_1().min_w_0(),
                |panel| panel.w(px(232.)).flex_none(),
            )
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                Surface::new(ColorRole::SurfaceContainerLow)
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(
                        div().flex_none().px_4().pt_2().pb_3().child(
                            div()
                                .id("gallery-search-field")
                                .rounded_md()
                                .border_1()
                                .border_color(outline)
                                .px_3()
                                .py_2()
                                .text_sm()
                                .child(Input::new(&self.search)),
                        ),
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
                            .children(results.into_iter().map(|(group, items)| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb_2()
                                    .child(
                                        div().px_2().pt_2().child(
                                            Text::new(group)
                                                .id(group)
                                                .role(TypeRole::LabelMedium)
                                                .color(ColorRole::OnSurfaceVariant),
                                        ),
                                    )
                                    .child(
                                        List::new((ElementId::from("catalog-group"), group))
                                            .variant(ListVariant::Segmented)
                                            .items(items.into_iter().map(|story| {
                                                let owner = owner.clone();
                                                ListItem::new(
                                                    (ElementId::from("gallery-story"), story.id()),
                                                    story.name(),
                                                )
                                                .selected(story == self.current)
                                                .on_activate(move |window, cx| {
                                                    owner.update(cx, |this, cx| {
                                                        this.navigate(story, compact, window, cx)
                                                    })
                                                })
                                            })),
                                    )
                            }))
                            .when(no_results, |panel| {
                                panel.child(
                                    div().p_2().child(
                                        Text::new("No matching components").id("no-components"),
                                    ),
                                )
                            }),
                    ),
            )
    }

    fn foundation(&self, window: &Window, cx: &App) -> impl IntoElement {
        let appearance = window_material_theme(window, cx).colors.appearance();
        let status = format!("Display scale {:.0}%", window.scale_factor() * 100.);
        let theme_status = format!("Window theme {appearance:?}");
        stack()
            .child(
                section("Window and typography")
                    .child(
                        Text::new("Material 3 · 中文排版")
                            .id("typography")
                            .role(TypeRole::HeadlineSmall),
                    )
                    .child(
                        Surface::new(ColorRole::Surface)
                            .tonal_elevation(Elevation::Level3)
                            .shadow_elevation(Elevation::Level2)
                            .rounded_lg()
                            .p_4()
                            .child(
                                div()
                                    .id("window-theme")
                                    .role(Role::Status)
                                    .aria_label(theme_status.clone())
                                    .test_support()
                                    .child(theme_status),
                            )
                            .child(
                                div()
                                    .id("display-scale")
                                    .role(Role::Status)
                                    .aria_label(status.clone())
                                    .test_support()
                                    .child(status),
                            ),
                    ),
            )
            .child(section("Shape and interaction").child(self.drawing.clone()))
            .child(section("Input and overlays").child(self.input.clone()))
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let compact = window.viewport_size().width < px(700.);
        let show_catalog = !compact || self.catalog_open;
        let show_story = !compact || !self.catalog_open;
        let toolbar_motion = cx.reduce_motion();
        let local_theme = self.local_theme;
        let story_title = self.current.name();
        let story_content = if self.current == Story::Foundation {
            self.foundation(window, cx).into_any_element()
        } else {
            self.stories
                .iter()
                .find(|(story, _)| *story == self.current)
                .map(|(_, view)| view.clone().into_any_element())
                .expect("registered story")
        };
        let focus = self.content_focus.clone();
        let search_focus = self.search.read(cx).focus_handle(cx);

        Surface::new(ColorRole::Surface)
            .size_full()
            .flex()
            .flex_col()
            .child(
                Surface::new(ColorRole::SurfaceContainerLow)
                    .flex_none()
                    .w_full()
                    .px_3()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_x_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .when(compact, |toolbar| {
                        toolbar.child(
                            Button::new(
                                "browse-components",
                                if self.catalog_open {
                                    "Back"
                                } else {
                                    "Components"
                                },
                            )
                            .variant(ButtonVariant::Outlined)
                            .size(ControlSize::ExtraSmall)
                            .on_activate({
                                let owner = cx.entity();
                                move |window, cx| {
                                    owner.update(cx, |this, cx| {
                                        this.catalog_open = !this.catalog_open;
                                        if this.catalog_open {
                                            search_focus.focus(window, cx);
                                        } else {
                                            focus.focus(window, cx);
                                        }
                                        cx.notify();
                                    })
                                }
                            }),
                        )
                    })
                    .when(!compact, |toolbar| {
                        toolbar.child(
                            div().min_w_0().flex_1().child(
                                Text::new(TITLE)
                                    .id("gallery-title")
                                    .role(TypeRole::TitleMedium),
                            ),
                        )
                    })
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
                            theme.colors = gpui_material::theme::ColorScheme::baseline(appearance);
                            if local_theme {
                                set_window_material_theme(Some(theme), window, cx);
                            } else {
                                set_material_theme(theme, cx);
                            }
                        }),
                    )
                    .child(
                        ToggleButton::new("toggle-motion", "Reduced motion", toolbar_motion)
                            .variant(ToggleButtonVariant::Outlined)
                            .size(ControlSize::ExtraSmall)
                            .on_change(|next, _, cx| cx.set_reduce_motion(next)),
                    ),
            )
            .child(Divider::horizontal())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .children(show_catalog.then(|| self.catalog(compact, window, cx)))
                    .when(!compact, |body| body.child(Divider::vertical()))
                    .children(show_story.then(|| {
                        div()
                            .id((ElementId::from("gallery-story-pane"), self.current.id()))
                            .track_focus(&self.content_focus)
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .child(div().flex_none().px_6().py_4().child(
                                Text::new(story_title).id("story-title").role(if compact {
                                    TypeRole::TitleLarge
                                } else {
                                    TypeRole::HeadlineSmall
                                }),
                            ))
                            .child(
                                div()
                                    .id("gallery-scroll")
                                    .tab_group()
                                    .track_scroll(&self.content_scroll)
                                    .overflow_y_scroll()
                                    .flex_1()
                                    .min_h_0()
                                    .p_6()
                                    .child(story_content),
                            )
                            .when(self.current == Story::Fab, |pane| {
                                pane.child(div().h(px(88.)).flex_none())
                            })
                    })),
            )
            .children((show_story && self.current == Story::Fab).then(|| {
                FloatingAction::new(
                    Fab::new("floating-motion", "Toggle reduced motion", "◐")
                        .on_activate(|_, cx| cx.set_reduce_motion(!cx.reduce_motion())),
                )
            }))
    }
}
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{InputEvent, TestAppContext};
    use gpui_base::{Theme, ThemeAppearance, test_support::find};

    #[gpui::test]
    fn catalog_filters_and_opens_a_story_on_a_compact_window(cx: &mut TestAppContext) {
        let handle = cx.update(|cx| {
            init_gallery(cx);
            open_gallery(cx, false).unwrap()
        });
        cx.simulate_window_resize(handle.into(), size(px(340.), px(560.)));

        let focus_control = |cx: &mut TestAppContext, id: ElementId| {
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let target = find(window, &[], &id).unwrap();
                assert!(target.visible());
                let position = target.bounds().center();
                for input in [
                    gpui::MouseDownEvent {
                        button: gpui::MouseButton::Left,
                        position,
                        click_count: 1,
                        first_mouse: false,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    gpui::MouseUpEvent {
                        button: gpui::MouseButton::Left,
                        position,
                        click_count: 1,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                ] {
                    window.dispatch_event(input, cx);
                }
            })
            .unwrap();
            cx.run_until_parked();
        };

        focus_control(cx, "browse-components".into());
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.simulate_input(handle.into(), "slider");
        cx.run_until_parked();
        let story = ElementId::from((ElementId::from("gallery-story"), "slider"));
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let excluded = ElementId::from((ElementId::from("gallery-story"), "checkbox"));
            assert!(find(window, &[], &(excluded, "activate").into()).is_none());
        })
        .unwrap();
        focus_control(cx, (story, "activate").into());
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let level = ElementId::from((ElementId::from("level"), "end"));
            let slider = find(window, &[], &level).unwrap();
            assert_eq!(slider.role(), Some(Role::Slider));
            assert!(slider.visible());
        })
        .unwrap();
    }

    #[gpui::test]
    fn projects_theme_and_paints_status_in_two_windows(cx: &mut TestAppContext) {
        let (first, second) = cx.update(|cx| {
            init_gallery(cx);
            (
                open_gallery(cx, false).unwrap(),
                open_gallery(cx, true).unwrap(),
            )
        });
        let label = |cx: &mut TestAppContext, handle: WindowHandle<Root>, id: &str| {
            let id = gpui::ElementId::from(gpui::SharedString::from(id.to_owned()));
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let status = find(window, &[], &id).expect("rendered status");
                assert_eq!(status.role(), Some(Role::Status));
                assert!(status.visible());
                let bounds = status.bounds();
                assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
                assert!(bounds.right() <= window.viewport_size().width);
                assert!(bounds.bottom() <= window.viewport_size().height);
                status.label().unwrap().to_owned()
            })
            .unwrap()
        };
        assert!(label(cx, first, "display-scale").starts_with("Display scale "));
        assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
        cx.simulate_window_scale_factor_change(second.into(), 1.5);
        assert_eq!(label(cx, second, "display-scale"), "Display scale 150%");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
        cx.simulate_window_scale_factor_change(second.into(), 1.);
        assert_eq!(label(cx, second, "display-scale"), "Display scale 100%");
        for (handle, appearance) in [(first, Appearance::Light), (second, Appearance::Dark)] {
            cx.update_window(handle.into(), |_, window, cx| {
                let theme = window_material_theme(window, cx);
                assert_eq!(theme.colors.appearance(), appearance);
                assert_eq!(
                    theme.colors.color(ColorRole::OnPrimary),
                    MaterialTheme::baseline(appearance)
                        .colors
                        .color(ColorRole::OnPrimary)
                );
            })
            .unwrap();
        }
        cx.update(|cx| set_material_theme(MaterialTheme::baseline(Appearance::Dark), cx));
        assert_eq!(label(cx, first, "window-theme"), "Window theme Dark");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
        cx.update(|cx| {
            let base = Theme::global(cx);
            assert_eq!(base.appearance, ThemeAppearance::Dark);
            assert_eq!(
                base.tokens.colors.primary,
                color(material_theme(cx).colors.color(ColorRole::Primary))
            );
        });
        cx.update(|cx| set_material_theme(MaterialTheme::baseline(Appearance::Light), cx));
        assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
        assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
    }

    #[gpui::test]
    fn mixed_script_uses_one_layout_for_metrics_and_paint(cx: &mut TestAppContext) {
        let handle = cx.update(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            open_gallery(cx, false).unwrap()
        });
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let theme = material_theme(cx);
            let style = TypeRole::BodyLarge.style(TypographyEmphasis::Baseline);
            let mut typography = theme.typography.clone();
            typography.plain.primary = "An unavailable Material test font".into();
            let line = shape_text(
                "A中文B",
                style,
                &typography,
                1.25,
                color(theme.colors.color(ColorRole::OnSurface)),
                window,
                cx,
            )
            .unwrap();
            assert_eq!(line.line_height(), px(style.line_height * 1.25));
            assert_eq!(line.line().len(), "A中文B".len());
            assert!(line.line().font_id_for_index(1).is_some());
            assert!(line.line().x_for_index(4) > line.line().x_for_index(1));
            assert_eq!(
                line.line().x_for_index(line.line().len()),
                line.line().width()
            );
        })
        .unwrap();
    }
}
