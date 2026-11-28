use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{Hsla, Role, TestAppContext, rgb};
use gpui_base::test_support::find;
use gpui_material::{
    MaterialRoot, MaterialTheme, SnackbarResult,
    foundation::set_window_material_theme,
    theme::{CornerShape, Direction},
};

use super::super::*;
use super::{QueuePreview, TintProbe, current, draw, enqueue, key, open};

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
