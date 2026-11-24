use super::*;
use gpui::{
    AnyWindowHandle, AppContext, ElementId, Entity, InputEvent, InteractiveElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Role,
    TestAppContext, WindowOptions, rgb, size,
};
use gpui_base::{
    TestSupportExt as _,
    test_support::{ElementSnapshot, find, snapshots},
};
use gpui_material::{
    Button, ButtonColors, ButtonGroupAlignment, ButtonGroupConnectedShapes, ButtonGroupItemStyle,
    ButtonGroupSelection, ButtonGroupVariant, ButtonGroupVerticalAlignment, ButtonShapes,
    InteractionSource, MaterialRoot, MaterialTheme,
    foundation::set_window_material_theme,
    theme::{CornerShape, Direction},
};

const COMMAND_TINT: u32 = 0x554377;
const DETAIL_TINT: u32 = 0x7f3157;
const DETAIL_CHECKED: u32 = 0x235a4c;

struct Preview {
    width: Pixels,
    mode: ButtonGroupSelection,
    variant: ButtonGroupVariant,
    selected: Vec<SharedString>,
    calls: usize,
    changes: usize,
    pinned: bool,
    source: Entity<InteractionSource>,
    open: bool,
    accept_open: bool,
    requests: Vec<bool>,
    reordered: bool,
    outside: FocusHandle,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let command = cx.weak_entity();
        let toggle = cx.weak_entity();
        let selection = cx.weak_entity();
        let overflow = cx.weak_entity();
        let mut items = vec![
            ButtonGroupItem::new("send", "Send")
                .leading_icon(|| Icon::new(IconName::Description))
                .action(move |_, cx| {
                    let _ = command.update(cx, |owner, cx| {
                        owner.calls += 1;
                        cx.notify();
                    });
                })
                .compression_limit(px(4.))
                .style(ButtonGroupItemStyle {
                    colors: ButtonColors {
                        container: Some(rgb(COMMAND_TINT).into()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ButtonGroupItem::new("details", "Details")
                .weight(1.)
                .interaction_source(&self.source)
                .vertical_alignment(ButtonGroupVerticalAlignment::Bottom)
                .style(ButtonGroupItemStyle {
                    colors: ButtonColors {
                        container: Some(rgb(DETAIL_TINT).into()),
                        checked_container: Some(rgb(DETAIL_CHECKED).into()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ButtonGroupItem::new("unavailable", "Unavailable")
                .disabled(true)
                .compression_limit(px(0.)),
            ButtonGroupItem::new("pin", "Pin results")
                .icon_only(|| Icon::new(IconName::Favorite))
                .toggle(self.pinned, move |value, _, cx| {
                    let _ = toggle.update(cx, |owner, cx| {
                        owner.pinned = value;
                        cx.notify();
                    });
                }),
            ButtonGroupItem::new("advanced", "Advanced")
                .weight(2.)
                .content(px(50.), || div().child("Advanced"))
                .overflow_item_content(|| div().child("Advanced filters")),
        ];
        if self.reordered {
            items.swap(1, 4);
        }
        div()
            .w(self.width)
            .child(Button::new("outside-group", "Outside").track_focus(&self.outside))
            .child(
                ButtonGroup::new("tools", "Tools", items)
                    .variant(self.variant)
                    .shape(ButtonShape::Square)
                    .spacing(px(4.))
                    .alignment(ButtonGroupAlignment::End)
                    .vertical_alignment(ButtonGroupVerticalAlignment::Center)
                    .connected_shapes(ButtonGroupConnectedShapes {
                        middle: ButtonShapes {
                            resting: Some(CornerShape::Radii([2.; 4])),
                            pressed: Some(CornerShape::Radii([8.; 4])),
                            checked: Some(CornerShape::Radii([11.; 4])),
                        },
                        ..Default::default()
                    })
                    .expanded_ratio(0.4)
                    .selection(self.mode, self.selected.clone())
                    .on_change(move |value, _, cx| {
                        let _ = selection.update(cx, |owner, cx| {
                            owner.changes += 1;
                            owner.selected = value;
                            cx.notify();
                        });
                    })
                    .overflow_label("More tools")
                    .overflow_trigger(px(58.), |open| {
                        div().child(if open { "Close" } else { "More" })
                    })
                    .overflow_content(|| {
                        div()
                            .id("menu-decoration")
                            .test_support()
                            .child("More tools")
                    })
                    .overflow_open(self.open)
                    .on_overflow_open_change(move |open, _, cx| {
                        let _ = overflow.update(cx, |owner, cx| {
                            owner.requests.push(open);
                            if owner.accept_open {
                                owner.open = open;
                            }
                            cx.notify();
                        });
                    }),
            )
    }
}

type Handle = AnyWindowHandle;

fn open(
    cx: &mut TestAppContext,
    mode: ButtonGroupSelection,
    variant: ButtonGroupVariant,
) -> (Handle, Entity<Preview>) {
    let pair = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|cx| Preview {
            width: px(520.),
            mode,
            variant,
            selected: vec!["details".into()],
            calls: 0,
            changes: 0,
            pinned: false,
            source: cx.new(|_| InteractionSource::new()),
            open: false,
            accept_open: true,
            requests: Vec::new(),
            reordered: false,
            outside: cx.focus_handle(),
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(pair.0, size(px(720.), px(560.)));
    cx.update_window(pair.0, |_, window, cx| {
        window.activate_window();
        pair.1.read(cx).outside.clone().focus(window, cx);
    })
    .unwrap();
    draw(cx, pair.0);
    pair
}

fn draw(cx: &mut TestAppContext, handle: Handle) {
    for _ in 0..3 {
        cx.update_window(handle, |_, window, cx| {
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn member_id(id: &'static str) -> ElementId {
    if id == "overflow" {
        (ElementId::from("tools"), "overflow").into()
    } else {
        (ElementId::from("tools"), SharedString::from(id)).into()
    }
}

fn member(window: &Window, id: &'static str) -> ElementSnapshot {
    find(window, &[], &member_id(id)).unwrap_or_else(|| panic!("entering member {id}"))
}

fn menu_row(window: &Window, label: &str) -> Option<ElementSnapshot> {
    snapshots(window).into_iter().find(|row| {
        row.label() == Some(label)
            && matches!(row.role(), Some(Role::MenuItem | Role::MenuItemCheckBox))
    })
}

fn key(cx: &mut TestAppContext, handle: Handle, name: &str) {
    cx.update_window(handle, |_, window, cx| {
        let keystroke = Keystroke::parse_with_os(name, cx.platform_info().os).unwrap();
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
    })
    .unwrap();
    draw(cx, handle);
}

fn pointer(cx: &mut TestAppContext, handle: Handle, point: Point<Pixels>, down: bool) {
    cx.update_window(handle, |_, window, cx| {
        if down {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: point,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        } else {
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: point,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        }
    })
    .unwrap();
    draw(cx, handle);
}

fn click_at(cx: &mut TestAppContext, handle: Handle, at: Point<Pixels>) {
    pointer(cx, handle, at, true);
    pointer(cx, handle, at, false);
}

fn click_member(cx: &mut TestAppContext, handle: Handle, id: &'static str) {
    let at = cx
        .update_window(handle, |_, window, _| member(window, id).bounds().center())
        .unwrap();
    click_at(cx, handle, at);
}

#[gpui::test]
fn button_group_mixes_commands_toggles_and_selection_with_stable_press_feedback(
    cx: &mut TestAppContext,
) {
    let (handle, view) = open(
        cx,
        ButtonGroupSelection::MultipleRequired,
        ButtonGroupVariant::Connected,
    );
    let (at, send_width, details_width) = cx
        .update_window(handle, |_, window, cx| {
            let send = member(window, "send");
            let details = member(window, "details");
            let unavailable = member(window, "unavailable");
            assert_eq!(send.role(), Some(Role::Button));
            assert_eq!(send.checked(), None);
            assert_eq!(details.checked(), Some(true));
            assert_eq!(member(window, "pin").checked(), Some(false));
            assert_eq!(unavailable.disabled(), Some(true));
            assert!(member(window, "advanced").bounds().size.width > details.bounds().size.width);
            let detail_quad = crate::painted_quads(window)
                .into_iter()
                .find(|quad| quad.background == rgb(DETAIL_CHECKED).into())
                .unwrap();
            assert_eq!(
                detail_quad.corner_radii.top_left,
                px(11.).scale(window.scale_factor())
            );
            window.focus_next(cx);
            (
                details.bounds().center(),
                send.bounds().size.width,
                details.bounds().size.width,
            )
        })
        .unwrap();
    draw(cx, handle);
    assert!(
        cx.update_window(handle, |_, window, _| member(window, "details").focused()
            == Some(true))
            .unwrap()
    );
    pointer(cx, handle, at, true);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(member(window, "send").bounds().size.width, send_width);
        assert_eq!(member(window, "details").bounds().size.width, details_width);
        let detail_quad = crate::painted_quads(window)
            .into_iter()
            .find(|quad| quad.background == rgb(DETAIL_CHECKED).into())
            .unwrap();
        assert_eq!(
            detail_quad.corner_radii.top_left,
            px(8.).scale(window.scale_factor())
        );
        assert!(
            view.read(cx)
                .source
                .read(cx)
                .state(window.window_handle().window_id(), &member_id("details"))
                .unwrap()
                .pressed
        );
    })
    .unwrap();
    pointer(cx, handle, at, false);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(member(window, "details").bounds().size.width, details_width);
        assert_eq!(view.read(cx).changes, 0);
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    draw(cx, handle);
    key(cx, handle, "left");
    cx.update_window(handle, |_, window, _| {
        assert!(member(window, "details").bounds().left() > member(window, "pin").bounds().left());
        assert_eq!(member(window, "pin").focused(), Some(true));
    })
    .unwrap();
    key(cx, handle, "space");
    click_member(cx, handle, "send");
    click_member(cx, handle, "advanced");
    cx.update(|cx| {
        let owner = view.read(cx);
        assert!(owner.pinned);
        assert_eq!(owner.calls, 1);
        assert_eq!(
            owner.selected,
            [SharedString::from("details"), "advanced".into()]
        );
    });
    click_member(cx, handle, "details");
    cx.update(|cx| {
        view.update(cx, |owner, cx| {
            owner.reordered = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(member(window, "details").focused(), Some(true));
        assert!(
            view.read(cx)
                .source
                .read(cx)
                .state(window.window_handle().window_id(), &member_id("details"))
                .is_some_and(|state| !state.pressed)
        );
        assert_eq!(view.read(cx).selected, [SharedString::from("advanced")]);
    })
    .unwrap();
}

#[gpui::test]
fn button_group_overflow_reuses_activation_and_closes_once_when_resized(cx: &mut TestAppContext) {
    let (handle, view) = open(
        cx,
        ButtonGroupSelection::Single,
        ButtonGroupVariant::Standard,
    );
    click_member(cx, handle, "advanced");
    cx.update(|cx| {
        view.update(cx, |owner, cx| {
            owner.width = px(165.);
            owner.accept_open = false;
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update_window(handle, |_, window, _| {
        let trigger = member(window, "overflow");
        assert_eq!(trigger.role(), Some(Role::Button));
        assert_eq!(trigger.label(), Some("More tools"));
        assert_eq!(trigger.focused(), Some(true));
    })
    .unwrap();
    key(cx, handle, "enter");
    cx.update_window(handle, |_, window, cx| {
        assert!(menu_row(window, "Advanced").is_none());
        assert_eq!(view.read(cx).requests, [true]);
        view.update(cx, |owner, cx| {
            owner.accept_open = true;
            cx.notify();
        });
    })
    .unwrap();
    key(cx, handle, "enter");
    let pin = cx
        .update_window(handle, |_, window, cx| {
            let advanced = menu_row(window, "Advanced").unwrap();
            assert_eq!(advanced.role(), Some(Role::MenuItemCheckBox));
            assert_eq!(advanced.checked(), Some(true));
            assert_eq!(
                menu_row(window, "Unavailable").unwrap().disabled(),
                Some(true)
            );
            assert!(find(window, &[], &"menu-decoration".into()).is_some());
            assert_eq!(view.read(cx).requests, [true, true]);
            menu_row(window, "Pin results").unwrap().bounds().center()
        })
        .unwrap();
    click_at(cx, handle, pin);
    cx.update(|cx| {
        let owner = view.read(cx);
        assert!(owner.pinned && !owner.open);
        assert_eq!(owner.requests, [true, true, false]);
        assert_eq!(owner.selected, [SharedString::from("advanced")]);
    });
    click_member(cx, handle, "overflow");
    key(cx, handle, "escape");
    cx.update(|cx| assert_eq!(view.read(cx).requests, [true, true, false, true, false]));
    click_member(cx, handle, "overflow");
    cx.update(|cx| {
        view.update(cx, |owner, cx| {
            owner.width = px(520.);
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert!(find(window, &[], &member_id("overflow")).is_none());
        assert!(menu_row(window, "Advanced").is_none());
        assert_eq!(member(window, "advanced").checked(), Some(true));
        assert_eq!(member(window, "pin").checked(), Some(true));
        assert!(
            ["send", "details", "pin", "advanced"]
                .into_iter()
                .any(|id| member(window, id).focused() == Some(true))
        );
        assert_eq!(
            view.read(cx).requests,
            [true, true, false, true, false, true, false]
        );
    })
    .unwrap();
    click_member(cx, handle, "advanced");
    cx.update(|cx| assert!(view.read(cx).selected.is_empty()));
}

#[gpui::test]
fn connected_single_select_and_optional_select_shortcuts_update_selection(cx: &mut TestAppContext) {
    struct ShortcutsView {
        required: SharedString,
        required_events: Vec<SharedString>,
        optional: Option<SharedString>,
        optional_events: Vec<Option<SharedString>>,
    }

    impl Render for ShortcutsView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
            let req_view = cx.weak_entity();
            let opt_view = cx.weak_entity();
            div()
                .w(px(520.))
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    ButtonGroup::connected(
                        "req-group",
                        "Required period",
                        [
                            ButtonGroupItem::new("day", "Day"),
                            ButtonGroupItem::new("week", "Week"),
                            ButtonGroupItem::new("month", "Month"),
                        ],
                    )
                    .single_select(self.required.clone(), move |id, _, cx| {
                        let _ = req_view.update(cx, |this, cx| {
                            this.required_events.push(id.clone());
                            this.required = id;
                            cx.notify();
                        });
                    }),
                )
                .child(
                    ButtonGroup::connected(
                        "opt-group",
                        "Optional filter",
                        [
                            ButtonGroupItem::new("open", "Open"),
                            ButtonGroupItem::new("closed", "Closed"),
                        ],
                    )
                    .single_optional_select(
                        self.optional.clone(),
                        move |id, _, cx| {
                            let _ = opt_view.update(cx, |this, cx| {
                                this.optional_events.push(id.clone());
                                this.optional = id;
                                cx.notify();
                            });
                        },
                    ),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| ShortcutsView {
            required: "day".into(),
            required_events: Vec::new(),
            optional: Some("open".into()),
            optional_events: Vec::new(),
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(handle, size(px(640.), px(480.)));
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    draw(cx, handle);

    let item_snapshot = |window: &Window, group: &'static str, item: &'static str| {
        find(
            window,
            &[],
            &ElementId::from((ElementId::from(group), SharedString::from(item))),
        )
        .unwrap_or_else(|| panic!("missing {group}/{item}"))
    };

    // Connected spacing is 2 px between consecutive segments.
    let (day_pt, week_pt, open_pt, closed_pt) = cx
        .update_window(handle, |_, window, _| {
            let day = item_snapshot(window, "req-group", "day");
            let week = item_snapshot(window, "req-group", "week");
            let open = item_snapshot(window, "opt-group", "open");
            let closed = item_snapshot(window, "opt-group", "closed");
            assert_eq!(day.checked(), Some(true));
            assert_eq!(week.checked(), Some(false));
            assert_eq!(open.checked(), Some(true));
            assert_eq!(closed.checked(), Some(false));
            assert_eq!(
                (week.bounds().origin.x - (day.bounds().origin.x + day.bounds().size.width))
                    .round(),
                px(2.)
            );
            (
                day.bounds().center(),
                week.bounds().center(),
                open.bounds().center(),
                closed.bounds().center(),
            )
        })
        .unwrap();

    // Clicking already-selected item in single_select does NOT deselect or fire callback.
    click_at(cx, handle, day_pt);
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.required, "day");
        assert!(state.required_events.is_empty());
    });

    // Clicking another item in single_select delivers the SharedString directly.
    click_at(cx, handle, week_pt);
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.required, "week");
        assert_eq!(state.required_events, vec![SharedString::from("week")]);
    });

    // Clicking already-selected item in single_optional_select clears selection (delivers None).
    click_at(cx, handle, open_pt);
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.optional, None);
        assert_eq!(state.optional_events, vec![None]);
    });

    // Clicking another item in single_optional_select selects it (delivers Some(id)).
    click_at(cx, handle, closed_pt);
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.optional, Some(SharedString::from("closed")));
        assert_eq!(
            state.optional_events,
            vec![None, Some(SharedString::from("closed"))]
        );
    });
}

#[gpui::test]
fn button_group_and_item_tone_applies_to_visible_and_overflow_items(cx: &mut TestAppContext) {
    use gpui_material::{
        ButtonVariant, SemanticTone,
        foundation::{color, window_material_theme},
        theme::ColorRole,
    };

    struct TonedGroup {
        width: Pixels,
    }

    impl Render for TonedGroup {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
            div().w(self.width).child(
                ButtonGroup::new(
                    "toned-group",
                    "Toned actions",
                    [
                        ButtonGroupItem::new("warn", "Warn").action(|_, _| {}),
                        ButtonGroupItem::new("delete", "Delete")
                            .tone(SemanticTone::Danger)
                            .action(|_, _| {}),
                    ],
                )
                .appearance(ButtonVariant::Filled)
                .tone(SemanticTone::Warning)
                .overflow_label("More toned"),
            )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| TonedGroup { width: px(320.) });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(handle, size(px(360.), px(260.)));
    draw(cx, handle);

    cx.update_window(handle, |_, window, cx| {
        let theme = window_material_theme(window, cx);
        let scale = window.scale_factor();
        let warn_bg = color(theme.colors.color(ColorRole::Warning));
        let danger_bg = color(theme.colors.color(ColorRole::Error));

        let warn_bounds = find(
            window,
            &[],
            &ElementId::from((ElementId::from("toned-group"), SharedString::from("warn"))),
        )
        .unwrap()
        .bounds()
        .scale(scale);
        let delete_bounds = find(
            window,
            &[],
            &ElementId::from((ElementId::from("toned-group"), SharedString::from("delete"))),
        )
        .unwrap()
        .bounds()
        .scale(scale);
        let quads = crate::painted_quads(window);
        assert!(quads.iter().any(|quad| {
            warn_bounds.contains(&quad.bounds.center())
                && quad.background.as_solid() == Some(warn_bg)
        }));
        assert!(quads.iter().any(|quad| {
            delete_bounds.contains(&quad.bounds.center())
                && quad.background.as_solid() == Some(danger_bg)
        }));
    })
    .unwrap();

    // Narrow to collapse into overflow and open overflow menu cleanly.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.width = px(110.);
            cx.notify();
        });
    });
    draw(cx, handle);
    let overflow_pt = cx
        .update_window(handle, |_, window, _| {
            find(
                window,
                &[],
                &ElementId::from((
                    ElementId::from("toned-group"),
                    SharedString::from("overflow"),
                )),
            )
            .unwrap()
            .bounds()
            .center()
        })
        .unwrap();
    click_at(cx, handle, overflow_pt);
    cx.update_window(handle, |_, window, _| {
        assert!(menu_row(window, "Warn").is_some());
        assert!(menu_row(window, "Delete").is_some());
    })
    .unwrap();
}

#[gpui::test]
fn standard_button_group_borrows_neighbor_width_on_press(cx: &mut TestAppContext) {
    let (handle, _) = open(
        cx,
        ButtonGroupSelection::MultipleRequired,
        ButtonGroupVariant::Standard,
    );
    let (at, send_width, details_width) = cx
        .update_window(handle, |_, window, _| {
            let send = member(window, "send");
            let details = member(window, "details");
            (
                details.bounds().center(),
                send.bounds().size.width,
                details.bounds().size.width,
            )
        })
        .unwrap();
    pointer(cx, handle, at, true);
    cx.update_window(handle, |_, window, _| {
        assert_eq!(
            member(window, "send").bounds().size.width,
            send_width - px(4.)
        );
        assert_eq!(
            member(window, "details").bounds().size.width,
            details_width + px(4.)
        );
    })
    .unwrap();
    pointer(cx, handle, at, false);
    cx.update_window(handle, |_, window, _| {
        assert_eq!(member(window, "send").bounds().size.width, send_width);
        assert_eq!(member(window, "details").bounds().size.width, details_width);
    })
    .unwrap();
}
