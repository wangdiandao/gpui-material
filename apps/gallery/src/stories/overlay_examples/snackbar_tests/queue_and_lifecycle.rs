use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{ElementId, Modifiers, MouseButton, Role, TestAppContext};
use gpui_base::test_support::find;
use gpui_material::{MaterialRoot, SnackbarResult};

use super::super::*;
use super::{current, draw, enqueue, open};

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
