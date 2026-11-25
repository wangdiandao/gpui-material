use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    App, ElementId, Hsla, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, RenderOnce,
    Role, StatefulInteractiveElement, TestAppContext, VisualTestContext, rgb,
};
use gpui_base::test_support::find;
use gpui_material::{
    MaterialRoot, MaterialTheme, SnackbarId, SnackbarResult,
    foundation::set_window_material_theme,
    theme::{CornerShape, Direction},
};

use super::*;

#[derive(IntoElement)]
struct TintProbe {
    region: &'static str,
    paints: Rc<RefCell<Vec<(&'static str, Hsla)>>>,
    text: &'static str,
}

impl RenderOnce for TintProbe {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.paints
            .borrow_mut()
            .push((self.region, window.text_style().color));
        div().child(self.text)
    }
}

struct QueuePreview {
    host: Option<Entity<SnackbarHost>>,
    outside: FocusHandle,
}

impl Render for QueuePreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(340.))
            .child(Button::new("outside", "Outside").track_focus(&self.outside))
            .children(self.host.clone())
    }
}

fn open(
    cx: &mut TestAppContext,
    host: SnackbarHost,
) -> (Entity<QueuePreview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        QueuePreview {
            host: Some(cx.new(|_| host)),
            outside: cx.focus_handle(),
        }
    });
    draw(cx);
    cx.update(|window, cx| view.read(cx).outside.clone().focus(window, cx));
    draw(cx);
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    cx.run_until_parked();
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

fn enqueue(
    host: &Entity<SnackbarHost>,
    cx: &mut VisualTestContext,
    message: Snackbar,
) -> SnackbarId {
    cx.update(|window, cx| host.update(cx, |host, cx| host.enqueue(message, window, cx)))
}

fn current(
    host: &Entity<SnackbarHost>,
    cx: &mut VisualTestContext,
) -> Option<(SnackbarId, SharedString)> {
    cx.update(|_, cx| {
        host.read(cx)
            .current()
            .map(|(id, message)| (id, message.message().clone()))
    })
}

