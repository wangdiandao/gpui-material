use gpui::{
    AnyElement, App, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::current_material_strings;
use gpui_material_theme::Density;

use crate::{ButtonColors, ControlSize, Icon, IconButton, IconName, IconToggleButton};

use super::{Editor, VisibilityToggle};

pub(super) struct TrailingControls {
    pub(super) element: Option<AnyElement>,
    pub(super) clearance: f32,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_trailing_controls(
    id: &ElementId,
    editor: &Editor,
    editor_focus: &FocusHandle,
    clearable: bool,
    clear_label: Option<SharedString>,
    visibility_toggle: Option<VisibilityToggle>,
    visibility_labels: Option<(SharedString, SharedString)>,
    custom_trailing: Option<AnyElement>,
    is_secure: bool,
    has_text: bool,
    disabled: bool,
    readonly: bool,
    rtl: bool,
    density: Density,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> TrailingControls {
    let mut items: Vec<AnyElement> = Vec::new();
    let mut width_sum = 0.0_f32;
    let button_width = density.interactive_size(32.);

    if clearable && !disabled && !readonly && has_text {
        let clear_focus = window
            .use_keyed_state((id.clone(), "clear-button-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let label = clear_label.unwrap_or_else(|| {
            SharedString::from(current_material_strings(window, cx).clear.clone())
        });
        let editor_click = editor.clone();
        let focus_click = editor_focus.clone();
        let clear_btn = div()
            .flex_none()
            .capture_any_mouse_down(|event, window, _| {
                if event.button == MouseButton::Left {
                    window.prevent_default();
                }
            })
            .child(
                IconButton::standard(
                    (id.clone(), "clear-button"),
                    label,
                    Icon::new(IconName::Close).size(px(20.)),
                )
                .size(ControlSize::ExtraSmall)
                .track_focus(&clear_focus)
                .colors(ButtonColors {
                    content: Some(tint),
                    ..Default::default()
                })
                .on_activate(move |window, cx| {
                    editor_click.replace_all(SharedString::default(), window, cx);
                    focus_click.focus(window, cx);
                }),
            );
        items.push(clear_btn.into_any_element());
        width_sum += button_width;
    }

    if is_secure && let Some(toggle) = visibility_toggle {
        let toggle_focus = window
            .use_keyed_state((id.clone(), "visibility-toggle-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let (show_label, hide_label) = visibility_labels.unwrap_or_else(|| {
            let strings = current_material_strings(window, cx);
            (
                SharedString::from(strings.show_password.clone()),
                SharedString::from(strings.hide_password.clone()),
            )
        });
        let label = if toggle.visible {
            hide_label
        } else {
            show_label
        };
        let on_toggle = toggle.on_toggle;
        let focus_click = editor_focus.clone();
        let toggle_btn = div()
            .flex_none()
            .when(!disabled, |slot| {
                slot.capture_any_mouse_down(|event, window, _| {
                    if event.button == MouseButton::Left {
                        window.prevent_default();
                    }
                })
            })
            .child(
                IconToggleButton::standard(
                    (id.clone(), "visibility-toggle"),
                    label,
                    Icon::new(IconName::Visibility).size(px(20.)),
                    toggle.visible,
                )
                .selected_icon(Icon::new(IconName::VisibilityOff).size(px(20.)))
                .size(ControlSize::ExtraSmall)
                .track_focus(&toggle_focus)
                .disabled(disabled)
                .colors(ButtonColors {
                    content: Some(tint),
                    checked_content: Some(tint),
                    disabled_content: Some(tint),
                    ..Default::default()
                })
                .on_change(move |next_visible, window, cx| {
                    on_toggle(next_visible, window, cx);
                    if !window.last_input_was_keyboard() {
                        focus_click.focus(window, cx);
                    }
                }),
            );
        items.push(toggle_btn.into_any_element());
        width_sum += button_width;
    }

    if let Some(custom) = custom_trailing {
        items.push(custom);
        width_sum += 40.;
    }

    let count = items.len();
    let clearance = if count == 0 {
        0.
    } else {
        width_sum + 4. * (count.saturating_sub(1) as f32) + 8.
    };

    let element = match count {
        0 => None,
        1 => items.into_iter().next(),
        _ => Some(
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .when(rtl, Styled::flex_row_reverse)
                .children(items)
                .into_any_element(),
        ),
    };

    TrailingControls { element, clearance }
}
