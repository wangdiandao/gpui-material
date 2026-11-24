use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AppContext, Bounds, Context, ElementId, Hsla, IntoElement, ParentElement, Pixels, Render, Role,
    ScrollHandle, SharedString, Styled, TestAppContext, Window, WindowHandle, canvas, div, point,
    px,
};
use gpui_base::{
    Root,
    test_support::{ElementSnapshot, find},
};
use gpui_material::{
    Badge, Icon, IconName, InteractionSource, MaterialRoot, MaterialTheme, NavigationItem,
    NavigationItemColors, NavigationItemIconPosition, NavigationItems, TabIndicatorLayout,
    TabIndicatorStyle, TabIndicatorWidth, TabVariant, Tabs, TabsColors,
    foundation::set_window_material_theme,
    theme::{CornerShape, CubicBezier, Direction, MotionSpec},
};

type TestWindow = WindowHandle<Root>;

const ROW: Hsla = gpui::hsla(0.52, 0.31, 0.66, 1.);
const DIVIDER: Hsla = gpui::hsla(0.89, 0.53, 0.31, 1.);
const MARK: Hsla = gpui::hsla(0.16, 0.82, 0.48, 1.);
const DEFAULT_MARK: Hsla = gpui::hsla(0.22, 0.13, 0.52, 1.);
const LABEL: Hsla = gpui::hsla(0.36, 0.71, 0.42, 1.);
const ICON: Hsla = gpui::hsla(0.75, 0.62, 0.6, 1.);
const BADGE: Hsla = gpui::hsla(0.02, 0.71, 0.45, 1.);
const DISABLED: Hsla = gpui::hsla(0.1, 0.24, 0.25, 1.);

fn frame(handle: TestWindow, cx: &mut TestAppContext) {
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn direction(handle: TestWindow, direction: Direction, cx: &mut TestAppContext) {
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = direction;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
}

fn node(window: &Window, id: &'static str) -> ElementSnapshot {
    find(window, &[], &id.into()).unwrap()
}

fn painted(window: &Window, tint: Hsla) -> (Bounds<Pixels>, f32) {
    let scale = window.scale_factor();
    let quad = crate::painted_quads(window)
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(tint))
        .expect("tab paint");
    (
        quad.bounds.map(|side| px(side.0 / scale)),
        quad.corner_radii.top_left.0 / scale,
    )
}

fn swatch(width: f32) -> impl IntoElement {
    canvas(
        |_, window, _| window.text_style().color,
        |bounds, ink, window, _| window.paint_quad(gpui::fill(bounds, ink)),
    )
    .w(px(width))
    .h(px(12.))
}

