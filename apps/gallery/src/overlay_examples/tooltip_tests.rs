use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    Anchor, AnyWindowHandle, AppContext, Bounds, Context, Entity, FocusHandle, InputEvent,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, Role, Styled, TestAppContext, Window, WindowOptions, div,
    point, px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Button, MaterialRoot, Tooltip, TooltipCaret, TooltipColors, TooltipPlacement,
    TooltipPositioner, TooltipState,
    foundation::{set_window_material_theme, window_material_theme},
    theme::{CornerShape, Direction, Elevation},
};

use super::MenuTooltipExamples;
use crate::story::Story;

fn frame(handle: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..3 {
        cx.update_window(handle, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn element(
    handle: AnyWindowHandle,
    id: &'static str,
    cx: &mut TestAppContext,
) -> Option<gpui_base::test_support::ElementSnapshot> {
    cx.update_window(handle, |_, window, _| find(window, &[], &id.into()))
        .unwrap()
}

fn click(handle: AnyWindowHandle, id: &'static str, cx: &mut TestAppContext) {
    let position = element(handle, id, cx).unwrap().bounds().center();
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
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
                position,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);
}

fn activate_focused(handle: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(handle, |_, window, cx| {
        let key = gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
        window.dispatch_event(
            gpui::KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(gpui::KeyUpEvent { keystroke: key }.to_platform_input(), cx);
    })
    .unwrap();
    frame(handle, cx);
}

fn move_pointer(handle: AnyWindowHandle, position: Point<Pixels>, cx: &mut TestAppContext) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);
}

#[gpui::test]
fn transient_tooltip_keeps_hover_delay_and_exit_grace(cx: &mut TestAppContext) {
    let handle: AnyWindowHandle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| MenuTooltipExamples::new(Story::Tooltip, cx))
        })
        .unwrap()
        .into()
    });
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    frame(handle, cx);
    let center = element(handle, "pin", cx).unwrap().bounds().center();
    move_pointer(handle, center, cx);
    cx.executor().advance_clock(Duration::from_millis(490));
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_none());
    cx.executor().advance_clock(Duration::from_millis(20));
    frame(handle, cx);
    assert_eq!(
        element(handle, "tooltip", cx).unwrap().label(),
        Some("Keep this roadmap at the top of the project list")
    );

    move_pointer(handle, point(px(1.), px(1.)), cx);
    assert!(element(handle, "tooltip", cx).is_some());
    cx.executor().advance_clock(Duration::from_millis(210));
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_none());
}

struct PersistentFixture {
    state: Entity<TooltipState>,
    trigger: FocusHandle,
    action: FocusHandle,
    automatic: bool,
    auto_focus: bool,
    accept_dismiss: bool,
    dismissals: usize,
    activations: usize,
    link_activations: usize,
    outside_activations: usize,
}

impl Render for PersistentFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dismiss = cx.entity();
        let action = cx.entity();
        let link = cx.entity();
        let outside = cx.entity();
        let underneath = outside.clone();
        div()
            .relative()
            .flex()
            .items_start()
            .gap_20()
            .p_4()
            .child(div().absolute().left(px(110.)).top(px(104.)).child(
                Button::new("tooltip-underlay", "Underlay").on_activate(move |_, cx| {
                    underneath.update(cx, |model, _| model.outside_activations += 1);
                }),
            ))
            .child(
                Tooltip::rich(
                    "persistent-test",
                    Button::new("persistent-trigger", "Sharing help")
                        .track_focus(&self.trigger)
                        .on_activate(|_, _| {}),
                    "Sharing settings",
                    "Tooltip description",
                )
                .persistent(&self.state)
                .enable_user_input(self.automatic)
                .track_focus(&self.trigger)
                .focus_target(&self.action)
                .focusable(self.auto_focus)
                .positioner(|_, _, _| {
                    TooltipPositioner::corner(Anchor::TopLeft, point(px(100.), px(100.)))
                })
                .title_content(
                    div()
                        .id("custom-tooltip-title")
                        .test_support()
                        .child("Custom title"),
                )
                .body_content(
                    div()
                        .id("custom-tooltip-body")
                        .test_support()
                        .child("Custom body"),
                )
                .action_content(
                    Button::new("custom-tooltip-action", "Apply sharing setting")
                        .track_focus(&self.action)
                        .on_activate(move |_, cx| {
                            action.update(cx, |model, _| model.activations += 1)
                        }),
                )
                .link("Sharing guide", move |_, cx| {
                    link.update(cx, |model, _| model.link_activations += 1)
                })
                .on_dismiss_request(move |_, cx| {
                    let accepted = dismiss.update(cx, |model, _| {
                        model.dismissals += 1;
                        model.accept_dismiss.then(|| model.state.clone())
                    });
                    if let Some(state) = accepted {
                        state.update(cx, |state, cx| state.dismiss(cx));
                    }
                }),
            )
            .child(
                Button::new("tooltip-outside", "Outside target").on_activate(move |_, cx| {
                    outside.update(cx, |model, _| model.outside_activations += 1);
                }),
            )
    }
}

