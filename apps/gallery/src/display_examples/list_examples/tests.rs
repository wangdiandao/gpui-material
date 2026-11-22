use std::time::Duration;

use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, Hsla, InteractiveElement as _,
    IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, LongPressEvent, Modifiers, MouseButton,
    ParentElement as _, Pixels, Point, Render, Role, ScrollDelta, ScrollEvent, ScrollHandle,
    StatefulInteractiveElement as _, Styled as _, TestAppContext, TouchPhase, VisualTestContext,
    Window, div, point, px, rgb,
};
use gpui_base::{
    TestSupportExt,
    test_support::{find, scope},
};
use gpui_material::{
    Button, List, ListItem, ListItemAlignment, ListItemColors, ListItemPadding, ListItemShapes,
    ListItemStateColors, ListVariant, MaterialRoot, MaterialTheme, SwipeToDismissState,
    SwipeToDismissValue, Text,
    foundation::{InteractionSource, InteractionState, set_window_material_theme},
    theme::{CornerShape, Direction},
};

#[derive(Default, Debug, PartialEq)]
struct Requests {
    activate: usize,
    long_press: usize,
    trail: usize,
    checked: Vec<bool>,
    expanded: Vec<bool>,
    dismissed: Vec<SwipeToDismissValue>,
}

struct Preview {
    variant: ListVariant,
    checked: bool,
    selected: bool,
    expanded: bool,
    disabled: bool,
    accept: bool,
    included: bool,
    reversed: bool,
    source: Entity<InteractionSource>,
    swipe: Entity<SwipeToDismissState>,
    scroll: ScrollHandle,
    requests: Requests,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let check = owner.clone();
        let expand = owner.clone();
        let hold = owner.clone();
        let follow = owner.clone();
        let dismiss = owner.clone();
        let row = ListItem::new("primary", "Current report")
            .content(Text::new("Visible title"))
            .overline_content(Text::new("Visible overline"))
            .supporting_content(Text::new("Visible supporting"))
            .leading(div().id("leading").test_support().size(px(15.)))
            .trailing(
                Button::new("independent-trailing", "Independent action").on_activate(
                    move |_, cx| {
                        follow.update(cx, |this, _| this.requests.trail += 1);
                    },
                ),
            )
            .checked(self.checked)
            .selected(self.selected)
            .on_checked_change(move |checked, _, cx| {
                check.update(cx, |this, cx| {
                    this.requests.checked.push(checked);
                    if this.accept {
                        this.checked = checked;
                        cx.notify();
                    }
                })
            })
            .expanded_content(
                self.expanded,
                Button::new("expanded-child", "Nested report action").on_activate(|_, _| {}),
            )
            .on_expanded_change(move |expanded, _, cx| {
                expand.update(cx, |this, cx| {
                    this.requests.expanded.push(expanded);
                    if this.accept {
                        this.expanded = expanded;
                        cx.notify();
                    }
                })
            })
            .on_long_press("Report operations", move |_, cx| {
                hold.update(cx, |this, _| this.requests.long_press += 1);
            })
            .long_press_button("context-command")
            .on_activate(move |_, cx| {
                owner.update(cx, |this, _| this.requests.activate += 1);
            })
            .colors(ListItemColors {
                selected: ListItemStateColors {
                    container: Some(rgb(0x81abc2).into()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .shapes(ListItemShapes {
                resting: Some(CornerShape::Radii([2., 4., 6., 8.])),
                selected: Some(CornerShape::Radii([2., 4., 6., 8.])),
                ..Default::default()
            })
            .vertical_alignment(ListItemAlignment::Start)
            .content_padding(ListItemPadding {
                start: px(22.),
                end: px(10.),
                top: px(4.),
                bottom: px(10.),
            })
            .interaction_source(&self.source)
            .disabled(self.disabled)
            .swipe_to_dismiss(&self.swipe, |swipe| {
                swipe.on_dismiss(move |direction, _, cx| {
                    dismiss.update(cx, |this, _| this.requests.dismissed.push(direction));
                })
            });
        let mut rows = vec![
            ListItem::new("static", "Information").content(Text::new("Visible information")),
            ListItem::new("other-row", "Another row").on_activate(|_, _| {}),
        ];
        if self.included {
            rows.insert(0, row);
        }
        if self.reversed {
            rows.reverse();
        }
        div()
            .id("viewport")
            .test_support()
            .w(px(500.))
            .h(px(310.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(List::new("test-list").variant(self.variant).items(rows))
            .child(div().h(px(380.)))
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (preview, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Preview {
            variant: ListVariant::Standard,
            checked: true,
            selected: false,
            expanded: false,
            disabled: false,
            accept: false,
            included: true,
            reversed: false,
            source: cx.new(|_| InteractionSource::new()),
            swipe: cx.new(|_| SwipeToDismissState::new()),
            scroll: ScrollHandle::new(),
            requests: Requests::default(),
        }
    });
    draw(cx);
    (preview, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

fn bounds(cx: &mut VisualTestContext, id: impl Into<ElementId>) -> Bounds<Pixels> {
    let id = id.into();
    cx.update(|window, _| find(window, &[], &id).unwrap().bounds())
}

fn slot(cx: &mut VisualTestContext, id: impl Into<ElementId>, label: &str) -> Bounds<Pixels> {
    let id = id.into();
    cx.update(|window, _| {
        let row = find(window, &[], &"primary".into()).unwrap();
        let path = scope(window, row.path(), &id);
        let element = find(window, &path, &"material-text".into()).unwrap();
        assert_eq!(element.label(), Some(label));
        element.bounds()
    })
}

fn primary_control() -> ElementId {
    (ElementId::from("primary"), "activate").into()
}

fn snapshot(cx: &mut VisualTestContext, preview: &Entity<Preview>) -> Option<InteractionState> {
    cx.update(|window, cx| {
        preview
            .read(cx)
            .source
            .read(cx)
            .state(window.window_handle().window_id(), &"primary".into())
    })
}

fn key(cx: &mut VisualTestContext, key: &str) {
    let keystroke =
        cx.update(|_, cx| Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
    draw(cx);
}

fn release(cx: &mut VisualTestContext, at: Point<Pixels>) {
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    draw(cx);
}

#[gpui::test]
fn list_slots_keep_grouping_and_controlled_semantics_in_both_directions(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let mut previous_focus = None;
    for (variant, direction, selected) in [
        (ListVariant::Standard, Direction::Ltr, false),
        (ListVariant::Segmented, Direction::Rtl, true),
    ] {
        cx.update(|window, cx| {
            let mut theme = MaterialTheme::default();
            theme.layout.direction = direction;
            set_window_material_theme(Some(theme), window, cx);
            preview.update(cx, |this, cx| {
                this.variant = variant;
                this.selected = selected;
                this.reversed = variant == ListVariant::Segmented;
                cx.notify();
            });
        });
        draw(cx);
        cx.update(|window, cx| {
            let list = find(window, &[], &"test-list".into()).unwrap();
            assert_eq!(list.role(), Some(Role::List));
            let info = find(window, list.path(), &"static".into()).unwrap();
            assert_eq!(info.role(), Some(Role::ListItem));
            assert_eq!(info.label(), Some("Information"));
            let item = find(window, list.path(), &"primary".into()).unwrap();
            assert_eq!(item.role(), Some(Role::ListItem));
            assert_eq!(item.selected(), Some(selected));
            let action = find(window, item.path(), &primary_control()).unwrap();
            assert_eq!(action.role(), Some(Role::CheckBox));
            assert_eq!(action.label(), Some("Current report"));
            assert_eq!(action.checked(), Some(true));
            assert_eq!(action.expanded(), Some(false));
            assert_eq!(action.description(), Some("Report operations"));
            assert!(find(window, &[], &"expanded-child".into()).is_none());
            let quad = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == Hsla::from(rgb(0x81abc2)).into())
                .unwrap();
            let scale = window.scale_factor();
            let is_rtl = direction == Direction::Rtl;
            assert_eq!(
                quad.corner_radii.top_left,
                px(if is_rtl { 4. } else { 2. }).scale(scale)
            );
            assert_eq!(
                quad.corner_radii.top_right,
                px(if is_rtl { 2. } else { 4. }).scale(scale)
            );
            if let Some(expected) = previous_focus.clone() {
                assert_eq!(window.focused(cx), Some(expected));
            } else {
                window.focus_next(cx);
                previous_focus = window.focused(cx);
            }
        });
        draw(cx);
        assert!(
            cx.update(|window, _| find(window, &[], &primary_control()).unwrap().focused())
                .unwrap()
        );
        let leading = bounds(cx, "leading");
        let overline = slot(
            cx,
            (ElementId::from("overline"), "slot"),
            "Visible overline",
        );
        let headline = slot(cx, "headline-slot", "Visible title");
        let supporting = slot(
            cx,
            (ElementId::from("supporting"), "slot"),
            "Visible supporting",
        );
        let trailing = bounds(cx, "independent-trailing");
        assert!(overline.top() < headline.top() && headline.top() < supporting.top());
        assert_eq!(leading.top(), bounds(cx, "primary").top() + px(6.));
        match direction {
            Direction::Ltr => {
                assert_eq!(leading.left(), px(24.));
                assert!(leading.right() < headline.left() && headline.right() < trailing.left());
            }
            Direction::Rtl => {
                assert_eq!(leading.right(), px(476.));
                assert!(trailing.right() < headline.left() && headline.right() < leading.left());
            }
        }
    }

    key(cx, "enter");
    cx.update(|window, cx| {
        let action = find(window, &[], &primary_control()).unwrap();
        assert_eq!(action.checked(), Some(true));
        assert_eq!(action.expanded(), Some(false));
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.activate, 1);
        assert_eq!(requests.checked, [false]);
        assert_eq!(requests.expanded, [true]);
        preview.update(cx, |this, cx| {
            this.accept = true;
            cx.notify();
        });
    });
    key(cx, "space");
    cx.update(|window, _| {
        let action = find(window, &[], &primary_control()).unwrap();
        assert_eq!(action.checked(), Some(false));
        assert_eq!(action.expanded(), Some(true));
        let child = find(window, &[], &"expanded-child".into()).unwrap();
        assert!(child.path().contains(&"primary".into()));
    });

    let nested = bounds(cx, "expanded-child").center();
    cx.simulate_click(nested, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            find(window, &[], &"expanded-child".into())
                .unwrap()
                .focused(),
            Some(true)
        );
        preview.update(cx, |this, cx| {
            this.expanded = false;
            cx.notify();
        });
    });
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            find(window, &[], &primary_control()).unwrap().focused(),
            Some(true)
        );
        preview.update(cx, |this, cx| {
            this.expanded = true;
            cx.notify();
        });
    });
    draw(cx);

    let trailing = bounds(cx, "independent-trailing").center();
    cx.simulate_click(trailing, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.trail, 1);
        assert_eq!(requests.activate, 2);
        let control = find(window, &[], &"independent-trailing".into()).unwrap();
        assert_eq!(control.label(), Some("Independent action"));
        assert_eq!(control.focused(), Some(true));
    });
    key(cx, "down");
    assert_eq!(
        cx.update(
            |window, _| find(window, &[], &"independent-trailing".into())
                .unwrap()
                .focused()
        ),
        Some(true),
    );
    let primary = slot(cx, "headline-slot", "Visible title").center();
    cx.simulate_click(primary, Modifiers::default());
    draw(cx);
    assert!(cx.update(|window, _| find(window, &[], &"expanded-child".into()).is_none()));
    key(cx, "down");
    assert_eq!(
        cx.update(|window, _| find(
            window,
            &[],
            &(ElementId::from("other-row"), "activate").into()
        )
        .unwrap()
        .focused()),
        Some(true),
    );
    cx.update(|_, cx| {
        preview.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
    });
    draw(cx);
    key(cx, "down");
    assert_eq!(
        cx.update(|window, _| find(window, &[], &primary_control()).unwrap().disabled()),
        Some(true)
    );
    assert!(snapshot(cx, &preview).unwrap().disabled);
}