#[gpui::test]
fn tabs_custom_layout_paints_all_variants_and_preserves_accessible_names(cx: &mut TestAppContext) {
    struct Example {
        variant: TabVariant,
        scrollable: bool,
        custom: bool,
        layout: Rc<RefCell<Option<TabIndicatorLayout>>>,
        source: gpui::Entity<InteractionSource>,
        scroll: ScrollHandle,
    }
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let items = NavigationItems::new(
                "tabs",
                "Workspace tabs",
                [
                    NavigationItem::new("home", "Overview").icon(Icon::new(IconName::Info)),
                    NavigationItem::new("off", "Unavailable")
                        .disabled(true)
                        .label_content(swatch(29.))
                        .colors(NavigationItemColors {
                            disabled_label: Some(DISABLED),
                            ..Default::default()
                        })
                        .interaction_source(&self.source),
                    NavigationItem::new("details", "People details")
                        .icon(Icon::custom(swatch(11.)).size(px(24.)))
                        .icon_position(NavigationItemIconPosition::Start)
                        .label_content(swatch(54.))
                        .badge(
                            Badge::standalone()
                                .container_color(BADGE)
                                .content("person-count", "4")
                                .accessible_label("person-status", "4 online"),
                        )
                        .colors(NavigationItemColors {
                            selected_icon: Some(ICON),
                            selected_label_start: Some(LABEL),
                            selected_indicator: Some(MARK),
                            ..Default::default()
                        })
                        .interaction_source(&self.source),
                ],
            )
            .selected("details");
            let mut tabs = Tabs::new(items)
                .variant(self.variant)
                .scrollable(self.scrollable)
                .scroll_handle(&self.scroll)
                .edge_padding(px(14.))
                .min_tab_width(px(109.))
                .colors(TabsColors {
                    container: Some(ROW),
                    ..Default::default()
                })
                .divider(div().w_full().h(px(2.)).bg(DIVIDER))
                .indicator_style(TabIndicatorStyle {
                    width: TabIndicatorWidth::Content,
                    height: Some(px(5.)),
                    color: Some(DEFAULT_MARK),
                    shape: Some(CornerShape::Radii([1., 3., 1., 0.])),
                    ..Default::default()
                });
            if self.custom {
                let layout = self.layout.clone();
                tabs = tabs.indicator_content(move |indicator| {
                    let tint = indicator.color;
                    *layout.borrow_mut() = Some(indicator);
                    div().size_full().bg(tint)
                });
            }
            div().w(px(408.)).child(tabs)
        }
    }

    let (handle, view, captured, source, scroll) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let captured = Rc::new(RefCell::new(None));
        let source = cx.new(|_| InteractionSource::new());
        let scroll = ScrollHandle::new();
        let view = cx.new(|_| Example {
            variant: TabVariant::Primary,
            scrollable: false,
            custom: false,
            layout: captured.clone(),
            source: source.clone(),
            scroll: scroll.clone(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view, captured, source, scroll)
    });
    for variant in [TabVariant::Primary, TabVariant::Secondary] {
        for scrollable in [false, true] {
            for layout_direction in [Direction::Ltr, Direction::Rtl] {
                let custom = variant == TabVariant::Secondary;
                cx.update(|cx| {
                    view.update(cx, |view, cx| {
                        view.variant = variant;
                        view.scrollable = scrollable;
                        view.custom = custom;
                        cx.notify();
                    })
                });
                direction(handle, layout_direction, cx);
                frame(handle, cx);
                frame(handle, cx);
                cx.update_window(handle.into(), |_, window, cx| {
                    let tabs = node(window, "tabs");
                    let tab = node(window, "details");
                    assert_eq!(tabs.role(), Some(Role::TabList));
                    assert_eq!(tab.role(), Some(Role::Tab));
                    assert_eq!(tab.label(), Some("People details"));
                    assert_eq!(tab.description(), Some("4 online"));
                    assert_eq!(tab.selected(), Some(true));
                    assert_eq!(node(window, "off").disabled(), Some(true));
                    assert_eq!(painted(window, ROW).0.size.width, tabs.bounds().size.width);
                    assert_eq!(
                        painted(window, DIVIDER).0.size.width,
                        tabs.bounds().size.width
                    );
                    assert!(
                        node(window, "off")
                            .bounds()
                            .contains(&painted(window, DISABLED).0.center())
                    );
                    let (mark, corner) = painted(window, MARK);
                    assert_eq!(mark.size.height, px(5.));
                    assert!(mark.size.width < tab.bounds().size.width);
                    assert!((mark.center().x - tab.bounds().center().x).abs() <= px(1.));
                    let label = painted(window, LABEL).0;
                    let icon = painted(window, ICON).0;
                    assert_eq!(
                        icon.center().x < label.center().x,
                        layout_direction == Direction::Ltr
                    );
                    assert_eq!(
                        painted(window, BADGE).0.center().x > icon.center().x,
                        layout_direction == Direction::Ltr
                    );
                    assert!(mark.size.width >= label.size.width + icon.size.width);
                    if custom {
                        let captured = captured.borrow();
                        let layout = captured.as_ref().unwrap();
                        assert_eq!(layout.selected, "details");
                        assert_eq!(layout.direction, layout_direction);
                        assert_eq!(mark.size.width, layout.content_width);
                    } else {
                        assert_eq!(
                            corner,
                            if layout_direction == Direction::Ltr {
                                1.
                            } else {
                                3.
                            }
                        );
                    }
                    if scrollable {
                        assert!(node(window, "off").bounds().size.width >= px(109.));
                        assert_eq!(scroll.bounds(), tabs.bounds());
                        assert_eq!(
                            scroll.bounds_for_item(0).unwrap().left() - scroll.bounds().left(),
                            px(14.)
                        );
                    }
                    let disabled_key =
                        ElementId::from((ElementId::from("tabs"), SharedString::from("off")));
                    assert!(
                        source
                            .read(cx)
                            .state(window.window_handle().window_id(), &disabled_key)
                            .unwrap()
                            .disabled
                    );
                })
                .unwrap();
            }
        }
    }
}

