use gpui::{
    Anchor, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, ScrollHandle, SharedString, Size, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::Button;
use gpui_material_components::{Fab, FabColor, FabSize, Icon, IconName, Menu, MenuItem};
use gpui_material_foundation::logical_anchor;
use gpui_material_theme::Direction;

use super::{
    FabContent, Invoke, Toolbar, ToolbarAction, ToolbarFab, activate_key, focus_handle,
    palette::ToolbarPalette, style_button,
};

pub(super) fn partition_actions(
    id: &ElementId,
    actions: Vec<ToolbarAction>,
    expanded: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Vec<(ToolbarAction, FocusHandle)>, Vec<ToolbarAction>) {
    let mut visible = Vec::new();
    let mut hidden = Vec::new();
    for action in actions {
        let action_id = ElementId::from((id.clone(), action.id.clone()));
        let handle = focus_handle(&action_id, "focus", window, cx);
        // Keep the active keyboard target mounted during scroll collapse.
        if expanded || !action.secondary || handle.is_focused(window) {
            visible.push((action, handle));
        } else {
            hidden.push(action);
        }
    }
    (visible, hidden)
}

pub(super) fn render_fab(
    toolbar: &Toolbar,
    fab: ToolbarFab,
    fab_focus: &FocusHandle,
    default_color: FabColor,
) -> AnyElement {
    let content = match fab.content {
        FabContent::Standard {
            icon,
            label,
            invoke,
        } => Fab::new((toolbar.id.clone(), "fab"), label, Icon::new(icon))
            .track_focus(fab_focus)
            .color(toolbar.fab_color.unwrap_or(default_color))
            .colors(toolbar.fab_colors)
            .size(if toolbar.expanded {
                FabSize::Small
            } else {
                FabSize::Medium
            })
            .on_activate(move |window, cx| invoke(window, cx))
            .into_any_element(),
        FabContent::Custom(content) => content,
    };
    div().flex_none().child(content).into_any_element()
}

pub(super) fn render_visible_actions(
    id: &ElementId,
    visible: Vec<(ToolbarAction, FocusHandle)>,
    first_tab: Option<usize>,
    action_size: Pixels,
    palette: ToolbarPalette,
) -> Vec<(AnyElement, FocusHandle)> {
    visible
        .into_iter()
        .enumerate()
        .map(|(index, (action, handle))| {
            let callback = action.invoke;
            let selected = action.selected.unwrap_or(false);
            let fill = if selected {
                palette.selected_fill
            } else {
                palette.surface
            };
            let foreground = if !action.enabled {
                palette.ink.opacity(0.38)
            } else if selected {
                palette.selected_ink
            } else {
                palette.ink
            };
            let button = Button::new((id.clone(), action.id))
                .track_focus(&handle)
                .tab_stop(first_tab == Some(index))
                .accessibility_label(action.label)
                .disabled(!action.enabled)
                .size(action_size)
                .flex_none()
                .rounded_full()
                .child(action.icon.render_inline())
                .when_some(action.selected, |button, selected| {
                    button.aria_toggled(if selected {
                        gpui::accesskit::Toggled::True
                    } else {
                        gpui::accesskit::Toggled::False
                    })
                })
                .when_some(callback.filter(|_| action.enabled), |button, callback| {
                    let key = callback.clone();
                    let a11y = callback.clone();
                    button
                        .on_click(move |_, window, cx| callback(window, cx))
                        .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                            a11y(window, cx)
                        })
                        .capture_key_down(move |event, window, cx| {
                            activate_key(event, window, cx, &*key)
                        })
                });
            (
                style_button(button, fill, foreground, palette.ring, action.enabled)
                    .into_any_element(),
                handle,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_overflow_menu(
    id: &ElementId,
    overflow_label: SharedString,
    hidden: Vec<ToolbarAction>,
    overflow_focus: FocusHandle,
    is_tab_stop: bool,
    horizontal: bool,
    direction: Direction,
    action_size: Pixels,
    palette: ToolbarPalette,
) -> (AnyElement, FocusHandle) {
    let entries = hidden.into_iter().map(|action| {
        let icon = action.icon;
        let item = MenuItem::new(action.id, action.label)
            .disabled(!action.enabled)
            .leading(move || icon.render_overflow());
        let item = if let Some(selected) = action.selected {
            item.checked(selected)
        } else {
            item
        };
        if let Some(invoke) = action.invoke {
            item.on_activate(move |window, cx| invoke(window, cx))
        } else {
            item
        }
    });
    let trigger_id = id.clone();
    let scroll_focus = overflow_focus.clone();
    let trigger = move |open, toggle: Invoke| {
        let focus = overflow_focus.clone();
        let accessibility_toggle = toggle.clone();
        let button = Button::new((trigger_id, "overflow"))
            .accessibility_label(overflow_label)
            .aria_expanded(open)
            .track_focus(&overflow_focus)
            .tab_stop(is_tab_stop)
            .size(action_size)
            .rounded_full()
            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                focus.focus(window, cx);
                toggle(window, cx);
            })
            .on_a11y_action(
                if open {
                    gpui::AccessibleAction::Collapse
                } else {
                    gpui::AccessibleAction::Expand
                },
                {
                    let focus = overflow_focus.clone();
                    move |_, window, cx| {
                        focus.focus(window, cx);
                        accessibility_toggle(window, cx);
                    }
                },
            )
            .child(Icon::new(IconName::ArrowDropDown).size(px(24.)));
        style_button(button, palette.surface, palette.ink, palette.ring, true).into_any_element()
    };
    let anchor = logical_anchor(
        if horizontal {
            Anchor::BottomRight
        } else {
            Anchor::LeftCenter
        },
        direction,
    );
    (
        div()
            .flex_none()
            .child(Menu::composed((id.clone(), "overflow-menu"), entries, trigger).anchor(anchor))
            .into_any_element(),
        scroll_focus,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_focused_scroll(
    id: &ElementId,
    strip_children: &[(AnyElement, FocusHandle)],
    first_command: usize,
    first_tab: Option<usize>,
    arranged: bool,
    mirrored: bool,
    toolbar_scroll: &ScrollHandle,
    window: &mut Window,
    cx: &mut App,
) {
    // Direct child indices include slot and spacer elements. The scroll handle
    // uses their physical order, including the reversal of a horizontal RTL strip.
    let mut children_count = strip_children.len();
    if arranged && children_count > 0 {
        children_count = children_count * 2 + 1;
    }
    let reveal_index = strip_children
        .iter()
        .position(|(_, handle)| handle.contains_focused(window, cx))
        .or_else(|| first_tab.map(|index| first_command + index))
        .map(|index| {
            let index = if arranged { index * 2 + 1 } else { index };
            if mirrored {
                children_count - index - 1
            } else {
                index
            }
        });
    let reveal = window.use_keyed_state((id.clone(), "focused-action"), cx, |_, _| {
        None::<(usize, Size<Pixels>)>
    });
    let target = reveal_index.map(|index| (index, toolbar_scroll.bounds().size));
    reveal.update(cx, |previous, _| {
        if *previous != target {
            if let Some((index, _)) = target {
                toolbar_scroll.scroll_to_item(index);
            }
            *previous = target;
        }
    });
}