#[gpui::test]
fn persistent_tooltip_uses_owner_visibility_input_and_focus_policy(cx: &mut TestAppContext) {
    let (handle, model, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(|_| TooltipState::new());
        let model = cx.new(|cx| PersistentFixture {
            state: state.clone(),
            trigger: cx.focus_handle(),
            action: cx.focus_handle(),
            automatic: true,
            auto_focus: false,
            accept_dismiss: true,
            dismissals: 0,
            activations: 0,
            link_activations: 0,
            outside_activations: 0,
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (handle.into(), model, state)
    });
    frame(handle, cx);
    let location = element(handle, "persistent-trigger", cx)
        .unwrap()
        .bounds()
        .center();
    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        window.dispatch_event(
            MouseMoveEvent {
                position: location,
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        model.read(cx).trigger.clone().focus(window, cx);
    })
    .unwrap();
    frame(handle, cx);
    cx.executor().advance_clock(Duration::from_millis(650));
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_none());
    assert!(!cx.update(|cx| state.read(cx).is_visible()));

    click(handle, "persistent-trigger", cx);
    let tip = element(handle, "tooltip", cx).unwrap();
    assert_eq!(tip.role(), Some(Role::Tooltip));
    assert_eq!(tip.label(), Some("Tooltip description"));
    assert!(element(handle, "custom-tooltip-title", cx).is_some());
    assert!(element(handle, "custom-tooltip-body", cx).is_some());
    assert_eq!(
        element(handle, "persistent-trigger", cx).unwrap().focused(),
        Some(true)
    );
    cx.simulate_keystrokes(handle, "tab");
    frame(handle, cx);
    assert_eq!(
        element(handle, "custom-tooltip-action", cx)
            .unwrap()
            .focused(),
        Some(true)
    );
    activate_focused(handle, cx);
    cx.update(|cx| assert_eq!(model.read(cx).activations, 1));
    cx.simulate_keystrokes(handle, "tab");
    frame(handle, cx);
    activate_focused(handle, cx);
    cx.update(|cx| assert_eq!(model.read(cx).link_activations, 1));

    cx.simulate_keystrokes(handle, "escape");
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_none());
    assert_eq!(
        element(handle, "persistent-trigger", cx).unwrap().focused(),
        Some(true)
    );
    cx.update(|cx| assert_eq!(model.read(cx).dismissals, 1));

    activate_focused(handle, cx);
    assert!(element(handle, "tooltip", cx).is_some());
    cx.update(|cx| {
        cx.set_reduce_motion(false);
        state.update(cx, |state, cx| state.dismiss(cx));
    });
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_none());
    let fading = element(handle, "tooltip-exit", cx).unwrap().bounds();
    let underlay = element(handle, "tooltip-underlay", cx).unwrap().bounds();
    assert!(fading.contains(&underlay.center()));
    click(handle, "tooltip-underlay", cx);
    cx.update(|cx| cx.set_reduce_motion(true));
    frame(handle, cx);

    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.automatic = false;
            model.auto_focus = true;
            model.accept_dismiss = false;
            cx.notify();
        })
    });
    frame(handle, cx);
    click(handle, "persistent-trigger", cx);
    assert!(element(handle, "tooltip", cx).is_none());
    cx.update(|cx| state.update(cx, |state, cx| state.show(cx)));
    frame(handle, cx);
    assert_eq!(
        element(handle, "custom-tooltip-action", cx)
            .unwrap()
            .focused(),
        Some(true)
    );
    cx.simulate_keystrokes(handle, "escape");
    frame(handle, cx);
    assert!(element(handle, "tooltip", cx).is_some());
    cx.update(|cx| assert_eq!(model.read(cx).dismissals, 2));

    cx.update(|cx| state.update(cx, |state, cx| state.dismiss(cx)));
    frame(handle, cx);
    assert!(element(handle, "custom-tooltip-action", cx).is_none());
    assert_eq!(
        element(handle, "persistent-trigger", cx).unwrap().focused(),
        Some(true)
    );
    cx.update(|cx| {
        assert_eq!(model.read(cx).dismissals, 2);
        model.update(cx, |model, cx| {
            model.accept_dismiss = true;
            cx.notify();
        });
        state.update(cx, |state, cx| state.show(cx));
    });
    frame(handle, cx);
    click(handle, "tooltip-outside", cx);
    assert!(element(handle, "tooltip", cx).is_none());
    assert_eq!(
        element(handle, "tooltip-outside", cx).unwrap().focused(),
        Some(true)
    );
    cx.update(|cx| {
        assert_eq!(model.read(cx).dismissals, 3);
        assert_eq!(model.read(cx).outside_activations, 2);
    });
}

