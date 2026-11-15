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

/// Position of the visual label. Filled uses Inside and Outlined uses Cutout by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFieldLabelPosition {
    /// The minimized label remains within the container, above the editor.
    Inside,
    /// The minimized label crosses the container's top edge.
    Cutout,
    /// The label is always minimized, with its own row above the container.
    Above,
}

/// Horizontal label alignment. Start and End follow the Material theme direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldLabelAlignment {
    #[default]
    Start,
    Center,
    End,
}

struct FieldDescription {
    text: SharedString,
    content: Option<AnyElement>,
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
    label_content: Option<AnyElement>,
    label_position: Option<TextFieldLabelPosition>,
    label_minimized: bool,
    expanded_alignment: TextFieldLabelAlignment,
    minimized_alignment: TextFieldLabelAlignment,
    placeholder: Option<(SharedString, AnyElement)>,
    supporting: Option<FieldDescription>,
    error: Option<SharedString>,
    invalid: bool,
    required: bool,
    character_count: Option<usize>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
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
            label_content: None,
            label_position: None,
            label_minimized: false,
            expanded_alignment: TextFieldLabelAlignment::Start,
            minimized_alignment: TextFieldLabelAlignment::Start,
            placeholder: None,
            supporting: None,
            error: None,
            invalid: false,
            required: false,
            character_count: None,
            leading: None,
            trailing: None,
            prefix: None,
            suffix: None,
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
        self.supporting = Some(FieldDescription {
            text: text.into(),
            content: None,
        });
        self
    }

    /// Replaces the visible label. The constructor label remains the accessible name.
    pub fn label_content(mut self, element: impl IntoElement) -> Self {
        self.label_content = Some(element.into_any_element());
        self
    }

    pub fn label_position(mut self, position: TextFieldLabelPosition) -> Self {
        self.label_position = Some(position);
        self
    }

    /// Keeps an Inside or Cutout label minimized so affixes and hints can remain visible.
    pub fn label_always_minimized(mut self, minimized: bool) -> Self {
        self.label_minimized = minimized;
        self
    }

    /// Configures the alignment in each label state. Above uses `minimized`.
    pub fn label_alignment(
        mut self,
        expanded: TextFieldLabelAlignment,
        minimized: TextFieldLabelAlignment,
    ) -> Self {
        self.expanded_alignment = expanded;
        self.minimized_alignment = minimized;
        self
    }

    /// Replaces the Base placeholder's presentation. `hint` is the accessible placeholder.
    pub fn placeholder_content(
        mut self,
        hint: impl Into<SharedString>,
        element: impl IntoElement,
    ) -> Self {
        self.placeholder = Some((hint.into(), element.into_any_element()));
        self
    }

    /// Supplies custom supporting content and the field's plain-text accessible description.
    pub fn supporting_content(
        mut self,
        description: impl Into<SharedString>,
        element: impl IntoElement,
    ) -> Self {
        self.supporting = Some(FieldDescription {
            text: description.into(),
            content: Some(element.into_any_element()),
        });
        self
    }

    /// Displays an error message in place of the supporting text.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self.invalid = true;
        self
    }

    /// Sets the invalid visual and accessibility state independently of the supporting content.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Displays a required mark and reports the required state to assistive technology.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Shows a Unicode scalar count and the supplied limit. Base still owns validation.
    /// The count is omitted for secure or masked fields.
    pub fn character_count(mut self, limit: usize) -> Self {
        self.character_count = Some(limit);
        self
    }

    /// Places an adornment in the 48 dp leading icon slot.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Places an addon at the logical end of the editing region.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Places an element before the value while the label is minimized.
    pub fn prefix(mut self, element: impl IntoElement) -> Self {
        self.prefix = Some(element.into_any_element());
        self
    }

    /// Places an element after the value while the label is minimized.
    pub fn suffix(mut self, element: impl IntoElement) -> Self {
        self.suffix = Some(element.into_any_element());
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
        let label_position = self.label_position.unwrap_or(if filled {
            TextFieldLabelPosition::Inside
        } else {
            TextFieldLabelPosition::Cutout
        });
        let label_inside = label_position == TextFieldLabelPosition::Inside;
        let label_above = label_position == TextFieldLabelPosition::Above;
        let has_cutout = label_position == TextFieldLabelPosition::Cutout && !filled;
        let error = self.invalid;
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
        let label_color = if !disabled && (focused || error) {
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
            f32::from(focused || has_text || self.label_minimized || label_above),
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
            // Base resolves unset palette fields against its application theme.
            // Supply every palette field so editor decorations also stay local
            // when another window uses a different Material appearance.
            background: tone(SurfaceContainer),
            border: tone(OutlineVariant),
            muted_foreground: match (has_text, self.placeholder.is_some()) {
                (true, _) => secondary,
                // Base treats exactly transparent ink as an unset palette override.
                (false, true) => secondary.opacity(f32::EPSILON),
                (false, false) => secondary.opacity(floating.max(f32::EPSILON)),
            },
            caret: if self.expanded.is_some() && readonly {
                // Base treats zero alpha as unset. A negligible nonzero alpha
                // hides the caret while the combobox retains focus.
                tone(Primary).opacity(f32::EPSILON)
            } else {
                tone(if error { Error } else { Primary })
            },
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
                state.update(cx, |state, cx| {
                    state.set_text_align(align, cx);
                    configure_editor(state, style);
                });
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
        let support = self.error.map_or(self.supporting, |text| {
            Some(FieldDescription {
                text,
                content: None,
            })
        });
        let count: Option<SharedString> = self
            .character_count
            .zip(value.as_ref())
            .map(|(limit, text)| format!("{} / {limit}", text.chars().count()).into());
        let description = match (support.as_ref(), count.as_ref()) {
            (Some(support), Some(count)) => Some(format!("{}. {count}", support.text).into()),
            (Some(support), None) => Some(support.text.clone()),
            (None, count) => count.cloned(),
        };
        let placeholder_hint = self.placeholder.as_ref().map_or_else(
            || presentation.placeholder().clone(),
            |(hint, _)| hint.clone(),
        );
        let label_size = type_style.size + (caption_style.size - type_style.size) * floating;
        let label_height = type_style.line_height
            + (caption_style.line_height - type_style.line_height) * floating;
        let resting_top = (56. - type_style.line_height) * 0.5;
        let floating_top = if label_inside {
            8.
        } else {
            -caption_style.line_height * 0.5
        };
        let cutout = Rc::new(Cell::new(None::<Bounds<gpui::Pixels>>));
        let measure_cutout = cutout.clone();
        let label_top = resting_top + (floating_top - resting_top) * floating;
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
                    // The outer edge and stroke width each snap independently.
                    // Account for their combined rounding when opening the notch.
                    let device_pixel = px(1. / window.scale_factor());
                    let Some(label_bounds) = cutout.get().filter(|label| {
                        label.top() < bounds.top() + outline_width + device_pixel
                            && label.bottom() > bounds.top() - device_pixel
                    }) else {
                        window.paint_quad(border);
                        return;
                    };
                    // The missing stroke must clear the complete label in each
                    // frame that overlaps it, including when focus is leaving.
                    let left = label_bounds.left().max(bounds.left());
                    let right = label_bounds.right().min(bounds.right());
                    for (min, max) in [
                        (bounds.origin, point(left, bounds.bottom())),
                        (point(right, bounds.top()), bounds.bottom_right()),
                        // Only the bottom edge must be restored between the two
                        // side masks. Start in the interior, so outward clip
                        // rounding cannot expose a snapped top-stroke pixel.
                        (
                            point(left, bounds.center().y),
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
        let rtl = direction == Direction::Rtl;
        let label_alignment = if floating >= 0.5 {
            self.minimized_alignment
        } else {
            self.expanded_alignment
        };
        let label_inset = if has_cutout { 12. } else { 16. };
        let leading_inset = if self.leading.is_some() {
            56. * (1. - floating)
        } else {
            0.
        };
        let label_content = self
            .label_content
            .unwrap_or_else(|| label.clone().into_any_element());
        let label_container = div()
            .id("label")
            .test_support()
            .when_else(
                label_above,
                |label| label.px(px(label_inset)),
                |label| {
                    label
                        .absolute()
                        .top(px(label_top))
                        .left(px(label_inset + if rtl { 0. } else { leading_inset }))
                        .right(px(label_inset + if rtl { leading_inset } else { 0. }))
                },
            )
            .text_size(px(label_size))
            .line_height(px(label_height))
            .text_color(label_color)
            .flex()
            .map(|label| match label_alignment {
                TextFieldLabelAlignment::Center => label.justify_center(),
                TextFieldLabelAlignment::Start if rtl => label.justify_end(),
                TextFieldLabelAlignment::End if !rtl => label.justify_end(),
                _ => label,
            })
            .child(
                div()
                    .id("label-text")
                    .test_support()
                    .min_w_0()
                    .max_w_full()
                    .relative()
                    .truncate()
                    .when(has_cutout, |label| label.px(px(4.)))
                    .role(Role::Label)
                    .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                    .when(self.required, |label| {
                        label
                            .flex()
                            .items_baseline()
                            .gap(px(4.))
                            .when(rtl, Styled::flex_row_reverse)
                    })
                    .child(label_content)
                    .children(self.required.then_some("*"))
                    .when(has_cutout, |label| {
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
        let (above_label, attached_label) = if label_above {
            (Some(label_container), None)
        } else {
            (None, Some(label_container))
        };
        let show_affixes = floating >= 0.5;
        div()
            .id(self.id)
            .test_support()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .font_family(font)
            .role(match (self.expanded, readonly) {
                (Some(_), true) => Role::ComboBox,
                (Some(_), false) => Role::EditableComboBox,
                _ => role,
            })
            .aria_label(label)
            .accessibility_id(accessible_id)
            .aria_placeholder(placeholder_hint)
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
            .when_some(description, |field, text| field.aria_description(text))
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
                if self.required {
                    node.set_required();
                }
            })
            .children(above_label)
            .child(
                div()
                    .id("container")
                    .relative()
                    .flex()
                    .flex_col()
                    .when_else(
                        label_inside || presentation.is_multi_line(),
                        Styled::justify_end,
                        Styled::justify_center,
                    )
                    .min_h(px(56.).max(px(type_style.line_height
                        + if label_inside {
                            caption_style.line_height + 16.
                        } else {
                            16.
                        })))
                    .rounded(radius)
                    .border_color(indicator)
                    .px(px(16.))
                    .py(px(4.))
                    .when(filled, |mut field| {
                        // Compose reserves 8 dp above the minimized label and below
                        // the editor; larger text grows the container instead of overlapping.
                        field = field.when(label_inside, |field| {
                            field
                                .pt(px(caption_style.line_height + 8.))
                                .pb(px(8.) - outline_width)
                        });
                        field.style().border_widths.bottom = Some(outline_width.into());
                        field
                            .bg(background)
                            .rounded_b_none()
                            .when(!disabled && !focused && !error, |field| {
                                field.hover(|style| style.border_color(foreground))
                            })
                    })
                    .when(label_inside && !filled, |field| {
                        field.pt(px(caption_style.line_height + 8.))
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
                            .gap(px(8.))
                            .min_w_0()
                            .text_size(px(type_style.size))
                            .line_height(px(type_style.line_height))
                            .when(rtl, Styled::flex_row_reverse)
                            .children(self.leading.map(|leading| {
                                div()
                                    .w(px(48.))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(leading)
                            }))
                            .children(self.prefix.filter(|_| show_affixes))
                            .child(div().flex_1().min_w_0().relative().child(editor).children(
                                self.placeholder.filter(|_| show_affixes && !has_text).map(
                                    |(_, content)| {
                                        div()
                                            .id("placeholder")
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .right_0()
                                            .text_color(secondary)
                                            .role(Role::Label)
                                            .a11y_synthetic_children(|tree| {
                                                tree.parent_node().set_hidden()
                                            })
                                            .child(content)
                                    },
                                ),
                            ))
                            .children(self.suffix.filter(|_| show_affixes))
                            .children(self.trailing),
                    )
                    .children(attached_label),
            )
            .when(support.is_some() || count.is_some(), |field| {
                field.child(
                    div()
                        .px(px(16.))
                        .text_size(px(caption_style.size))
                        .line_height(px(caption_style.line_height))
                        .text_color(if error && !disabled {
                            indicator
                        } else {
                            secondary
                        })
                        .flex()
                        .gap(px(8.))
                        .justify_between()
                        .when(rtl, Styled::flex_row_reverse)
                        .child(div().flex_1().min_w_0().children(support.map(|support| {
                            support
                                .content
                                .unwrap_or_else(|| support.text.into_any_element())
                        })))
                        .children(count.map(|count| div().flex_none().child(count))),
                )
            })
    }
}
