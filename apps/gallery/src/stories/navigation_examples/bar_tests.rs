use gpui::{
    App, AppContext, Bounds, Context, ElementId, Entity, Hsla, InputEvent, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    Pixels, Point, Render, Role, SharedString, Styled, TestAppContext, Window, canvas, div, px,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::{
    Badge, Button, Icon, InteractionSource, MaterialRoot, NavigationBar, NavigationBarArrangement,
    NavigationBarColors, NavigationBarLayout, NavigationItem, NavigationItemColors,
    NavigationItemIconPosition, NavigationItems,
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

const BAR: Hsla = gpui::hsla(0.55, 0.51, 0.71, 1.);
const SHARED_INK: Hsla = gpui::hsla(0.57, 0.63, 0.38, 1.);
const TOP_LABEL: Hsla = gpui::hsla(0.23, 0.81, 0.29, 1.);
const START_LABEL: Hsla = gpui::hsla(0.82, 0.61, 0.48, 1.);
const ICON: Hsla = gpui::hsla(0.48, 0.84, 0.37, 1.);
const INDICATOR: Hsla = gpui::hsla(0.91, 0.71, 0.64, 1.);
const BADGE: Hsla = gpui::hsla(0.07, 0.82, 0.58, 1.);
const DISABLED: Hsla = gpui::hsla(0.35, 0.33, 0.42, 1.);

fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

fn set_direction(direction: Direction, window: &mut Window, cx: &mut App) {
    let mut theme = window_material_theme(window, cx).clone();
    theme.layout.direction = direction;
    set_window_material_theme(Some(theme), window, cx);
}

fn item(window: &Window, id: &'static str) -> ElementSnapshot {
    let scoped: ElementId = (ElementId::from("bar"), id).into();
    find(window, &[], &scoped)
        .or_else(|| find(window, &[], &id.into()))
        .unwrap()
}

fn swatch(width: f32) -> impl IntoElement {
    canvas(
        |_, window, _| window.text_style().color,
        |bounds, color, window, _| window.paint_quad(gpui::fill(bounds, color)),
    )
    .w(px(width))
    .h(px(10.))
}

fn painted(window: &Window, tint: Hsla) -> Bounds<Pixels> {
    window
        .painted_quads()
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(tint))
        .expect("expected navigation pigment")
        .bounds
        .map(|side| px(side.0 / window.scale_factor()))
}

#[gpui::test]
fn short_navigation_bar_places_and_colors_mixed_destinations(cx: &mut TestAppContext) {
    struct Example {
        arrangement: NavigationBarArrangement,
        layout: NavigationBarLayout,
        selected: &'static str,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let opposite = match self.layout {
                NavigationBarLayout::Vertical => NavigationItemIconPosition::Start,
                NavigationBarLayout::Horizontal => NavigationItemIconPosition::Top,
            };
            let items = NavigationItems::new(
                "bar",
                "Primary navigation",
                [
                    NavigationItem::new("inbox", "Inbox")
                        .icon(Icon::custom(swatch(10.)))
                        .label_content(swatch(36.))
                        .badge(
                            Badge::standalone()
                                .container_color(BADGE)
                                .content("unread-count", "3")
                                .accessible_label("badge-status", "3 unread messages"),
                        )
                        .colors(NavigationItemColors {
                            selected_icon: Some(ICON),
                            selected_label_top: Some(TOP_LABEL),
                            selected_label_start: Some(START_LABEL),
                            selected_indicator: Some(INDICATOR),
                            ..Default::default()
                        }),
                    NavigationItem::new("archive", "Archive")
                        .icon(Icon::custom(div().size(px(10.)).bg(ICON.opacity(0.5))))
                        .icon_position(opposite)
                        .label_content(swatch(180.)),
                    NavigationItem::new("off", "Unavailable")
                        .icon(Icon::new(gpui_material::IconName::Info))
                        .label_content(swatch(51.))
                        .disabled(true)
                        .colors(NavigationItemColors {
                            disabled_label: Some(DISABLED),
                            selected_label_top: Some(TOP_LABEL),
                            selected_label_start: Some(START_LABEL),
                            ..Default::default()
                        }),
                ],
            )
            .selected(self.selected);
            div().w(px(720.)).child(
                NavigationBar::new(items)
                    .layout(self.layout)
                    .arrangement(self.arrangement)
                    .colors(NavigationBarColors {
                        container: Some(BAR),
                        content: Some(SHARED_INK),
                    }),
            )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Example {
            arrangement: NavigationBarArrangement::EqualWeight,
            layout: NavigationBarLayout::Vertical,
            selected: "inbox",
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });

    for arrangement in [
        NavigationBarArrangement::EqualWeight,
        NavigationBarArrangement::Centered,
    ] {
        for layout in [
            NavigationBarLayout::Vertical,
            NavigationBarLayout::Horizontal,
        ] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                cx.update_window(handle, |_, window, cx| {
                    view.update(cx, |view, cx| {
                        view.arrangement = arrangement;
                        view.layout = layout;
                        cx.notify();
                    });
                    set_direction(direction, window, cx);
                    draw(window, cx);
                    let row = painted(window, BAR);
                    let inbox = item(window, "inbox");
                    let archive = item(window, "archive");
                    let off = item(window, "off");
                    assert_eq!(inbox.label(), Some("Inbox"));
                    assert_eq!(inbox.description(), Some("3 unread messages"));
                    assert_eq!(archive.description(), None);
                    assert_eq!(inbox.role(), Some(Role::ListBoxOption));
                    assert_eq!(inbox.selected(), Some(true));
                    assert_eq!(off.disabled(), Some(true));
                    let first = inbox.bounds();
                    let middle = archive.bounds();
                    let last = off.bounds();
                    let used_left = first.left().min(last.left());
                    let used_right = first.right().max(last.right());
                    let spare_left = used_left - row.left();
                    let spare_right = row.right() - used_right;
                    assert!((spare_left - spare_right).abs() <= px(1.));
                    assert_eq!(first.left() < last.left(), direction == Direction::Ltr);
                    if arrangement == NavigationBarArrangement::EqualWeight {
                        assert_eq!(first.size.width, px(240.));
                        assert_eq!(middle.size.width, px(240.));
                        assert_eq!(last.size.width, px(240.));
                        assert_eq!(spare_left, px(0.));
                    } else {
                        assert_eq!(first.size.width, px(144.));
                        assert!(
                            middle.size.width > first.size.width && middle.size.width <= px(240.)
                        );
                        assert!(spare_left > px(0.));
                    }

                    let icon = painted(window, ICON);
                    let second_icon = painted(window, ICON.opacity(0.5));
                    let second_label = painted(window, SHARED_INK);
                    let indicator = painted(window, INDICATOR);
                    let badge = painted(window, BADGE);
                    assert!(indicator.contains(&icon.center()));
                    assert!(badge.top() < icon.top());
                    assert_eq!(
                        badge.center().x > icon.center().x,
                        direction == Direction::Ltr
                    );
                    assert!(off.bounds().contains(&painted(window, DISABLED).center()));
                    if layout == NavigationBarLayout::Vertical {
                        assert!(painted(window, TOP_LABEL).top() >= icon.bottom());
                        assert_eq!(
                            second_icon.center().x < second_label.center().x,
                            direction == Direction::Ltr
                        );
                    } else {
                        assert!(second_label.top() >= second_icon.bottom());
                        let label = painted(window, START_LABEL);
                        assert_eq!(
                            icon.center().x < label.center().x,
                            direction == Direction::Ltr
                        );
                        assert!(indicator.contains(&label.center()));
                    }
                })
                .unwrap();
            }
        }
    }

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.selected = "off";
            cx.notify();
        });
        draw(window, cx);
        let off = item(window, "off");
        assert_eq!(off.selected(), Some(true));
        assert_eq!(off.disabled(), Some(true));
        assert!(off.bounds().contains(&painted(window, DISABLED).center()));
    })
    .unwrap();
}