struct PositionFixture {
    state: Entity<TooltipState>,
    origin: Point<Pixels>,
    placement: TooltipPlacement,
    override_position: Option<Anchor>,
    caret_side: Rc<Cell<Option<TooltipPlacement>>>,
}

impl Render for PositionFixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let observed_caret = self.caret_side.clone();
        let tooltip = Tooltip::rich(
            "placement-test",
            div()
                .id("placement-trigger")
                .test_support()
                .w(px(52.))
                .h(px(24.)),
            "Position",
            "A description to wrap to the configured width.",
        )
        .persistent(&self.state)
        .placement(self.placement)
        .offset(px(12.))
        .caret(
            TooltipCaret::new(size(px(18.), px(7.))).path(move |bounds, side| {
                assert!(bounds.size.width <= px(18.) && bounds.size.height <= px(18.));
                observed_caret.set(Some(side));
                let mut path = gpui::PathBuilder::fill();
                path.move_to(bounds.origin);
                path.line_to(bounds.top_right());
                path.line_to(bounds.bottom_right());
                path.line_to(bounds.bottom_left());
                path.close();
                path.build().ok()
            }),
        )
        .max_width(px(112.))
        .shape(CornerShape::Radii([4., 8., 9., 6.]))
        .colors(TooltipColors {
            container: Some(rgb(0x456789).into()),
            ..Default::default()
        })
        .tonal_elevation(Elevation::Level2)
        .shadow_elevation(Elevation::Level0);
        let tooltip = if let Some(anchor) = self.override_position {
            tooltip.positioner(move |_, viewport, _| {
                let x = if matches!(anchor, Anchor::TopLeft | Anchor::BottomLeft) {
                    px(-200.)
                } else {
                    viewport.width + px(200.)
                };
                let y = if matches!(anchor, Anchor::TopLeft | Anchor::TopRight) {
                    px(-200.)
                } else {
                    viewport.height + px(200.)
                };
                TooltipPositioner::corner(anchor, point(x, y))
            })
        } else {
            tooltip
        };
        div().relative().size_full().child(
            div()
                .absolute()
                .left(self.origin.x)
                .top(self.origin.y)
                .child(tooltip),
        )
    }
}

#[gpui::test]
fn tooltip_placement_flips_mirrors_and_clamps_a_custom_positioner(cx: &mut TestAppContext) {
    let (handle, view, caret) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(|cx| {
            let mut state = TooltipState::new();
            state.show(cx);
            state
        });
        let caret = Rc::new(Cell::new(None));
        let view = cx.new(|_| PositionFixture {
            state,
            origin: point(px(160.), px(120.)),
            placement: TooltipPlacement::Start,
            override_position: None,
            caret_side: caret.clone(),
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, caret)
    });
    let viewport = size(px(360.), px(320.));
    cx.simulate_window_resize(handle, viewport);
    let bounds = |cx: &mut TestAppContext| -> Bounds<Pixels> {
        element(handle, "tooltip", cx).unwrap().bounds()
    };
    frame(handle, cx);
    assert_eq!(caret.get(), Some(TooltipPlacement::Left));
    assert!(bounds(cx).right() < px(160.));
    cx.update_window(handle, |_, window, cx| {
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    frame(handle, cx);
    assert_eq!(caret.get(), Some(TooltipPlacement::Right));
    assert!(bounds(cx).left() > px(212.));

    for (x, y, requested, expected) in [
        (0., 120., TooltipPlacement::Left, TooltipPlacement::Right),
        (306., 120., TooltipPlacement::Right, TooltipPlacement::Left),
        (160., 0., TooltipPlacement::Above, TooltipPlacement::Below),
        (160., 296., TooltipPlacement::Below, TooltipPlacement::Above),
    ] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.origin = point(px(x), px(y));
                view.placement = requested;
                cx.notify();
            })
        });
        frame(handle, cx);
        assert_eq!(caret.get(), Some(expected));
    }

    for anchor in [
        Anchor::TopLeft,
        Anchor::TopRight,
        Anchor::BottomLeft,
        Anchor::BottomRight,
    ] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.override_position = Some(anchor);
                cx.notify();
            })
        });
        frame(handle, cx);
        let tip = bounds(cx);
        assert!(tip.size.width <= px(112.));
        assert!(tip.left() >= px(8.) && tip.top() >= px(8.));
        assert!(tip.right() <= viewport.width - px(8.) && tip.bottom() <= viewport.height - px(8.));
    }
    cx.update_window(handle, |_, window, _| {
        let fill: gpui::Hsla = rgb(0x456789).into();
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == fill.into())
        );
    })
    .unwrap();
}

