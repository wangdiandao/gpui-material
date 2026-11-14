use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material::{Button, Snackbar, SnackbarHost};

use crate::story::stack;

pub(crate) struct SnackbarExamples {
    host: Entity<SnackbarHost>,
}

impl SnackbarExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            host: cx.new(|_| SnackbarHost::default()),
        }
    }
}

impl Render for SnackbarExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let timed = self.host.clone();
        let persistent = self.host.clone();
        stack()
            .max_w(px(480.))
            .items_start()
            .child(div().id("save-region").test_support().child(
                Button::new("save-message", "Save changes").on_activate(move |window, cx| {
                    timed.update(cx, |host, cx| {
                        host.push(Snackbar::new("Changes saved"), window, cx);
                    });
                }),
            ))
            .child(div().id("archive-region").test_support().child(
                Button::new("archive-message", "Archive draft").on_activate(move |window, cx| {
                    let host = persistent.downgrade();
                    persistent.update(cx, |queue, cx| {
                        queue.push(
                            Snackbar::new("Draft archived").action("Undo", move |window, cx| {
                                let _ = host.update(cx, |host, cx| {
                                    host.push(Snackbar::new("Draft restored"), window, cx);
                                });
                            }),
                            window,
                            cx,
                        );
                    });
                }),
            ))
            .child(self.host.clone())
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn snackbar_advances_after_timeout_and_accepts_undo(cx: &mut gpui::TestAppContext) {
    use gpui::{ElementId, InputEvent as _, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(SnackbarExamples::new)
        })
        .unwrap()
    });
    let click = |cx: &mut gpui::TestAppContext, id: ElementId| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let position = find(window, &[], &id).unwrap().bounds().center();
            window.dispatch_event(
                gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    click_count: 1,
                    position,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
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
    let message = |cx: &mut gpui::TestAppContext, id| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &("snackbar-message", id).into())
                .and_then(|message| message.label().map(str::to_owned))
        })
        .unwrap()
    };
    click(cx, "save-region".into());
    click(cx, "archive-region".into());
    assert_eq!(message(cx, 1usize).as_deref(), Some("Changes saved"));
    assert_eq!(message(cx, 2usize), None);
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    assert_eq!(message(cx, 2usize).as_deref(), Some("Draft archived"));
    click(cx, "snackbar-action-region".into());
    assert_eq!(message(cx, 3usize).as_deref(), Some("Draft restored"));
}
