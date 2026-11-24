use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, App, Bounds, EntityInputHandler, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, StatefulInteractiveElement, Styled, TextAlign, Window,
    div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    InteractionSource, InteractionState, current_material_theme, font_for_style,
};
use gpui_material_theme::{CornerRole, Direction, LayoutPolicy, TypeRole, TypographyEmphasis};

use super::{
    Editor, FieldDescription, TextField, TextFieldLabelPosition, TextFieldVariant,
    controls::build_trailing_controls,
    editor::{
        CharacterCountCache, EditorSlots, build_editor_row, build_layout_probe,
        build_supporting_row, configure_and_build_editor,
    },
    label::{LabelConfig, build_label},
    metrics::resolve_metrics,
    outline::build_outline_canvas,
    palette::{build_editor_style, resolve_palette, sample_animation},
};

fn interaction_state(disabled: bool, hovered: bool, focus_visible: bool) -> InteractionState {
    let mut state = InteractionState::default();
    state.disabled = disabled;
    state.hovered = hovered && !disabled;
    state.focus_visible = focus_visible && !disabled;
    state
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx).clone();
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let component_disabled = window
            .use_keyed_state((self.id.clone(), "component-disabled"), cx, |_, _| {
                Rc::new(Cell::new(false))
            })
            .read(cx)
            .clone();
        let state_disabled = match &self.editor {
            Editor::Single(state) | Editor::Secure(state) => {
                state.read(cx).presentation().is_disabled()
            }
            Editor::Multiple(state) => state.read(cx).presentation().is_disabled(),
        };
        if self.disabled {
            if !state_disabled {
                component_disabled.set(true);
                self.editor.set_disabled(true, cx);
            }
        } else if component_disabled.replace(false) {
            self.editor.set_disabled(false, cx);
        }
        let presentation = match &self.editor {
            Editor::Single(state) | Editor::Secure(state) => state.read(cx).presentation(),
            Editor::Multiple(state) => state.read(cx).presentation(),
        };
        let disabled = self.disabled || presentation.is_disabled();
        let readonly = presentation.is_readonly();
        let focus = presentation.focus_handle().clone();
        if disabled && focus.is_focused(window) {
            let blur_focus = focus.clone();
            window.defer(cx, move |window, cx| {
                if blur_focus.is_focused(window) {
                    window.blur(cx);
                }
            });
        }
        let focused = !disabled && focus.is_focused(window);
        let filled = self.variant == TextFieldVariant::Filled;
        let label_position = self.label_position.unwrap_or(if filled {
            TextFieldLabelPosition::Inside
        } else {
            TextFieldLabelPosition::Cutout
        });
        let label_inside = self.show_label && label_position == TextFieldLabelPosition::Inside;
        let label_above = self.show_label && label_position == TextFieldLabelPosition::Above;
        let has_cutout =
            self.show_label && label_position == TextFieldLabelPosition::Cutout && !filled;
        let error = self.invalid;

        let shape = self.shape.unwrap_or(
            if filled {
                CornerRole::ExtraSmallTop
            } else {
                CornerRole::ExtraSmall
            }
            .shape(),
        );
        let type_style = LayoutPolicy::scale_text(
            theme.typography.scale_style(
                self.text_style
                    .unwrap_or(TypeRole::BodyLarge.style(TypographyEmphasis::Baseline)),
            ),
            self.font_scale,
        );
        let caption_style = LayoutPolicy::scale_text(
            theme
                .typography
                .style(TypeRole::BodySmall, TypographyEmphasis::Baseline),
            self.font_scale,
        );
        let font = font_for_style(&theme.typography, type_style, cx);
        let has_text = match &self.editor {
            Editor::Single(state) | Editor::Secure(state) => state.update(cx, |state, cx| {
                !state.value().is_empty()
                    || state
                        .marked_text_range(window, cx)
                        .is_some_and(|range| !range.is_empty())
            }),
            Editor::Multiple(state) => state.update(cx, |state, cx| {
                !state.value().is_empty()
                    || state
                        .marked_text_range(window, cx)
                        .is_some_and(|range| !range.is_empty())
            }),
        };
        let animation = sample_animation(
            &self.id,
            &theme,
            focused || has_text || self.label_minimized || label_above || !self.show_label,
            focused,
            error && !disabled,
            window,
            cx,
        );
        let effective_floating = if self.show_label {
            animation.floating
        } else {
            1.
        };
        let hover = window
            .use_keyed_state((self.id.clone(), "field-hover"), cx, |_, _| {
                Rc::new(Cell::new(false))
            })
            .read(cx)
            .clone();
        let tracker = self.interaction_source.map(|source| {
            let tracker = InteractionSource::track(&source, &self.id, window, cx);
            tracker.record(
                interaction_state(
                    disabled,
                    hover.get(),
                    focused && window.last_input_was_keyboard(),
                ),
                cx,
            );
            tracker
        });
        let palette = resolve_palette(
            &theme,
            self.colors,
            filled,
            disabled,
            error,
            focused,
            hover.get(),
            &animation,
        );
        let style = build_editor_style(
            &theme,
            &palette,
            has_text,
            self.placeholder.is_some(),
            effective_floating,
            self.dropdown_editor && readonly,
            error,
        );
        let label = self.label;
        let accessible_id = self.id.to_string();
        let align = if rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        };
        let layout_probe =
            build_layout_probe(&self.id, &self.editor, self.on_text_layout, window, cx);
        let editor_for_clear = self.editor.clone();
        let is_secure = matches!(self.editor, Editor::Secure(_));
        let configured = configure_and_build_editor(
            &self.id,
            self.editor,
            self.obfuscation_mode,
            &presentation,
            align,
            style,
            &theme,
            readonly,
            disabled,
            window,
            cx,
        );
        let support = self.error.map_or(self.supporting, |text| {
            Some(FieldDescription {
                text,
                content: None,
            })
        });
        let support_text = support.as_ref().map(|support| &support.text);
        let (count, description) =
            if let Some((limit, text)) = self.character_count.zip(configured.value.as_ref()) {
                let cache = window.use_keyed_state((self.id.clone(), "char-count"), cx, |_, _| {
                    CharacterCountCache::default()
                });
                cache.update(cx, |cache, _| {
                    let text_or_limit_changed = cache.limit != Some(limit) || &cache.text != text;
                    if text_or_limit_changed {
                        cache.text = text.clone();
                        cache.limit = Some(limit);
                        cache.count_label = format!("{} / {limit}", text.chars().count()).into();
                    }
                    if text_or_limit_changed || cache.support_text.as_ref() != support_text {
                        cache.support_text = support_text.cloned();
                        cache.combined_support = Some(match support_text {
                            Some(support) => format!("{support}. {}", cache.count_label).into(),
                            None => cache.count_label.clone(),
                        });
                    }
                    (
                        Some(cache.count_label.clone()),
                        cache.combined_support.clone(),
                    )
                })
            } else {
                (None, support_text.cloned())
            };
        let placeholder_hint = self.placeholder.as_ref().map_or_else(
            || presentation.placeholder().clone(),
            |(hint, _)| hint.clone(),
        );
        let metrics = resolve_metrics(
            type_style,
            caption_style,
            &animation,
            theme.layout.density,
            direction,
            self.min_height,
            self.indicator_width,
            self.content_padding,
            label_inside,
            filled,
        );
        let cutout = has_cutout.then(|| Rc::new(Cell::new(None::<Bounds<Pixels>>)));
        let outline_canvas = build_outline_canvas(
            shape,
            direction,
            palette.background,
            filled,
            metrics.outline_width,
            palette.outline_tint,
            cutout.clone(),
        );
        let trailing_tint = palette.state_colors.trailing.unwrap_or(palette.secondary);
        let trailing = build_trailing_controls(
            &self.id,
            &editor_for_clear,
            &focus,
            self.clearable,
            self.clear_label,
            self.visibility_toggle,
            self.visibility_labels,
            self.trailing,
            is_secure,
            has_text,
            disabled,
            readonly,
            rtl,
            theme.layout.density,
            trailing_tint,
            window,
            cx,
        );
        let (above_label, attached_label) = if self.show_label {
            build_label(
                LabelConfig {
                    label: label.clone(),
                    label_content: self.label_content,
                    label_above,
                    has_cutout,
                    required: self.required,
                    disabled,
                    rtl,
                    has_leading: self.leading.is_some(),
                    trailing_clearance: trailing.clearance,
                    floating: animation.floating,
                    expanded_alignment: self.expanded_alignment,
                    minimized_alignment: self.minimized_alignment,
                    label_color: palette.label_color,
                },
                &metrics,
                &focus,
                cutout,
            )
        } else {
            (None, None)
        };
        let show_affixes = effective_floating >= 0.5;
        let editor_row = build_editor_row(
            EditorSlots {
                leading: self.leading,
                prefix: self.prefix,
                editor: configured.element,
                placeholder: self.placeholder,
                layout_probe,
                suffix: self.suffix,
                trailing: trailing.element,
                readonly_dropdown: self.dropdown_editor && readonly,
            },
            font.clone(),
            type_style,
            rtl,
            show_affixes,
            has_text,
            &palette,
        );

        div()
            .id(self.id)
            .test_support()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .font_family(font.family)
            .role(match (self.expanded, readonly) {
                (Some(_), true) => Role::ComboBox,
                (Some(_), false) => Role::EditableComboBox,
                _ => configured.role,
            })
            .aria_label(label)
            .accessibility_id(accessible_id)
            .aria_placeholder(placeholder_hint)
            .when_some(self.expanded, |field, expanded| {
                field.aria_expanded(expanded)
            })
            .when_some(
                self.dropdown_request.filter(|_| !disabled),
                |field, request| {
                    let open = self.expanded.unwrap_or_default();
                    let click = request.clone();
                    field
                        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            click(!open, window, cx);
                        })
                        .on_a11y_action(
                            if open {
                                AccessibleAction::Collapse
                            } else {
                                AccessibleAction::Expand
                            },
                            move |_, window, cx| request(!open, window, cx),
                        )
                },
            )
            .when(!disabled, |field| field.track_focus(&focus).tab_stop(false))
            .when_some(configured.value, |field, value| field.aria_value(value))
            .when_some(configured.set_value, |field, editor| {
                field.on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(gpui::accesskit::ActionData::Value(value)) = data {
                        editor.replace_all(value.to_string().into(), window, cx);
                    }
                })
            })
            .when_some(description, |field, text| field.aria_description(text))
            .a11y_synthetic_children(move |tree| {
                let node = tree.parent_node();
                if error && !disabled {
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
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .when(!disabled && self.expanded != Some(true), |region| {
                        let blur_focus = focus.clone();
                        region.on_mouse_down_out(move |_, window, cx| {
                            if blur_focus.is_focused(window) {
                                window.blur(cx);
                            }
                        })
                    })
                    .children(above_label)
                    .child(
                        div()
                            .id("container")
                            .test_support()
                            .relative()
                            .flex()
                            .flex_col()
                            .when_else(
                                label_inside || presentation.is_multi_line(),
                                Styled::justify_end,
                                Styled::justify_center,
                            )
                            .min_h(metrics.minimum_height)
                            .pt(metrics.editor_top_padding)
                            .pr(metrics.padding.right)
                            .pb(metrics.editor_bottom_padding)
                            .pl(metrics.padding.left)
                            .when(!disabled, |field| {
                                field.on_hover(move |hovering, window, cx| {
                                    if hover.replace(*hovering) != *hovering {
                                        if let Some(tracker) = &tracker {
                                            tracker.record(
                                                interaction_state(
                                                    false,
                                                    *hovering,
                                                    focus.is_focused(window)
                                                        && window.last_input_was_keyboard(),
                                                ),
                                                cx,
                                            );
                                        }
                                        window.refresh();
                                    }
                                })
                            })
                            .child(outline_canvas)
                            .when(rtl, |field| field.text_align(TextAlign::Right))
                            .child(editor_row)
                            .children(attached_label),
                    ),
            )
            .when(support.is_some() || count.is_some(), |field| {
                field.child(build_supporting_row(
                    support,
                    count,
                    metrics.padding,
                    caption_style,
                    &palette,
                    error,
                    disabled,
                    rtl,
                ))
            })
    }
}
