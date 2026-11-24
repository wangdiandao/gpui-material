use std::time::Duration;

use gpui::{
    AppContext, Bounds, Context, ElementId, Entity, Hsla, IntoElement, ParentElement, Pixels,
    Render, Role, Styled, TestAppContext, Window, WindowHandle, div, px,
};
use gpui_base::{
    Root,
    test_support::{ElementSnapshot, find},
};
use gpui_material::{
    Button, ButtonColors, ContentArrangement, ControlElevations, FabColor, IconName, LogicalEdges,
    MaterialRoot, MaterialTheme, TextField, TextFieldState, Toolbar, ToolbarAction, ToolbarColors,
    ToolbarExitDirection, ToolbarFabPosition, ToolbarFling, ToolbarOrientation, ToolbarPlacement,
    ToolbarScrollBehavior, ToolbarScrollState,
    foundation::set_window_material_theme,
    theme::{CornerShape, CubicBezier, Direction, Elevation, MotionSpec},
};

const BAR: Hsla = gpui::hsla(0.52, 0.42, 0.42, 1.);
const FAB: Hsla = gpui::hsla(0.81, 0.65, 0.51, 1.);

type TestWindow = WindowHandle<Root>;

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

fn observed(window: &Window, id: impl Into<ElementId>) -> Option<ElementSnapshot> {
    find(window, &[], &id.into())
}

fn lookup(
    handle: TestWindow,
    id: impl Into<ElementId>,
    cx: &mut TestAppContext,
) -> ElementSnapshot {
    let id = id.into();
    cx.update_window(handle.into(), |_, window, _| {
        observed(window, id.clone()).unwrap_or_else(|| panic!("missing toolbar element {id:?}"))
    })
    .unwrap()
}

fn part(id: &'static str) -> ElementId {
    (ElementId::from("toolbar"), id).into()
}

fn paint(window: &Window, tint: Hsla) -> (Bounds<Pixels>, f32) {
    let quad = crate::painted_quads(window)
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(tint))
        .unwrap();
    (
        quad.bounds.map(|side| px(side.0 / window.scale_factor())),
        quad.corner_radii.top_left.0 / window.scale_factor(),
    )
}