#[gpui::test]
fn snackbar_styles_live_content_and_keeps_keyboard_focus_local(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SnackbarHost::default());
    let host = cx.update(|window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        view.read(cx).host.clone().unwrap()
    });
    let paints = Rc::new(RefCell::new(Vec::new()));
    let results = Rc::new(RefCell::new(Vec::new()));
    let plain_results = results.clone();
    let action_results = results.clone();
    let completion_results = results.clone();
    let dismissed_styled = results.clone();
    let last_results = results.clone();
    let paint_message = paints.clone();
    let paint_action = paints.clone();
    let paint_dismiss = paints.clone();
    let first = enqueue(
        &host,
        cx,
        Snackbar::new("Saved")
            .on_result(move |result, _, _| plain_results.borrow_mut().push(("plain", result))),
    );
    let second = enqueue(
        &host,
        cx,
        Snackbar::new("Imported five items")
            .message_content(move |_, _| TintProbe {
                region: "message",
                paints: paint_message.clone(),
                text: "Five items were imported. Open the import list to review each item.",
            })
            .action("Review items", move |_, _| {
                action_results
                    .borrow_mut()
                    .push(("action", SnackbarResult::ActionPerformed))
            })
            .action_content(move |_, _| TintProbe {
                region: "action",
                paints: paint_action.clone(),
                text: "Review all imported items",
            })
            .dismiss_content("Close import message", move |_, _| TintProbe {
                region: "dismiss",
                paints: paint_dismiss.clone(),
                text: "Close",
            })
            .action_on_new_line(true)
            .shape(CornerShape::Radii([3., 7., 11., 5.]))
            .colors(SnackbarColors {
                container: Some(rgb(0x172b49).into()),
                content: Some(rgb(0xffeeee).into()),
                action: Some(rgb(0xb4ffff).into()),
                dismiss: Some(rgb(0xffd0f0).into()),
            })
            .on_result(move |result, _, _| completion_results.borrow_mut().push(("styled", result)))
            .on_dismiss(move |_, _| {
                dismissed_styled
                    .borrow_mut()
                    .push(("wrong dismissal", SnackbarResult::Dismissed))
            }),
    );
    let third = enqueue(
        &host,
        cx,
        Snackbar::new("Message without a close button")
            .show_dismiss(false)
            .timeout(None)
            .on_result(move |result, _, _| last_results.borrow_mut().push(("hidden", result))),
    );
    draw(cx);
    let short_height = cx.update(|window, cx| {
        assert!(view.read(cx).outside.is_focused(window));
        let status = find(window, &[], &("snackbar-message", 1usize).into()).unwrap();
        assert_eq!(status.role(), Some(Role::Status));
        assert_eq!(status.label(), Some("Saved"));
        assert_eq!(
            find(window, &[], &"snackbar-dismiss".into())
                .unwrap()
                .label(),
            Some("Dismiss message")
        );
        find(window, &[], &("snackbar", 1usize).into())
            .unwrap()
            .bounds()
            .size
            .height
    });
    assert_eq!(current(&host, cx).unwrap().0, first);
    assert!(results.borrow().is_empty());
    key(cx, "escape");
    assert_eq!(current(&host, cx).unwrap().0, first);
    cx.executor().advance_clock(Duration::from_secs(4));
    draw(cx);
    assert_eq!(current(&host, cx).unwrap().0, second);
    assert_eq!(*results.borrow(), [("plain", SnackbarResult::Dismissed)]);
    cx.update(|window, cx| {
        assert!(view.read(cx).outside.is_focused(window));
        let status = find(window, &[], &("snackbar-message", 2usize).into()).unwrap();
        let action = find(window, &[], &"snackbar-action".into()).unwrap();
        let dismiss = find(window, &[], &"snackbar-dismiss".into()).unwrap();
        assert_eq!(status.role(), Some(Role::Status));
        assert_eq!(status.label(), Some("Imported five items"));
        assert_eq!(action.label(), Some("Review items"));
        assert_eq!(dismiss.label(), Some("Close import message"));
        assert_eq!(action.role(), Some(Role::Button));
        assert_eq!(dismiss.role(), Some(Role::Button));
        assert!(action.bounds().top() >= status.bounds().bottom());
        assert!(dismiss.bounds().top() >= status.bounds().bottom());
        assert!(dismiss.bounds().right() < action.bounds().left());
        assert!(
            find(window, &[], &("snackbar", 2usize).into())
                .unwrap()
                .bounds()
                .size
                .height
                > short_height
        );
        let quads = crate::painted_quads(window);
        let custom = quads
            .iter()
            .find(|quad| quad.background == rgb(0x172b49).into())
            .expect("custom container");
        let scale = window.scale_factor();
        assert_eq!(custom.corner_radii.top_left, px(7.).scale(scale));
        assert_eq!(custom.corner_radii.top_right, px(3.).scale(scale));
        assert_eq!(custom.corner_radii.bottom_left, px(11.).scale(scale));
        assert_eq!(custom.corner_radii.bottom_right, px(5.).scale(scale));
        assert_eq!(
            paints
                .borrow()
                .iter()
                .rev()
                .find(|(part, _)| *part == "message")
                .unwrap()
                .1,
            rgb(0xffeeee).into()
        );
        assert_eq!(
            paints
                .borrow()
                .iter()
                .rev()
                .find(|(part, _)| *part == "action")
                .unwrap()
                .1,
            rgb(0xb4ffff).into()
        );
        assert_eq!(
            paints
                .borrow()
                .iter()
                .rev()
                .find(|(part, _)| *part == "dismiss")
                .unwrap()
                .1,
            rgb(0xffd0f0).into()
        );
        window.focus_next(cx);
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &"snackbar-action".into())
                .unwrap()
                .focused(),
            Some(true)
        )
    });
    key(cx, "enter");
    assert_eq!(current(&host, cx).unwrap().0, third);
    assert_eq!(
        *results.borrow(),
        [
            ("plain", SnackbarResult::Dismissed),
            ("action", SnackbarResult::ActionPerformed),
            ("styled", SnackbarResult::ActionPerformed)
        ]
    );
    cx.update(|window, cx| {
        assert!(view.read(cx).outside.is_focused(window));
        assert!(find(window, &[], &"snackbar-dismiss".into()).is_none());
        host.update(cx, |host, cx| {
            assert!(!host.dismiss_message(first, window, cx));
            assert!(!host.perform_action(second, window, cx));
            assert!(!host.perform_action(third, window, cx));
            assert!(host.dismiss_message(third, window, cx));
            assert!(!host.dismiss_message(third, window, cx));
        });
    });
    draw(cx);
    assert!(current(&host, cx).is_none());
    assert_eq!(
        results.borrow().last(),
        Some(&("hidden", SnackbarResult::Dismissed))
    );

    let programmatic_results = results.clone();
    let programmatic = enqueue(
        &host,
        cx,
        Snackbar::new("Ready")
            .action("Confirm", |_, _| {})
            .on_result(move |result, _, _| {
                programmatic_results
                    .borrow_mut()
                    .push(("programmatic", result))
            }),
    );
    cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            assert!(host.perform_action(programmatic, window, cx));
            assert!(!host.perform_action(programmatic, window, cx));
        })
    });
    assert_eq!(
        results.borrow().last(),
        Some(&("programmatic", SnackbarResult::ActionPerformed))
    );
    enqueue(&host, cx, Snackbar::new("Escape test").timeout(None));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    key(cx, "escape");
    assert!(current(&host, cx).is_none());
    cx.update(|window, cx| assert!(view.read(cx).outside.is_focused(window)));
}

