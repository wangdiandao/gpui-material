use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, Styled, Window, canvas, div, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    AdaptiveNavigation, AppBar, Button, ButtonVariant, Card, CardVariant, Fab, Feed, Icon,
    IconButton, IconName, List, ListDetail, ListDetailPane, ListItem, NavigationItem,
    NavigationItems, Scaffold, Snackbar, SnackbarHost, SupportingPane, Text,
    foundation::isolated_scroll, theme::TypeRole,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Destination {
    Inbox,
    Activity,
    Feed,
}

impl Destination {
    fn id(self) -> &'static str {
        match self {
            Self::Inbox => "adaptive-inbox",
            Self::Activity => "adaptive-activity",
            Self::Feed => "adaptive-feed",
        }
    }
}

pub(crate) struct AdaptiveExamples {
    width: Rc<Cell<Pixels>>,
    destination: Destination,
    detail: ListDetailPane,
    support: bool,
    document: usize,
    inbox_scroll: ScrollHandle,
    detail_scroll: ScrollHandle,
    activity_scroll: ScrollHandle,
    support_scroll: ScrollHandle,
    feed_scroll: ScrollHandle,
    inbox_focus: FocusHandle,
    detail_focus: FocusHandle,
    activity_focus: FocusHandle,
    support_focus: FocusHandle,
    snackbar: Entity<SnackbarHost>,
}

impl AdaptiveExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            width: Rc::new(Cell::new(px(0.))),
            destination: Destination::Inbox,
            detail: ListDetailPane::List,
            support: false,
            document: 0,
            inbox_scroll: ScrollHandle::new(),
            detail_scroll: ScrollHandle::new(),
            activity_scroll: ScrollHandle::new(),
            support_scroll: ScrollHandle::new(),
            feed_scroll: ScrollHandle::new(),
            inbox_focus: cx.focus_handle(),
            detail_focus: cx.focus_handle(),
            activity_focus: cx.focus_handle(),
            support_focus: cx.focus_handle(),
            snackbar: cx.new(|_| SnackbarHost::default()),
        }
    }

    fn inbox(&self, width: Pixels, cx: &Context<Self>) -> ListDetail {
        let owner = cx.weak_entity();
        let list = List::new("documents-list").items((0..24).map(|index| {
            let owner = owner.clone();
            ListItem::new(("document", index), format!("Document {}", index + 1))
                .selected(index == self.document)
                .on_activate(move |window, cx| {
                    owner
                        .update(cx, |this, cx| {
                            if this.document != index {
                                this.detail_scroll.set_offset(gpui::point(px(0.), px(0.)));
                            }
                            this.document = index;
                            this.detail = ListDetailPane::Detail;
                            if ListDetail::is_single_pane(width) {
                                this.detail_focus.focus(window, cx);
                            }
                            cx.notify();
                        })
                        .ok();
                })
        }));
        let detail_title = format!("Document {}", self.document + 1);
        ListDetail::new(
            "inbox-layout",
            width,
            isolated_scroll("inbox-scroll", &self.inbox_scroll)
                .test_support()
                .track_focus(&self.inbox_focus)
                .flex_1()
                .p_4()
                .child(list),
            isolated_scroll("document-scroll", &self.detail_scroll)
                .test_support()
                .track_focus(&self.detail_focus)
                .flex_1()
                .p_4()
                .child(Text::new(detail_title).role(TypeRole::HeadlineSmall))
                .children(lines("Document section")),
        )
        .active_pane(self.detail)
    }

    fn activity(&self, width: Pixels) -> SupportingPane {
        SupportingPane::new(
            "activity-layout",
            width,
            isolated_scroll("activity-scroll", &self.activity_scroll)
                .test_support()
                .track_focus(&self.activity_focus)
                .flex_1()
                .p_4()
                .children(lines("Recent change")),
            isolated_scroll("support-scroll", &self.support_scroll)
                .test_support()
                .track_focus(&self.support_focus)
                .flex_1()
                .p_4()
                .child(Text::new("Workspace details").role(TypeRole::TitleMedium))
                .children(lines("Activity detail")),
        )
        .show_support(self.support)
    }

    fn feed(&self, width: Pixels) -> Feed {
        Feed::new("adaptive-feed-scroll", width, &self.feed_scroll).children((0_usize..36).map(
            |index| {
                Card::new(("feed-card", index))
                    .variant(CardVariant::Outlined)
                    .child(Text::new(format!("Update {}", index + 1)).role(TypeRole::TitleMedium))
                    .child(
                        Text::new("Workspace activity")
                            .role(TypeRole::BodyMedium)
                            .wrap(),
                    )
            },
        ))
    }

    fn app_bar(&self, inner_width: Pixels, cx: &Context<Self>) -> AppBar {
        let single = ListDetail::is_single_pane(inner_width);
        let title = match self.destination {
            Destination::Inbox => "Inbox",
            Destination::Activity => "Activity",
            Destination::Feed => "Feed",
        };
        let app_bar = AppBar::new("adaptive-app-bar", title);
        let owner = cx.weak_entity();
        match self.destination {
            Destination::Inbox if single && self.detail == ListDetailPane::Detail => app_bar
                .navigation(
                    IconButton::new(
                        "adaptive-back",
                        "Back to documents",
                        Icon::new(IconName::ArrowForward)
                            .mirror_in_rtl(true)
                            .rotate(gpui::radians(std::f32::consts::PI)),
                    )
                    .on_activate(move |window, cx| {
                        owner
                            .update(cx, |this, cx| {
                                this.detail = ListDetailPane::List;
                                this.inbox_focus.focus(window, cx);
                                cx.notify();
                            })
                            .ok();
                    }),
                ),
            Destination::Activity if single => app_bar.actions(
                Button::new(
                    "adaptive-support",
                    if self.support {
                        "Back to activity"
                    } else {
                        "Workspace details"
                    },
                )
                .variant(ButtonVariant::Text)
                .on_activate(move |window, cx| {
                    owner
                        .update(cx, |this, cx| {
                            this.support = !this.support;
                            let focus = if this.support {
                                &this.support_focus
                            } else {
                                &this.activity_focus
                            };
                            focus.focus(window, cx);
                            cx.notify();
                        })
                        .ok();
                }),
            ),
            _ => app_bar,
        }
    }
}

