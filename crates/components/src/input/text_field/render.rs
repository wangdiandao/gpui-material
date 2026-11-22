use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, Bounds, ClickEvent, ContentMask, Corners,
    Edges, ElementId, FocusHandle, Font, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    TextAlign, Window, canvas, div, point, prelude::FluentBuilder, px, quad,
};
use gpui_base::{
    ScrollbarAxis, TestSupportExt,
    input::{
        DiagnosticColors, InputAccessibility, InputBaseState, InputEditorStyle, InputModeKind,
        InputPresentation,
    },
};
use gpui_material_foundation::{
    InteractionSource, InteractionState, ThemeScope, animate, color, current_material_theme,
    font_for_style,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Density, Direction, LayoutPolicy, MaterialTheme,
    MotionKind, MotionSpeed, TypeRole, TypeStyle, TypographyEmphasis,
};

use super::{
    Editor, FieldDescription, OnTextLayout, TextField, TextFieldColorOverrides, TextFieldColors,
    TextFieldLabelAlignment, TextFieldLabelPosition, TextFieldLayout, TextFieldObfuscationMode,
    TextFieldVariant, VisibilityToggle,
};

struct FieldAnimation {
    floating: f32,
    focus_amount: f32,
    error_amount: f32,
}

#[derive(Default)]
struct CharacterCountCache {
    text: SharedString,
    limit: Option<usize>,
    support_text: Option<SharedString>,
    count_label: SharedString,
    combined_support: Option<SharedString>,
}

struct FieldPalette {
    state_colors: TextFieldColorOverrides,
    secondary: Hsla,
    foreground: Hsla,
    placeholder_color: Hsla,
    background: Hsla,
    indicator: Hsla,
    label_color: Hsla,
    outline_tint: Hsla,
}

struct FieldMetrics {
    label_size: f32,
    label_height: f32,
    outline_width: Pixels,
    padding: Edges<Pixels>,
    editor_top_padding: Pixels,
    editor_bottom_padding: Pixels,
    minimum_height: Pixels,
    label_top: f32,
}

struct ConfiguredEditor {
    element: AnyElement,
    role: Role,
    value: Option<SharedString>,
    set_value: Option<Editor>,
}

fn configure_editor<M: InputModeKind>(state: &mut InputBaseState<M>, style: InputEditorStyle) {
    // The surrounding element exposes one semantic node for the Base focus
    // handle. Base still owns text layout, key dispatch, and input handling.
    state.set_accessibility(InputAccessibility::new(None).with_value_exposed(false));
    state.set_editor_style(style);
}

fn editor_ink(mut color: Hsla) -> Hsla {
    // Base resolves zero alpha to its application palette.
    if color.a == 0. {
        color.a = f32::EPSILON;
    }
    color
}

fn interaction_state(disabled: bool, hovered: bool, focus_visible: bool) -> InteractionState {
    let mut state = InteractionState::default();
    state.disabled = disabled;
    state.hovered = hovered && !disabled;
    state.focus_visible = focus_visible && !disabled;
    state
}

