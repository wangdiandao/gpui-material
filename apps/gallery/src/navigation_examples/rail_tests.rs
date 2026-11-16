use std::time::Duration;

use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement as _, IntoElement, Modifiers, ParentElement as _, Pixels, Render, Role,
    SharedString, Styled as _, TestAppContext, VisualTestContext, Window, div, point, px, rgb,
    size,
};
use gpui_base::{
    TestSupportExt,
    test_support::{find, snapshots},
};
use gpui_material::{
    Badge, Button, DialogCloseReason, DialogProperties, DialogState, Icon, IconName, MaterialTheme,
    NavigationItem, NavigationItemColors, NavigationItemIconPosition, NavigationItems,
    NavigationRail, NavigationRailArrangement, NavigationRailColors, NavigationRailPadding,
    NavigationRailShapes, NavigationRailState, RailModalOptions, RailMode, SheetMargins,
    SheetPlacement,
    foundation::set_window_material_theme,
    theme::{CornerShape, Direction},
};

struct Preview {
    rail: Entity<NavigationRailState>,
    modal: Entity<DialogState>,
    open_focus: FocusHandle,
    selected: SharedString,
    width: Pixels,
    allow_close: bool,
    custom: bool,
    custom_padding: bool,
    hide_on_collapse: bool,
    commands: usize,
    close_requests: Vec<DialogCloseReason>,
    closed: Vec<DialogCloseReason>,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rail = self.rail.clone();
        let modal = self.modal.clone();
        let request = cx.entity();
        let report = cx.entity();
        let outside = cx.entity();
        let mut mail = NavigationItem::new("mail-destination", "Mail")
            .icon(Icon::new(IconName::Inbox))
            .icon_position(NavigationItemIconPosition::Top)
            .badge(Badge::standalone().accessible_label("unread-badge", "Five unread messages"))
            .colors(NavigationItemColors {
                selected_indicator: Some(rgb(0x93afdc).into()),
                ..Default::default()
            });
        if self.custom_padding {
            mail = mail.indicator_padding(
                NavigationRailPadding {
                    start: px(12.),
                    end: px(6.),
                    top: px(3.),
                    bottom: px(5.),
                },
                NavigationRailPadding {
                    start: px(18.),
                    end: px(8.),
                    top: px(7.),
                    bottom: px(3.),
                },
            );
        }
        let items = NavigationItems::new(
            "rail-test-items",
            "Audit destinations",
            [
                mail,
                NavigationItem::new("disabled-destination", "Unavailable").disabled(true),
                NavigationItem::new("archive-destination", "Archive")
                    .icon(Icon::new(IconName::Archive))
                    .icon_position(NavigationItemIconPosition::Start),
            ],
        )
        .selected(self.selected.clone())
        .on_select(cx.listener(|this, name: &SharedString, _, cx| {
            this.selected = name.clone();
            cx.notify();
        }));
        let mut nav = NavigationRail::new("audit-rail", items)
            .state(&rail)
            .title("Audit rail")
            .expanded_width(self.width)
            .modal(&modal)
            .modal_options(RailModalOptions {
                properties: DialogProperties {
                    dismiss_on_escape: true,
                    dismiss_on_outside_click: false,
                },
                placement: SheetPlacement::Detached,
                margins: SheetMargins {
                    start: px(18.),
                    end: px(12.),
                    top: px(22.),
                    bottom: px(28.),
                },
                hide_on_collapse: self.hide_on_collapse,
                ..Default::default()
            })
            .on_dismiss_request(move |reason, _, cx| {
                request.update(cx, |view, _| {
                    view.close_requests.push(reason);
                    view.allow_close
                })
            })
            .on_modal_close(move |reason, _, cx| {
                report.update(cx, |view, _| view.closed.push(reason))
            })
            .content_padding(NavigationRailPadding {
                start: px(7.),
                end: px(19.),
                top: px(5.),
                bottom: px(8.),
            })
            .expanded_header_top_padding(px(24.))
            .header_spacing(px(10.))
            .destination_spacing(px(8.))
            .arrangement(NavigationRailArrangement::Center)
            .colors(NavigationRailColors {
                container: Some(rgb(0xededf5).into()),
                modal_container: Some(rgb(0x436582).into()),
                scrim: Some(rgb(0x1f3040).into()),
                scrim_opacity: Some(0.35),
                ..Default::default()
            })
            .shapes(NavigationRailShapes {
                collapsed: Some(CornerShape::Radii([2., 4., 6., 8.])),
                expanded: Some(CornerShape::Radii([6., 12., 6., 12.])),
                modal: Some(CornerShape::Radii([14.; 4])),
            })
            .header(
                div()
                    .id("header-body")
                    .test_support()
                    .h(px(24.))
                    .child("Workspace"),
            )
            .divider_after(
                "mail-destination",
                div().id("rail-marker").test_support().h(px(2.)),
            );
        if self.custom {
            nav = nav.destination_content(|_| {
                Button::new("custom-rail-command", "Custom destination").on_activate(|_, _| {})
            });
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("open-audit-rail", "Open audit navigation")
                            .track_focus(&self.open_focus)
                            .on_activate(move |window, cx| {
                                modal.update(cx, |state, cx| state.open(window, cx))
                            }),
                    )
                    .child(Button::new("outside-rail", "Outside command").on_activate(
                        move |_, cx| outside.update(cx, |view, _| view.commands += 1),
                    )),
            )
            .child(div().w_full().h(px(450.)).child(nav))
    }
}