fn lines(label: &'static str) -> impl Iterator<Item = gpui::Div> {
    (1..=24).map(move |index| {
        div().py_3().child(
            Text::new(format!("{label} {index}"))
                .id((label, index as usize))
                .role(TypeRole::BodyMedium)
                .wrap(),
        )
    })
}

impl Render for AdaptiveExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = self.width.get();
        let inner_width = if ListDetail::is_single_pane(width) {
            width
        } else {
            width - px(96.)
        };
        let content: AnyElement = match self.destination {
            Destination::Inbox => self.inbox(inner_width, cx).into_any_element(),
            Destination::Activity => self.activity(inner_width).into_any_element(),
            Destination::Feed => self.feed(inner_width).into_any_element(),
        };
        let destinations = [
            (Destination::Inbox, "Inbox", IconName::Inbox),
            (Destination::Activity, "Activity", IconName::Notifications),
            (Destination::Feed, "Feed", IconName::Description),
        ];
        let items = NavigationItems::new(
            "adaptive-destinations",
            "Workspace destinations",
            destinations.map(|(destination, label, icon)| {
                NavigationItem::new(destination.id(), label).icon(Icon::new(icon))
            }),
        )
        .selected(self.destination.id())
        .on_select(cx.listener(move |this, id: &SharedString, _, cx| {
            if let Some((destination, _, _)) = destinations
                .iter()
                .find(|entry| entry.0.id() == id.as_ref())
            {
                this.destination = *destination;
                cx.notify();
            }
        }));
        let snackbar = self.snackbar.clone();
        let scaffold = Scaffold::new("adaptive-scaffold", content)
            .app_bar(self.app_bar(inner_width, cx))
            .fab(
                Fab::new(
                    "adaptive-create",
                    "Create document",
                    Icon::new(IconName::Add),
                )
                .on_activate(move |window, cx| {
                    snackbar.update(cx, |host, cx| {
                        host.push(Snackbar::new("Draft created"), window, cx);
                    });
                }),
            )
            .snackbar(self.snackbar.clone());
        let size = self.width.clone();
        let owner = cx.weak_entity();
        let focus = [
            (self.inbox_focus.clone(), self.detail_focus.clone()),
            (self.activity_focus.clone(), self.support_focus.clone()),
        ];
        div()
            .relative()
            .size_full()
            .min_h_0()
            .min_w_0()
            .child(AdaptiveNavigation::new(scaffold, items, width))
            .child(
                canvas(
                    move |bounds, window, cx| {
                        if size.replace(bounds.size.width) != bounds.size.width {
                            let focus = focus.map(|(main, secondary)| {
                                (
                                    main.contains_focused(window, cx),
                                    secondary.contains_focused(window, cx),
                                )
                            });
                            window.defer(cx, move |_, cx| {
                                owner
                                    .update(cx, |this, cx| {
                                        if focus[0].0 {
                                            this.detail = ListDetailPane::List;
                                        }
                                        if focus[0].1 {
                                            this.detail = ListDetailPane::Detail;
                                        }
                                        if focus[1].0 {
                                            this.support = false;
                                        }
                                        if focus[1].1 {
                                            this.support = true;
                                        }
                                        cx.notify();
                                    })
                                    .ok();
                            });
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0()
                .size_full(),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{ElementId, InputEvent, TestAppContext, WindowOptions, point, size};
    use gpui_base::test_support::{ElementSnapshot, find};
    use gpui_material::{
        foundation::{MaterialRoot, material_theme, set_material_theme},
        theme::Direction,
    };

    use super::*;

    #[gpui::test]
    fn destinations_keep_focus_selection_and_pane_scroll_across_breakpoints(
        cx: &mut TestAppContext,
    ) {
        let mut example = None;
        let window = cx.update(|cx| {
            crate::init_gallery(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
                let entity = cx.new(AdaptiveExamples::new);
                example = Some(entity.clone());
                entity
            })
            .unwrap()
        });
        let example = example.unwrap();
        let draw = |cx: &mut TestAppContext| {
            for _ in 0..2 {
                cx.update_window(window.into(), |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .unwrap();
                cx.run_until_parked();
            }
        };
        let observe = |cx: &mut TestAppContext, id: ElementId| -> Option<ElementSnapshot> {
            cx.update_window(window.into(), |_, window, _| find(window, &[], &id))
                .unwrap()
        };
        let resize = |cx: &mut TestAppContext, width| {
            cx.simulate_window_resize(window.into(), size(px(width), px(540.)));
            draw(cx);
        };
        let click = |cx: &mut TestAppContext, id: ElementId| {
            let position = observe(cx, id.clone())
                .unwrap_or_else(|| panic!("missing {id:?}"))
                .bounds()
                .center();
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
            draw(cx);
        };
        resize(cx, 599.);
        click(
            cx,
            (ElementId::from(("document", 2_usize)), "activate").into(),
        );
        assert!(observe(cx, "detail-pane".into()).is_some());
        assert!(observe(cx, "list-pane".into()).is_none());
        click(cx, "adaptive-back".into());
        assert!(observe(cx, "list-pane".into()).is_some());
        assert!(observe(cx, "detail-pane".into()).is_none());
        click(
            cx,
            (ElementId::from(("document", 2_usize)), "activate").into(),
        );
        cx.update(|cx| {
            example
                .read(cx)
                .detail_scroll
                .set_offset(point(px(0.), px(-120.)));
        });
        click(cx, "adaptive-inbox".into());

        for width in [599., 600., 839., 840., 936., 1040., 839., 599., 1040.] {
            resize(cx, width);
            let item = observe(cx, "adaptive-inbox".into()).unwrap();
            let content = observe(cx, "scaffold-content".into()).unwrap().bounds();
            assert_eq!(item.selected(), Some(true));
            assert_eq!(
                item.focused(),
                Some(true),
                "focused destination at {width}dp"
            );
            if width < 840. {
                assert!(item.bounds().top() >= content.bottom());
            } else {
                assert!(item.bounds().right() <= content.left());
            }
            assert_eq!(observe(cx, "list-pane".into()).is_some(), width >= 936.);
            assert!(observe(cx, "detail-pane".into()).is_some());
            cx.update(|cx| {
                let example = example.read(cx);
                assert_eq!(example.document, 2);
                assert_eq!(example.detail_scroll.offset().y, px(-120.));
                assert_eq!(example.inbox_scroll.offset().y, px(0.));
            });
        }

        cx.update(|cx| {
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = Direction::Rtl;
            set_material_theme(theme, cx);
        });
        draw(cx);
        assert!(
            observe(cx, "adaptive-inbox".into())
                .unwrap()
                .bounds()
                .left()
                >= observe(cx, "scaffold-content".into())
                    .unwrap()
                    .bounds()
                    .right()
        );
        assert!(
            observe(cx, "list-pane".into()).unwrap().bounds().left()
                > observe(cx, "detail-pane".into()).unwrap().bounds().right()
        );
    }
}