fn mouse_down(position: Point<Pixels>, window: &mut Window, cx: &mut App) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
}

fn mouse_up(position: Point<Pixels>, window: &mut Window, cx: &mut App) {
    window.dispatch_event(
        MouseUpEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
}

fn enter(window: &mut Window, cx: &mut App) {
    let keystroke = Keystroke::parse("enter").unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
}

#[gpui::test]
fn short_navigation_bar_roving_focus_and_observers_follow_item_ids(cx: &mut TestAppContext) {
    struct Example {
        arrangement: NavigationBarArrangement,
        order: Vec<&'static str>,
        selected: SharedString,
        requested: Vec<SharedString>,
        source: Entity<InteractionSource>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let items = NavigationItems::new(
                "bar",
                "Primary navigation",
                self.order.iter().map(|&id| {
                    NavigationItem::new(id, id)
                        .disabled(id == "off")
                        .interaction_source(&self.source)
                }),
            )
            .selected(self.selected.clone())
            .on_select(cx.listener(|this, id: &SharedString, _, cx| {
                this.requested.push(id.clone());
                cx.notify();
            }));
            div()
                .w(px(720.))
                .flex()
                .flex_col()
                .child(Button::new("before", "Before navigation"))
                .child(NavigationBar::new(items).arrangement(self.arrangement))
                .child(Button::new("after", "After navigation"))
        }
    }

    let (handle, view, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|_| Example {
            arrangement: NavigationBarArrangement::EqualWeight,
            order: vec!["inbox", "off", "archive", "settings"],
            selected: "inbox".into(),
            requested: Vec::new(),
            source: source.clone(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, source)
    });
    let snapshot = |id, cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            draw(window, cx);
            item(window, id)
        })
        .unwrap()
    };
    let source_id =
        |id: &'static str| ElementId::from((ElementId::from("bar"), SharedString::from(id)));
    for arrangement in [
        NavigationBarArrangement::EqualWeight,
        NavigationBarArrangement::Centered,
    ] {
        cx.update_window(handle, |_, window, cx| {
            window.activate_window();
            window.blur(cx);
            view.update(cx, |view, cx| {
                view.arrangement = arrangement;
                view.selected = "inbox".into();
                view.requested.clear();
                cx.notify();
            });
            set_direction(Direction::Ltr, window, cx);
            draw(window, cx);
            window.focus_next(cx);
            draw(window, cx);
            assert_eq!(
                find(window, &[], &"before".into()).unwrap().focused(),
                Some(true)
            );
            window.focus_next(cx);
            draw(window, cx);
            assert_eq!(item(window, "inbox").focused(), Some(true));
            window.focus_next(cx);
            draw(window, cx);
            assert_eq!(
                find(window, &[], &"after".into()).unwrap().focused(),
                Some(true)
            );
        })
        .unwrap();
        cx.update_window(handle, |_, window, cx| {
            window.focus_prev(cx);
            draw(window, cx);
        })
        .unwrap();
        cx.simulate_keystrokes(handle, "right");
        assert_eq!(snapshot("archive", cx).focused(), Some(true));
        assert_eq!(snapshot("inbox", cx).selected(), Some(true));
        cx.update_window(handle, |_, window, cx| enter(window, cx))
            .unwrap();
        assert_eq!(snapshot("archive", cx).selected(), Some(false));
        cx.update(|cx| assert_eq!(view.read(cx).requested, vec![SharedString::from("archive")]));
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.selected = "archive".into();
                cx.notify();
            });
        });
        assert_eq!(snapshot("archive", cx).selected(), Some(true));

        cx.update_window(handle, |_, window, cx| {
            set_direction(Direction::Rtl, window, cx);
            draw(window, cx);
        })
        .unwrap();
        cx.simulate_keystrokes(handle, "right");
        assert_eq!(snapshot("inbox", cx).focused(), Some(true));
        cx.simulate_keystrokes(handle, "end");
        assert_eq!(snapshot("settings", cx).focused(), Some(true));
        cx.simulate_keystrokes(handle, "home");
        assert_eq!(snapshot("inbox", cx).focused(), Some(true));
        cx.update_window(handle, |_, window, cx| {
            let off = item(window, "off").bounds().center();
            mouse_down(off, window, cx);
            mouse_up(off, window, cx);
            assert_eq!(view.read(cx).requested.len(), 1);
        })
        .unwrap();
    }

    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.order.reverse();
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(item(window, "inbox").focused(), Some(true));
        let archive = item(window, "archive").bounds().center();
        mouse_down(archive, window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        let id = window.window_handle().window_id();
        assert!(
            source
                .read(cx)
                .state(id, &source_id("archive"))
                .unwrap()
                .pressed
        );
        assert!(
            source
                .read(cx)
                .state(id, &source_id("off"))
                .unwrap()
                .disabled
        );
        mouse_up(item(window, "archive").bounds().center(), window, cx);
        assert_eq!(
            view.read(cx).requested.last().map(AsRef::as_ref),
            Some("archive")
        );
        view.update(cx, |view, cx| {
            view.order.retain(|&id| id != "archive");
            view.selected = "inbox".into();
            cx.notify();
        });
        draw(window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        let id = window.window_handle().window_id();
        assert!(source.read(cx).state(id, &source_id("archive")).is_none());
        assert!(source.read(cx).state(id, &source_id("inbox")).is_some());
    })
    .unwrap();
}

