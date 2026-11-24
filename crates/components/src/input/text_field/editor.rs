use gpui::StatefulInteractiveElement;
use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Edges, ElementId, Font, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Role, SharedString, Styled, TextAlign, Window, canvas, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{
    ScrollbarAxis, TestSupportExt,
    input::{
        InputAccessibility, InputBaseState, InputEditorStyle, InputModeKind, InputPresentation,
    },
};
use gpui_material_foundation::ThemeScope;
use gpui_material_theme::{MaterialTheme, TypeStyle};

use super::{
    Editor, FieldDescription, OnTextLayout, TextFieldLayout, TextFieldObfuscationMode,
    palette::FieldPalette,
};

#[derive(Default)]
pub(super) struct CharacterCountCache {
    pub(super) text: SharedString,
    pub(super) limit: Option<usize>,
    pub(super) support_text: Option<SharedString>,
    pub(super) count_label: SharedString,
    pub(super) combined_support: Option<SharedString>,
}

pub(super) struct ConfiguredEditor {
    pub(super) element: AnyElement,
    pub(super) role: Role,
    pub(super) value: Option<SharedString>,
    pub(super) set_value: Option<Editor>,
}

pub(super) struct EditorSlots {
    pub(super) leading: Option<AnyElement>,
    pub(super) prefix: Option<AnyElement>,
    pub(super) editor: AnyElement,
    pub(super) placeholder: Option<(SharedString, AnyElement)>,
    pub(super) layout_probe: Option<gpui::Canvas<()>>,
    pub(super) suffix: Option<AnyElement>,
    pub(super) trailing: Option<AnyElement>,
    pub(super) readonly_dropdown: bool,
}

fn configure_editor<M: InputModeKind>(state: &mut InputBaseState<M>, style: InputEditorStyle) {
    // The surrounding element exposes one semantic node for the Base focus
    // handle. Base still owns text layout, key dispatch, and input handling.
    state.set_accessibility(InputAccessibility::new(None).with_value_exposed(false));
    state.set_editor_style(style);
}

