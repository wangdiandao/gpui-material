use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Context, DragMoveEvent, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Render, Role,
    SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::DragCapture;

use super::super::{SheetClose, sheet::Change};
use super::state::{BottomSheetState, BottomSheetThresholds, BottomSheetValue};

#[derive(Default)]
pub(in crate::overlay) struct ExpandableGesture {
    pub(in crate::overlay) origin: Option<Pixels>,
    pub(in crate::overlay) capture: Option<DragCapture>,
    pub(in crate::overlay) offset: Pixels,
}

#[derive(Clone)]
struct Driver {
    state: WeakEntity<BottomSheetState>,
    close: SheetClose,
    thresholds: BottomSheetThresholds,
}

impl Driver {
    fn navigate(&self, key: Option<&str>, window: &mut Window, cx: &mut App) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        if let Some(target) = state.read(cx).destination(key) {
            self.request(target, window, cx);
            true
        } else {
            false
        }
    }

    fn request(&self, target: BottomSheetValue, window: &mut Window, cx: &mut App) -> bool {
        if target == BottomSheetValue::Hidden {
            self.close.request(window, cx)
        } else {
            self.state
                .update(cx, |state, cx| state.animate_to(target, window, cx))
                .unwrap_or(false)
        }
    }

    fn release(&self, y: Pixels, window: &mut Window, cx: &mut App) {
        let target = self
            .state
            .update(cx, |state, cx| {
                state.finish_drag(y, self.thresholds, window, cx)
            })
            .ok()
            .flatten();
        if let Some(target) = target {
            if !self.request(target, window, cx) {
                let _ = self.state.update(cx, |state, cx| {
                    state.target = state.current;
                    state.move_to_target(true, cx);
                    cx.notify();
                    window.refresh();
                });
            }
            cx.stop_propagation();
        }
    }
}

struct DragPayload;

impl Render for DragPayload {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(in crate::overlay) fn drag_handle_mark(tint: Hsla) -> gpui::Div {
    div().w(px(32.)).h(px(4.)).rounded_full().bg(tint)
}

pub(in crate::overlay) struct BottomHandle {
    pub(in crate::overlay) id: ElementId,
    pub(in crate::overlay) state: Entity<BottomSheetState>,
    pub(in crate::overlay) close: SheetClose,
    pub(in crate::overlay) decoration: Option<AnyElement>,
    pub(in crate::overlay) label: SharedString,
    pub(in crate::overlay) grip: bool,
    pub(in crate::overlay) thresholds: BottomSheetThresholds,
    pub(in crate::overlay) tint: Hsla,
    pub(in crate::overlay) active: bool,
}

impl BottomHandle {
    pub(in crate::overlay) fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let model = self.state.read(cx);
        let enabled = self.active && model.available_values().len() > 1;
        let draggable =
            enabled && self.grip && model.layout.is_some_and(|layout| layout.drag_enabled);
        let focus = window
            .use_keyed_state((self.id.clone(), "bottom-handle"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let driver = Driver {
            state: self.state.downgrade(),
            close: self.close,
            thresholds: self.thresholds,
        };
        let decoration = if self.grip {
            self.decoration
                .unwrap_or_else(|| drag_handle_mark(self.tint).into_any_element())
        } else {
            crate::Text::new("Change sheet height")
                .id("sheet-height-label")
                .into_any_element()
        };
        let grip = div()
            .id("bottom-sheet-resize")
            .test_support()
            .w_full()
            .h(px(48.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .child(decoration);
        if !enabled {
            return grip.into_any_element();
        }
        let keyboard = driver.clone();
        let accessibility = driver.clone();
        let activate = driver.clone();
        grip.role(Role::Button)
            .aria_label(self.label)
            .track_focus(&focus)
            .tab_stop(true)
            .on_click(move |_, window, cx| {
                activate.navigate(None, window, cx);
            })
            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                accessibility.navigate(None, window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if keyboard.navigate(Some(&event.keystroke.key), window, cx) {
                    cx.stop_propagation();
                }
            })
            .when(draggable, |grip| {
                let pressed = driver.state.clone();
                let start = driver.state.clone();
                let moving = driver.state.clone();
                let outside = driver.clone();
                grip.cursor_ns_resize()
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        let _ =
                            pressed.update(cx, |state, _| state.pressed = Some(event.position.y));
                    })
                    .on_drag(DragPayload, move |_, _, window, cx| {
                        focus.focus(window, cx);
                        let y = window.mouse_position().y;
                        let _ = start.update(cx, |state, cx| {
                            state.begin_drag(y, window, cx);
                            state.drag_to(y, cx);
                        });
                        cx.stop_propagation();
                        cx.new(|_| DragPayload)
                    })
                    .on_drag_move(move |event: &DragMoveEvent<DragPayload>, window, cx| {
                        if let Ok(Some(changed)) =
                            moving.update(cx, |state, cx| state.drag_to(event.event.position.y, cx))
                        {
                            if changed {
                                window.refresh();
                            }
                            cx.stop_propagation();
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        driver.release(event.position.y, window, cx)
                    })
                    .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                        outside.release(event.position.y, window, cx)
                    })
            })
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::overlay) fn render_expandable(
        id: &ElementId,
        gesture: Rc<RefCell<ExpandableGesture>>,
        travel: Pixels,
        expanded: bool,
        change: Change,
        tint: Hsla,
        open: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        if !open {
            return div()
                .w_full()
                .h(px(48.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(drag_handle_mark(tint))
                .into_any_element();
        }
        let focus = window
            .use_keyed_state((id.clone(), "drag-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
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
        let settle = move |gesture: &RefCell<ExpandableGesture>,
                           y: Pixels,
                           change: &Change,
                           window: &mut Window,
                           cx: &mut App| {
            let active = std::mem::take(&mut *gesture.borrow_mut());
            window.refresh();
            if active.capture.is_none() || !f32::from(y).is_finite() {
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
            .tab_stop(false)
            .w_full()
            .h(px(48.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .cursor_ns_resize()
            .child(drag_handle_mark(tint))
            .on_mouse_down(MouseButton::Left, move |event, _, _| {
                if f32::from(event.position.y).is_finite() {
                    down.borrow_mut().origin = Some(event.position.y);
                }
            })
            .on_drag(DragPayload, move |_, _, window, cx| {
                drag_focus.focus(window, cx);
                let weak = Rc::downgrade(&start);
                start.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |_, _| {
                    if let Some(state) = weak.upgrade() {
                        *state.borrow_mut() = ExpandableGesture::default();
                    }
                }));
                cx.stop_propagation();
                cx.new(|_| DragPayload)
            })
            .on_drag_move(move |event: &DragMoveEvent<DragPayload>, window, _| {
                if !f32::from(event.event.position.y).is_finite() {
                    return;
                }
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
}