#[gpui::test]
fn rich_tooltip_keeps_its_action_available_while_focused(cx: &mut TestAppContext) {
    use super::MenuTooltipExamples;
    use crate::story::Story;
    use std::time::Duration;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| MenuTooltipExamples::new(Story::Tooltip, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"tooltip".into()).is_some()
        })
        .unwrap()
    };
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx).rich_trigger.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(10));
    cx.run_until_parked();
    assert!(frame(cx));
    let bounds = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, _| {
            find(window, &[], &"tooltip".into()).unwrap().bounds()
        })
        .unwrap()
    };
    let collapsed = bounds(cx);
    cx.simulate_keystrokes(window.into(), "tab");
    frame(cx);
    cx.update_window(window.into(), |_, window, cx| {
        assert!(
            view.read(cx).rich_action.is_focused(window),
            "tooltip action focused"
        );
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    cx.run_until_parked();
    assert!(frame(cx));
    let expanded = bounds(cx);
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert!((collapsed.center().x - expanded.center().x).abs() <= px(1.));
    cx.simulate_keystrokes(window.into(), "enter");
    cx.update(|cx| assert_eq!(view.read(cx).output.as_ref(), "30 days"));
    cx.simulate_keystrokes(window.into(), "escape");
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, cx| {
        assert!(view.read(cx).rich_trigger.is_focused(window));
        assert!(find(window, &[], &"tooltip-exit".into()).is_some());
        assert!(find(window, &[], &"tooltip-action".into()).is_none());
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(200));
    assert!(!frame(cx));
    cx.update_window(window.into(), |_, window, _| {
        assert!(find(window, &[], &"tooltip-exit".into()).is_none());
    })
    .unwrap();
}

#[gpui::test]
fn tooltip_delays_entrance_motion_until_anchor_is_measured(cx: &mut TestAppContext) {
    use gpui_material::Label;

    struct OffscreenLabel {
        offscreen: bool,
    }

    impl Render for OffscreenLabel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().h(px(100.)).w(px(300.)).overflow_hidden().child(
                div()
                    .mt(if self.offscreen { px(240.) } else { px(16.) })
                    .child(
                        Label::new(
                            "clipped-label",
                            Button::new("clipped-trigger", "Anchor"),
                            "Persistent label copy",
                        )
                        .persistent(true),
                    ),
            )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| OffscreenLabel { offscreen: true });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });

    // While clipped outside the content mask, anchor is None and motion must not start.
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &"label".into()).is_none());
    })
    .unwrap();

    // Bring the anchor into the content mask: first draw measures anchor, second starts motion at scale 0.8.
    let initial = cx
        .update_window(window.into(), |_, window, cx| {
            view.update(cx, |view, cx| {
                view.offscreen = false;
                cx.notify();
            });
            window.refresh();
            window.draw(cx).clear(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"label".into()).unwrap().bounds()
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    let settled = cx
        .update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"label".into()).unwrap().bounds()
        })
        .unwrap();
    assert!(initial.size.width < settled.size.width);
}