#[gpui::test]
fn toolbar_content_keeps_native_editor_focus_and_independent_slots_during_resize(
    cx: &mut TestAppContext,
) {
    struct Example {
        editor: Entity<TextFieldState>,
        leading: bool,
        trailing: bool,
        expanded: bool,
        orientation: ToolbarOrientation,
        placement: ToolbarPlacement,
        width: Pixels,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(self.width).h(px(208.)).child(
                Toolbar::new(
                    "toolbar",
                    "Search actions",
                    [
                        ToolbarAction::new("edit", "Edit", IconName::Edit),
                        ToolbarAction::new("archive", "Archive", IconName::Archive).secondary(true),
                    ],
                )
                .placement(self.placement)
                .orientation(self.orientation)
                .arrangement(ContentArrangement::SpaceBetween)
                .spacing(px(6.))
                .expanded_height(px(76.))
                .padding(LogicalEdges::new(3., 11., 5., 7.))
                .shape(CornerShape::Radii([6., 12., 18., 8.]))
                .colors(ToolbarColors {
                    container: Some(BAR),
                    ..Default::default()
                })
                .leading_content(div().w(px(96.)).child(TextField::new(
                    "editor",
                    "Filter by title",
                    &self.editor,
                )))
                .content(Button::new("body", "Refine"))
                .trailing_content(Button::new("tail", "Done"))
                .expanded(self.expanded)
                .transition_motion(tween(200))
                .leading_expanded(self.leading)
                .trailing_expanded(self.trailing),
            )
        }
    }

    let (handle, view, editor) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entities = None;
        let handle = MaterialRoot::open_window(Default::default(), cx, |window, cx| {
            let editor = cx.new(|cx| TextFieldState::new(window, cx).default_value("ab"));
            let view = cx.new(|_| Example {
                editor: editor.clone(),
                leading: true,
                trailing: true,
                expanded: true,
                orientation: ToolbarOrientation::Horizontal,
                placement: ToolbarPlacement::Docked,
                width: px(440.),
            });
            entities = Some((view.clone(), editor));
            view
        })
        .unwrap();
        let (view, editor) = entities.unwrap();
        (handle, view, editor)
    });
    for (placement, orientation) in [
        (ToolbarPlacement::Docked, ToolbarOrientation::Horizontal),
        (ToolbarPlacement::Floating, ToolbarOrientation::Horizontal),
        (ToolbarPlacement::Floating, ToolbarOrientation::Vertical),
    ] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.placement = placement;
                view.orientation = orientation;
                cx.notify();
            })
        });
        for layout_direction in [Direction::Ltr, Direction::Rtl] {
            direction(handle, layout_direction, cx);
            frame(handle, cx);
            let editor = lookup(handle, "editor", cx).bounds();
            let middle = lookup(handle, "body", cx).bounds();
            let strip = lookup(handle, part("actions"), cx).bounds();
            if orientation == ToolbarOrientation::Vertical {
                assert!(editor.bottom() <= middle.top());
            } else if layout_direction == Direction::Ltr {
                assert!(editor.right() <= middle.left());
            } else {
                assert!(middle.right() <= editor.left());
            }
            if placement == ToolbarPlacement::Docked {
                assert_eq!(strip.size.height, px(76.));
                let tail = lookup(handle, "tail", cx).bounds();
                let edge_gap = if layout_direction == Direction::Ltr {
                    strip.right() - tail.right()
                } else {
                    tail.left() - strip.left()
                };
                assert_eq!(edge_gap, px(11.));
            }
            cx.update_window(handle.into(), |_, window, _| {
                let (_, corner) = paint(window, BAR);
                assert_eq!(
                    corner,
                    if layout_direction == Direction::Ltr {
                        6.
                    } else {
                        12.
                    }
                );
            })
            .unwrap();
        }
    }

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        editor
            .read(cx)
            .presentation()
            .focus_handle()
            .clone()
            .focus(window, cx);
        view.update(cx, |view, cx| {
            view.placement = ToolbarPlacement::Docked;
            view.orientation = ToolbarOrientation::Horizontal;
            view.leading = false;
            view.expanded = false;
            view.width = px(128.);
            cx.notify();
        });
    })
    .unwrap();
    frame(handle, cx);
    frame(handle, cx);
    assert_eq!(lookup(handle, "editor", cx).focused(), Some(true));
    assert!(lookup(handle, "editor", cx).visible());
    assert_eq!(
        lookup(handle, part("overflow"), cx).label(),
        Some("More actions")
    );
    cx.simulate_keystrokes(handle.into(), "end");
    cx.simulate_input(handle.into(), "c");
    cx.update(|cx| assert_eq!(editor.read(cx).value().as_ref(), "abc"));
    frame(handle, cx);
    assert_eq!(lookup(handle, "editor", cx).focused(), Some(true));
    cx.update_window(handle.into(), |_, window, cx| window.blur(cx))
        .unwrap();
    frame(handle, cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(observed(window, "editor").is_none());
        assert!(observed(window, "tail").is_some());
        assert!(observed(window, "body").is_some());
        assert!(observed(window, part("archive")).is_none());
    })
    .unwrap();

    cx.update(|cx| {
        cx.set_reduce_motion(false);
        view.update(cx, |view, cx| {
            view.leading = true;
            view.width = px(440.);
            cx.notify();
        });
    });
    frame(handle, cx);
    cx.executor().advance_clock(Duration::from_millis(100));
    frame(handle, cx);
    let partial_width = lookup(handle, part("leading"), cx).bounds().size.width;
    assert!(px(0.) < partial_width && partial_width < px(96.));
    cx.executor().advance_clock(Duration::from_millis(200));
    frame(handle, cx);
    assert_eq!(
        lookup(handle, part("leading"), cx).bounds().size.width,
        px(96.)
    );
    assert!(lookup(handle, "tail", cx).visible());
}

fn tween(duration_ms: u16) -> MotionSpec {
    MotionSpec::Tween {
        duration_ms,
        easing: CubicBezier {
            x1: 0.,
            y1: 0.,
            x2: 1.,
            y2: 1.,
        },
    }
}

