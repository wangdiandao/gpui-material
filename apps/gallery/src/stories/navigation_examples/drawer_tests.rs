use gpui::{
    AppContext, Context, ElementId, Entity, InputEvent, IntoElement, KeyDownEvent, KeyUpEvent,
    Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    SharedString, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Badge, Button, DialogCloseReason, DialogState, Icon, IconName, NavigationDrawer,
    NavigationDrawerMode, NavigationItem, NavigationItems,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

#[gpui::test]
fn navigation_drawer_supports_standard_roving_keyboard_rtl_and_modal_dismissal(
    cx: &mut TestAppContext,
) {
    struct Example {
        selected: SharedString,
        modal: Entity<DialogState>,
        closed_reasons: Vec<DialogCloseReason>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let on_open = self.modal.clone();
            let on_closed = cx.entity();
            let make_items = |id: &'static str| {
                NavigationItems::new(id, "Main navigation", [])
                    .selected(self.selected.clone())
                    .on_select(cx.listener(|this, id: &SharedString, _, cx| {
                        this.selected = id.clone();
                        cx.notify();
                    }))
            };
            div()
                .size_full()
                .flex()
                .child(
                    NavigationDrawer::new("std-drawer", make_items("std-nav"))
                        .headline("Workspace")
                        .section(
                            "Primary",
                            [
                                NavigationItem::new("inbox", "Inbox")
                                    .icon(Icon::new(IconName::Inbox))
                                    .badge(
                                        Badge::standalone()
                                            .content("cnt", "5")
                                            .accessible_label("unread", "5 unread"),
                                    ),
                                NavigationItem::new("starred", "Starred").disabled(true),
                                NavigationItem::new("archive", "Archive")
                                    .icon(Icon::new(IconName::Archive)),
                            ],
                        )
                        .divider()
                        .section("More", [NavigationItem::new("docs", "Documents")]),
                )
                .child(
                    div()
                        .p_4()
                        .child(Button::new("open-drawer", "Open drawer").on_activate(
                            move |window, cx| {
                                on_open.update(cx, |state, cx| state.open(window, cx));
                            },
                        )),
                )
                .child(
                    NavigationDrawer::new("modal-drawer", make_items("modal-nav"))
                        .mode(NavigationDrawerMode::Modal)
                        .headline("Modal workspace")
                        .modal_state(&self.modal)
                        .section(
                            "Primary",
                            [
                                NavigationItem::new("inbox", "Inbox"),
                                NavigationItem::new("archive", "Archive"),
                            ],
                        )
                        .on_closed(move |reason, _, cx| {
                            on_closed.update(cx, |this, cx| {
                                this.closed_reasons.push(reason);
                                cx.notify();
                            });
                        }),
                )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let modal = cx.new(DialogState::new);
            let e = cx.new(|_| Example {
                selected: "inbox".into(),
                modal,
                closed_reasons: Vec::new(),
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    cx.simulate_window_resize(handle.into(), size(px(800.), px(520.)));

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    // 1. Standard drawer geometry, role, badge description, roving keyboard skipping disabled item.
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let drawer = find(window, &[], &"std-drawer".into()).unwrap();
        assert_eq!(drawer.role(), Some(Role::Navigation));
        assert_eq!(drawer.bounds().size.width, px(360.));

        let inbox_id: ElementId = (ElementId::from("std-nav"), SharedString::from("inbox")).into();
        let archive_id: ElementId =
            (ElementId::from("std-nav"), SharedString::from("archive")).into();
        let inbox = find(window, &[], &inbox_id).unwrap();
        assert_eq!(inbox.selected(), Some(true));
        assert_eq!(inbox.description(), Some("5 unread"));

        // Click inbox to focus it, then press Down (skips disabled "starred" -> focuses "archive").
        let center = inbox.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);

        let down = Keystroke::parse("down").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: down.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: down }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &archive_id).unwrap().focused(),
            Some(true)
        );

        let enter = Keystroke::parse("enter").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: enter.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: enter }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(view.read(cx).selected, "archive");

        // Home and End move roving focus to first ("inbox") and last ("docs") destinations.
        let docs_id: ElementId = (ElementId::from("std-nav"), SharedString::from("docs")).into();
        let end_key = Keystroke::parse("end").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: end_key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: end_key }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(find(window, &[], &docs_id).unwrap().focused(), Some(true));

        let home_key = Keystroke::parse("home").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: home_key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            KeyUpEvent {
                keystroke: home_key,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(find(window, &[], &inbox_id).unwrap().focused(), Some(true));
    })
    .unwrap();

    // 2. Modal drawer opens via DialogState, renders in RTL on right edge, closes on Escape, restores focus.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);

        let open_btn = find(window, &[], &"open-drawer".into()).unwrap();
        let open_center = open_btn.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: open_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: open_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert!(view.read(cx).modal.read(cx).is_open());

        let modal_panel_id: ElementId = (ElementId::from("modal-drawer"), "drawer-panel").into();
        let modal_panel = find(window, &[], &modal_panel_id).unwrap();
        // In RTL, SheetEdge::Start docks on the right half of the 800px window.
        assert!(modal_panel.bounds().left() > px(300.));

        let esc = Keystroke::parse("escape").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: esc.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: esc }.to_platform_input(), cx);
        draw(window, cx);

        assert!(!view.read(cx).modal.read(cx).is_open());
        assert_eq!(view.read(cx).closed_reasons, [DialogCloseReason::Escape]);
        assert_eq!(
            find(window, &[], &"open-drawer".into()).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
}

