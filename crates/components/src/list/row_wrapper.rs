use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, Keystroke, ParentElement, Role, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, accesskit::Toggled, div, prelude::FluentBuilder,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::ThemeScope;

use crate::action::{Activate, FeedbackState, activate_feedback, feedback_target};

use super::long_press::LongPress;
use super::{Expansion, SecondaryAction, ValueChange};

pub(super) fn merge_aria_descriptions(
    primary: Option<SharedString>,
    secondary: Option<SharedString>,
) -> Option<SharedString> {
    match (
        primary.filter(|text| !text.is_empty()),
        secondary.filter(|text| !text.is_empty()),
    ) {
        (Some(a), Some(b)) if a == b => Some(a),
        (Some(a), Some(b)) => Some(format!("{a}. {b}").into()),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

pub(super) fn render_expanded_content(
    expansion: Option<Expansion>,
    interactive: bool,
    enabled: bool,
    focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> Option<Stateful<Div>> {
    expansion.and_then(|expansion| {
        let scope = window
            .use_keyed_state("disclosure-focus", cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        if !expansion.expanded && interactive && enabled && scope.contains_focused(window, cx) {
            focus.focus(window, cx);
        }
        expansion.expanded.then(|| {
            div()
                .id("list-expanded-content")
                .track_focus(&scope)
                .child(expansion.content)
        })
    })
}

pub(super) fn interactive_row_role(
    checked: Option<Toggled>,
    checked_role: Role,
    selectable: bool,
) -> Role {
    if checked.is_some() {
        checked_role
    } else if selectable {
        Role::ListBoxOption
    } else {
        Role::Button
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn wrap_interactive_row(
    id: &ElementId,
    label: SharedString,
    aria_description: Option<SharedString>,
    surface: Stateful<Div>,
    focus: FocusHandle,
    tab_stop: bool,
    selected: bool,
    selectable: bool,
    enabled: bool,
    dragged: bool,
    checked: Option<Toggled>,
    checked_role: Role,
    invalid: bool,
    expanded: Option<bool>,
    menu_a11y_toggle: Option<Activate>,
    ring: Hsla,
    feedback: Rc<RefCell<FeedbackState>>,
    primary: Option<Activate>,
    expanded_change: Option<ValueChange>,
    secondary_action: Option<SecondaryAction>,
    long_press: Option<Rc<LongPress>>,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let activate = feedback.clone();
    let secondary_label = secondary_action
        .as_ref()
        .filter(|_| enabled && !dragged)
        .map(|secondary| secondary.label.clone());
    let merged_description = merge_aria_descriptions(aria_description, secondary_label);
    let role = interactive_row_role(checked, checked_role, selectable);

    let mut control = gpui_base::Button::new((id.clone(), "activate"))
        .track_focus(&focus)
        .tab_stop(tab_stop)
        .w_full()
        .role(role)
        .accessibility_label(label)
        .when_some(merged_description, |control, description| {
            control.aria_description(description)
        })
        .disabled(!enabled)
        .selected(selected)
        .when(selectable, |control| control.aria_selected(selected))
        .when_some(checked, |control, checked| control.aria_toggled(checked))
        .when(invalid, |control| {
            control.a11y_synthetic_children(|tree| {
                tree.parent_node()
                    .set_invalid(gpui::accesskit::Invalid::True);
            })
        })
        .when_some(expanded, |control, expanded| {
            control.aria_expanded(expanded)
        })
        .border_2()
        .border_color(gpui::transparent_black())
        .rounded_md()
        .focus_visible(move |style| style.border_color(ring))
        .when(enabled && !dragged, |control| {
            feedback_target(control, feedback, true, Some(focus.clone()))
        });
    if let Some(toggle) = menu_a11y_toggle.filter(|_| enabled && !dragged) {
        let return_focus = focus.clone();
        control = control.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            return_focus.focus(window, cx);
            toggle(window, cx);
        });
    } else if let Some(primary) = primary.filter(|_| enabled && !dragged) {
        let invoke = primary.clone();
        let hold = long_press.clone();
        control = control
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                invoke(window, cx)
            })
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                let native_drag = matches!(event, ClickEvent::Mouse(_)) && cx.has_active_drag();
                let allowed = !native_drag
                    && !activate.borrow().blocks_activation(event)
                    && !hold
                        .as_ref()
                        .is_some_and(|hold| hold.blocks_activation(event));
                if allowed || matches!(event, ClickEvent::Mouse(_)) {
                    activate_feedback(event, &activate, window, cx);
                }
                if allowed {
                    primary(window, cx);
                }
            });
    }
    if let Some((expanded, change)) = expanded
        .zip(expanded_change)
        .filter(|_| enabled && !dragged)
    {
        let change =
            ThemeScope::bind_action(move |window, cx| change(!expanded, window, cx), window, cx);
        control = control.on_a11y_action(
            if expanded {
                AccessibleAction::Collapse
            } else {
                AccessibleAction::Expand
            },
            move |_, window, cx| change(window, cx),
        );
    }
    if let Some((secondary, hold)) = secondary_action
        .zip(long_press)
        .filter(|_| enabled && !dragged)
    {
        let invoke = secondary.handler.clone();
        control = control
            .on_a11y_action(AccessibleAction::ShowContextMenu, move |_, window, cx| {
                invoke(window, cx)
            })
            .on_key_down(move |event, window, cx| {
                if !focus.is_focused(window) {
                    return;
                }
                if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified() {
                    hold.cancel();
                }
                if is_long_press_shortcut(&event.keystroke) {
                    cx.stop_propagation();
                    if !event.is_held {
                        (secondary.handler)(window, cx);
                    }
                }
            });
    }
    control.child(surface).into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn wrap_static_row(
    id: ElementId,
    label: SharedString,
    aria_description: Option<SharedString>,
    selected: bool,
    selectable: bool,
    disabled: bool,
    checked: Option<Toggled>,
    checked_role: Role,
    invalid: bool,
    expanded: Option<bool>,
    surface: Stateful<Div>,
    expanded_content: Option<Stateful<Div>>,
) -> AnyElement {
    let role = if checked.is_some() {
        checked_role
    } else {
        Role::ListItem
    };
    div()
        .id(id)
        .role(role)
        .test_support()
        .aria_label(label)
        .when(selectable, |row| row.aria_selected(selected))
        .when_some(aria_description, |row, description| {
            row.aria_description(description)
        })
        .when_some(checked, |row, checked| row.aria_toggled(checked))
        .when_some(expanded, |row, expanded| row.aria_expanded(expanded))
        .when(
            invalid || (disabled && (checked.is_some() || expanded.is_some())),
            |row| {
                row.a11y_synthetic_children(move |tree| {
                    let node = tree.parent_node();
                    if disabled && (checked.is_some() || expanded.is_some()) {
                        node.set_disabled();
                    }
                    if invalid {
                        node.set_invalid(gpui::accesskit::Invalid::True);
                    }
                })
            },
        )
        .flex()
        .flex_col()
        .w_full()
        .child(surface)
        .children(expanded_content)
        .into_any_element()
}

fn is_long_press_shortcut(keystroke: &Keystroke) -> bool {
    let modifiers = keystroke.modifiers;
    let is_shift_f10 = keystroke.key == "f10"
        && modifiers.shift
        && !modifiers.control
        && !modifiers.alt
        && !modifiers.platform;
    let is_context_menu = keystroke.key == "contextmenu" && !modifiers.modified();
    is_shift_f10 || is_context_menu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_aria_descriptions_preserves_caller_and_secondary_descriptions() {
        assert_eq!(merge_aria_descriptions(None, None), None);
        assert_eq!(
            merge_aria_descriptions(Some("Current: Dark".into()), None),
            Some("Current: Dark".into())
        );
        assert_eq!(
            merge_aria_descriptions(None, Some("More actions".into())),
            Some("More actions".into())
        );
        assert_eq!(
            merge_aria_descriptions(Some("Current: Dark".into()), Some("More actions".into())),
            Some("Current: Dark. More actions".into())
        );
        assert_eq!(
            merge_aria_descriptions(Some("Same".into()), Some("Same".into())),
            Some("Same".into())
        );
        assert_eq!(
            merge_aria_descriptions(Some("".into()), Some("Only".into())),
            Some("Only".into())
        );
    }

    #[test]
    fn interactive_row_role_maps_selectable_checked_and_action_rows() {
        assert_eq!(
            interactive_row_role(None, Role::CheckBox, false),
            Role::Button
        );
        assert_eq!(
            interactive_row_role(None, Role::CheckBox, true),
            Role::ListBoxOption
        );
        assert_eq!(
            interactive_row_role(Some(Toggled::True), Role::CheckBox, true),
            Role::CheckBox
        );
        assert_eq!(
            interactive_row_role(Some(Toggled::False), Role::Switch, false),
            Role::Switch
        );
    }
}
