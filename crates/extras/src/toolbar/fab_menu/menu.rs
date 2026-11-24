use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_components::Text;
use gpui_material_foundation::move_roving_focus;
use gpui_material_theme::{Direction, TypeRole};

use super::{
    Expand, FabMenu, FabMenuItem, MenuFocus, MenuState, MenuStyle, activate_key, focus_handle,
    style_button,
    trigger::{decorative_content, icon_content},
};

pub(super) fn item_part_id(
    menu_id: &ElementId,
    part: &'static str,
    item_id: SharedString,
) -> ElementId {
    ElementId::from((ElementId::from((menu_id.clone(), part)), item_id))
}

impl FabMenu {
    pub(super) fn prepare_focus(
        &self,
        open: bool,
        was_open: bool,
        state: &Rc<MenuState>,
        window: &mut Window,
        cx: &mut App,
    ) -> MenuFocus {
        let interactive = state.interactive.get();
        let has_items = !self.items.is_empty();
        let trigger = self
            .focus
            .clone()
            .unwrap_or_else(|| focus_handle(&self.id, "trigger-focus", window, cx));
        let scope = focus_handle(&self.id, "menu-scope", window, cx);
        let handles = self
            .items
            .iter()
            .map(|item| {
                let item_id = ElementId::from((self.id.clone(), item.id.clone()));
                (focus_handle(&item_id, "focus", window, cx), !item.disabled)
            })
            .collect::<Vec<_>>();
        let first = handles
            .iter()
            .find(|(_, enabled)| *enabled)
            .map(|(handle, _)| handle.clone());
        let open_focus = first.unwrap_or_else(|| scope.clone());
        if open && !was_open {
            let state = state.clone();
            let open_focus = open_focus.clone();
            window.defer(cx, move |window, cx| {
                if state.open.get() {
                    open_focus.focus(window, cx);
                }
            });
        } else if !open
            && ((was_open && scope.contains_focused(window, cx))
                || (!interactive && trigger.is_focused(window)))
        {
            let state = state.clone();
            let trigger_focus = trigger.clone();
            let previous_focus = window.focused(cx);
            window.defer(cx, move |window, cx| {
                let current = window.focused(cx);
                if state.open.get()
                    || interactive != state.interactive.get()
                    || current.is_some() && current != previous_focus
                {
                    return;
                }
                if interactive && has_items {
                    trigger_focus.focus(window, cx);
                } else if current.is_some() {
                    window.blur(cx);
                }
            });
        }
        MenuFocus {
            trigger,
            scope,
            open_focus,
            handles,
        }
    }

    pub(super) fn expansion_request(
        &self,
        state: &Rc<MenuState>,
        focus: &MenuFocus,
        has_items: bool,
    ) -> Expand {
        let state = state.clone();
        let trigger_focus = focus.trigger.clone();
        let scope = focus.scope.clone();
        let open_focus = focus.open_focus.clone();
        let controlled = self.expanded.is_some();
        let on_change = self.on_expanded_change.clone();
        Rc::new(move |expanded, window, cx| {
            if !state.interactive.get() || !has_items || state.open.get() == expanded {
                return;
            }
            if !controlled {
                state.open.set(expanded);
                if expanded {
                    open_focus.focus(window, cx);
                } else if scope.contains_focused(window, cx) {
                    trigger_focus.focus(window, cx);
                }
                window.refresh();
            }
            if let Some(on_change) = &on_change {
                on_change(expanded, window, cx);
            }
        })
    }

