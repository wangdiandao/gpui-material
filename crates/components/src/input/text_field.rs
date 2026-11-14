use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    TextAlign, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{
    TestSupportExt,
    input::{InputAccessibility, InputBaseState, InputEditorStyle, InputModeKind},
};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, LayoutPolicy, TypeRole, TypographyEmphasis,
};

use super::{TextAreaState, TextFieldState};

/// Material text field container treatment. The default is Filled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    #[default]
    Filled,
    Outlined,
}

/// A Material presentation for a caller-owned Base text editing entity.
///
/// Keep the entity across renders. It owns selection, undo, composition, focus,
/// and text. Subscribe to [`super::TextFieldEvent`] on the entity to observe user
/// edits. Set disabled and read-only on the entity before rendering.
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    label: SharedString,
    editor: Editor,
    variant: TextFieldVariant,
    supporting_text: Option<SharedString>,
    error: Option<SharedString>,
    trailing: Option<AnyElement>,
    expanded: Option<bool>,
    font_scale: f32,
}

enum Editor {
    Single(Entity<TextFieldState>),
    Multiple(Entity<TextAreaState>),
    Secure(Entity<TextFieldState>),
}

impl TextField {
    /// Renders a single-line editor.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Single(state.clone()))
    }

    /// Renders a multi-line editor using the row policy configured in its state.
    pub fn multiline(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextAreaState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Multiple(state.clone()))
    }

    /// Always masks the editor and suppresses its value from accessibility.
    /// Use a dedicated entity initialized with `TextFieldState::masked(true)`
    /// so the text is protected even before this field is first rendered.
    pub fn secure(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Secure(state.clone()))
    }

    fn with_editor(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        editor: Editor,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor,
            variant: TextFieldVariant::Filled,
            supporting_text: None,
            error: None,
            trailing: None,
            expanded: None,
            font_scale: 1.,
        }
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Applies the caller's user text scale to the editor, label, and supporting text.
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// Displays an error message in place of the supporting text.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self
    }

    /// Places an addon at the logical end of the editing region.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    pub(super) fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }
}

fn configure_editor<M: InputModeKind>(state: &mut InputBaseState<M>, style: InputEditorStyle) {
    // The surrounding element exposes one semantic node for the Base focus
    // handle. Base still owns text layout, key dispatch, and input handling.
    state.set_accessibility(InputAccessibility::new(None).with_value_exposed(false));
    state.set_editor_style(style);
}

type SetValue = Box<dyn Fn(SharedString, &mut Window, &mut App)>;