fn open(cx: &mut TestAppContext, reduced: bool) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(reduced);
    });
    let mut preview = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let view = cx.new(|cx| Preview {
            rail: cx.new(|_| {
                NavigationRailState::default().transition_duration(Duration::from_millis(250))
            }),
            modal: cx.new(DialogState::new),
            open_focus: cx.focus_handle(),
            selected: "mail-destination".into(),
            width: px(280.),
            allow_close: false,
            custom: false,
            custom_padding: false,
            hide_on_collapse: false,
            commands: 0,
            close_requests: Vec::new(),
            closed: Vec::new(),
        });
        preview = Some(view.clone());
        gpui_base::Root::new(view, window, cx)
    });
    draw(cx);
    (preview.unwrap(), cx)
}

fn draw(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }
}

fn bounds(cx: &mut VisualTestContext, id: ElementId) -> Bounds<Pixels> {
    cx.update(|window, _| {
        find(window, &[], &id)
            .unwrap_or_else(|| panic!("missing {id:?}"))
            .bounds()
    })
}

fn frame(cx: &mut VisualTestContext) -> Bounds<Pixels> {
    bounds(cx, (ElementId::from("audit-rail"), "rail-frame").into())
}

fn painted(cx: &mut VisualTestContext, pigment: u32) -> gpui::Quad {
    cx.update(|window, _| {
        window
            .painted_quads()
            .into_iter()
            .find(|quad| quad.background == Hsla::from(rgb(pigment)).into())
            .unwrap_or_else(|| panic!("missing paint {pigment:x}"))
    })
}

#[gpui::test]
fn retained_rail_retargets_and_fits_the_resized_viewport(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, false);
    assert_eq!(frame(cx).size.width, px(96.));
    let position = bounds(cx, "mail-destination".into()).center();
    cx.simulate_click(position, Modifiers::default());
    draw(cx);
    cx.simulate_keystrokes("down");
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            find(window, &[], &"archive-destination".into())
                .unwrap()
                .focused(),
            Some(true)
        );
        let rail = view.read(cx).rail.clone();
        rail.update(cx, |rail, cx| {
            rail.expand(cx);
            assert_eq!(rail.current_value(), RailMode::Collapsed);
            assert_eq!(rail.target_value(), RailMode::Expanded);
            assert!(rail.is_animating());
        });
    });
    cx.executor().advance_clock(Duration::from_millis(95));
    draw(cx);
    let midway = frame(cx).size.width;
    assert!(midway > px(96.) && midway < px(280.));
    cx.update(|_, cx| {
        view.read(cx)
            .rail
            .clone()
            .update(cx, |rail, cx| rail.toggle(cx));
    });
    draw(cx);
    assert!((frame(cx).size.width - midway).abs() <= px(1.));
    let window = cx.update(|window, cx| {
        view.read(cx)
            .rail
            .clone()
            .update(cx, |rail, cx| rail.expand(cx));
        window.window_handle()
    });
    cx.simulate_window_resize(window, size(px(184.), px(520.)));
    cx.executor().advance_clock(Duration::from_millis(260));
    draw(cx);
    cx.update(|window, cx| {
        let rail = view.read(cx).rail.read(cx);
        assert_eq!(rail.current_value(), RailMode::Expanded);
        assert_eq!(rail.progress(), 1.);
        assert!(!rail.is_animating());
        assert_eq!(
            find(window, &[], &"archive-destination".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    });
    assert_eq!(frame(cx).size.width, px(184.));
    let rail = cx.update(|_, cx| view.read(cx).rail.clone());
    cx.update(|_, cx| rail.update(cx, |rail, cx| rail.hide(cx)));
    cx.executor().advance_clock(Duration::from_millis(260));
    draw(cx);
    assert_eq!(frame(cx).size.width, px(0.));
    cx.update(|window, cx| {
        assert_eq!(rail.read(cx).current_value(), RailMode::Hidden);
        assert!(find(window, &[], &"mail-destination".into()).is_none());
        rail.update(cx, |rail, cx| rail.snap_to(RailMode::Collapsed, cx));
    });
    draw(cx);
    assert_eq!(frame(cx).size.width, px(96.));
}

#[gpui::test]
fn rail_indicator_padding_keeps_target_badge_and_layout_in_rtl(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, true);
    let before = bounds(cx, "mail-destination".into()).size;
    let original = painted(cx, 0x93afdc);
    let scale = cx.update(|window, _| window.scale_factor());
    assert_eq!(original.bounds.size.width, px(56.).scale(scale));
    cx.update(|window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        view.update(cx, |view, cx| {
            view.custom_padding = true;
            cx.notify();
        });
    });
    draw(cx);
    let custom = painted(cx, 0x93afdc);
    assert!(custom.bounds.size.width < original.bounds.size.width);
    assert_eq!(bounds(cx, "mail-destination".into()).size, before);
    let frame = frame(cx);
    let header = bounds(cx, "header-body".into());
    assert_eq!(header.left() - frame.left(), px(19.));
    assert_eq!(frame.right() - header.right(), px(7.));
    let mail = bounds(cx, "mail-destination".into());
    assert!(mail.top() > header.bottom() + px(10.));
    assert!(bounds(cx, "rail-marker".into()).top() >= mail.bottom());
    cx.update(|window, cx| {
        let mail = find(window, &[], &"mail-destination".into()).unwrap();
        assert_eq!(mail.label(), Some("Mail"));
        assert_eq!(mail.description(), Some("Five unread messages"));
        let rail = view.read(cx).rail.clone();
        rail.update(cx, |rail, cx| rail.snap_to(RailMode::Expanded, cx));
    });
    draw(cx);
    assert!(bounds(cx, "header-body".into()).top() >= header.top() + px(24.));
    assert!(painted(cx, 0x93afdc).bounds.size.width > custom.bounds.size.width);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.custom = true;
            cx.notify();
        });
    });
    draw(cx);
    cx.update(|window, _| {
        assert!(find(window, &[], &"mail-destination".into()).is_none());
        let command = find(window, &[], &"custom-rail-command".into()).unwrap();
        assert_eq!(command.role(), Some(Role::Button));
        assert_eq!(command.label(), Some("Custom destination"));
    });
}