fn sample_animation(
    id: &ElementId,
    theme: &MaterialTheme,
    should_float: bool,
    focused: bool,
    invalid_active: bool,
    window: &mut Window,
    cx: &mut App,
) -> FieldAnimation {
    let effects = theme
        .motion
        .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion());
    let floating = animate(
        (id.clone(), "floating-label"),
        f32::from(should_float),
        theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion()),
        window,
        cx,
    )
    .clamp(0., 1.);
    let focus_amount = animate(
        (id.clone(), "focus-indicator"),
        f32::from(focused),
        effects,
        window,
        cx,
    )
    .clamp(0., 1.);
    let error_amount = animate(
        (id.clone(), "error-indicator"),
        f32::from(invalid_active),
        effects,
        window,
        cx,
    )
    .clamp(0., 1.);
    FieldAnimation {
        floating,
        focus_amount,
        error_amount,
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_palette(
    theme: &MaterialTheme,
    colors: TextFieldColors,
    filled: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    hovered: bool,
    animation: &FieldAnimation,
) -> FieldPalette {
    use ColorRole::*;

    let tone = |role| color(theme.colors.color(role));
    let state_colors = match (disabled, error, focused) {
        (true, _, _) => colors.disabled,
        (_, true, _) => colors.invalid,
        (_, _, true) => colors.focused,
        _ => colors.unfocused,
    };
    let secondary = if disabled {
        tone(OnSurface).opacity(0.38)
    } else {
        tone(OnSurfaceVariant)
    };
    let foreground =
        state_colors
            .text
            .unwrap_or(if disabled { secondary } else { tone(OnSurface) });
    let placeholder_color = state_colors.placeholder.unwrap_or(secondary);
    let background = state_colors
        .container
        .unwrap_or_else(|| match (filled, disabled) {
            (false, _) => gpui::transparent_black(),
            (true, true) => tone(OnSurface).opacity(0.04),
            (true, false) => tone(SurfaceContainerHighest),
        });
    let animated_color = |idle, active, invalid| {
        let focused_tint = crate::motion::mix(idle, active, animation.focus_amount);
        crate::motion::mix(focused_tint, invalid, animation.error_amount)
    };
    let indicator = if disabled {
        colors
            .disabled
            .indicator
            .unwrap_or(tone(OnSurface).opacity(0.12))
    } else {
        animated_color(
            colors.unfocused.indicator.unwrap_or(tone(if filled {
                OnSurfaceVariant
            } else {
                Outline
            })),
            colors.focused.indicator.unwrap_or(tone(Primary)),
            colors.invalid.indicator.unwrap_or(tone(Error)),
        )
    };
    let label_color = if disabled {
        colors.disabled.label.unwrap_or(secondary)
    } else {
        animated_color(
            colors.unfocused.label.unwrap_or(secondary),
            colors.focused.label.unwrap_or(tone(Primary)),
            colors.invalid.label.unwrap_or(tone(Error)),
        )
    };
    let outline_tint = if hovered && !disabled && !error {
        crate::motion::mix(
            colors.hovered_indicator.unwrap_or(foreground),
            colors.focused.indicator.unwrap_or(tone(Primary)),
            animation.focus_amount,
        )
    } else {
        indicator
    };
    FieldPalette {
        state_colors,
        secondary,
        foreground,
        placeholder_color,
        background,
        indicator,
        label_color,
        outline_tint,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_editor_style(
    theme: &MaterialTheme,
    palette: &FieldPalette,
    has_text: bool,
    has_custom_placeholder: bool,
    floating: f32,
    hide_caret: bool,
    error: bool,
) -> InputEditorStyle {
    use ColorRole::*;

    let tone = |role| color(theme.colors.color(role));
    InputEditorStyle {
        foreground: editor_ink(palette.foreground),
        // Base resolves unset (zero-alpha) palette fields against its
        // application theme. Supply every palette and diagnostic field with
        // local Material roles so editor decorations stay isolated inside
        // window and subtree theme scopes.
        background: editor_ink(tone(SurfaceContainer)),
        border: editor_ink(tone(OutlineVariant)),
        muted_foreground: match (has_text, has_custom_placeholder) {
            (true, _) => editor_ink(palette.placeholder_color),
            (false, true) => editor_ink(gpui::transparent_black()),
            (false, false) => editor_ink(palette.placeholder_color.opacity(floating)),
        },
        caret: if hide_caret {
            // Base treats zero alpha as unset. A negligible nonzero alpha
            // hides the caret while the combobox retains focus.
            editor_ink(gpui::transparent_black())
        } else {
            editor_ink(palette.state_colors.caret.unwrap_or(tone(if error {
                Error
            } else {
                Primary
            })))
        },
        selection: editor_ink(
            palette
                .state_colors
                .selection
                .unwrap_or(tone(Primary).opacity(0.3)),
        ),
        diagnostics: DiagnosticColors {
            error: editor_ink(tone(Error)),
            warning: editor_ink(tone(Warning)),
            info: editor_ink(tone(Info)),
            hint: editor_ink(tone(OnSurfaceVariant)),
        },
        ..Default::default()
    }
}

fn build_layout_probe(
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
fn configure_and_build_editor(
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
            let scrollbar_id = (id.clone(), "editor-scrollbar");
            let scrollbar_theme = theme.clone();
            state.update(cx, |state, cx| {
                state.set_text_align(align, cx);
                configure_editor(state, style);
                state.set_scrollbar_renderer(Some(Rc::new(move |scroll, _, _| {
                    let content = match scroll.axis() {
                        ScrollbarAxis::Vertical => crate::DisplayScrollbar::new(
                            scrollbar_id.clone(),
                            gpui::Axis::Vertical,
                            &scroll,
                        )
                        .into_any_element(),
                        ScrollbarAxis::Horizontal => crate::DisplayScrollbar::new(
                            scrollbar_id.clone(),
                            gpui::Axis::Horizontal,
                            &scroll,
                        )
                        .into_any_element(),
                        ScrollbarAxis::Both => div()
                            .absolute()
                            .inset_0()
                            .child(crate::DisplayScrollbar::new(
                                scrollbar_id.clone(),
                                gpui::Axis::Vertical,
                                &scroll,
                            ))
                            .child(crate::DisplayScrollbar::new(
                                scrollbar_id.clone(),
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
fn resolve_metrics(
    type_style: TypeStyle,
    caption_style: TypeStyle,
    animation: &FieldAnimation,
    density: Density,
    min_height: Option<Pixels>,
    indicator_width: (Pixels, Pixels),
    content_padding: Option<Edges<Pixels>>,
    label_inside: bool,
    filled: bool,
) -> FieldMetrics {
    let label_size = type_style.size + (caption_style.size - type_style.size) * animation.floating;
    let label_height = type_style.line_height
        + (caption_style.line_height - type_style.line_height) * animation.floating;
    let (unfocused_indicator, focused_indicator) = indicator_width;
    let unfocused_w = f32::from(unfocused_indicator);
    let focused_w = f32::from(focused_indicator);
    let outline_width = px(unfocused_w + (focused_w - unfocused_w) * animation.focus_amount);
    let inside_vertical_pad = if density == Density::Compact { 6. } else { 8. };
    let padding = content_padding.unwrap_or(Edges {
        left: px(16.),
        right: px(16.),
        top: px(if label_inside {
            inside_vertical_pad
        } else {
            4.
        }),
        bottom: if label_inside && filled {
            (px(inside_vertical_pad) - outline_width).max(px(0.))
        } else {
            px(4.)
        },
    });
    let label_reserve = if label_inside {
        px(caption_style.line_height)
    } else {
        px(0.)
    };
    let editor_top_padding = padding.top + label_reserve;
    // The canvas covers the whole container, including the filled indicator.
    let editor_bottom_padding = padding.bottom + if filled { outline_width } else { px(0.) };
    let content_min_height =
        px(type_style.line_height) + editor_top_padding + editor_bottom_padding;
    let minimum_height = match min_height {
        Some(height) => height.max(content_min_height),
        None => px(density.visual_size(56.))
            .max(px(type_style.line_height + 2. * inside_vertical_pad) + label_reserve)
            .max(content_min_height),
    };
    let resting_top = f32::from(minimum_height - px(type_style.line_height)) * 0.5;
    let floating_top = if label_inside {
        f32::from(padding.top)
    } else {
        -caption_style.line_height * 0.5
    };
    let label_top = resting_top + (floating_top - resting_top) * animation.floating;
    FieldMetrics {
        label_size,
        label_height,
        outline_width,
        padding,
        editor_top_padding,
        editor_bottom_padding,
        minimum_height,
        label_top,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_outline_canvas(
    shape: CornerShape,
    direction: Direction,
    background: Hsla,
    filled: bool,
    outline_width: Pixels,
    outline_tint: Hsla,
    cutout: Option<Rc<Cell<Option<Bounds<Pixels>>>>>,
) -> gpui::Canvas<()> {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let [top_left, top_right, bottom_right, bottom_left] = shape
                .radii(
                    bounds.size.width.into(),
                    bounds.size.height.into(),
                    direction,
                )
                .map(px);
            let corners = Corners {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            };
            if background.a > 0. {
                window.paint_quad(quad(
                    bounds,
                    corners,
                    background,
                    px(0.),
                    gpui::transparent_black(),
                    BorderStyle::Solid,
                ));
            }
            if outline_width <= px(0.) {
                return;
            }
            let border = quad(
                bounds,
                corners,
                gpui::transparent_black(),
                if filled {
                    Edges {
                        bottom: outline_width,
                        ..Default::default()
                    }
                } else {
                    Edges::all(outline_width)
                },
                outline_tint,
                BorderStyle::Solid,
            );
            // The outer edge and stroke width each snap independently.
            // Account for their combined rounding when opening the notch.
            let Some(label_bounds) =
                cutout
                    .as_ref()
                    .and_then(|cutout| cutout.get())
                    .filter(|label| {
                        let device_pixel = px(1. / window.scale_factor());
                        label.top() < bounds.top() + outline_width + device_pixel
                            && label.bottom() > bounds.top() - device_pixel
                    })
            else {
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
}

struct LabelConfig {
    label: SharedString,
    label_content: Option<AnyElement>,
    label_above: bool,
    has_cutout: bool,
    required: bool,
    disabled: bool,
    rtl: bool,
    has_leading: bool,
    has_trailing: bool,
    floating: f32,
    expanded_alignment: TextFieldLabelAlignment,
    minimized_alignment: TextFieldLabelAlignment,
    label_color: Hsla,
}

fn build_label(
    config: LabelConfig,
    metrics: &FieldMetrics,
    focus: &FocusHandle,
    measure_cutout: Option<Rc<Cell<Option<Bounds<Pixels>>>>>,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let label_alignment = if config.floating >= 0.5 {
        config.minimized_alignment
    } else {
        config.expanded_alignment
    };
    let label_padding = if config.has_cutout { px(4.) } else { px(0.) };
    let label_left = (metrics.padding.left - label_padding).max(px(0.));
    let label_right = (metrics.padding.right - label_padding).max(px(0.));
    let leading_inset = if config.has_leading {
        56. * (1. - config.floating)
    } else {
        0.
    };
    let trailing_inset = if config.has_trailing {
        48. * (1. - config.floating)
    } else {
        0.
    };
    let (left_inset, right_inset) = if config.rtl {
        (trailing_inset, leading_inset)
    } else {
        (leading_inset, trailing_inset)
    };
    let label_content = config
        .label_content
        .unwrap_or_else(|| config.label.into_any_element());
    let label_container = div()
        .id("label")
        .test_support()
        .when_else(
            config.label_above,
            |label| label.pl(label_left).pr(label_right),
            |label| {
                label
                    .absolute()
                    .top(px(metrics.label_top))
                    .left(label_left + px(left_inset))
                    .right(label_right + px(right_inset))
            },
        )
        .text_size(px(metrics.label_size))
        .line_height(px(metrics.label_height))
        .text_color(config.label_color)
        .flex()
        .map(|label| match label_alignment {
            TextFieldLabelAlignment::Center => label.justify_center(),
            TextFieldLabelAlignment::Start if config.rtl => label.justify_end(),
            TextFieldLabelAlignment::End if !config.rtl => label.justify_end(),
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
                .when(config.has_cutout, |label| label.px(px(4.)))
                .role(Role::Label)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .when(config.required, |label| {
                    label
                        .flex()
                        .items_baseline()
                        .gap(px(4.))
                        .when(config.rtl, Styled::flex_row_reverse)
                })
                .child(label_content)
                .children(config.required.then_some("*"))
                .when_some(measure_cutout, |label, measure_cutout| {
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
        .when(!config.disabled, |label| {
            let focus = focus.clone();
            label.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus.focus(window, cx);
            })
        });
    if config.label_above {
        (Some(label_container.into_any_element()), None)
    } else {
        (None, Some(label_container.into_any_element()))
    }
}

struct EditorSlots {
    leading: Option<AnyElement>,
    prefix: Option<AnyElement>,
    editor: AnyElement,
    placeholder: Option<(SharedString, AnyElement)>,
    layout_probe: Option<gpui::Canvas<()>>,
    suffix: Option<AnyElement>,
    trailing: Option<AnyElement>,
}

#[allow(clippy::too_many_arguments)]
fn build_editor_row(
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
                .children(slots.layout_probe),
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
fn build_trailing_controls(
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
    tint: Hsla,
    focus_ring: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let mut items: Vec<AnyElement> = Vec::new();

    if clearable && !disabled && !readonly && has_text {
        let clear_focus = window
            .use_keyed_state((id.clone(), "clear-button-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let label = clear_label.unwrap_or_else(|| "Clear".into());
        let editor_click = editor.clone();
        let editor_a11y = editor.clone();
        let focus_click = editor_focus.clone();
        let focus_a11y = editor_focus.clone();
        let clear_btn = gpui_base::Button::new((id.clone(), "clear-button"))
            .track_focus(&clear_focus)
            .accessibility_label(label)
            .size(px(36.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .text_color(tint)
            .border_2()
            .border_color(gpui::transparent_black())
            .hover(move |style| style.bg(tint.opacity(0.08)))
            .active(move |style| style.bg(tint.opacity(0.12)))
            .focus_visible(move |style| style.border_color(focus_ring))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                editor_click.replace_all(SharedString::default(), window, cx);
                focus_click.focus(window, cx);
            })
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                editor_a11y.replace_all(SharedString::default(), window, cx);
                focus_a11y.focus(window, cx);
            })
            .child(crate::Icon::new(crate::IconName::Close).size(px(20.)));
        items.push(clear_btn.into_any_element());
    }

    if is_secure && let Some(toggle) = visibility_toggle {
        let toggle_focus = window
            .use_keyed_state((id.clone(), "visibility-toggle-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let (show_label, hide_label) =
            visibility_labels.unwrap_or_else(|| ("Show password".into(), "Hide password".into()));
        let label = if toggle.visible {
            hide_label
        } else {
            show_label
        };
        let icon = if toggle.visible {
            crate::IconName::VisibilityOff
        } else {
            crate::IconName::Visibility
        };
        let next_visible = !toggle.visible;
        let mut button = gpui_base::Button::new((id.clone(), "visibility-toggle"))
            .track_focus(&toggle_focus)
            .disabled(disabled)
            .accessibility_label(label)
            .size(px(36.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .text_color(tint)
            .border_2()
            .border_color(gpui::transparent_black())
            .focus_visible(move |style| style.border_color(focus_ring))
            .child(crate::Icon::new(icon).size(px(20.)));
        if !disabled {
            let on_toggle = toggle.on_toggle;
            let bound = Rc::new(ThemeScope::bind_action(
                move |window, cx| on_toggle(next_visible, window, cx),
                window,
                cx,
            ));
            let invoke_click = bound.clone();
            let focus_click = editor_focus.clone();
            button = button
                .hover(move |style| style.bg(tint.opacity(0.08)))
                .active(move |style| style.bg(tint.opacity(0.12)))
                .on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    invoke_click(window, cx);
                    if matches!(event, ClickEvent::Mouse(_)) {
                        focus_click.focus(window, cx);
                    }
                })
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    bound(window, cx);
                });
        }
        items.push(button.into_any_element());
    }

    if let Some(custom) = custom_trailing {
        items.push(custom);
    }

    match items.len() {
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
    }
}

#[allow(clippy::too_many_arguments)]
fn build_supporting_row(
    support: Option<FieldDescription>,
    count: Option<SharedString>,
    padding: Edges<Pixels>,
    caption_style: TypeStyle,
    palette: &FieldPalette,
    error: bool,
    disabled: bool,
    rtl: bool,
) -> gpui::Div {
    div()
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
        .child(div().flex_1().min_w_0().children(support.map(|support| {
            support
                .content
                .unwrap_or_else(|| support.text.into_any_element())
        })))
        .children(count.map(|count| div().flex_none().child(count)))
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx).clone();
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
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
            Editor::Single(state) | Editor::Secure(state) => !state.read(cx).value().is_empty(),
            Editor::Multiple(state) => !state.read(cx).value().is_empty(),
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
        let focus_ring = color(theme.colors.color(ColorRole::Secondary));
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
            trailing_tint,
            focus_ring,
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
                    has_trailing: trailing.is_some(),
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
                trailing,
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
