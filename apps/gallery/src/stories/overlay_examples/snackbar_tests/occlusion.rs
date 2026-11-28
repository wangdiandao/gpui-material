use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{Modifiers, StatefulInteractiveElement, TestAppContext};
use gpui_base::test_support::find;
use gpui_material::{MaterialRoot, MaterialTheme, SnackbarResult};

use super::super::*;
use super::{current, draw, enqueue};

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
