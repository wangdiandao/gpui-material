use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, ContentMask, ElementId, Entity,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, TextAlign, Window, canvas, div, point,
    prelude::FluentBuilder, px, quad,
};
use gpui_base::{
    TestSupportExt,
    input::{InputAccessibility, InputBaseState, InputEditorStyle, InputModeKind},
};
use gpui_material_foundation::{animate, color, font_for_style, window_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, LayoutPolicy, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
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

        let theme = window_material_theme(window, cx).clone();
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
        let font = font_for_style(&theme.typography, type_style, cx).family;
        let has_text = match &self.editor {
            Editor::Single(state) | Editor::Secure(state) => !state.read(cx).value().is_empty(),
            Editor::Multiple(state) => !state.read(cx).value().is_empty(),
        };
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion());
        let floating = animate(
            (self.id.clone(), "floating-label"),
            f32::from(focused || has_text),
            theme
                .motion
                .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion()),
            window,
            cx,
        )
        .clamp(0., 1.);
        let focus_amount = animate(
            (self.id.clone(), "focus-indicator"),
            f32::from(focused),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let error_amount = animate(
            (self.id.clone(), "error-indicator"),
            f32::from(error && !disabled),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let indicator = if disabled {
            indicator
        } else {
            let idle = tone(if filled { OnSurfaceVariant } else { Outline });
            let active = crate::motion::mix(idle, tone(Primary), focus_amount);
            crate::motion::mix(active, tone(Error), error_amount)
        };
        let label_color = if disabled {
            label_color
        } else {
            let active = crate::motion::mix(secondary, tone(Primary), focus_amount);
            crate::motion::mix(active, tone(Error), error_amount)
        };
        let style = InputEditorStyle {
            foreground,
            muted_foreground: if has_text {
                secondary
            } else {
                // Base treats exactly transparent ink as an unset palette override.
                secondary.opacity(floating.max(f32::EPSILON))
            },
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
        let label_size = type_style.size + (caption_style.size - type_style.size) * floating;
        let label_height = type_style.line_height
            + (caption_style.line_height - type_style.line_height) * floating;
        let resting_top = (56. - type_style.line_height) * 0.5;
        let floating_top = if filled {
            4.
        } else {
            -caption_style.line_height * 0.5
        };
        let cutout = Rc::new(Cell::new(None));
        let measure_cutout = cutout.clone();
        let label_top = resting_top + (floating_top - resting_top) * floating;
        let cut_outline = !filled && floating > 0. && label_top < 1. + focus_amount;
        let outline_width = px(1. + focus_amount);
        let hover = window
            .use_keyed_state((self.id.clone(), "field-hover"), cx, |_, _| {
                Rc::new(Cell::new(false))
            })
            .read(cx)
            .clone();
        let outline_tint = if hover.get() && !disabled && !error {
            crate::motion::mix(foreground, tone(Primary), focus_amount)
        } else {
            indicator
        };
        let outline_canvas = (!filled).then(|| {
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let border = quad(
                        bounds,
                        radius,
                        gpui::transparent_black(),
                        outline_width,
                        outline_tint,
                        BorderStyle::Solid,
                    );
                    let Some(label_bounds) = cutout.get() else {
                        window.paint_quad(border);
                        return;
                    };
                    let label_bounds: Bounds<gpui::Pixels> = label_bounds;
                    let label_center = label_bounds.center().x;
                    let half_cut = label_bounds.size.width * floating / 2.;
                    let left = (label_center - half_cut).max(bounds.left());
                    let right = (label_center + half_cut).min(bounds.right());
                    for (min, max) in [
                        (bounds.origin, point(left, bounds.bottom())),
                        (point(right, bounds.top()), bounds.bottom_right()),
                        (
                            point(left, bounds.top() + outline_width),
                            point(right, bounds.bottom()),
                        ),
                    ] {
                        window.with_content_mask(
                            Some(ContentMask {
                                bounds: Bounds::from_corners(min, max),
                            }),
                            |window| window.paint_quad(border.clone()),
                        );
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        let label_container = div()
            .id("label")
            .test_support()
            .absolute()
            .top(px(label_top))
            .left(px(16.))
            .right(px(16.))
            .text_size(px(label_size))
            .line_height(px(label_height))
            .text_color(label_color)
            .flex()
            .when(direction == Direction::Rtl, Styled::justify_end)
            .child(
                div()
                    .min_w_0()
                    .max_w_full()
                    .relative()
                    .truncate()
                    .when(!filled, |label| label.px_1())
                    .child(label.clone())
                    .when(cut_outline, |label| {
                        label.child(
                            canvas(
                                move |bounds, _, _| measure_cutout.set(Some(bounds)),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        )
                    }),
            )
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
                    .relative()
                    .flex()
                    .flex_col()
                    .when_else(
                        filled || presentation.is_multi_line(),
                        Styled::justify_end,
                        Styled::justify_center,
                    )
                    .min_h(px(56.).max(px(type_style.line_height + 16.)))
                    .rounded(radius)
                    .border_color(indicator)
                    .px_4()
                    .py_1()
                    .when(filled, |mut field| {
                        field.style().border_widths.bottom = Some(outline_width.into());
                        field
                            .bg(background)
                            .rounded_b_none()
                            .when(!disabled && !focused && !error, |field| {
                                field.hover(|style| style.border_color(foreground))
                            })
                    })
                    .when(!filled && !disabled, |field| {
                        field.on_hover(move |hovering, window, _| {
                            if hover.replace(*hovering) != *hovering {
                                window.refresh();
                            }
                        })
                    })
                    .children(outline_canvas)
                    .when(direction == Direction::Rtl, |field| {
                        field.text_align(TextAlign::Right)
                    })
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
                    )
                    .child(label_container),
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