#[gpui::test]
fn tooltip_shortcut_and_element_ext_chaining(cx: &mut TestAppContext) {
    use gpui_material::{MaterialElementExt, MenuItem};

    struct ShortcutTooltips {
        plain_state: Entity<TooltipState>,
        rich_state: Entity<TooltipState>,
        empty_state: Entity<TooltipState>,
    }

    impl Render for ShortcutTooltips {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .gap_4()
                .p_6()
                .child(
                    Button::filled("chained-btn", "Search")
                        .badge_dot()
                        .tooltip("plain-ext-tip", "Search files")
                        .shortcut("Ctrl+K")
                        .persistent(&self.plain_state)
                        .context_menu(
                            "chained-menu",
                            [MenuItem::new("chained-action", "Open").shortcut("Enter")],
                        ),
                )
                .child(
                    Tooltip::rich(
                        "rich-shortcut-tip",
                        Button::tonal("rich-shortcut-trigger", "Palette"),
                        "Command Palette",
                        "Run any workspace action.",
                    )
                    .shortcut("Shift+P")
                    .persistent(&self.rich_state),
                )
                .child(
                    Button::text("empty-shortcut-trigger", "No Shortcut")
                        .tooltip("empty-shortcut-tip", "Just text")
                        .shortcut("")
                        .persistent(&self.empty_state),
                )
        }
    }

    let (handle, plain_state, rich_state, empty_state) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let plain = cx.new(|_| TooltipState::new());
        let rich = cx.new(|_| TooltipState::new());
        let empty = cx.new(|_| TooltipState::new());
        let p = plain.clone();
        let r = rich.clone();
        let e = empty.clone();
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, move |_, cx| {
            cx.new(|_| ShortcutTooltips {
                plain_state: p,
                rich_state: r,
                empty_state: e,
            })
        })
        .unwrap();
        (handle.into(), plain, rich, empty)
    });
    frame(handle, cx);

    // Show plain tooltip with shortcut created via MaterialElementExt::tooltip.
    cx.update(|cx| plain_state.update(cx, |state, cx| state.show(cx)));
    frame(handle, cx);
    cx.update_window(handle, |_, window, _| {
        let tip = find(window, &[], &"tooltip".into()).unwrap();
        assert_eq!(tip.role(), Some(Role::Tooltip));
        assert!(
            find(window, tip.path(), &"tooltip-shortcut".into()).is_some(),
            "plain tooltip renders formatted shortcut pill"
        );
    })
    .unwrap();
    cx.update(|cx| plain_state.update(cx, |state, cx| state.dismiss(cx)));
    frame(handle, cx);

    // Show rich tooltip with title and shortcut.
    cx.update(|cx| rich_state.update(cx, |state, cx| state.show(cx)));
    frame(handle, cx);
    cx.update_window(handle, |_, window, _| {
        let tip = find(window, &[], &"tooltip".into()).unwrap();
        assert!(
            find(window, tip.path(), &"tooltip-shortcut".into()).is_some(),
            "rich tooltip renders formatted shortcut pill in header"
        );
    })
    .unwrap();
    cx.update(|cx| rich_state.update(cx, |state, cx| state.dismiss(cx)));
    frame(handle, cx);

    // Empty shortcut string is ignored.
    cx.update(|cx| empty_state.update(cx, |state, cx| state.show(cx)));
    frame(handle, cx);
    cx.update_window(handle, |_, window, _| {
        let tip = find(window, &[], &"tooltip".into()).unwrap();
        assert!(
            find(window, tip.path(), &"tooltip-shortcut".into()).is_none(),
            "empty shortcut must not render a shortcut pill"
        );
    })
    .unwrap();
    cx.update(|cx| empty_state.update(cx, |state, cx| state.dismiss(cx)));
    frame(handle, cx);

    // Chained .badge_dot().tooltip(...).context_menu(...) opens context menu on right click.
    cx.update_window(handle, |_, window, cx| {
        let trigger = find(window, &[], &"chained-btn".into()).unwrap();
        let pos = trigger.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Right,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Right,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);
    assert!(element(handle, "chained-action", cx).is_some());
}

#[gpui::test]
fn tooltip_custom_show_and_hide_delays(cx: &mut TestAppContext) {
    struct DelayTooltipView;

    impl Render for DelayTooltipView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().p_8().child(
                Tooltip::plain(
                    "fast-tip",
                    div()
                        .id("fast-trigger")
                        .test_support()
                        .w(px(80.))
                        .h(px(32.)),
                    "Fast tooltip",
                )
                .show_delay(Duration::from_millis(100))
                .hide_delay(Duration::from_millis(50)),
            )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| DelayTooltipView)
        })
        .unwrap()
        .into()
    });
    frame(handle, cx);

    let trigger_center = cx
        .update_window(handle, |_, window, _| {
            find(window, &[], &"fast-trigger".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: trigger_center,
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);
    cx.executor().advance_clock(Duration::from_millis(120));
    frame(handle, cx);
    cx.update_window(handle, |_, window, _| {
        assert!(
            find(window, &[], &"tooltip".into()).is_some(),
            "custom 100 ms show_delay opens before default 500 ms"
        );
    })
    .unwrap();

    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: point(px(2.), px(2.)),
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);
    cx.executor().advance_clock(Duration::from_millis(70));
    frame(handle, cx);
    cx.update_window(handle, |_, window, _| {
        assert!(
            find(window, &[], &"tooltip".into()).is_none(),
            "custom 50 ms hide_delay closes before default 200 ms"
        );
    })
    .unwrap();
}
