use gpui::{
    AppContext as _, Context, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Render, Role, ScrollHandle, Styled as _, TestAppContext, Window, WindowOptions, div,
    px, size,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{
    Card, Feed, MaterialTheme, SupportingPane,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction},
};

#[gpui::test]
fn feed_responsive_columns_rtl_role_and_scroll_anchor(cx: &mut TestAppContext) {
    struct FeedPreview {
        width: Pixels,
        scroll: ScrollHandle,
    }

    impl Render for FeedPreview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(self.width).h(px(260.)).flex().flex_col().child(
                Feed::new("feed", self.width, &self.scroll)
                    .accessible_label("Updates feed")
                    .children((0..8usize).map(|index| {
                        Card::outlined(("feed-card", index)).child(
                            div()
                                .id(("feed-card-body", index))
                                .test_support()
                                .h(px(120.))
                                .child(format!("Item {index}")),
                        )
                    })),
            )
        }
    }

    assert_eq!(Feed::columns(px(400.)), 1);
    assert_eq!(Feed::columns(px(600.)), 2);
    assert_eq!(Feed::columns(px(840.)), 3);
    assert_eq!(Feed::columns(px(1200.)), 4);
    assert_eq!(Feed::columns(px(1600.)), 4);

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| FeedPreview {
                width: px(600.),
                scroll: ScrollHandle::new(),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.simulate_window_resize(handle.into(), size(px(1700.), px(400.)));

    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };

    // 2 columns at Medium width (600 dp).
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let feed = find(window, &[], &"feed".into()).unwrap();
        assert_eq!(feed.role(), Some(Role::List));
        assert_eq!(feed.label(), Some("Updates feed"));

        let c0 = find(window, &[], &ElementId::from(("feed-card-body", 0usize)))
            .unwrap()
            .bounds();
        let c1 = find(window, &[], &ElementId::from(("feed-card-body", 1usize)))
            .unwrap()
            .bounds();
        let c2 = find(window, &[], &ElementId::from(("feed-card-body", 2usize)))
            .unwrap()
            .bounds();
        assert_eq!(c0.top(), c1.top());
        assert!(c0.right() < c1.left());
        assert!(c2.top() > c0.bottom());
    })
    .unwrap();

    // 3 columns at Expanded width (840 dp) and 4 columns at Large width (1200 dp).
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.width = px(840.);
            cx.notify();
        });
    });
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let c0 = find(window, &[], &ElementId::from(("feed-card-body", 0usize)))
            .unwrap()
            .bounds();
        let c2 = find(window, &[], &ElementId::from(("feed-card-body", 2usize)))
            .unwrap()
            .bounds();
        let c3 = find(window, &[], &ElementId::from(("feed-card-body", 3usize)))
            .unwrap()
            .bounds();
        assert_eq!(c0.top(), c2.top());
        assert!(c3.top() > c0.bottom());
    })
    .unwrap();

    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.width = px(1200.);
            cx.notify();
        });
    });
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let c0 = find(window, &[], &ElementId::from(("feed-card-body", 0usize)))
            .unwrap()
            .bounds();
        let c3 = find(window, &[], &ElementId::from(("feed-card-body", 3usize)))
            .unwrap()
            .bounds();
        let c4 = find(window, &[], &ElementId::from(("feed-card-body", 4usize)))
            .unwrap()
            .bounds();
        assert_eq!(c0.top(), c3.top());
        assert!(c4.top() > c0.bottom());
    })
    .unwrap();

    // RTL reverses card order within each row.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let c0 = find(window, &[], &ElementId::from(("feed-card-body", 0usize)))
            .unwrap()
            .bounds();
        let c1 = find(window, &[], &ElementId::from(("feed-card-body", 1usize)))
            .unwrap()
            .bounds();
        assert!(c0.left() > c1.right());
    })
    .unwrap();

    // Changing from 1 column (scrolled to item 4) to 2 columns anchors scroll at row 2 (= 4 / 2).
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.width = px(400.);
            cx.notify();
        });
    });
    draw(cx);
    cx.update(|cx| {
        view.read(cx).scroll.scroll_to_top_of_item(4);
    });
    draw(cx);
    cx.update(|cx| {
        assert_eq!(view.read(cx).scroll.top_item(), 4);
        view.update(cx, |this, cx| {
            this.width = px(600.);
            cx.notify();
        });
    });
    draw(cx);
    cx.update(|cx| {
        assert_eq!(view.read(cx).scroll.top_item(), 2);
    });
}

#[gpui::test]
fn supporting_pane_narrow_toggle_and_rtl(cx: &mut TestAppContext) {
    struct Preview {
        width: Pixels,
        show_support: bool,
    }

    impl Render for Preview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(self.width).h(px(400.)).child(
                SupportingPane::new(
                    "supporting-standalone",
                    self.width,
                    div().child("Primary"),
                    div().child("Support"),
                )
                .show_support(self.show_support),
            )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| Preview {
                width: px(599.),
                show_support: false,
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1200.), px(500.)));

    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };

    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"primary-pane".into()).is_some());
        assert!(find(window, &[], &"supporting-pane".into()).is_none());
    })
    .unwrap();

    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.show_support = true;
            cx.notify();
        });
    });
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"primary-pane".into()).is_none());
        assert!(find(window, &[], &"supporting-pane".into()).is_some());
    })
    .unwrap();

    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.width = px(1000.);
            cx.notify();
        });
    });
    draw(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let primary = find(window, &[], &"primary-pane".into()).unwrap().bounds();
        let support = find(window, &[], &"supporting-pane".into())
            .unwrap()
            .bounds();
        assert!(primary.right() < support.left());

        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let primary = find(window, &[], &"primary-pane".into()).unwrap().bounds();
        let support = find(window, &[], &"supporting-pane".into())
            .unwrap()
            .bounds();
        assert!(primary.left() > support.right());
    })
    .unwrap();
}