    pub(super) fn render_actions(
        &mut self,
        handles: &[(FocusHandle, bool)],
        state: &Rc<MenuState>,
        request: &Expand,
        style: MenuStyle,
        window: &Window,
    ) -> AnyElement {
        let available_height = (window.viewport_size().height - px(24.)).max(px(0.));
        let menu_id = self.id.clone();
        let items = std::mem::take(&mut self.items).into_iter().zip(handles);
        if !state.open.get() {
            return div()
                .id((menu_id.clone(), "exit"))
                .test_support()
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .max_h(available_height)
                .overflow_y_hidden()
                .track_scroll(&state.scroll)
                .children(items.map(|(item, _)| {
                    let colors = style.colors(&item);
                    menu_row(
                        menu_pill(&menu_id, div(), item, style, colors),
                        style.anchor,
                    )
                }))
                .into_any_element();
        }

        let key_handles = handles.to_vec();
        let key_scroll = state.scroll.clone();
        div()
            .id((menu_id.clone(), "actions"))
            .test_support()
            .role(Role::Menu)
            .aria_label(self.open_label.clone())
            .max_h(available_height)
            .overflow_y_scroll()
            .track_scroll(&state.scroll)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let focus = key_handles
                    .iter()
                    .map(|(handle, enabled)| (handle, *enabled))
                    .collect::<Vec<_>>();
                if let Some(index) = move_roving_focus(
                    &focus,
                    &event.keystroke.key,
                    Axis::Vertical,
                    Direction::Ltr,
                    window,
                    cx,
                ) {
                    key_scroll.scroll_to_item(index);
                    cx.stop_propagation();
                }
            })
            .children(items.map(|(mut item, (focus, enabled))| {
                let invoke = item.invoke.take();
                let request = request.clone();
                let open = state.clone();
                let activate = move |window: &mut Window, cx: &mut App| {
                    if open.open.get() {
                        request(false, window, cx);
                        if let Some(invoke) = &invoke {
                            invoke(window, cx);
                        }
                    }
                };
                let keyboard = activate.clone();
                let accessibility = activate.clone();
                let colors = style.colors(&item);
                let button = Button::new((menu_id.clone(), item.id.clone()))
                    .role(Role::MenuItem)
                    .accessibility_label(item.label.clone())
                    .track_focus(focus)
                    .disabled(!enabled)
                    .when(*enabled, |button| {
                        button
                            .on_click(move |_, window, cx| activate(window, cx))
                            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                                accessibility(window, cx)
                            })
                            .capture_key_down(move |event, window, cx| {
                                activate_key(event, window, cx, &keyboard)
                            })
                    });
                menu_row(
                    menu_pill(
                        &menu_id,
                        style_button(button, colors.0, colors.1, style.ring, *enabled),
                        item,
                        style,
                        colors,
                    ),
                    style.anchor,
                )
            }))
            .into_any_element()
    }
}

fn menu_row(child: impl IntoElement, anchor: Anchor) -> impl IntoElement {
    let row = div().flex().px(px(4.)).py(px(2.));
    match anchor {
        Anchor::BottomLeft => row.justify_start(),
        Anchor::BottomCenter => row.justify_center(),
        _ => row.justify_end(),
    }
    .child(child)
}

fn menu_pill<E>(
    menu_id: &ElementId,
    element: E,
    item: FabMenuItem,
    style: MenuStyle,
    (fill, ink): (Hsla, Hsla),
) -> E
where
    E: Styled + ParentElement + FluentBuilder,
{
    let width = style.max_width * style.reveal;
    let label = item.content.unwrap_or_else(|| {
        Text::new(item.label)
            .role(TypeRole::TitleMedium)
            .into_any_element()
    });
    element
        .flex()
        .flex_row()
        .flex_none()
        .when(style.rtl, Styled::flex_row_reverse)
        .items_center()
        .justify_end()
        .overflow_hidden()
        .min_h(px(56.))
        .max_w(width)
        .rounded_full()
        .bg(fill)
        .text_color(ink)
        .opacity(style.alpha)
        .px(px(24.).min(width / 3.))
        .gap(px(8.))
        .children(item.icon.map(|icon| {
            icon_content(icon, px(24.))
                .id(item_part_id(menu_id, "fab-menu-icon", item.id.clone()))
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        }))
        .child(decorative_content(
            item_part_id(menu_id, "fab-menu-label", item.id),
            label,
        ))
}