#[gpui::test]
fn toolbar_exit_scroll_retargets_and_keeps_the_adjacent_fab_available(cx: &mut TestAppContext) {
    struct Example(Entity<ToolbarScrollState>);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Toolbar::new(
                        "toolbar",
                        "Panning actions",
                        [
                            ToolbarAction::new("edit", "Edit", IconName::Edit),
                            ToolbarAction::new("search", "Search", IconName::Search),
                        ],
                    )
                    .scroll_state(&self.0)
                    .scroll_behavior(ToolbarScrollBehavior {
                        exit_direction: Some(ToolbarExitDirection::Start),
                        collapse_threshold: px(14.),
                        expand_threshold: px(30.),
                        snap_motion: Some(tween(200)),
                        fling: ToolbarFling {
                            projection: Duration::from_millis(160),
                            motion: Some(tween(400)),
                        },
                    })
                    .colors(ToolbarColors {
                        container: Some(BAR),
                        ..Default::default()
                    })
                    .elevations(ControlElevations {
                        resting: Some(Elevation::Level2),
                        disabled: Some(Elevation::Level0),
                        ..Default::default()
                    })
                    .fab(
                        ToolbarFabPosition::End,
                        "New note",
                        IconName::Add,
                        |_, _| {},
                    )
                    .fab_color(FabColor::Tertiary)
                    .fab_colors(ButtonColors {
                        container: Some(FAB),
                        ..Default::default()
                    }),
                )
        }
    }

    let (handle, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        let state = cx.new(|_| ToolbarScrollState::new());
        let handle = MaterialRoot::open_window(Default::default(), cx, |_, cx| {
            cx.new(|_| Example(state.clone()))
        })
        .unwrap();
        (handle, state)
    });
    let offset = |value, cx: &mut TestAppContext| {
        cx.update(|cx| state.update(cx, |state, cx| state.set_content_offset(px(value), cx)))
    };
    let read = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            let state = state.read(cx);
            (state.offset(), state.offset_limit(), state.content_offset())
        })
    };
    frame(handle, cx);
    frame(handle, cx);
    let (initial, limit, _) = read(cx);
    assert_eq!(initial, px(0.));
    assert!(limit > px(70.));
    let original = lookup(handle, part("actions"), cx).bounds();
    offset(20., cx);
    frame(handle, cx);
    let (partial, _, raw) = read(cx);
    assert!((partial - px(20.)).abs() < px(0.1));
    assert_eq!(raw, px(20.));
    let ltr = lookup(handle, part("actions"), cx).bounds();
    assert!(ltr.left() < original.left());
    direction(handle, Direction::Rtl, cx);
    frame(handle, cx);
    let rtl = lookup(handle, part("actions"), cx).bounds();
    assert!(rtl.left() > ltr.left());
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(paint(window, FAB).0.size.width, px(80.));
    })
    .unwrap();

    cx.update(|cx| state.update(cx, |state, cx| state.settle(900., cx)));
    frame(handle, cx);
    cx.executor().advance_clock(Duration::from_millis(100));
    frame(handle, cx);
    let (settling, limit, _) = read(cx);
    assert!(settling > partial && settling < limit);
    offset(14., cx);
    frame(handle, cx);
    let (interrupted, _, _) = read(cx);
    assert!((interrupted - (settling - px(6.))).abs() <= px(0.1));

    cx.update(|cx| {
        cx.set_reduce_motion(true);
        state.update(cx, |state, cx| state.set_collapsed_fraction(0.8, cx));
    });
    frame(handle, cx);
    cx.update(|cx| state.update(cx, |state, cx| state.settle(0., cx)));
    frame(handle, cx);
    assert_eq!(read(cx).0, read(cx).1);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(observed(window, part("edit")).is_none());
        assert_eq!(
            observed(window, part("fab")).unwrap().role(),
            Some(Role::Button)
        );
    })
    .unwrap();
    offset(0., cx);
    frame(handle, cx);
    assert_eq!(read(cx).0, px(0.));
    assert!(lookup(handle, part("edit"), cx).visible());

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.blur(cx);
        window.focus_next(cx);
    })
    .unwrap();
    frame(handle, cx);
    assert_eq!(lookup(handle, part("edit"), cx).focused(), Some(true));
    cx.update(|cx| state.update(cx, |state, cx| state.set_collapsed_fraction(1., cx)));
    frame(handle, cx);
    assert_eq!(read(cx).0, px(0.));
    assert_eq!(lookup(handle, part("edit"), cx).focused(), Some(true));
    cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
        .unwrap();
    frame(handle, cx);
    assert_eq!(read(cx).0, read(cx).1);
    assert_eq!(lookup(handle, part("fab"), cx).focused(), Some(true));
}

#[gpui::test]
fn toolbar_action_with_icon_renders_custom_icons_inline_and_in_overflow(cx: &mut TestAppContext) {
    use gpui::{InputEvent as _, InteractiveElement as _};
    use gpui_base::TestSupportExt as _;

    struct Preview {
        expanded: bool,
    }

    impl Render for Preview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(320.)).child(
                Toolbar::new(
                    "toolbar",
                    "Custom icon actions",
                    [
                        ToolbarAction::with_icon("alpha", "Alpha", || {
                            div().id("custom-alpha-icon").test_support().size(px(18.))
                        }),
                        ToolbarAction::with_icon("beta", "Beta", || {
                            div().id("custom-beta-icon").test_support().size(px(18.))
                        })
                        .secondary(true),
                    ],
                )
                .placement(ToolbarPlacement::Docked)
                .expanded(self.expanded)
                .overflow_label("More actions"),
            )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Preview { expanded: true });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view)
    });
    cx.simulate_window_resize(handle.into(), gpui::size(px(400.), px(280.)));
    frame(handle, cx);

    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"custom-alpha-icon".into()).is_some());
        assert!(find(window, &[], &"custom-beta-icon".into()).is_some());
    })
    .unwrap();

    // Collapse toolbar so secondary "Beta" moves into overflow and renders its custom icon inside the overflow menu.
    cx.update(|cx| {
        view.update(cx, |this, cx| {
            this.expanded = false;
            cx.notify();
        });
    });
    frame(handle, cx);

    let overflow_center = cx
        .update_window(handle.into(), |_, window, _| {
            assert!(find(window, &[], &"custom-beta-icon".into()).is_none());
            find(window, &[], &part("overflow"))
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position: overflow_center,
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
                position: overflow_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(handle, cx);

    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"custom-beta-icon".into()).is_some());
    })
    .unwrap();
}