#[gpui::test]
fn tabs_reveal_selection_and_focus_without_overriding_manual_scroll(cx: &mut TestAppContext) {
    struct Example {
        order: Vec<&'static str>,
        selected: &'static str,
        width: Pixels,
        scroll: ScrollHandle,
        source: gpui::Entity<InteractionSource>,
    }
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(self.width).child(
                Tabs::new(
                    NavigationItems::new(
                        "tabs",
                        "Folders",
                        self.order.iter().map(|&name| {
                            NavigationItem::new(name, name)
                                .disabled(name == "Fifth")
                                .interaction_source(&self.source)
                        }),
                    )
                    .selected(self.selected),
                )
                .scrollable(true)
                .scroll_handle(&self.scroll)
                .edge_padding(px(0.))
                .min_tab_width(px(104.))
                .indicator_style(TabIndicatorStyle {
                    width: TabIndicatorWidth::Fixed(px(34.)),
                    color: Some(MARK),
                    ..Default::default()
                }),
            )
        }
    }

    let (handle, view, scroll, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let scroll = ScrollHandle::new();
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|_| Example {
            order: vec!["First", "Second", "Third", "Fourth", "Fifth", "Last"],
            selected: "First",
            width: px(235.),
            scroll: scroll.clone(),
            source: source.clone(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view, scroll, source)
    });
    let rendered = |name, cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, _| node(window, name))
            .unwrap()
    };
    frame(handle, cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.focus_next(cx);
    })
    .unwrap();
    frame(handle, cx);
    assert_eq!(rendered("First", cx).focused(), Some(true));
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.selected = "Last";
            cx.notify();
        })
    });
    frame(handle, cx);
    frame(handle, cx);
    let chosen = rendered("Last", cx);
    assert_eq!(chosen.selected(), Some(true));
    assert!(scroll.bounds().contains(&chosen.bounds().center()));
    assert_eq!(rendered("First", cx).focused(), Some(true));
    cx.update_window(handle.into(), |_, window, _| {
        assert!((painted(window, MARK).0.center().x - chosen.bounds().center().x).abs() <= px(1.));
    })
    .unwrap();

    scroll.set_offset(point(px(0.), px(0.)));
    frame(handle, cx);
    frame(handle, cx);
    assert_eq!(scroll.offset().x, px(0.));
    assert!(!rendered("Last", cx).visible());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.order.swap(1, 5);
            view.width = px(149.);
            cx.notify();
        })
    });
    frame(handle, cx);
    frame(handle, cx);
    assert!(
        scroll
            .bounds()
            .contains(&rendered("Last", cx).bounds().center())
    );

    let alternate = ScrollHandle::new();
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.scroll = alternate.clone();
            cx.notify();
        })
    });
    frame(handle, cx);
    assert!(
        alternate
            .bounds()
            .contains(&rendered("Last", cx).bounds().center())
    );
    direction(handle, Direction::Rtl, cx);
    frame(handle, cx);
    assert!(
        alternate
            .bounds()
            .contains(&rendered("Last", cx).bounds().center())
    );
    cx.simulate_keystrokes(handle.into(), "left");
    frame(handle, cx);
    assert_eq!(rendered("Last", cx).focused(), Some(true));
    let key = ElementId::from((ElementId::from("tabs"), SharedString::from("Last")));
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(
            source
                .read(cx)
                .state(window.window_handle().window_id(), &key)
                .unwrap()
                .focus_visible
        );
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "end");
    frame(handle, cx);
    assert_eq!(rendered("Second", cx).focused(), Some(true));
    assert_eq!(rendered("Last", cx).selected(), Some(true));
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.order.retain(|&name| name != "Last");
            view.selected = "Second";
            cx.notify();
        })
    });
    frame(handle, cx);
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(find(window, &[], &"Last".into()).is_none());
        assert!(
            source
                .read(cx)
                .state(window.window_handle().window_id(), &key)
                .is_none()
        );
    })
    .unwrap();
}

#[gpui::test]
fn tabs_custom_indicator_animates_between_tab_centers(cx: &mut TestAppContext) {
    struct Example(bool);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            Tabs::new(
                NavigationItems::new(
                    "tabs",
                    "Messages",
                    [
                        NavigationItem::new("initial", "Initial"),
                        NavigationItem::new("next", "Next"),
                    ],
                )
                .selected(if self.0 { "next" } else { "initial" }),
            )
            .indicator_style(TabIndicatorStyle {
                width: TabIndicatorWidth::Tab,
                color: Some(MARK),
                motion: Some(MotionSpec::Tween {
                    duration_ms: 200,
                    easing: CubicBezier {
                        x1: 0.,
                        y1: 0.,
                        x2: 1.,
                        y2: 1.,
                    },
                }),
                ..Default::default()
            })
        }
    }
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example(false));
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view)
    });
    let center = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, _| {
            painted(window, MARK).0.center().x
        })
        .unwrap()
    };
    frame(handle, cx);
    let before = center(cx);
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.0 = true;
            cx.notify();
        })
    });
    frame(handle, cx);
    let start = center(cx);
    assert_eq!(start, before);
    cx.executor().advance_clock(Duration::from_millis(100));
    frame(handle, cx);
    let midway = center(cx);
    cx.executor().advance_clock(Duration::from_millis(300));
    frame(handle, cx);
    assert!(before < midway && midway < center(cx));
}