#[gpui::test]
fn snackbar_renderer_reentrant_dismissal_and_teardown_complete_once(cx: &mut TestAppContext) {
    let (view, cx) = open(
        cx,
        SnackbarHost::default().renderer(|_, parts, _, _| {
            div()
                .id("custom-snackbar-layout")
                .test_support()
                .flex()
                .flex_col()
                .child(parts.message)
                .children(parts.action)
                .children(parts.dismiss)
        }),
    );
    let host = cx.update(|_, cx| view.read(cx).host.clone().unwrap());
    let log = Rc::new(RefCell::new(Vec::new()));
    let weak = host.downgrade();
    let log_one = log.clone();
    let log_two = log.clone();
    let log_three = log.clone();
    let dismiss_three = log.clone();
    let log_four = log.clone();
    let short = Duration::from_millis(10);
    let first = enqueue(
        &host,
        cx,
        Snackbar::new("First")
            .timeout(Some(short))
            .on_dismiss(move |window, cx| {
                let log_four = log_four.clone();
                weak.update(cx, |host, cx| {
                    assert_eq!(
                        host.current()
                            .map(|(_, message)| message.message().as_ref()),
                        Some("Second")
                    );
                    host.enqueue(
                        Snackbar::new("Fourth").timeout(Some(short)).on_result(
                            move |result, _, _| log_four.borrow_mut().push(("fourth", result)),
                        ),
                        window,
                        cx,
                    );
                    host.dismiss(window, cx);
                })
                .unwrap();
            })
            .on_result(move |result, _, _| log_one.borrow_mut().push(("first", result))),
    );
    let second = enqueue(
        &host,
        cx,
        Snackbar::new("Second")
            .timeout(Some(short))
            .on_result(move |result, _, _| log_two.borrow_mut().push(("second", result))),
    );
    let third = enqueue(
        &host,
        cx,
        Snackbar::new("Third")
            .timeout(None)
            .dismiss_content("Dismiss third", |_, _| div().child("Close"))
            .on_dismiss(move |_, _| {
                dismiss_three
                    .borrow_mut()
                    .push(("dismiss-third", SnackbarResult::Dismissed))
            })
            .on_result(move |result, _, _| log_three.borrow_mut().push(("third", result))),
    );
    draw(cx);
    cx.executor().advance_clock(short);
    draw(cx);
    assert_eq!(current(&host, cx), Some((third, "Third".into())));
    assert_eq!(
        *log.borrow(),
        [
            ("first", SnackbarResult::Dismissed),
            ("second", SnackbarResult::Dismissed)
        ]
    );
    cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            assert!(!host.dismiss_message(first, window, cx));
            assert!(!host.dismiss_message(second, window, cx));
        })
    });
    cx.executor().advance_clock(short);
    draw(cx);
    assert_eq!(current(&host, cx), Some((third, "Third".into())));
    let dismiss = cx.update(|window, _| {
        let custom = find(window, &[], &"custom-snackbar-layout".into()).unwrap();
        let status = find(window, &[], &("snackbar-message", 3usize).into()).unwrap();
        let dismiss = find(window, &[], &ElementId::from("snackbar-dismiss")).unwrap();
        assert_eq!(status.role(), Some(Role::Status));
        assert_eq!(dismiss.label(), Some("Dismiss third"));
        assert!(custom.bounds().contains(&dismiss.bounds().center()));
        dismiss.bounds().center()
    });
    cx.simulate_mouse_down(dismiss, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(dismiss, MouseButton::Left, Modifiers::default());
    draw(cx);
    let (fourth, message) = current(&host, cx).unwrap();
    assert_eq!(message.as_ref(), "Fourth");
    assert_eq!(
        log.borrow()[2..],
        [
            ("dismiss-third", SnackbarResult::Dismissed),
            ("third", SnackbarResult::Dismissed)
        ]
    );
    let race = host.downgrade();
    cx.update(|window, cx| {
        view.update(cx, |_, cx| {
            cx.spawn_in(window, async move |_, cx| {
                cx.background_executor().timer(short).await;
                let _ = race.update_in(cx, |host, window, cx| {
                    host.dismiss_message(fourth, window, cx)
                });
            })
            .detach();
        })
    });
    let log_teardown = log.clone();
    let final_id = enqueue(
        &host,
        cx,
        Snackbar::new("Teardown")
            .timeout(Some(short))
            .on_result(move |result, _, _| log_teardown.borrow_mut().push(("teardown", result))),
    );
    draw(cx);
    cx.executor().advance_clock(short);
    draw(cx);
    assert_eq!(current(&host, cx), Some((final_id, "Teardown".into())));
    assert_eq!(log.borrow()[4..], [("fourth", SnackbarResult::Dismissed)]);
    let weak = host.downgrade();
    drop(host);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.host = None;
            cx.notify();
        })
    });
    draw(cx);
    assert!(weak.upgrade().is_none());
    cx.executor().advance_clock(short);
    draw(cx);
    assert_eq!(log.borrow().len(), 5);
}

