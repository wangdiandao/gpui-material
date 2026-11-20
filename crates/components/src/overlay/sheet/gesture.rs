use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Context, DragMoveEvent, ElementId, Hsla, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, Render, StatefulInteractiveElement, Styled,
    Window, div, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{DragCapture, color, current_material_theme};
use gpui_material_theme::ColorRole;

use super::Change;

struct SheetDrag;

impl Render for SheetDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Default)]
pub(super) struct Gesture {
    pub(super) origin: Option<Pixels>,
    pub(super) capture: Option<DragCapture>,
    pub(super) offset: Pixels,
}

pub(super) fn drag_handle(
    id: &ElementId,
    gesture: Rc<RefCell<Gesture>>,
    travel: Pixels,
    expanded: bool,
    change: Change,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let tint = color(
        current_material_theme(window, cx)
            .colors
            .color(ColorRole::OnSurfaceVariant),
    );
    let focus = window
        .use_keyed_state((id.clone(), "drag-focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let down = gesture.clone();
    let start = gesture.clone();
    let release = gesture.clone();
    let outside = gesture.clone();
    let moving = gesture.clone();
    let cancel = gesture;
    let drag_focus = focus.clone();
    let release_change = change.clone();
    let settle = move |gesture: &RefCell<Gesture>,
                       y: Pixels,
                       change: &Change,
                       window: &mut Window,
                       cx: &mut App| {
        let active = std::mem::take(&mut *gesture.borrow_mut());
        window.refresh();
        if active.capture.is_none() {
            return;
        }
        if let Some(origin) = active.origin {
            let distance = y - origin;
            if distance.abs() >= px(32.) && expanded == (distance > px(0.)) {
                change(!expanded, window, cx);
            }
        }
    };
    div()
        .id("drag-handle")
        .test_support()
        .track_focus(&focus)
        .w_full()
        .h(px(48.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .cursor_ns_resize()
        .child(drag_handle_mark(tint))
        .on_mouse_down(MouseButton::Left, move |event, _, _| {
            down.borrow_mut().origin = Some(event.position.y);
        })
        .on_drag(SheetDrag, move |_, _, window, cx| {
            drag_focus.focus(window, cx);
            let weak = Rc::downgrade(&start);
            start.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |_, _| {
                if let Some(state) = weak.upgrade() {
                    *state.borrow_mut() = Gesture::default();
                }
            }));
            cx.stop_propagation();
            cx.new(|_| SheetDrag)
        })
        .on_drag_move(move |event: &DragMoveEvent<SheetDrag>, window, _| {
            let mut state = moving.borrow_mut();
            if state.capture.is_some()
                && let Some(origin) = state.origin
            {
                let offset = event.event.position.y - origin;
                state.offset = if expanded {
                    offset.clamp(px(0.), travel)
                } else {
                    offset.clamp(-travel, px(0.))
                };
                window.refresh();
            }
        })
        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
            settle(&release, event.position.y, &release_change, window, cx);
        })
        .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
            settle(&outside, event.position.y, &change, window, cx);
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && let Some(capture) = std::mem::take(&mut *cancel.borrow_mut()).capture
            {
                capture.cancel(window, cx);
                window.refresh();
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

pub(super) fn drag_handle_mark(tint: Hsla) -> gpui::Div {
    div().w(px(32.)).h(px(4.)).rounded_full().bg(tint)
}