#[gpui::test]
fn rapid_modal_reopen_uses_the_same_session_and_restores_the_entry(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, false);
    cx.update(|_, cx| {
        view.read(cx)
            .rail
            .clone()
            .update(cx, |rail, cx| rail.expand(cx))
    });
    cx.executor().advance_clock(Duration::from_millis(65));
    draw(cx);
    let trigger = bounds(cx, "open-audit-rail".into()).center();
    cx.simulate_click(trigger, Modifiers::default());
    cx.executor().advance_clock(Duration::from_millis(85));
    draw(cx);
    cx.update(|window, cx| {
        let model = view.read(cx);
        assert!(model.modal.read(cx).is_open());
        assert!(model.rail.read(cx).is_animating());
        assert!(!model.open_focus.is_focused(window));
        assert!(gpui_base::active_focus_trap(window, cx).is_some());
    });
    let viewport = cx.update(|window, _| window.viewport_size());
    cx.simulate_click(
        point(viewport.width - px(12.), px(240.)),
        Modifiers::default(),
    );
    cx.simulate_keystrokes("escape");
    draw(cx);
    cx.update(|_, cx| {
        let model = view.read(cx);
        assert!(model.modal.read(cx).is_open());
        assert_eq!(model.close_requests, [DialogCloseReason::Escape]);
        assert!(model.closed.is_empty());
        assert_eq!(model.commands, 0);
        view.update(cx, |view, cx| {
            view.allow_close = true;
            cx.notify();
        });
    });
    cx.simulate_keystrokes("escape");
    draw(cx);
    cx.update(|window, cx| {
        let model = view.read(cx);
        assert_eq!(model.closed, [DialogCloseReason::Escape]);
        assert!(model.open_focus.is_focused(window));
        assert!(!model.modal.read(cx).is_open());
    });
    // Reopen before the modal exit and docked expansion have settled.
    cx.update(|window, cx| {
        let modal = view.read(cx).modal.clone();
        modal.update(cx, |state, cx| state.open(window, cx));
    });
    draw(cx);
    cx.update(|window, cx| {
        assert!(view.read(cx).modal.read(cx).is_open());
        assert_eq!(
            snapshots(window)
                .iter()
                .filter(|node| node.role() == Some(Role::Dialog))
                .count(),
            1
        );
    });
    cx.simulate_keystrokes("escape");
    cx.executor().advance_clock(Duration::from_millis(260));
    draw(cx);
    cx.update(|window, cx| {
        let model = view.read(cx);
        assert_eq!(
            model.closed,
            [DialogCloseReason::Escape, DialogCloseReason::Escape]
        );
        assert_eq!(model.close_requests.len(), 3);
        assert!(model.open_focus.is_focused(window));
        assert!(find(window, &[], &"audit-rail".into()).is_none());
        assert_eq!(model.rail.read(cx).current_value(), RailMode::Expanded);
        let state = model.rail.clone();
        view.update(cx, |view, cx| {
            view.hide_on_collapse = true;
            cx.notify();
        });
        state.update(cx, |state, cx| state.snap_to(RailMode::Collapsed, cx));
    });
    draw(cx);
    assert_eq!(frame(cx).size.width, px(0.));
}