#[gpui::test]
fn snackbar_advances_after_timeout_and_accepts_undo(cx: &mut TestAppContext) {
    use super::SnackbarExamples;
    use gpui::{InputEvent as _, MouseDownEvent, MouseUpEvent, WindowOptions};

    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(SnackbarExamples::new)
        })
        .unwrap()
    });
    let click = |cx: &mut TestAppContext, id: ElementId| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let position = find(window, &[], &id).unwrap().bounds().center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    click_count: 1,
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    click_count: 1,
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        cx.run_until_parked();
    };
    let message = |cx: &mut TestAppContext, id| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &("snackbar-message", id).into())
                .and_then(|message| message.label().map(str::to_owned))
        })
        .unwrap()
    };
    click(cx, "save-region".into());
    let snackbar_bounds = |cx: &mut TestAppContext, state, id| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &(state, id).into()).map(|surface| surface.bounds())
        })
        .unwrap()
    };
    let collapsed = snackbar_bounds(cx, "snackbar", 1usize).unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    let expanded = snackbar_bounds(cx, "snackbar", 1usize).unwrap();
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert!((collapsed.center().x - expanded.center().x).abs() <= px(1.));
    click(cx, "archive-region".into());
    assert_eq!(message(cx, 1usize).as_deref(), Some("Changes saved"));
    assert_eq!(message(cx, 2usize), None);
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(message(cx, 2usize).as_deref(), Some("Draft archived"));
    click(cx, "snackbar-action-region".into());
    assert_eq!(message(cx, 3usize).as_deref(), Some("Draft restored"));
}

