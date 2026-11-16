use std::time::Duration;

use gpui::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, ParentElement as _, Pixels, Point,
    Render, Role, ScrollDelta, ScrollEvent, ScrollHandle, StatefulInteractiveElement as _,
    Styled as _, TestAppContext, TouchDragEvent, TouchPhase, VisualTestContext, Window, div, point,
    px, rgb,
};
use gpui_base::{Button, test_support::find};
use gpui_material::SwipeToDismissValue::{self, EndToStart, Settled, StartToEnd};
use gpui_material::foundation::set_window_material_theme;
use gpui_material::theme::Direction;
use gpui_material::{MaterialRoot, MaterialTheme, SwipeToDismiss, SwipeToDismissState};

struct SwipePreview {
    state: Entity<SwipeToDismissState>,
    archive_focus: FocusHandle,
    remove_focus: FocusHandle,
    scroll: ScrollHandle,
    completions: Vec<SwipeToDismissValue>,
    foreground_clicks: usize,
    vertical_moves: usize,
    mounted: bool,
    unmount_on_dismiss: bool,
    gestures: bool,
    start_to_end: bool,
    end_to_start: bool,
}

impl Render for SwipePreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut viewport = div()
            .id("viewport")
            .track_scroll(&self.scroll)
            .overflow_y_scroll()
            .w(px(240.))
            .h(px(120.));
        if self.mounted {
            let owner = cx.weak_entity();
            let foreground = div()
                .w_full()
                .h(px(80.))
                .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, _| {
                    if event.pressed_button == Some(MouseButton::Left) {
                        this.vertical_moves += 1;
                    }
                }))
                .child(
                    Button::new("foreground")
                        .size_full()
                        .bg(rgb(0xffffff))
                        .accessibility_label("Message")
                        .on_click(cx.listener(|this, _, _, _| this.foreground_clicks += 1)),
                );
            viewport = viewport.child(
                SwipeToDismiss::new("swipe", &self.state, foreground)
                    .background(div().size_full().bg(rgb(0x00539c)))
                    .enable_start_to_end(self.start_to_end)
                    .enable_end_to_start(self.end_to_start)
                    .gestures_enabled(self.gestures)
                    .on_dismiss(move |value, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.completions.push(value);
                            if this.unmount_on_dismiss {
                                this.mounted = false;
                            }
                            cx.notify();
                        });
                    }),
            );
        }
        div()
            .child(viewport.child(div().h(px(240.))))
            .child(self.action("archive", "Archive item", StartToEnd, &self.archive_focus))
            .child(self.action("remove", "Remove item", EndToStart, &self.remove_focus))
    }
}

impl SwipePreview {
    fn action(
        &self,
        id: &'static str,
        label: &'static str,
        direction: SwipeToDismissValue,
        focus: &FocusHandle,
    ) -> Button {
        let state = self.state.clone();
        Button::new(id)
            .track_focus(focus)
            .accessibility_label(label)
            .child(label)
            .on_click(move |_, window, cx| {
                state.update(cx, |state, cx| {
                    state.dismiss(direction, window, cx);
                });
            })
    }
}

fn open(
    cx: &mut TestAppContext,
    state: SwipeToDismissState,
    reduce_motion: bool,
) -> (Entity<SwipePreview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(reduce_motion);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        SwipePreview {
            state: cx.new(|_| state),
            archive_focus: cx.focus_handle(),
            remove_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            completions: Vec::new(),
            foreground_clicks: 0,
            vertical_moves: 0,
            mounted: true,
            unmount_on_dismiss: false,
            gestures: true,
            start_to_end: true,
            end_to_start: true,
        }
    });
    draw(cx);
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

fn drag(cx: &mut VisualTestContext, delta: Point<Pixels>) -> Point<Pixels> {
    let start = point(px(120.), px(20.));
    let end = start + delta;
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
    end
}

fn release(cx: &mut VisualTestContext, end: Point<Pixels>) {
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
    draw(cx);
}

