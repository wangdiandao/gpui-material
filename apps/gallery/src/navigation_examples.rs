use gpui::{
    AnyElement, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_material::{
    AppBar, AppBarVariant, Button, ButtonVariant, DialogState, Fab, Icon, IconButton, IconName,
    NavigationBar, NavigationBarLayout, NavigationItem, NavigationItems, NavigationRail, RailMode,
    TabVariant, Tabs, Text, theme::TypeRole,
};

use crate::story::{Story, section, stack};

const DESTINATIONS: &[(&str, &str, IconName)] = &[
    ("inbox", "Inbox", IconName::Inbox),
    ("favorites", "Favorites", IconName::Favorite),
    ("archive", "Archive", IconName::Archive),
    ("documents", "Documents", IconName::Description),
    ("notifications", "Notifications", IconName::Notifications),
    ("info", "Information", IconName::Info),
];

pub(crate) struct NavigationExamples {
    story: Story,
    selected: SharedString,
    expanded: bool,
    font_scale: f32,
    modal: Entity<DialogState>,
    scroll: ScrollHandle,
}

impl NavigationExamples {
    pub(crate) fn new(story: Story, cx: &mut Context<Self>) -> Self {
        Self {
            story,
            selected: "inbox".into(),
            expanded: false,
            font_scale: 1.,
            modal: cx.new(DialogState::new),
            scroll: ScrollHandle::new(),
        }
    }

    fn items(
        &self,
        id: &'static str,
        label: &'static str,
        count: usize,
        icons: bool,
        close_modal: bool,
        cx: &mut Context<Self>,
    ) -> NavigationItems {
        NavigationItems::new(
            id,
            label,
            DESTINATIONS.iter().take(count).map(|&(id, label, glyph)| {
                let item = NavigationItem::new(id, label).disabled(id == "favorites");
                if icons {
                    item.icon(Icon::new(glyph))
                } else {
                    item
                }
            }),
        )
        .selected(self.selected.clone())
        .font_scale(self.font_scale)
        .on_select(cx.listener(move |this, id: &SharedString, window, cx| {
            this.selected = id.clone();
            if close_modal {
                this.modal.update(cx, |state, cx| state.close(window, cx));
            }
            cx.notify();
        }))
    }

    fn tabs(&self, cx: &mut Context<Self>) -> AnyElement {
        stack()
            .child(section("Fixed primary").child(Tabs::new(self.items(
                "fixed-primary",
                "Fixed primary tabs",
                3,
                true,
                false,
                cx,
            ))))
            .child(
                section("Fixed secondary").child(
                    Tabs::new(self.items(
                        "fixed-secondary",
                        "Fixed secondary tabs",
                        3,
                        false,
                        false,
                        cx,
                    ))
                    .variant(TabVariant::Secondary),
                ),
            )
            .child(
                section("Scrolling primary").child(
                    div().max_w(px(320.)).child(
                        Tabs::new(self.items("scroll-tabs", "Scrolling tabs", 6, true, false, cx))
                            .scrollable(true),
                    ),
                ),
            )
            .into_any_element()
    }

    fn bar(&self, cx: &mut Context<Self>) -> AnyElement {
        stack()
            .child(
                section("Vertical items").child(NavigationBar::new(self.items(
                    "vertical-bar",
                    "Vertical navigation",
                    3,
                    true,
                    false,
                    cx,
                ))),
            )
            .child(
                section("Horizontal items").child(
                    NavigationBar::new(self.items(
                        "horizontal-bar",
                        "Horizontal navigation",
                        3,
                        true,
                        false,
                        cx,
                    ))
                    .layout(NavigationBarLayout::Horizontal),
                ),
            )
            .into_any_element()
    }

    fn rail(&self, cx: &mut Context<Self>) -> AnyElement {
        let modal = self.modal.clone();
        let rail_owner = cx.weak_entity();
        let fab_owner = rail_owner.clone();
        let toolbar = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .child(
                Button::new("rail-modal-open", "Open modal rail")
                    .variant(ButtonVariant::Outlined)
                    .on_activate(move |window, cx| {
                        modal.update(cx, |state, cx| state.open(window, cx));
                    }),
            )
            .child(
                Button::new(
                    "rail-toggle",
                    if self.expanded {
                        "Collapse rail"
                    } else {
                        "Expand rail"
                    },
                )
                .variant(ButtonVariant::Outlined)
                .on_activate(move |_, cx| {
                    rail_owner
                        .update(cx, |this, cx| {
                            this.expanded = !this.expanded;
                            cx.notify();
                        })
                        .ok();
                }),
            );
        let selected = DESTINATIONS
            .iter()
            .find(|(id, _, _)| *id == self.selected.as_ref())
            .map_or("Inbox", |(_, label, _)| *label);
        section("Expanded rail")
            .child(toolbar)
            .child(
                div()
                    .flex()
                    .h(px(288.))
                    .min_w_0()
                    .gap_2()
                    .child(
                        NavigationRail::new(
                            "standard-rail",
                            self.items("rail-items", "Destinations", 6, true, false, cx),
                        )
                        .mode(if self.expanded {
                            RailMode::Expanded
                        } else {
                            RailMode::Collapsed
                        })
                        .title("Workspace")
                        .fab(
                            Fab::new("rail-create", "Create document", Icon::new(IconName::Add))
                                .on_activate(move |_, cx| {
                                    fab_owner
                                        .update(cx, |this, cx| {
                                            this.selected = "documents".into();
                                            cx.notify();
                                        })
                                        .ok();
                                }),
                        ),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .pt_3()
                            .child(Text::new(selected).role(TypeRole::TitleMedium).wrap()),
                    ),
            )
            .child(
                NavigationRail::new(
                    "modal-rail",
                    self.items("modal-rail-items", "Modal destinations", 6, true, true, cx),
                )
                .mode(RailMode::Hidden)
                .modal(&self.modal)
                .title("Workspace navigation"),
            )
            .into_any_element()
    }

    fn app_bar(&self) -> AnyElement {
        let scroll = self.scroll.clone();
        stack()
            .child(
                section("Small and centered")
                    .child(AppBar::new("small-app-bar", "Inbox"))
                    .child(
                        AppBar::new("centered-app-bar", "Favorites")
                            .variant(AppBarVariant::Centered),
                    ),
            )
            .child(
                section("Medium flexible").child(
                    AppBar::new("medium-app-bar", "Shared documents")
                        .variant(AppBarVariant::MediumFlexible)
                        .subtitle("Workspace"),
                ),
            )
            .child(
                section("Large flexible").child(
                    div()
                        .h(px(320.))
                        .flex()
                        .flex_col()
                        .child(
                            AppBar::new("large-app-bar", "Documents shared across your workspace")
                                .variant(AppBarVariant::LargeFlexible)
                                .subtitle("Workspace")
                                .font_scale(self.font_scale)
                                .navigation(
                                    IconButton::new(
                                        "app-bar-back",
                                        "Scroll to top",
                                        Icon::new(IconName::ArrowForward)
                                            .mirror_in_rtl(true)
                                            .rotate(gpui::radians(std::f32::consts::PI)),
                                    )
                                    .on_activate(
                                        move |window, _| {
                                            scroll.set_offset(gpui::point(px(0.), px(0.)));
                                            window.refresh();
                                        },
                                    ),
                                )
                                .track_scroll(&self.scroll),
                        )
                        .child(
                            div()
                                .id("app-bar-scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .occlude()
                                .track_scroll(&self.scroll)
                                .children(DESTINATIONS.iter().cycle().take(24).enumerate().map(
                                    |(index, (_, title, _))| {
                                        div()
                                            .p_3()
                                            .child(Text::new(*title).id(("app-bar-line", index)))
                                    },
                                )),
                        ),
                ),
            )
            .into_any_element()
    }
}

impl Render for NavigationExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let examples = match self.story {
            Story::Tabs => self.tabs(cx),
            Story::NavigationBar => self.bar(cx),
            Story::NavigationRail => self.rail(cx),
            Story::AppBar => self.app_bar(),
            _ => unreachable!(),
        };
        stack()
            .child(
                Button::new(
                    "navigation-font-scale",
                    if self.font_scale == 1. {
                        "Enlarge labels"
                    } else {
                        "Reset label size"
                    },
                )
                .variant(ButtonVariant::Outlined)
                .on_activate(move |_, cx| {
                    owner
                        .update(cx, |this, cx| {
                            this.font_scale = if this.font_scale == 1. { 1.5 } else { 1. };
                            cx.notify();
                        })
                        .ok();
                }),
            )
            .child(examples)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{ElementId, InputEvent, Role, TestAppContext, WindowOptions, size};
    use gpui_base::test_support::find_all;
    use gpui_material::{
        foundation::{MaterialRoot, material_theme, set_material_theme},
        theme::Direction,
    };

    use super::*;

    #[gpui::test]
    fn tabs_move_focus_without_selecting_and_reveal_the_last_tab(cx: &mut TestAppContext) {
        let window = cx.update(|cx| {
            crate::init_gallery(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| NavigationExamples::new(Story::Tabs, cx))
            })
            .unwrap()
        });
        cx.simulate_window_resize(window.into(), size(px(480.), px(520.)));
        let tab = |cx: &mut TestAppContext, row: &'static str, id: &'static str| {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                find_all(window, &[], &id.into())
                    .into_iter()
                    .find(|tab| tab.path().contains(&ElementId::from(row)))
                    .unwrap()
            })
            .unwrap()
        };
        let click = |cx: &mut TestAppContext, row, id| {
            let position = tab(cx, row, id).bounds().center();
            cx.update_window(window.into(), |_, window, cx| {
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
            cx.run_until_parked();
        };
        let activate = |cx: &mut TestAppContext, key: &'static str| {
            cx.update_window(window.into(), |_, window, cx| {
                let keystroke = gpui::Keystroke::parse(key).unwrap();
                window.dispatch_event(
                    gpui::KeyDownEvent {
                        keystroke: keystroke.clone(),
                        is_held: false,
                        prefer_character_input: false,
                    }
                    .to_platform_input(),
                    cx,
                );
                window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
            })
            .unwrap();
            cx.run_until_parked();
        };
        let first = tab(cx, "fixed-primary", "inbox");
        assert_eq!(first.role(), Some(Role::Tab));
        assert_eq!(first.selected(), Some(true));
        assert_eq!(tab(cx, "fixed-primary", "favorites").disabled(), Some(true));
        click(cx, "fixed-primary", "inbox");
        cx.simulate_keystrokes(window.into(), "right");
        assert_eq!(tab(cx, "fixed-primary", "archive").focused(), Some(true));
        assert_eq!(tab(cx, "fixed-primary", "inbox").selected(), Some(true));
        activate(cx, "enter");
        assert_eq!(tab(cx, "fixed-primary", "archive").selected(), Some(true));

        cx.update(|cx| {
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = Direction::Rtl;
            set_material_theme(theme, cx);
        });
        tab(cx, "fixed-primary", "archive");
        cx.simulate_keystrokes(window.into(), "right");
        assert_eq!(tab(cx, "fixed-primary", "inbox").focused(), Some(true));
        click(cx, "scroll-tabs", "archive");
        cx.simulate_keystrokes(window.into(), "end");
        let last = tab(cx, "scroll-tabs", "info");
        assert_eq!(last.focused(), Some(true));
        assert!(last.visible());
        activate(cx, "space");
        assert_eq!(tab(cx, "scroll-tabs", "info").selected(), Some(true));
    }
}