#[gpui::test]
fn snackbar_renders_nonzero_fallback_width_on_first_frame(cx: &mut TestAppContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|window, cx| {
        let host = cx.new(|_| SnackbarHost::default());
        host.update(cx, |host, cx| {
            host.push(Snackbar::new("First frame message"), window, cx);
        });
        QueuePreview {
            host: Some(host),
            outside: cx.focus_handle(),
        }
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let bounds = find(window, &[], &("snackbar", 1usize).into())
            .unwrap()
            .bounds();
        assert!(
            bounds.size.width > px(0.),
            "first-frame snackbar uses fallback width instead of 0px: {bounds:?}"
        );
    });
}

#[gpui::test]
fn snackbar_tone_push_message_and_replace_precedence(cx: &mut TestAppContext) {
    use gpui_material::{
        SemanticTone,
        foundation::{color, window_material_theme},
        theme::ColorRole,
    };

    let (view, cx) = open(cx, SnackbarHost::default());
    let host = cx.update(|_, cx| view.read(cx).host.clone().unwrap());

    // 1. push_message queues a plain message and returns its SnackbarId.
    let pushed_id = cx.update(|window, cx| {
        host.update(cx, |host, cx| host.push_message("Plain push", window, cx))
    });
    draw(cx);
    assert_eq!(current(&host, cx), Some((pushed_id, "Plain push".into())));
    cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            assert!(host.dismiss_message(pushed_id, window, cx));
        })
    });
    draw(cx);
    assert!(current(&host, cx).is_none());

    // 2. replace works when host is idle, and when active dismisses active cleanly ahead of pending entries.
    let events = Rc::new(RefCell::new(Vec::new()));
    let ev_a_dismiss = events.clone();
    let ev_a_result = events.clone();
    let ev_b_result = events.clone();
    let ev_c_result = events.clone();

    let id_a = cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            host.replace(
                Snackbar::new("First active")
                    .timeout(None)
                    .on_dismiss(move |_, _| ev_a_dismiss.borrow_mut().push("a:dismiss"))
                    .on_result(move |res, _, _| {
                        assert_eq!(res, SnackbarResult::Dismissed);
                        ev_a_result.borrow_mut().push("a:result");
                    }),
                window,
                cx,
            )
        })
    });
    let id_b = enqueue(
        &host,
        cx,
        Snackbar::new("Pending B")
            .timeout(None)
            .on_result(move |res, _, _| {
                assert_eq!(res, SnackbarResult::Dismissed);
                ev_b_result.borrow_mut().push("b:result");
            }),
    );
    draw(cx);
    assert_eq!(current(&host, cx), Some((id_a, "First active".into())));

    // Replacing while A is active and B is pending must dismiss A and show C ahead of B.
    let tints = Rc::new(RefCell::new(Vec::new()));
    let msg_tints = tints.clone();
    let action_tints = tints.clone();
    let id_c = cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            host.replace(
                Snackbar::new("Urgent danger")
                    .timeout(None)
                    .tone(SemanticTone::Danger)
                    .message_content(move |_, _| TintProbe {
                        region: "message",
                        paints: msg_tints.clone(),
                        text: "Urgent danger",
                    })
                    .action("Retry", |_, _| {})
                    .action_content(move |_, _| TintProbe {
                        region: "action",
                        paints: action_tints.clone(),
                        text: "Retry",
                    })
                    .on_result(move |res, _, _| {
                        assert_eq!(res, SnackbarResult::Dismissed);
                        ev_c_result.borrow_mut().push("c:result");
                    }),
                window,
                cx,
            )
        })
    });
    draw(cx);
    assert_eq!(current(&host, cx), Some((id_c, "Urgent danger".into())));
    assert_eq!(*events.borrow(), ["a:dismiss", "a:result"]);

    // Verify Danger tone paints ErrorContainer, OnErrorContainer, and Error action color.
    cx.update(|window, cx| {
        let theme = window_material_theme(window, cx);
        let error_container = color(theme.colors.color(ColorRole::ErrorContainer));
        let on_error_container = color(theme.colors.color(ColorRole::OnErrorContainer));
        let error = color(theme.colors.color(ColorRole::Error));

        assert!(
            crate::painted_quads(window)
                .iter()
                .any(|quad| quad.background == error_container.into()),
            "Danger snackbar must paint ErrorContainer"
        );
        let painted = tints.borrow();
        assert_eq!(
            painted
                .iter()
                .rev()
                .find(|(region, _)| *region == "message")
                .map(|(_, c)| *c),
            Some(on_error_container)
        );
        assert_eq!(
            painted
                .iter()
                .rev()
                .find(|(region, _)| *region == "action")
                .map(|(_, c)| *c),
            Some(error)
        );
    });

    // Dismissing C advances to the previously pending B.
    cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            assert!(host.dismiss_message(id_c, window, cx));
        })
    });
    draw(cx);
    assert_eq!(current(&host, cx), Some((id_b, "Pending B".into())));
    assert_eq!(*events.borrow(), ["a:dismiss", "a:result", "c:result"]);

    // 3. Explicit SnackbarColors overrides SemanticTone.
    let explicit_bg: Hsla = rgb(0x224466).into();
    let _id_override = cx.update(|window, cx| {
        host.update(cx, |host, cx| {
            host.replace(
                Snackbar::new("Override")
                    .timeout(None)
                    .tone(SemanticTone::Success)
                    .colors(SnackbarColors {
                        container: Some(explicit_bg),
                        ..Default::default()
                    }),
                window,
                cx,
            )
        })
    });
    draw(cx);
    cx.update(|window, _| {
        assert!(
            crate::painted_quads(window)
                .iter()
                .any(|quad| quad.background == explicit_bg.into()),
            "explicit SnackbarColors::container must override SemanticTone"
        );
    });
}