#[gpui::test]
fn swipe_projects_logical_directions_to_the_painted_foreground(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SwipeToDismissState::new(), true);
    cx.simulate_click(point(px(120.), px(20.)), Modifiers::default());
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).foreground_clicks, 1);
        assert!(window.focused(cx).is_some());
    });
    let mut completions = Vec::new();
    for (direction, dx, expected) in [
        (Direction::Ltr, 64., StartToEnd),
        (Direction::Ltr, -64., EndToStart),
        (Direction::Rtl, -64., StartToEnd),
        (Direction::Rtl, 64., EndToStart),
    ] {
        cx.update(|window, cx| {
            let mut theme = MaterialTheme::default();
            theme.layout.direction = direction;
            set_window_material_theme(Some(theme), window, cx);
            view.read(cx).state.clone().update(cx, |state, cx| {
                state.snap_to(Settled, window, cx);
            });
        });
        draw(cx);
        let end = drag(cx, point(px(dx), px(0.)));
        draw(cx);
        cx.update(|window, cx| {
            let preview = view.read(cx);
            let state = preview.state.read(cx);
            assert_eq!(state.target_value(), expected);
            assert_eq!(state.dismiss_direction(), expected);
            assert_eq!(state.current_value(), Settled);
            assert_eq!(state.settled_value(), Settled);
            assert!((state.progress() - 64. / 240.).abs() < 0.001);
            assert!((f32::from(state.offset()) - dx).abs() < 0.01);
            let frame = find(window, &[], &"swipe".into()).unwrap();
            let foreground = find(window, frame.path(), &"foreground".into()).unwrap();
            assert_eq!(foreground.bounds().origin.x, px(dx));
            let quads = window.painted_quads();
            assert!(quads.iter().any(|quad| {
                quad.background == gpui::Hsla::from(rgb(0x00539c)).into()
                    && quad.bounds.origin.x == px(0.).scale(window.scale_factor())
            }));
            assert_eq!(preview.completions, completions);
        });
        release(cx, end);
        release(cx, end);
        completions.push(expected);
        cx.update(|window, cx| {
            let preview = view.read(cx);
            let state = preview.state.read(cx);
            assert_eq!(state.settled_value(), expected);
            assert_eq!(state.current_value(), expected);
            assert_eq!(state.progress(), 1.);
            assert_eq!(preview.completions, completions);
            assert_eq!(preview.foreground_clicks, 1);
            assert!(window.focused(cx).is_none());
            assert!(find(window, &[], &"foreground".into()).is_none());
        });
    }
}

#[gpui::test]
fn swipe_respects_threshold_axis_cancellation_and_direction_controls(cx: &mut TestAppContext) {
    let threshold = SwipeToDismissState::new().positional_threshold(|width| width * 0.5);
    let (view, cx) = open(cx, threshold, true);
    let end = drag(cx, point(px(90.), px(0.)));
    release(cx, end);
    let end = drag(cx, point(px(4.), px(30.)));
    release(cx, end);
    cx.update(|_, cx| {
        let preview = view.read(cx);
        assert!(preview.vertical_moves > 0);
        assert_eq!(preview.state.read(cx).offset(), px(0.));
        assert!(preview.completions.is_empty());
    });

    let end = drag(cx, point(px(130.), px(0.)));
    draw(cx);
    cx.simulate_keystrokes("escape");
    release(cx, end);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).state.read(cx).offset(), px(0.));
    });

    cx.update(|_, cx| {
        view.update(cx, |this, cx| {
            this.end_to_start = false;
            cx.notify();
        });
    });
    draw(cx);
    let end = drag(cx, point(px(-130.), px(0.)));
    release(cx, end);
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(!state.dismiss(EndToStart, window, cx))
        });
    });

    // Touch cancellation and vertical scroll use the same foreground as mouse swipes.
    let start = point(px(80.), px(25.));
    let touch = |cx: &mut VisualTestContext, phase| {
        cx.simulate_event(TouchDragEvent {
            phase,
            start_position: start,
            position: start + point(px(132.), px(0.)),
        });
    };
    touch(cx, TouchPhase::Started);
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).target_value(), StartToEnd));
    touch(cx, TouchPhase::Cancelled);
    cx.simulate_event(ScrollEvent {
        position: start,
        delta: ScrollDelta::Pixels(point(px(0.), px(-20.))),
        touch_phase: TouchPhase::Moved,
        ..Default::default()
    });
    draw(cx);
    cx.update(|_, cx| {
        let preview = view.read(cx);
        assert!(preview.completions.is_empty());
        assert_eq!(preview.state.read(cx).settled_value(), Settled);
        assert!(preview.scroll.offset().y < px(0.));
    });
}

