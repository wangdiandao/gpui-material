use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, EntityId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Role, Size,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_base::{
    Popover, PopoverState, Popup, Positioner,
    actions::{Cancel, Confirm},
};
use gpui_material_foundation::{OverlayMotion, ScaledOverlay, ThemeScope};
use gpui_material_theme::Direction;

use crate::menu_appearance::menu_pivot;

use super::content::menu_content;
use super::item::{Action, Change, ComposedTrigger, MenuItem};
use super::state::{MenuPresentation, PopupBounds};

pub(super) struct PopupHostConfig {
    pub(super) id: ElementId,
    pub(super) trigger: AnyElement,
    pub(super) composed_trigger: Option<ComposedTrigger>,
    pub(super) items: Option<Vec<MenuItem>>,
    pub(super) mouse_button: MouseButton,
    pub(super) anchor: Anchor,
    pub(super) offset: Pixels,
    pub(super) explicit_position: Option<Point<Pixels>>,
    pub(super) open: bool,
    pub(super) controlled: bool,
    pub(super) submenu: bool,
    pub(super) direction: Direction,
    pub(super) motion: Option<OverlayMotion>,
    pub(super) presentation: MenuPresentation,
    pub(super) bounds: Rc<PopupBounds>,
    pub(super) ancestor: Option<Rc<PopupBounds>>,
    pub(super) close_parent: Option<Action>,
    pub(super) request: Change,
    pub(super) view: EntityId,
}

#[allow(clippy::too_many_arguments)]
fn build_popup_overlay(
    menu_id: ElementId,
    items: Vec<MenuItem>,
    close_level: Action,
    close_parent: Option<Action>,
    submenu: bool,
    is_context: bool,
    controlled: bool,
    content_focus: FocusHandle,
    direction: Direction,
    bounds: Rc<PopupBounds>,
    presentation: MenuPresentation,
    motion: OverlayMotion,
    anchor: Anchor,
    explicit_position: Option<Point<Pixels>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let close: Action = {
        let close_level = close_level.clone();
        Rc::new(move |window, cx| {
            close_level(window, cx);
            if let Some(close_parent) = &close_parent {
                close_parent(window, cx);
            }
        })
    };
    let dismissal = close_level.clone();
    let escape = close_level.clone();
    let item_bounds = bounds.clone();
    let body = window.with_id(menu_id, |window| {
        menu_content(
            items,
            close,
            submenu.then_some(close_level),
            content_focus,
            (direction, bounds.clone(), presentation, motion),
            window,
            cx,
        )
    });
    let dismiss_shell = div()
        .id("dismiss-menu")
        .opacity(motion.alpha)
        .when(is_context, |shell| {
            let escape = escape.clone();
            shell.on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified() {
                    escape(window, cx);
                    cx.stop_propagation();
                }
            })
        })
        .when(!is_context && controlled, |shell| {
            shell.on_action(move |_: &Cancel, window, cx| escape(window, cx))
        })
        .on_mouse_down_out(move |event, window, cx| {
            let point = &event.position;
            let inside = if is_context {
                bounds.popup_contains(point)
                    || (event.button == MouseButton::Right && bounds.trigger.get().contains(point))
            } else {
                bounds.contains(point)
            };
            if inside {
                return;
            }
            dismissal(window, cx);
        })
        .child(body);
    let pivot = item_bounds.clone();
    let overlay = ScaledOverlay::new(dismiss_shell, motion, move |menu| {
        let anchor_bounds = explicit_position.map_or_else(
            || pivot.trigger.get(),
            |pos| Bounds::new(pos, Size::default()),
        );
        menu_pivot(menu, anchor_bounds)
    });
    let overlay = ThemeScope::inherit(overlay, window, cx);
    if let Some(position) = explicit_position {
        Positioner::corner(anchor, position)
            .on_position(move |resolved| item_bounds.popup.set(resolved.bounds))
            .occlude()
            .child(overlay)
            .into_any_element()
    } else {
        overlay
    }
}