#[gpui::test]
fn snackbar_dismiss_label_queue_controls_and_hover_focus_pause(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SnackbarHost::default());
    let host = cx.update(|_, cx| view.read(cx).host.clone().unwrap());

    let first = enqueue(
        &host,
        cx,
        Snackbar::new("Timed notice")
            .timeout(Some(Duration::from_millis(200)))
            .dismiss_label("Close notice"),
    );
    let _second = enqueue(&host, cx, Snackbar::new("Queued 1"));
    let _third = enqueue(&host, cx, Snackbar::new("Queued 2"));
    draw(cx);

    cx.update(|window, cx| {
        assert_eq!(host.read(cx).pending_len(), 2);
        let dismiss = find(window, &[], &"snackbar-dismiss".into()).unwrap();
        assert_eq!(dismiss.label(), Some("Close notice"));
        host.update(cx, |host, cx| host.clear_queue(cx));
        assert_eq!(host.read(cx).pending_len(), 0);
    });
    draw(cx);

    // Hovering pauses the auto-dismiss timer beyond its original 200 ms timeout.
    let center = cx.update(|window, _| {
        find(window, &[], &("snackbar", 1usize).into())
            .unwrap()
            .bounds()
            .center()
    });
    cx.simulate_mouse_move(center, None, Modifiers::default());
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(350));
    draw(cx);
    assert_eq!(
        current(&host, cx),
        Some((first, "Timed notice".into())),
        "hovered snackbar must pause its auto-dismiss timer"
    );

    // Leaving hover restarts the timer and dismisses once elapsed.
    cx.simulate_mouse_move(gpui::point(px(0.), px(0.)), None, Modifiers::default());
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(250));
    draw(cx);
    assert_eq!(current(&host, cx), None);
}