#[gpui::test]
fn list_hold_keeps_separate_commands_and_yields_to_swipe_or_scroll(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let center = slot(cx, "headline-slot", "Visible title").center();
    let trailing = bounds(cx, "independent-trailing").center();
    let tick_hold = |cx: &mut VisualTestContext| {
        cx.executor().advance_clock(Duration::from_millis(600));
        draw(cx);
    };

    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
    draw(cx);
    let state = snapshot(cx, &preview).unwrap();
    assert!(state.hovered && state.pressed);
    tick_hold(cx);
    release(cx, center);
    cx.update(|_, cx| {
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.long_press, 1);
        assert_eq!(requests.activate, 0);
    });
    assert!(!snapshot(cx, &preview).unwrap().pressed);
    key(cx, "shift-f10");
    cx.update(|_, cx| assert_eq!(preview.read(cx).requests.long_press, 2));

    // An early release inside is an ordinary click.
    cx.simulate_click(center, Modifiers::default());
    tick_hold(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).requests.activate, 1));
    // Releasing outside and moving far enough to pan cancel the pending hold.
    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
    release(cx, center + point(px(0.), px(105.)));
    tick_hold(cx);
    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
    let y_move = center + point(px(2.), px(30.));
    cx.simulate_mouse_move(y_move, MouseButton::Left, Modifiers::default());
    tick_hold(cx);
    release(cx, y_move);
    cx.update(|_, cx| {
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.activate, 1);
        assert_eq!(requests.long_press, 2);
        assert!(requests.dismissed.is_empty());
    });

    cx.simulate_mouse_down(trailing, MouseButton::Left, Modifiers::default());
    tick_hold(cx);
    release(cx, trailing);
    cx.update(|_, cx| {
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.trail, 1);
        assert_eq!(requests.long_press, 2);
    });
    let alternate = bounds(cx, "context-command").center();
    cx.simulate_click(alternate, Modifiers::default());
    draw(cx);
    let down = LongPressEvent {
        phase: TouchPhase::Started,
        start_position: center,
        position: center,
    };
    cx.simulate_event(down.clone());
    cx.simulate_event(LongPressEvent {
        phase: TouchPhase::Ended,
        ..down
    });
    draw(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).requests.long_press, 4));

    cx.simulate_event(ScrollEvent {
        position: center,
        delta: ScrollDelta::Pixels(point(px(0.), px(-45.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
        source: gpui::ScrollSource::Pointer,
    });
    draw(cx);
    cx.update(|_, cx| {
        let scroll = &preview.read(cx).scroll;
        assert!(scroll.offset().y < px(0.));
        scroll.set_offset(point(px(0.), px(0.)));
    });
    draw(cx);

    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
    let x_move = center + point(px(80.), px(0.));
    cx.simulate_mouse_move(x_move, MouseButton::Left, Modifiers::default());
    tick_hold(cx);
    let state = snapshot(cx, &preview).unwrap();
    assert!(state.dragged && !state.pressed);
    release(cx, x_move);
    cx.update(|window, cx| {
        let requests = &preview.read(cx).requests;
        assert_eq!(requests.activate, 1);
        assert_eq!(requests.long_press, 4);
        assert_eq!(requests.dismissed, [SwipeToDismissValue::StartToEnd]);
        assert!(find(window, &[], &primary_control()).is_none());
        window.focus_next(cx);
    });
    assert_ne!(
        snapshot(cx, &preview).map(|state| state.dragged),
        Some(true)
    );

    // Reset restores the original row and interaction tracking. Removing it
    // again cancels a pending hold and releases the source entry.
    cx.update(|window, cx| {
        let state = preview.read(cx).swipe.clone();
        state.update(cx, |state, cx| state.reset(window, cx));
    });
    draw(cx);
    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
    cx.update(|_, cx| {
        preview.update(cx, |this, cx| {
            this.included = false;
            cx.notify();
        });
    });
    draw(cx);
    tick_hold(cx);
    release(cx, center);
    assert_eq!(snapshot(cx, &preview), None);
    cx.update(|_, cx| assert_eq!(preview.read(cx).requests.long_press, 4));
}

#[gpui::test]
fn list_roves_skips_disabled_and_wraps_supporting_text(cx: &mut TestAppContext) {
    use gpui::WindowOptions;

    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(220.)).child(List::new("audit-list").items([
                ListItem::new("first", "First").on_activate(|_, _| {}),
                ListItem::new("selected", "Selected")
                    .selected(true)
                    .on_activate(|_, _| {}),
                ListItem::new("disabled", "Disabled")
                    .disabled(true)
                    .on_activate(|_, _| {}),
                ListItem::new("last", "Last")
                    .supporting_text(
                        "中文 supporting text that wraps across several lines and remains readable",
                    )
                    .on_activate(|_, _| {}),
            ]))
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Example))
            .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        for _ in 0..3 {
            window.refresh();
            window.draw(cx).clear(cx);
        }
        window.focus_next(cx);
    })
    .unwrap();
    let focused = |cx: &mut TestAppContext, id: &str| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(
                window,
                &[],
                &(ElementId::from(id.to_owned()), "activate").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap()
    };
    assert_eq!(focused(cx, "selected"), Some(true));
    cx.simulate_keystrokes(handle.into(), "down");
    assert_eq!(focused(cx, "last"), Some(true));
    cx.simulate_keystrokes(handle.into(), "down");
    assert_eq!(focused(cx, "first"), Some(true));
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let row = find(window, &[], &"last".into()).unwrap();
        let supporting = find(window, row.path(), &"supporting".into()).unwrap();
        assert!(
            supporting.bounds().size.height >= px(40.),
            "supporting text must wrap rather than truncate"
        );
        assert!(row.bounds().size.height >= px(88.));
        assert!(supporting.bounds().right() <= row.bounds().right());
    })
    .unwrap();
}

