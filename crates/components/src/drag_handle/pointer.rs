use std::rc::Rc;

use gpui::{
    App, Entity, Hitbox, MouseButton, MouseDownEvent, MouseExitEvent, MouseMoveEvent, MouseUpEvent,
    Subscription, TouchDragEvent, TouchPhase, WeakEntity, Window,
};
use gpui_material_foundation::DragCapture;

use super::state::{
    HandleState, PointerPress, ResizeBinding, ResizeRequest, starts_horizontal_drag,
};

fn watch_escape(owner: WeakEntity<HandleState>, window: &Window, cx: &mut App) -> Subscription {
    let window_id = window.window_handle().window_id();
    cx.intercept_keystrokes(move |event, window, cx| {
        if event.keystroke.key == "escape"
            && !event.keystroke.modifiers.modified()
            && window.window_handle().window_id() == window_id
        {
            let cancelled = owner
                .update(cx, |state, cx| {
                    if state.press.is_some() {
                        state.end(window);
                        state.publish(window, cx);
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if cancelled {
                cx.stop_propagation();
            }
        }
    })
}

pub(super) fn pointer_listeners(
    state: Entity<HandleState>,
    hitbox: Hitbox,
    resize: Option<Rc<ResizeBinding>>,
    on_resize_end: Option<ResizeRequest>,
    window: &mut Window,
) {
    window.on_mouse_event({
        let state = state.clone();
        let focus = resize.is_some();
        let initial_width = resize.as_ref().map(|resize| resize.width);
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if !phase.bubble()
                || event.button != MouseButton::Left
                || !hitbox.is_hovered(window)
                || window.captured_hitbox().is_some()
                || state.read(cx).press.is_some()
            {
                return;
            }
            state.update(cx, |state, cx| {
                if focus {
                    state.focus.focus(window, cx);
                }
                let owner = cx.weak_entity();
                window.capture_pointer(hitbox.id);
                let escape = watch_escape(owner.clone(), window, cx);
                let watch = DragCapture::watch_pointer(window, cx, move |window, cx| {
                    let _ = owner.update(cx, |state, cx| {
                        state.end(window);
                        state.hovered = false;
                        state.publish(window, cx);
                    });
                });
                state.press = Some(PointerPress::new(
                    event.position,
                    initial_width,
                    Some((hitbox.id, watch)),
                    escape,
                ));
                state.hovered = true;
                state.publish(window, cx);
            });
            cx.stop_propagation();
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let resize = resize.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            let current = state.read(cx);
            let press = current.mouse_pressed();
            // An active gesture still receives moves over other controls.
            if phase.capture() != press {
                return;
            }
            let hovered = hitbox.is_hovered_at(event.position, window);
            if !press && hovered == current.hovered {
                return;
            }
            let request = state.update(cx, |state, cx| {
                state.hovered = hovered;
                let request = if press && event.pressed_button == Some(MouseButton::Left) {
                    state.move_pointer(event.position, resize.as_deref())
                } else {
                    if press {
                        state.end(window);
                    }
                    None
                };
                state.publish(window, cx);
                request
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if press && event.pressed_button == Some(MouseButton::Left) {
                cx.stop_propagation();
            }
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        let resize = resize.clone();
        let on_resize_end = on_resize_end.clone();
        move |event: &MouseUpEvent, phase, window, cx| {
            if !phase.capture()
                || event.button != MouseButton::Left
                || !state.read(cx).mouse_pressed()
            {
                return;
            }
            let (request, finished) = state.update(cx, |state, cx| {
                let request = state.move_pointer(event.position, resize.as_deref());
                state.hovered = hitbox.is_hovered_at(event.position, window);
                let finished = state.final_pointer_width();
                state.end(window);
                state.publish(window, cx);
                (request, finished)
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if let (Some(position), Some(end)) = (finished, &on_resize_end) {
                end(position, window, cx);
            }
            cx.stop_propagation();
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        move |event: &TouchDragEvent, phase, window, cx| {
            let current = state.read(cx);
            if event.phase == TouchPhase::Started {
                if !phase.bubble()
                    || current.press.is_some()
                    || !hitbox.is_hovered_at(event.start_position, window)
                    || !starts_horizontal_drag(event.position - event.start_position)
                {
                    return;
                }
                window.prevent_default();
            } else if !phase.capture() || current.press.is_none() || current.mouse_pressed() {
                return;
            }
            let (request, finished) = state.update(cx, |state, cx| {
                if event.phase == TouchPhase::Started {
                    let width = resize.as_ref().map(|resize| {
                        state.focus.focus(window, cx);
                        resize.width
                    });
                    let escape = watch_escape(cx.weak_entity(), window, cx);
                    state.press =
                        Some(PointerPress::new(event.start_position, width, None, escape));
                }
                let request = if event.phase == TouchPhase::Cancelled {
                    None
                } else {
                    state.move_pointer(event.position, resize.as_deref())
                };
                let finished = (event.phase == TouchPhase::Ended)
                    .then(|| state.final_pointer_width())
                    .flatten();
                if matches!(event.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                    state.end(window);
                }
                state.publish(window, cx);
                (request, finished)
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if let (Some(position), Some(end)) = (finished, &on_resize_end) {
                end(position, window, cx);
            }
            cx.stop_propagation();
        }
    });
    window.on_mouse_event(move |_: &MouseExitEvent, phase, window, cx| {
        if phase.bubble() && state.read(cx).hovered {
            state.update(cx, |state, cx| {
                state.hovered = false;
                state.publish(window, cx);
            });
        }
    });
}