#[gpui::test]
fn snackbar_occludes_underlying_hover_and_stops_control_click_propagation(cx: &mut TestAppContext) {
    use std::cell::Cell;

    struct OverlapPreview {
        host: Entity<SnackbarHost>,
        behind_hovered: Rc<Cell<bool>>,
        behind_clicks: Rc<Cell<usize>>,
        ancestor_clicks: Rc<Cell<usize>>,
    }

    impl Render for OverlapPreview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let behind_hovered = self.behind_hovered.clone();
            let behind_clicks = self.behind_clicks.clone();
            let ancestor_clicks = self.ancestor_clicks.clone();
            div()
                .relative()
                .w(px(360.))
                .h(px(120.))
                .child(
                    div()
                        .id("behind-surface")
                        .test_support()
                        .size_full()
                        .on_hover(move |hovering, _, _| behind_hovered.set(*hovering))
                        .on_click(move |_, _, _| {
                            behind_clicks.set(behind_clicks.get() + 1);
                        }),
                )
                .child(
                    div()
                        .id("snackbar-ancestor")
                        .absolute()
                        .inset_0()
                        .on_click(move |_, _, _| {
                            ancestor_clicks.set(ancestor_clicks.get() + 1);
                        })
                        .child(self.host.clone()),
                )
        }
    }

    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let behind_hovered = Rc::new(Cell::new(false));
    let behind_clicks = Rc::new(Cell::new(0));
    let ancestor_clicks = Rc::new(Cell::new(0));
    let (view, cx) = cx.add_window_view({
        let behind_hovered = behind_hovered.clone();
        let behind_clicks = behind_clicks.clone();
        let ancestor_clicks = ancestor_clicks.clone();
        move |window, cx| {
            window.activate_window();
            OverlapPreview {
                host: cx.new(|_| SnackbarHost::default()),
                behind_hovered,
                behind_clicks,
                ancestor_clicks,
            }
        }
    });
    draw(cx);
    let host = cx.update(|_, cx| view.read(cx).host.clone());
    let results = Rc::new(RefCell::new(Vec::new()));

    // 1. Active snackbar occludes hover & clicks on underlying content while keeping hover-pause.
    let first_results = results.clone();
    let first = enqueue(
        &host,
        cx,
        Snackbar::new("Warning notice")
            .action("Undo", |_, _| {})
            .timeout(Some(Duration::from_millis(200)))
            .on_result(move |result, _, _| first_results.borrow_mut().push(result)),
    );
    draw(cx);

    let (message_center, dismiss_center) = cx.update(|window, _| {
        let message = find(window, &[], &("snackbar-message", 1usize).into())
            .unwrap()
            .bounds()
            .center();
        let dismiss = find(window, &[], &"snackbar-dismiss".into())
            .unwrap()
            .bounds()
            .center();
        (message, dismiss)
    });

    // Hovering over the snackbar body pauses its timer and must not hover the surface underneath.
    cx.simulate_mouse_move(message_center, None, Modifiers::default());
    draw(cx);
    assert!(
        !behind_hovered.get(),
        "active snackbar body must occlude hover on underlying elements"
    );
    cx.executor().advance_clock(Duration::from_millis(350));
    draw(cx);
    assert_eq!(
        current(&host, cx),
        Some((first, "Warning notice".into())),
        "occluding snackbar must preserve its own hover timer pause"
    );

    // Clicking the snackbar body must not click through to the underlying surface or ancestor.
    cx.simulate_click(message_center, Modifiers::default());
    draw(cx);
    assert_eq!(behind_clicks.get(), 0);
    assert_eq!(ancestor_clicks.get(), 0);

    // Hovering and clicking the dismiss button must not hover or click underlying/ancestor targets.
    cx.simulate_mouse_move(dismiss_center, None, Modifiers::default());
    draw(cx);
    assert!(
        !behind_hovered.get(),
        "active snackbar dismiss button must occlude hover on underlying elements"
    );
    cx.simulate_click(dismiss_center, Modifiers::default());
    draw(cx);
    assert_eq!(current(&host, cx), None);
    assert_eq!(*results.borrow(), [SnackbarResult::Dismissed]);
    assert_eq!(
        behind_clicks.get(),
        0,
        "dismissing snackbar must not click through to underlying elements"
    );
    assert_eq!(
        ancestor_clicks.get(),
        0,
        "dismiss button click must stop propagation"
    );

    // Once dismissed, the same point hovers and clicks the underlying surface normally.
    cx.simulate_mouse_move(dismiss_center, None, Modifiers::default());
    draw(cx);
    assert!(behind_hovered.get());
    cx.simulate_click(dismiss_center, Modifiers::default());
    draw(cx);
    assert_eq!(behind_clicks.get(), 1);
    assert_eq!(ancestor_clicks.get(), 1);

    // 2. Clicking the action button also blocks click-through and stops propagation.
    let second_results = results.clone();
    enqueue(
        &host,
        cx,
        Snackbar::new("Archived")
            .action("Undo", |_, _| {})
            .on_result(move |result, _, _| second_results.borrow_mut().push(result)),
    );
    draw(cx);
    let action_center = cx.update(|window, _| {
        find(window, &[], &"snackbar-action".into())
            .unwrap()
            .bounds()
            .center()
    });
    cx.simulate_click(action_center, Modifiers::default());
    draw(cx);
    assert_eq!(current(&host, cx), None);
    assert_eq!(
        *results.borrow(),
        [SnackbarResult::Dismissed, SnackbarResult::ActionPerformed]
    );
    assert_eq!(behind_clicks.get(), 1);
    assert_eq!(
        ancestor_clicks.get(),
        1,
        "action button click must stop propagation"
    );
}

