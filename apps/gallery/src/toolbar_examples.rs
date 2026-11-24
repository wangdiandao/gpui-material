use gpui::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_material::{
    Button, ContentArrangement, FabColor, FabMenu, FabMenuAlignment, FabMenuItem,
    FabMenuTriggerAppearance, FabMenuTriggerStyle, FabSize, Icon, IconName, NavigationBar,
    NavigationItem, NavigationItems, Scaffold, Switch, Text, TextField, TextFieldState, Toolbar,
    ToolbarAction, ToolbarColor, ToolbarFabPosition, ToolbarOrientation, ToolbarPlacement,
    theme::{CornerShape, TypeRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod fab_menu_tests;

#[cfg(all(test, feature = "test-support"))]
mod toolbar_content_tests;

pub(crate) struct ToolbarExamples {
    story: Story,
    menu_focus: FocusHandle,
    document_scroll: ScrollHandle,
    last_action: SharedString,
    show_navigation: bool,
    favorite: bool,
    menu_expanded: bool,
    toolbar_editor: Option<Entity<TextFieldState>>,
}

impl ToolbarExamples {
    pub(crate) fn new(story: Story, cx: &Context<Self>) -> Self {
        Self {
            story,
            menu_focus: cx.focus_handle(),
            document_scroll: ScrollHandle::new(),
            last_action: "No action yet".into(),
            show_navigation: false,
            favorite: false,
            menu_expanded: false,
            toolbar_editor: None,
        }
    }

    fn report(label: &'static str, cx: &Context<Self>) -> impl Fn(&mut Window, &mut App) + use<> {
        let view = cx.weak_entity();
        move |_, cx| {
            let _ = view.update(cx, |view, cx| {
                view.last_action = label.into();
                cx.notify();
            });
        }
    }

    fn menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let controlled = FabMenu::new(
            "controlled-create",
            "Compose actions",
            IconName::Add,
            [
                FabMenuItem::custom("draft", "Start a draft", div().child("Start a draft"))
                    .on_activate(Self::report("Draft", cx)),
                FabMenuItem::new("note", "Quick note", IconName::Edit)
                    .content(div().child("Quick note"))
                    .on_activate(Self::report("Note", cx)),
            ],
        )
        .expanded(self.menu_expanded)
        .on_expanded_change(move |expanded, _, cx| {
            let _ = owner.update(cx, |view, cx| {
                view.menu_expanded = expanded;
                cx.notify();
            });
        })
        .alignment(FabMenuAlignment::Center)
        .open_icon(Icon::new(IconName::Edit))
        .close_content(Icon::new(IconName::Close).size(px(20.)))
        .close_label("Close compose actions")
        .trigger_style(FabMenuTriggerStyle {
            collapsed: FabMenuTriggerAppearance {
                size: Some(px(64.)),
                shape: Some(CornerShape::Radii([12., 20., 20., 12.])),
                ..Default::default()
            },
            ..Default::default()
        });
        let examples = [
            ("small", FabSize::Small, FabColor::Primary, 2),
            ("medium", FabSize::Medium, FabColor::Secondary, 4),
            ("large", FabSize::Large, FabColor::Tertiary, 6),
        ];
        let entries = [
            ("note", "New note", IconName::Edit),
            ("document", "New document", IconName::Description),
            ("inbox", "Open inbox", IconName::Inbox),
            ("archive", "Archive", IconName::Archive),
            ("favorite", "Favourites", IconName::Favorite),
            ("alerts", "Notifications", IconName::Notifications),
        ];
        stack()
            .child(
                section("Create actions")
                    .description("Open a menu to compare the available actions at each size.")
                    .child(div().flex().flex_wrap().items_end().gap_4().children(
                        examples.into_iter().map(|(id, size, color, count)| {
                            let menu = FabMenu::new(
                                id,
                                "Create",
                                IconName::Add,
                                entries[..count].iter().map(|&(id, label, icon)| {
                                    FabMenuItem::new(id, label, icon)
                                        .disabled(id == "archive")
                                        .on_activate(Self::report(label, cx))
                                }),
                            )
                            .size(size)
                            .color(color)
                            .when(size == FabSize::Small, |menu| {
                                menu.track_focus(&self.menu_focus)
                            });
                            div()
                                .id(id)
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child(Text::new(match size {
                                    FabSize::Small => "Small",
                                    FabSize::Medium => "Medium",
                                    FabSize::Large => "Large",
                                }))
                                .child(menu)
                        }),
                    )),
            )
            .child(section("Controlled menu").child(controlled))
    }

    fn actions(&self, cx: &Context<Self>) -> Vec<ToolbarAction> {
        let view = cx.weak_entity();
        vec![
            ToolbarAction::new("edit", "Edit", IconName::Edit)
                .on_activate(Self::report("Edit", cx)),
            ToolbarAction::new("archive", "Archive", IconName::Archive).disabled(true),
            ToolbarAction::new("favorite", "Favorite", IconName::Favorite)
                .selected(self.favorite)
                .on_activate(move |_, cx| {
                    let _ = view.update(cx, |view, cx| {
                        view.favorite = !view.favorite;
                        view.last_action = (if view.favorite {
                            "Added to favorites"
                        } else {
                            "Removed from favorites"
                        })
                        .into();
                        cx.notify();
                    });
                }),
            ToolbarAction::new("search", "Search", IconName::Search)
                .secondary(true)
                .on_activate(Self::report("Search", cx)),
            ToolbarAction::new("notifications", "Notifications", IconName::Notifications)
                .secondary(true)
                .on_activate(Self::report("Notifications", cx)),
            ToolbarAction::new("inbox", "Inbox", IconName::Inbox)
                .secondary(true)
                .on_activate(Self::report("Inbox", cx)),
        ]
    }

    fn toolbars(&self, cx: &Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let docked = Toolbar::new("docked", "Document actions", self.actions(cx))
            .placement(ToolbarPlacement::Docked)
            .track_scroll(&self.document_scroll)
            .when(self.show_navigation, |toolbar| {
                toolbar.navigation(NavigationBar::new(
                    NavigationItems::new(
                        "navigation",
                        "Documents",
                        [
                            NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
                            NavigationItem::new("archive", "Archive")
                                .icon(Icon::new(IconName::Archive)),
                        ],
                    )
                    .selected("inbox"),
                ))
            });
        stack()
            .child(section("Floating horizontal toolbar").child(
                div().w_full().max_w(px(260.)).child(
                    Toolbar::new("horizontal", "Editing actions", self.actions(cx)).fab(
                        ToolbarFabPosition::End,
                        "New note",
                        IconName::Add,
                        Self::report("New note", cx),
                    ),
                ),
            ))
            .child(
                section("Floating and docked vertical toolbars").child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            Toolbar::new("vertical-floating", "Editing actions", self.actions(cx))
                                .orientation(ToolbarOrientation::Vertical)
                                .color(ToolbarColor::Vibrant)
                                .fab(
                                    ToolbarFabPosition::Start,
                                    "New note",
                                    IconName::Add,
                                    Self::report("New note", cx),
                                ),
                        )
                        .child(
                            Toolbar::new("vertical-docked", "Document actions", self.actions(cx))
                                .orientation(ToolbarOrientation::Vertical)
                                .placement(ToolbarPlacement::Docked)
                                .color(ToolbarColor::Vibrant),
                        ),
                ),
            )
            .child(
                section("Docked horizontal toolbar")
                    .description(
                        "Scroll the document to see the toolbar respond to content scrolling.",
                    )
                    .child(
                        Switch::new(
                            "navigation-toggle",
                            "Show navigation in the bottom region",
                            self.show_navigation,
                        )
                        .on_change(move |enabled, _, cx| {
                            let _ = owner.update(cx, |view, cx| {
                                view.show_navigation = enabled;
                                cx.notify();
                            });
                        }),
                    )
                    .child(
                        div().h(px(224.)).w_full().child(
                            Scaffold::new(
                                "document",
                                div()
                                    .id("document-scroll")
                                    .overflow_y_scroll()
                                    .occlude()
                                    .track_scroll(&self.document_scroll)
                                    .size_full()
                                    .p_4()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .children((1..=12).map(|index| {
                                        div()
                                            .id(("document-section", index as usize))
                                            .flex_none()
                                            .child(Text::new(format!("Document section {index}")))
                                    })),
                            )
                            .bottom_bar(docked),
                        ),
                    ),
            )
            .children(self.toolbar_editor.as_ref().map(|editor| {
                section("Editable docked toolbar").child(
                    Toolbar::new("editable-toolbar", "Editable document toolbar", [])
                        .placement(ToolbarPlacement::Docked)
                        .arrangement(ContentArrangement::SpaceBetween)
                        .shape(CornerShape::Radii([10., 10., 0., 0.]))
                        .leading_content(Button::new("editor-filter", "Filter documents"))
                        .content(div().w(px(192.)).child(TextField::new(
                            "toolbar-query",
                            "Find a document",
                            editor,
                        )))
                        .trailing_content(
                            Button::new("editor-save", "Save view")
                                .on_activate(Self::report("View saved", cx)),
                        ),
                )
            }))
    }
}

