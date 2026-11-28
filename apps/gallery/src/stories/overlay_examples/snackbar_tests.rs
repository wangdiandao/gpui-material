mod occlusion;
mod queue_and_lifecycle;
mod styling_and_focus;

use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, Hsla, KeyDownEvent, KeyUpEvent, Keystroke, RenderOnce, TestAppContext, VisualTestContext,
};
use gpui_material::{MaterialRoot, MaterialTheme, SnackbarId};

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