#[gpui::test]
fn snackbar_custom_renderer_stops_propagation_and_exit_does_not_occlude(cx: &mut TestAppContext) {
    use std::cell::Cell;

    struct CustomRendererPreview {
        host: Entity<SnackbarHost>,
        behind_clicks: Rc<Cell<usize>>,
    }

    impl Render for CustomRendererPreview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let behind_clicks = self.behind_clicks.clone();
            div()
                .relative()
                .w(px(360.))
                .h(px(120.))
                .child(
                    div()
                        .id("behind-custom")
                        .test_support()
                        .size_full()
                        .on_click(move |_, _, _| {
                            behind_clicks.set(behind_clicks.get() + 1);
                        }),
                )
                .child(div().absolute().inset_0().child(self.host.clone()))
        }
    }

    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(false);
    });
    let behind_clicks = Rc::new(Cell::new(0));
    let wrapper_clicks = Rc::new(Cell::new(0));
    let (view, cx) = cx.add_window_view({
        let behind_clicks = behind_clicks.clone();
        let wrapper_clicks = wrapper_clicks.clone();
        move |window, cx| {
            window.activate_window();
            let host = cx.new(|_| {
                SnackbarHost::default().renderer(move |_, parts, _, _| {
                    let wrapper_clicks = wrapper_clicks.clone();
                    div()
                        .id("custom-clickable-wrapper")
                        .flex()
                        .items_center()
                        .on_click(move |_, _, _| {
                            wrapper_clicks.set(wrapper_clicks.get() + 1);
                        })
                        .child(parts.message)
                        .children(parts.action)
                        .children(parts.dismiss)
                })
            });
            CustomRendererPreview {
                host,
                behind_clicks,
            }
        }
    });
    draw(cx);
    let host = cx.update(|_, cx| view.read(cx).host.clone());
    enqueue(&host, cx, Snackbar::new("Custom notice").timeout(None));
    draw(cx);
    // Let enter motion settle.
    cx.executor().advance_clock(Duration::from_millis(250));
    draw(cx);

    let dismiss_center = cx.update(|window, _| {
        find(window, &[], &"snackbar-dismiss".into())
            .unwrap()
            .bounds()
            .center()
    });
    cx.simulate_click(dismiss_center, Modifiers::default());
    draw(cx);

    assert_eq!(current(&host, cx), None);
    assert_eq!(
        wrapper_clicks.get(),
        0,
        "dismiss click must not propagate to a custom renderer container"
    );
    assert_eq!(
        behind_clicks.get(),
        0,
        "active custom-rendered snackbar must occlude underlying elements"
    );
    // While the exit transition is still mounted, it must not block clicks underneath.
    cx.update(|window, _| {
        assert!(
            find(window, &[], &("snackbar-exit", 1usize).into()).is_some(),
            "exiting snackbar should still be mid-animation"
        );
    });
    cx.simulate_click(dismiss_center, Modifiers::default());
    draw(cx);
    assert_eq!(
        behind_clicks.get(),
        1,
        "exiting snackbar must not occlude underlying elements"
    );
}