#[gpui::test]
fn navigation_bar_animates_pill_expand_color_interpolation_and_pill_ripple(
    cx: &mut TestAppContext,
) {
    use std::time::Duration;

    const UNSELECTED_ICON: Hsla = gpui::hsla(0.12, 0.72, 0.35, 1.);
    const UNSELECTED_LABEL: Hsla = gpui::hsla(0.42, 0.65, 0.32, 1.);

    struct Example {
        selected: &'static str,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let colors = NavigationItemColors {
                selected_icon: Some(ICON),
                selected_label_top: Some(TOP_LABEL),
                selected_indicator: Some(INDICATOR),
                unselected_icon: Some(UNSELECTED_ICON),
                unselected_label: Some(UNSELECTED_LABEL),
                ..Default::default()
            };
            let items = NavigationItems::new(
                "bar",
                "Primary navigation",
                [
                    NavigationItem::new("inbox", "Inbox")
                        .icon(Icon::custom(swatch(10.)))
                        .label_content(swatch(36.))
                        .colors(colors),
                    NavigationItem::new("archive", "Archive")
                        .icon(Icon::custom(swatch(12.)))
                        .label_content(swatch(44.))
                        .colors(colors),
                ],
            )
            .selected(self.selected);
            div().w(px(480.)).child(NavigationBar::new(items))
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example { selected: "inbox" });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });

    let draw_and_count_frames = |window: &mut Window, cx: &mut App| {
        window.simulate_next_frame(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        window.simulate_next_frame(cx)
    };

    cx.update_window(handle, |_, window, _| {
        window.activate_window();
    })
    .unwrap();
    cx.run_until_parked();

    // 1. Settled bar schedules zero frames; pressing an item without interaction_source
    // paints animated pill-clipped ripple/state-layer frames without a rectangular outer button quad.
    let archive_bounds = cx
        .update_window(handle, |_, window, cx| {
            assert_eq!(draw_and_count_frames(window, cx), 0);
            let bounds = item(window, "archive").bounds();
            mouse_down(bounds.center(), window, cx);
            bounds
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(40));
    cx.update_window(handle, |_, window, cx| {
        assert!(
            draw_and_count_frames(window, cx) > 0,
            "pressing a NavigationBar item without interaction_source must paint animated ripple/state-layer frames"
        );
        // No outer rectangular button-width hover/active quad should be painted.
        let scale = window.scale_factor();
        let outer_rect_state_layer = window.painted_quads().into_iter().any(|quad| {
            let bounds = quad.bounds.map(|side| px(side.0 / scale));
            archive_bounds.contains(&bounds.center())
                && bounds.size.width > px(120.)
                && quad
                    .background
                    .as_solid()
                    .is_some_and(|color| color.a > 0. && color.a < 0.2)
        });
        assert!(
            !outer_rect_state_layer,
            "outer NavigationBar button must not paint a rectangular hover/active quad"
        );
        mouse_up(archive_bounds.center(), window, cx);
    })
    .unwrap();

    // 2. Switching selection animates pill horizontal width expansion and icon/label color interpolation.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.selected = "archive";
            cx.notify();
        });
    });
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(45));
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        let indicator = quads
            .iter()
            .filter_map(|quad| {
                let solid = quad.background.as_solid()?;
                let bounds = quad.bounds.map(|side| px(side.0 / scale));
                (archive_bounds.contains(&bounds.center())
                    && (solid.h - INDICATOR.h).abs() < 0.02
                    && (solid.s - INDICATOR.s).abs() < 0.02
                    && (solid.l - INDICATOR.l).abs() < 0.02)
                    .then_some((bounds, solid.a))
            })
            .next()
            .expect("mid-transition archive pill indicator quad");
        assert!(
            indicator.0.size.width > px(0.) && indicator.0.size.width < px(56.),
            "bar pill indicator should expand horizontally mid-transition: {:?}",
            indicator.0.size.width
        );
        assert!(indicator.1 > 0. && indicator.1 < 1.);

        let icon_swatch = quads
            .iter()
            .find(|quad| {
                let bounds = quad.bounds.map(|side| px(side.0 / scale));
                archive_bounds.contains(&bounds.center())
                    && (bounds.size.width - px(12.)).abs() <= px(0.5)
            })
            .expect("archive icon swatch");
        let mid_icon = icon_swatch.background.as_solid().unwrap();
        assert_ne!(mid_icon, UNSELECTED_ICON);
        assert_ne!(mid_icon, ICON);

        let label_swatch = quads
            .iter()
            .find(|quad| {
                let bounds = quad.bounds.map(|side| px(side.0 / scale));
                archive_bounds.contains(&bounds.center())
                    && (bounds.size.width - px(44.)).abs() <= px(0.5)
            })
            .expect("archive label swatch");
        let mid_label = label_swatch.background.as_solid().unwrap();
        assert_ne!(mid_label, UNSELECTED_LABEL);
        assert_ne!(mid_label, TOP_LABEL);
    })
    .unwrap();

    // 3. Settled frame reaches exact 56px pill width and exact selected colors.
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        let settled_indicator = quads
            .iter()
            .find(|quad| quad.background.as_solid() == Some(INDICATOR))
            .expect("settled indicator quad");
        let width = px(settled_indicator.bounds.size.width.0 / scale);
        assert!((width - px(56.)).abs() <= px(0.5));
        assert!(
            quads
                .iter()
                .any(|quad| quad.background.as_solid() == Some(ICON))
        );
        assert!(
            quads
                .iter()
                .any(|quad| quad.background.as_solid() == Some(TOP_LABEL))
        );
    })
    .unwrap();
}
