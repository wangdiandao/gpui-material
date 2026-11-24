use gpui::{App, ElementId, Hsla, Window};
use gpui_base::input::{DiagnosticColors, InputEditorStyle};
use gpui_material_foundation::{animate, color};
use gpui_material_theme::{ColorRole, MaterialTheme, MotionKind, MotionSpeed};

use crate::control_style::mix_color;

use super::{TextFieldColorOverrides, TextFieldColors};

pub(super) struct FieldAnimation {
    pub(super) floating: f32,
    pub(super) focus_amount: f32,
    pub(super) error_amount: f32,
}

pub(super) struct FieldPalette {
    pub(super) state_colors: TextFieldColorOverrides,
    pub(super) secondary: Hsla,
    pub(super) foreground: Hsla,
    pub(super) placeholder_color: Hsla,
    pub(super) background: Hsla,
    pub(super) indicator: Hsla,
    pub(super) label_color: Hsla,
    pub(super) outline_tint: Hsla,
}

fn editor_ink(mut color: Hsla) -> Hsla {
    // Base resolves zero alpha to its application palette.
    if color.a == 0. {
        color.a = f32::EPSILON;
    }
    color
}

pub(super) fn sample_animation(
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
pub(super) fn resolve_palette(
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
        let focused_tint = mix_color(idle, active, animation.focus_amount);
        mix_color(focused_tint, invalid, animation.error_amount)
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
        mix_color(
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
pub(super) fn build_editor_style(
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