impl Render for ToolbarExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.story == Story::Toolbars && self.toolbar_editor.is_none() {
            self.toolbar_editor = Some(cx.new(|cx| TextFieldState::new(window, cx)));
        }
        let body = match self.story {
            Story::FabMenu => self.menu(cx).into_any_element(),
            _ => self.toolbars(cx).into_any_element(),
        };
        stack()
            .child(body)
            .child(Text::new(self.last_action.clone()).role(TypeRole::BodyMedium))
    }
}
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{AppContext, ElementId, Role, TestAppContext, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::{MaterialRoot, foundation::set_window_material_theme, theme::Direction};

    fn open(
        cx: &mut TestAppContext,
        story: Story,
    ) -> (
        gpui::WindowHandle<gpui_base::Root>,
        gpui::Entity<ToolbarExamples>,
    ) {
        cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                let entity = cx.new(|cx| ToolbarExamples::new(story, cx));
                view = Some(entity.clone());
                entity
            })
            .unwrap();
            (window, view.unwrap())
        })
    }

    fn frame(cx: &mut TestAppContext, window: gpui::WindowHandle<gpui_base::Root>) {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    #[gpui::test]
    fn fab_menu_selects_once_and_restores_focus_during_exit(cx: &mut TestAppContext) {
        let (handle, view) = open(cx, Story::FabMenu);
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            view.read(cx).menu_focus.clone().focus(window, cx);
        })
        .unwrap();
        frame(cx, handle);
        cx.simulate_keystrokes(handle.into(), "enter");
        frame(cx, handle);
        frame(cx, handle);
        let item = ElementId::from((ElementId::from("small"), "note"));
        let sibling = ElementId::from((ElementId::from("small"), "document"));
        cx.update_window(handle.into(), |_, window, _| {
            let target = find(window, &[], &item).unwrap();
            assert_eq!(target.role(), Some(Role::MenuItem));
            assert_eq!(target.focused(), Some(true));
        })
        .unwrap();
        let executor = cx.executor();
        executor.advance_clock(std::time::Duration::from_secs(1));
        frame(cx, handle);
        cx.simulate_keystrokes(handle.into(), "escape");
        frame(cx, handle);
        let exit = ElementId::from((ElementId::from("small"), "exit"));
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(view.read(cx).menu_focus.is_focused(window));
            assert_eq!(&*view.read(cx).last_action, "No action yet");
            assert!(find(window, &[], &item).is_none());
            assert!(find(window, &[], &exit).is_some());
        })
        .unwrap();
        executor.advance_clock(std::time::Duration::from_secs(2));
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            assert!(find(window, &[], &exit).is_none());
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "space");
        frame(cx, handle);
        frame(cx, handle);
        cx.simulate_keystrokes(handle.into(), "down");
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            assert_eq!(find(window, &[], &sibling).unwrap().focused(), Some(true));
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "enter");
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            assert_eq!(&*view.read(cx).last_action, "New document");
            assert!(view.read(cx).menu_focus.is_focused(window));
        })
        .unwrap();
        cx.update(|cx| cx.set_reduce_motion(true));
        cx.simulate_keystrokes(handle.into(), "space");
        frame(cx, handle);
        frame(cx, handle);
        cx.simulate_keystrokes(handle.into(), "escape");
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            assert!(find(window, &[], &exit).is_none());
        })
        .unwrap();
    }

    #[gpui::test]
    fn toolbar_directional_focus_reveals_actions_and_skips_disabled(cx: &mut TestAppContext) {
        let (handle, view) = open(cx, Story::Toolbars);
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            let mut theme = gpui_material::MaterialTheme::default();
            theme.layout.direction = Direction::Rtl;
            set_window_material_theme(Some(theme), window, cx);
        })
        .unwrap();
        frame(cx, handle);
        let toolbar = ElementId::from("horizontal");
        let edit = ElementId::from((toolbar.clone(), "edit"));
        let favorite = ElementId::from((toolbar.clone(), "favorite"));
        let strip = ElementId::from((toolbar.clone(), "actions"));
        cx.update_window(handle.into(), |_, window, cx| {
            assert_eq!(
                find(window, &[], &toolbar).unwrap().role(),
                Some(Role::Toolbar)
            );
            let position = find(window, &[], &edit).unwrap().bounds().center();
            use gpui::InputEvent as _;
            window.dispatch_event(
                gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
                    position,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        frame(cx, handle);
        cx.simulate_keystrokes(handle.into(), "left");
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let item = find(window, &[], &favorite).unwrap();
            assert_eq!(item.focused(), Some(true));
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "left left left");
        frame(cx, handle);
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let last = find(window, &[], &(toolbar.clone(), "inbox").into()).unwrap();
            let strip = find(window, &[], &strip).unwrap();
            assert_eq!(last.focused(), Some(true));
            assert!(strip.bounds().contains(&last.bounds().center()));
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "enter");
        frame(cx, handle);
        cx.update(|cx| assert_eq!(&*view.read(cx).last_action, "Inbox"));
    }

    #[gpui::test]
    fn vertical_toolbar_shows_the_entire_docked_search_button(cx: &mut TestAppContext) {
        let (handle, _) = open(cx, Story::Toolbars);
        frame(cx, handle);
        let docked = ElementId::from("vertical-docked");
        cx.update_window(handle.into(), |_, window, _| {
            let search = find(window, &[], &(docked.clone(), "search").into())
                .unwrap()
                .bounds();
            let strip = find(window, &[], &(docked, "actions").into())
                .unwrap()
                .bounds();
            assert!(
                search.top() >= strip.top() && search.bottom() <= strip.bottom(),
                "search button bounds {search:?} exceed toolbar bounds {strip:?}"
            );
        })
        .unwrap();
    }

    #[gpui::test]
    fn docked_toolbar_moves_secondary_actions_to_overflow_and_restores_them(
        cx: &mut TestAppContext,
    ) {
        let (handle, view) = open(cx, Story::Toolbars);
        frame(cx, handle);
        let docked = ElementId::from("docked");
        let search = ElementId::from((docked.clone(), "search"));
        let overflow = ElementId::from((docked.clone(), "overflow"));
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(find(window, &[], &search).is_some());
            view.read(cx)
                .document_scroll
                .set_offset(gpui::point(px(0.), px(-100.)));
        })
        .unwrap();
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(find(window, &[], &search).is_none());
            assert!(find(window, &[], &overflow).is_some());
            view.read(cx)
                .document_scroll
                .set_offset(gpui::point(px(0.), px(0.)));
        })
        .unwrap();
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            assert!(find(window, &[], &search).is_some());
            assert!(find(window, &[], &overflow).is_none());
            view.update(cx, |view, cx| {
                view.show_navigation = true;
                cx.notify();
            });
        })
        .unwrap();
        frame(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            assert!(find(window, &[], &docked).is_none());
            assert!(find(window, &[], &"navigation".into()).is_some());
        })
        .unwrap();
    }
}