pub(super) fn build_layout_probe(
    id: &ElementId,
    editor: &Editor,
    on_text_layout: Option<OnTextLayout>,
    window: &mut Window,
    cx: &mut App,
) -> Option<gpui::Canvas<()>> {
    on_text_layout.map(|listener| {
        let editor = editor.clone();
        let previous = window
            .use_keyed_state((id.clone(), "field-layout"), cx, |_, _| {
                Rc::new(Cell::new(None::<TextFieldLayout>))
            })
            .read(cx)
            .clone();
        canvas(
            |_, _, _| (),
            move |_, _, window, cx| {
                let layout = editor.layout(cx);
                if previous.replace(Some(layout)) != Some(layout) {
                    window.defer(cx, move |window, cx| listener(layout, window, cx));
                }
            },
        )
        .absolute()
        .inset_0()
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn configure_and_build_editor(
    id: &ElementId,
    editor: Editor,
    obfuscation_mode: TextFieldObfuscationMode,
    presentation: &InputPresentation,
    align: TextAlign,
    style: InputEditorStyle,
    theme: &MaterialTheme,
    readonly: bool,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> ConfiguredEditor {
    match editor {
        Editor::Single(state) => {
            let role = if presentation.is_masked() {
                Role::PasswordInput
            } else {
                Role::TextInput
            };
            state.update(cx, |state, cx| {
                state.set_text_align(align, cx);
                configure_editor(state, style);
            });
            let value = (role != Role::PasswordInput).then(|| state.read(cx).value());
            let set_value = (!readonly && !disabled && role != Role::PasswordInput)
                .then(|| Editor::Single(state.clone()));
            ConfiguredEditor {
                element: state.into_any_element(),
                role,
                value,
                set_value,
            }
        }
        Editor::Multiple(state) => {
            let vertical_scrollbar_id = (id.clone(), "editor-scrollbar-v");
            let horizontal_scrollbar_id = (id.clone(), "editor-scrollbar-h");
            let scrollbar_theme = theme.clone();
            state.update(cx, |state, cx| {
                state.set_text_align(align, cx);
                configure_editor(state, style);
                state.set_scrollbar_renderer(Some(Rc::new(move |scroll, _, _| {
                    let content = match scroll.axis() {
                        ScrollbarAxis::Vertical => crate::DisplayScrollbar::new(
                            vertical_scrollbar_id.clone(),
                            gpui::Axis::Vertical,
                            &scroll,
                        )
                        .into_any_element(),
                        ScrollbarAxis::Horizontal => crate::DisplayScrollbar::new(
                            horizontal_scrollbar_id.clone(),
                            gpui::Axis::Horizontal,
                            &scroll,
                        )
                        .into_any_element(),
                        ScrollbarAxis::Both => div()
                            .absolute()
                            .inset_0()
                            .child(crate::DisplayScrollbar::new(
                                vertical_scrollbar_id.clone(),
                                gpui::Axis::Vertical,
                                &scroll,
                            ))
                            .child(crate::DisplayScrollbar::new(
                                horizontal_scrollbar_id.clone(),
                                gpui::Axis::Horizontal,
                                &scroll,
                            ))
                            .into_any_element(),
                    };
                    ThemeScope::new(content)
                        .theme(scrollbar_theme.clone())
                        .into_any_element()
                })));
            });
            let value = state.read(cx).value();
            let set_value = (!readonly && !disabled).then(|| Editor::Multiple(state.clone()));
            ConfiguredEditor {
                element: state.into_any_element(),
                role: Role::MultilineTextInput,
                value: Some(value),
                set_value,
            }
        }
        Editor::Secure(state) => {
            let masked = obfuscation_mode == TextFieldObfuscationMode::Hidden;
            state.update(cx, |state, cx| {
                if presentation.is_masked() != masked {
                    state.set_masked(masked, window, cx);
                }
                state.set_text_align(align, cx);
                configure_editor(state, style);
            });
            let role = if masked {
                Role::PasswordInput
            } else {
                Role::TextInput
            };
            let value = (!masked).then(|| state.read(cx).value());
            let set_value =
                (!readonly && !disabled && !masked).then(|| Editor::Secure(state.clone()));
            ConfiguredEditor {
                element: state.into_any_element(),
                role,
                value,
                set_value,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_editor_row(
    slots: EditorSlots,
    font: Font,
    type_style: TypeStyle,
    rtl: bool,
    show_affixes: bool,
    has_text: bool,
    palette: &FieldPalette,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .min_w_0()
        .font(font)
        .text_size(px(type_style.size))
        .line_height(px(type_style.line_height))
        .when(rtl, Styled::flex_row_reverse)
        .children(slots.leading.map(|leading| {
            div()
                .w(px(48.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .when_some(palette.state_colors.leading, Styled::text_color)
                .child(leading)
        }))
        .children(slots.prefix.filter(|_| show_affixes).map(|prefix| {
            div()
                .when_some(palette.state_colors.prefix, Styled::text_color)
                .child(prefix)
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .relative()
                .child(slots.editor)
                .children(slots.placeholder.filter(|_| show_affixes && !has_text).map(
                    |(_, content)| {
                        div()
                            .id("placeholder")
                            .test_support()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .text_color(palette.placeholder_color)
                            .role(Role::Label)
                            .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                            .child(content)
                    },
                ))
                .children(slots.layout_probe)
                // Keep the default cursor over a read-only dropdown editor's I-beam
                // without covering trailing controls such as the secondary trigger.
                .when(slots.readonly_dropdown, |slot| {
                    slot.child(div().absolute().inset_0().cursor_default())
                }),
        )
        .children(slots.suffix.filter(|_| show_affixes).map(|suffix| {
            div()
                .when_some(palette.state_colors.suffix, Styled::text_color)
                .child(suffix)
        }))
        .children(slots.trailing.map(|trailing| {
            div()
                .when_some(palette.state_colors.trailing, Styled::text_color)
                .child(trailing)
        }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_supporting_row(
    support: Option<FieldDescription>,
    count: Option<SharedString>,
    padding: Edges<Pixels>,
    caption_style: TypeStyle,
    palette: &FieldPalette,
    error: bool,
    disabled: bool,
    rtl: bool,
) -> impl IntoElement {
    div()
        .id("supporting")
        .test_support()
        .pl(padding.left)
        .pr(padding.right)
        .text_size(px(caption_style.size))
        .line_height(px(caption_style.line_height))
        .text_color(
            palette
                .state_colors
                .supporting
                .unwrap_or(if error && !disabled {
                    palette.indicator
                } else {
                    palette.secondary
                }),
        )
        .flex()
        .gap(px(8.))
        .justify_between()
        .when(rtl, Styled::flex_row_reverse)
        .on_mouse_down(MouseButton::Left, |_, window, _| {
            window.prevent_default();
        })
        .on_mouse_down(MouseButton::Right, |_, window, _| {
            window.prevent_default();
        })
        .on_mouse_down(MouseButton::Middle, |_, window, _| {
            window.prevent_default();
        })
        .child(div().flex_1().min_w_0().children(support.map(|support| {
            support
                .content
                .unwrap_or_else(|| support.text.into_any_element())
        })))
        .children(count.map(|count| div().flex_none().child(count)))
}