#[gpui::test]
fn swipe_completes_programmatic_motion_once_and_retargets_from_current_position(
    cx: &mut TestAppContext,
) {
    let clock = cx.executor();
    let (view, cx) = open(cx, SwipeToDismissState::new(), false);
    let advance = |cx: &mut VisualTestContext, frames: usize| {
        for _ in 0..frames {
            clock.advance_clock(Duration::from_millis(16));
            draw(cx);
        }
    };

    cx.update(|window, cx| {
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert!(state.dismiss(StartToEnd, window, cx));
            assert_eq!(state.target_value(), StartToEnd);
            assert_eq!(state.settled_value(), Settled);
        });
    });
    draw(cx);
    advance(cx, 4);
    cx.update(|window, cx| {
        assert!(view.read(cx).completions.is_empty());
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert!(state.progress() > 0. && state.progress() < 1.);
            state.reset(window, cx);
        });
    });
    advance(cx, 80);
    cx.update(|window, cx| {
        assert!(view.read(cx).completions.is_empty());
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert_eq!(state.progress(), 0.);
            assert!(state.dismiss(EndToStart, window, cx));
        });
    });
    advance(cx, 80);
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).completions, [EndToStart]);
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert_eq!(state.settled_value(), EndToStart);
            assert!(state.snap_to(EndToStart, window, cx));
            assert!(state.snap_to(StartToEnd, window, cx));
            assert_eq!(state.settled_value(), StartToEnd);
        });
    });
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).completions, [EndToStart, StartToEnd]));
}

#[gpui::test]
fn swipe_named_commands_work_with_disabled_gestures_and_caller_removal(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SwipeToDismissState::new(), true);
    cx.update(|_, cx| {
        view.update(cx, |this, cx| {
            this.gestures = false;
            this.unmount_on_dismiss = true;
            cx.notify();
        });
    });
    draw(cx);
    let end = drag(cx, point(px(64.), px(0.)));
    release(cx, end);
    cx.update(|_, cx| assert!(view.read(cx).completions.is_empty()));

    let mut completions = Vec::new();
    for (id, label, expected) in [
        ("archive", "Archive item", StartToEnd),
        ("remove", "Remove item", EndToStart),
    ] {
        cx.update(|window, cx| {
            let button = find(window, &[], &id.into()).unwrap();
            assert_eq!(button.role(), Some(Role::Button));
            assert_eq!(button.label(), Some(label));
            let preview = view.read(cx);
            let focus = if expected == StartToEnd {
                preview.archive_focus.clone()
            } else {
                preview.remove_focus.clone()
            };
            focus.focus(window, cx);
        });
        draw(cx);
        let keystroke =
            cx.update(|_, cx| Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap());
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        draw(cx);
        completions.push(expected);
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).completions, completions);
            assert_eq!(view.read(cx).state.read(cx).settled_value(), expected);
            assert!(find(window, &[], &"swipe".into()).is_none());
            let state = view.read(cx).state.clone();
            state.update(cx, |state, cx| state.reset(window, cx));
            view.update(cx, |this, cx| {
                this.mounted = true;
                cx.notify();
            });
        });
        draw(cx);
    }
}