fn edit_handler<M: InputModeKind>(state: Entity<InputBaseState<M>>) -> SetValue {
    Box::new(move |value, window, cx| {
        state.update(cx, |state, cx| {
            if state.is_editable() {
                state.replace_all(value, window, cx);
            }
        });
    })
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use ColorRole::*;

        let theme = window_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        let direction = theme.layout.direction;
        let presentation = match &self.editor {
            Editor::Single(state) | Editor::Secure(state) => state.read(cx).presentation(),
            Editor::Multiple(state) => state.read(cx).presentation(),
        };
        let disabled = presentation.is_disabled();
        let readonly = presentation.is_readonly();
        let focus = presentation.focus_handle().clone();
        let focused = !disabled && focus.is_focused(window);
        let filled = self.variant == TextFieldVariant::Filled;
        let error = self.error.is_some();
        let indicator = match (disabled, error, focused) {
            (true, _, _) => tone(OnSurface).opacity(0.12),
            (_, true, _) => tone(Error),
            (_, _, true) => tone(Primary),
            _ if filled => tone(OnSurfaceVariant),
            _ => tone(Outline),
        };
        let foreground = if disabled {
            tone(OnSurface).opacity(0.38)
        } else {
            tone(OnSurface)
        };
        let secondary = if disabled {
            foreground
        } else {
            tone(OnSurfaceVariant)
        };
        let label_color = if focused || error {
            indicator
        } else {
            secondary
        };
        let background = if disabled {
            tone(OnSurface).opacity(0.04)
        } else {
            tone(SurfaceContainerHighest)
        };
        let radius = px(CornerRole::ExtraSmall.shape().radii(56., 56., direction)[0]);
        let type_style = LayoutPolicy::scale_text(
            TypeRole::BodyLarge.style(TypographyEmphasis::Baseline),
            self.font_scale,
        );
        let caption_style = LayoutPolicy::scale_text(
            TypeRole::BodySmall.style(TypographyEmphasis::Baseline),
            self.font_scale,
        );
        let families = theme.typography.families(type_style.typeface);
        let installed = cx.text_system().all_font_names();
        let font = std::iter::once(&families.primary)
            .chain(&families.fallback)
            .find(|family| {
                installed
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(family))
            })
            .map_or(".SystemUIFont", String::as_str)
            .to_owned();
        let style = InputEditorStyle {
            foreground,
            muted_foreground: secondary,
            caret: if error { tone(Error) } else { tone(Primary) },
            selection: tone(Primary).opacity(0.3),
            ..Default::default()
        };
        let label = self.label;
        let accessible_id = self.id.to_string();
        let align = if direction == Direction::Rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        };
        let (editor, role, value, set_value): (_, _, _, Option<SetValue>) = match self.editor {
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
                    .then(|| edit_handler(state.clone()));
                (state.into_any_element(), role, value, set_value)
            }
            Editor::Multiple(state) => {
                state.update(cx, |state, _| configure_editor(state, style));
                let value = state.read(cx).value();
                let set_value = (!readonly && !disabled).then(|| edit_handler(state.clone()));
                (
                    state.into_any_element(),
                    Role::MultilineTextInput,
                    Some(value),
                    set_value,
                )
            }
            Editor::Secure(state) => {
                state.update(cx, |state, cx| {
                    if !presentation.is_masked() {
                        state.set_masked(true, window, cx);
                    }
                    state.set_text_align(align, cx);
                    configure_editor(state, style);
                });
                (state.into_any_element(), Role::PasswordInput, None, None)
            }
        };
        let support = self.error.or(self.supporting_text);
        let label_container = div()
            .id("label")
            .text_size(px(caption_style.size))
            .line_height(px(caption_style.line_height))
            .text_color(label_color)
            .child(label.clone())
            .when(!disabled, |label| {
                let focus = focus.clone();
                label.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx);
                })
            });
        div()
            .id(self.id)
            .test_support()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_1()
            .font_family(font)
            .role(match (self.expanded, readonly) {
                (Some(_), true) => Role::ComboBox,
                (Some(_), false) => Role::EditableComboBox,
                _ => role,
            })
            .aria_label(label)
            .accessibility_id(accessible_id)
            .aria_placeholder(presentation.placeholder().clone())
            .when_some(self.expanded, |field, expanded| {
                field.aria_expanded(expanded)
            })
            .when(!disabled, |field| field.track_focus(&focus).tab_stop(false))
            .when_some(value, |field, value| field.aria_value(value))
            .when_some(set_value, |field, set_value| {
                field.on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(gpui::accesskit::ActionData::Value(value)) = data {
                        set_value(value.to_string().into(), window, cx);
                    }
                })
            })
            .when_some(support.clone(), |field, text| field.aria_description(text))
            .a11y_synthetic_children(move |tree| {
                let node = tree.parent_node();
                if error {
                    node.set_invalid(gpui::accesskit::Invalid::True);
                }
                if disabled {
                    node.set_disabled();
                }
                if readonly || disabled {
                    node.set_read_only();
                }
            })
            .child(
                div()
                    .id("container")
                    .flex()
                    .flex_col()
                    .min_h(px(56.))
                    .rounded(radius)
                    .border_color(indicator)
                    .px_4()
                    .py_1()
                    .when_else(
                        filled,
                        |field| field.bg(background).rounded_b_none().border_b_1(),
                        |field| field.border_1(),
                    )
                    .when(focused, |field| {
                        if filled {
                            field.border_b_2()
                        } else {
                            field.border_2()
                        }
                    })
                    .when(!disabled && !focused && !error, |field| {
                        field.hover(|style| style.border_color(foreground))
                    })
                    .when(direction == Direction::Rtl, |field| {
                        field.text_align(TextAlign::Right)
                    })
                    .child(label_container)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .text_size(px(type_style.size))
                            .line_height(px(type_style.line_height))
                            .when(direction == Direction::Rtl, |row| row.flex_row_reverse())
                            .child(div().flex_1().min_w_0().child(editor))
                            .children(self.trailing),
                    ),
            )
            .when_some(support, |field, text| {
                field.child(
                    div()
                        .px_4()
                        .text_size(px(caption_style.size))
                        .line_height(px(caption_style.line_height))
                        .text_color(if error && !disabled {
                            indicator
                        } else {
                            secondary
                        })
                        .child(text),
                )
            })
    }
}