#[gpui::test]
fn list_item_builder_tone_right_click_accessible_label_and_preserved_track_focus(
    cx: &mut TestAppContext,
) {
    use gpui::{FocusHandle, InputEvent as _, WindowOptions, size};
    use gpui_material::SemanticTone;

    struct Example {
        focus: FocusHandle,
        long_presses: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity();
            div().w(px(320.)).child(
                List::new("custom-list")
                    .accessible_label("Inbox items")
                    .item(
                        ListItem::new("item-tone", "Completed")
                            .tone(SemanticTone::Success)
                            .track_focus(&self.focus, false)
                            .on_long_press("Item options", move |_, cx| {
                                owner.update(cx, |this, cx| {
                                    this.long_presses += 1;
                                    cx.notify();
                                });
                            }),
                    )
                    .item(ListItem::new("item-next", "Next item").on_activate(|_, _| {})),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|cx| Example {
                focus: cx.focus_handle(),
                long_presses: 0,
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    cx.simulate_window_resize(handle.into(), size(px(400.), px(300.)));

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        let list = find(window, &[], &"custom-list".into()).unwrap();
        assert_eq!(list.role(), Some(Role::List));
        assert_eq!(list.label(), Some("Inbox items"));

        // Focusing caller-supplied handle targets item-tone activate node, and Down moves to item-next.
        view.read(cx).focus.clone().focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        let first_act = find(
            window,
            &[],
            &(ElementId::from("item-tone"), "activate").into(),
        )
        .unwrap();
        assert_eq!(first_act.focused(), Some(true));

        let center = first_act.bounds().center();
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: MouseButton::Right,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(view.read(cx).long_presses, 1);

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
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(
                window,
                &[],
                &(ElementId::from("item-next"), "activate").into()
            )
            .unwrap()
            .focused(),
            Some(true)
        );
    })
    .unwrap();
}