#[gpui::test]
fn navigation_drawer_animates_standard_open_indicator_colors_and_pill_ripple(
    cx: &mut TestAppContext,
) {
    use std::time::Duration;

    use gpui::{Hsla, canvas};
    use gpui_material::NavigationItemColors;

    const INDICATOR: Hsla = gpui::hsla(0.62, 0.72, 0.74, 1.);
    const ACTIVE_INK: Hsla = gpui::hsla(0.95, 0.80, 0.42, 1.);
    const INACTIVE_INK: Hsla = gpui::hsla(0.32, 0.70, 0.36, 1.);

    fn swatch(width: f32) -> impl IntoElement {
        canvas(
            |_, window, _| window.text_style().color,
            |bounds, color, window, _| window.paint_quad(gpui::fill(bounds, color)),
        )
        .w(px(width))
        .h(px(10.))
    }

    struct Example {
        open: bool,
        selected: SharedString,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let colors = NavigationItemColors {
                selected_indicator: Some(INDICATOR),
                selected_icon: Some(ACTIVE_INK),
                selected_label_start: Some(ACTIVE_INK),
                unselected_icon: Some(INACTIVE_INK),
                unselected_label: Some(INACTIVE_INK),
                ..Default::default()
            };
            let items = NavigationItems::new(
                "anim-nav",
                "Animated navigation",
                [
                    NavigationItem::new("inbox", "Inbox")
                        .icon(Icon::custom(swatch(11.)))
                        .label_content(swatch(31.))
                        .colors(colors),
                    NavigationItem::new("archive", "Archive")
                        .icon(Icon::custom(swatch(13.)))
                        .label_content(swatch(37.))
                        .colors(colors),
                ],
            )
            .selected(self.selected.clone())
            .on_select(cx.listener(|this, id: &SharedString, _, cx| {
                this.selected = id.clone();
                cx.notify();
            }));
            div()
                .size_full()
                .flex()
                .child(
                    NavigationDrawer::new("anim-drawer", items)
                        .headline("Workspace")
                        .open(self.open),
                )
                .child(div().p_4().child(Button::new("outside-action", "Outside")))
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example {
            open: true,
            selected: "inbox".into(),
        });
        let handle =
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| view.clone())
                .unwrap();
        (handle, view)
    });
    cx.simulate_window_resize(handle.into(), size(px(800.), px(520.)));

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };
    let draw_and_count_frames = |window: &mut Window, cx: &mut gpui::App| {
        window.simulate_next_frame(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        window.simulate_next_frame(cx)
    };

    let archive_id: ElementId = (ElementId::from("anim-nav"), SharedString::from("archive")).into();
    let panel_id: ElementId = (ElementId::from("anim-drawer"), "drawer-panel").into();

    cx.update_window(handle.into(), |_, window, _| {
        window.activate_window();
    })
    .unwrap();
    cx.run_until_parked();

    // 1. Settled drawer schedules zero frames; pressing an item without interaction_source
    // paints ripple + state-layer animation frames and emits no static outer button active quad.
    let full_row_width = cx
        .update_window(handle.into(), |_, window, cx| {
            assert_eq!(draw_and_count_frames(window, cx), 0);
            assert_eq!(
                find(window, &[], &"anim-drawer".into())
                    .unwrap()
                    .bounds()
                    .size
                    .width,
                px(360.)
            );
            let archive_bounds = find(window, &[], &archive_id).unwrap().bounds();
            let center = archive_bounds.center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: center,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            archive_bounds.size.width
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(40));
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(
            draw_and_count_frames(window, cx) > 0,
            "pressing a drawer item without interaction_source must paint animated ripple/state-layer frames"
        );
        let center = find(window, &[], &archive_id).unwrap().bounds().center();
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();

    // 2. Selection transition animates pill indicator width/opacity and interpolates icon/label colors.
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(view.read(cx).selected, "archive");
        draw(window, cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(45));
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        let scale = window.scale_factor();
        let archive_bounds = find(window, &[], &archive_id).unwrap().bounds();
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
            indicator.0.size.width > px(0.) && indicator.0.size.width < full_row_width,
            "drawer indicator should expand horizontally mid-transition: {:?}",
            indicator.0.size.width
        );
        assert!(
            indicator.1 > 0. && indicator.1 < 1.,
            "drawer indicator opacity should animate mid-transition: {}",
            indicator.1
        );
        // Swatch quads for archive icon (13px) and label (37px) should have interpolated color.
        let label_swatch = quads
            .iter()
            .find(|quad| {
                let bounds = quad.bounds.map(|side| px(side.0 / scale));
                archive_bounds.contains(&bounds.center())
                    && (bounds.size.width - px(37.)).abs() <= px(0.5)
            })
            .expect("archive label swatch");
        let mid_color = label_swatch.background.as_solid().unwrap();
        assert_ne!(mid_color, INACTIVE_INK);
        assert_ne!(mid_color, ACTIVE_INK);
    })
    .unwrap();

    cx.executor().advance_clock(Duration::from_millis(500));
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        let scale = window.scale_factor();
        let archive_bounds = find(window, &[], &archive_id).unwrap().bounds();
        let settled_label = window
            .painted_quads()
            .into_iter()
            .find(|quad| {
                let bounds = quad.bounds.map(|side| px(side.0 / scale));
                archive_bounds.contains(&bounds.center())
                    && (bounds.size.width - px(37.)).abs() <= px(0.5)
            })
            .expect("settled archive label swatch");
        assert_eq!(settled_label.background.as_solid(), Some(ACTIVE_INK));
    })
    .unwrap();

    // 3. Standard drawer open(false) animates width smoothly without squishing inner panel,
    // removes destinations and moves focus out at settlement, and snaps under reduced motion.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.open = false;
            cx.notify();
        });
    });
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(75));
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        let drawer_width = find(window, &[], &"anim-drawer".into())
            .unwrap()
            .bounds()
            .size
            .width;
        assert!(
            drawer_width > px(0.) && drawer_width < px(360.),
            "standard drawer should animate width mid-close: {drawer_width:?}"
        );
        let panel_width = find(window, &[], &panel_id).unwrap().bounds().size.width;
        assert_eq!(
            panel_width,
            px(360.),
            "inner drawer panel must keep full width while sliding closed"
        );
    })
    .unwrap();

    cx.executor().advance_clock(Duration::from_millis(600));
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"anim-drawer".into())
                .unwrap()
                .bounds()
                .size
                .width,
            px(0.)
        );
        assert!(find(window, &[], &archive_id).is_none());
        assert_eq!(
            find(window, &[], &"outside-action".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();

    // Reopen under reduced motion settles immediately.
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        view.update(cx, |this, cx| {
            this.open = true;
            cx.notify();
        });
    });
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"anim-drawer".into())
                .unwrap()
                .bounds()
                .size
                .width,
            px(360.)
        );
        assert!(find(window, &[], &archive_id).is_some());
    })
    .unwrap();
}