pub(super) fn render_popup_host(
    config: PopupHostConfig,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let is_context = config.mouse_button == MouseButton::Right;
    let open = config.open;
    let controlled = config.controlled;
    let trigger_state = config.bounds.open.clone();
    let trigger: Action = {
        let request = config.request.clone();
        Rc::new(move |window, cx| {
            let displayed = if controlled {
                open
            } else {
                trigger_state.get()
            };
            request(!displayed, window, cx);
        })
    };
    let position = config.bounds.clone();
    let ancestor = config.ancestor.clone();
    let explicit_position = config.explicit_position;
    let on_position = move |resolved_bounds: Bounds<Pixels>, trigger_bounds: Bounds<Pixels>| {
        position.trigger.set(trigger_bounds);
        if explicit_position.is_none() {
            position.popup.set(resolved_bounds);
        }
        if let Some(ancestor) = &ancestor {
            *ancestor.child.borrow_mut() = Some(position.clone());
        }
    };

    if is_context {
        let content_focus = {
            let mut slot = config.bounds.context_focus.borrow_mut();
            slot.get_or_insert_with(|| cx.focus_handle()).clone()
        };
        let was_open = config.bounds.context_was_open.replace(open);
        if open && !was_open {
            *config.bounds.context_previous_focus.borrow_mut() = window.focused(cx);
            content_focus.focus(window, cx);
        } else if !open && was_open {
            let previous = config.bounds.context_previous_focus.borrow_mut().take();
            if content_focus.contains_focused(window, cx) {
                match previous {
                    Some(previous) => previous.focus(window, cx),
                    None => window.blur(cx),
                }
            }
        }
        let context_mouse_bounds = config.bounds.clone();
        let context_key_bounds = config.bounds.clone();
        let context_open_state = config.bounds.open.clone();
        let context_open: Action = {
            let request = config.request.clone();
            Rc::new(move |window, cx| request(true, window, cx))
        };
        let context_key_open = context_open.clone();
        let view = config.view;
        let content = config
            .composed_trigger
            .map_or(config.trigger, |build| build(open, trigger));
        let trigger_el = div()
            .id("menu-trigger")
            .role(Role::Group)
            .aria_expanded(open)
            .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                cx.stop_propagation();
                context_mouse_bounds
                    .context_position
                    .set(Some(event.position));
                if context_open_state.get() {
                    cx.notify(view);
                } else {
                    context_open(window, cx);
                }
            })
            .on_key_down(move |event, window, cx| {
                if is_context_menu_key(&event.keystroke) {
                    context_key_bounds.context_position.set(None);
                    context_key_open(window, cx);
                    cx.stop_propagation();
                }
            })
            .child(content);

        let menu_id = config.id.clone();
        let popup = Popup::new(config.id, trigger_el)
            .anchor(config.anchor)
            .offset(config.offset)
            .on_position(move |resolved, trigger| on_position(resolved.bounds, trigger));

        if open {
            let motion = config.motion.expect("open menu has motion");
            let popup_request = config.request;
            let close_level: Action = Rc::new(move |window, cx| popup_request(false, window, cx));
            let positioned = build_popup_overlay(
                menu_id,
                config.items.unwrap_or_default(),
                close_level,
                config.close_parent,
                config.submenu,
                true,
                controlled,
                content_focus.clone(),
                config.direction,
                config.bounds,
                config.presentation,
                motion,
                config.anchor,
                explicit_position,
                window,
                cx,
            );
            popup
                .content(
                    div()
                        .id("content")
                        .role(Role::Dialog)
                        .occlude()
                        .tab_group()
                        .track_focus(&content_focus)
                        .child(positioned),
                )
                .into_any_element()
        } else {
            popup.into_any_element()
        }
    } else {
        let forward = if config.direction == Direction::Rtl {
            "left"
        } else {
            "right"
        };
        let key_trigger = trigger.clone();
        let mouse_trigger = trigger.clone();
        let key_state = config.bounds.open.clone();
        let forward_request = config.request.clone();
        let popover_request = config.request.clone();
        let popup_request = config.request;
        let menu_id = config.id.clone();
        let mouse_button = config.mouse_button;
        let submenu = config.submenu;
        Popover::new(config.id)
            .open(open)
            .on_open_change(move |requested, window, cx| popover_request(*requested, window, cx))
            .anchor(config.anchor)
            .offset(config.offset)
            .mouse_button(mouse_button)
            .overlay_closable(false)
            .on_position(move |resolved, trigger| on_position(resolved.bounds, trigger))
            .trigger_with(move |open, _, _| {
                let content = config
                    .composed_trigger
                    .map_or(config.trigger, |build| build(open, trigger));
                div()
                    .id("menu-trigger")
                    .role(Role::Group)
                    .aria_expanded(open)
                    .when(controlled, |trigger| {
                        // Accepting the request through Popover::open applies
                        // the Base focus lifecycle only after the owner agrees.
                        trigger
                            .on_action(move |_: &Confirm, window, cx| key_trigger(window, cx))
                            .on_mouse_down(mouse_button, move |_, window, cx| {
                                cx.stop_propagation();
                                mouse_trigger(window, cx);
                            })
                    })
                    .on_key_down(move |event, window, cx| {
                        if submenu && event.keystroke.key == forward && !key_state.get() {
                            forward_request(true, window, cx);
                            cx.stop_propagation();
                        }
                    })
                    .child(content)
                    .into_any_element()
            })
            .content(move |state, window, cx| {
                let motion = config.motion.expect("open menu has motion");
                let close_level: Action = if controlled {
                    Rc::new(move |window, cx| popup_request(false, window, cx))
                } else {
                    let own = cx.entity().downgrade();
                    Rc::new(move |window, cx| {
                        let _ = own.update(cx, |state: &mut PopoverState, cx| {
                            state.dismiss(window, cx);
                        });
                    })
                };
                build_popup_overlay(
                    menu_id,
                    config.items.unwrap_or_default(),
                    close_level,
                    config.close_parent,
                    submenu,
                    false,
                    controlled,
                    state.focus_handle(cx),
                    config.direction,
                    config.bounds,
                    config.presentation,
                    motion,
                    config.anchor,
                    explicit_position,
                    window,
                    cx,
                )
            })
            .into_any_element()
    }
}

fn is_context_menu_key(keystroke: &gpui::Keystroke) -> bool {
    let mods = &keystroke.modifiers;
    if keystroke.key == "contextmenu" && !mods.modified() {
        return true;
    }
    keystroke.key == "f10"
        && mods.shift
        && !mods.control
        && !mods.alt
        && !mods.platform
        && !mods.function
}
