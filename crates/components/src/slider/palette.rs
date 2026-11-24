use gpui::{App, Hsla, Window};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::ColorRole;

use super::SliderColors;

#[derive(Clone, Copy)]
pub(super) struct SliderPalette {
    pub(super) active: Hsla,
    pub(super) inactive: Hsla,
    pub(super) active_tick: Hsla,
    pub(super) inactive_tick: Hsla,
    pub(super) thumb: Hsla,
    pub(super) ring: Hsla,
    pub(super) active_icon: Hsla,
    pub(super) inactive_icon: Hsla,
    pub(super) disabled_icon: Hsla,
    pub(super) inverse_surface: Hsla,
    pub(super) inverse_on_surface: Hsla,
}

pub(super) fn resolve_palette(
    colors: SliderColors,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> SliderPalette {
    let theme = current_material_theme(window, cx);
    let scheme = &theme.colors;
    let resolve = |role| color(scheme.color(role));
    let muted = resolve(ColorRole::OnSurface);
    let (active, inactive, active_tick, inactive_tick, thumb) = if disabled {
        (
            colors
                .disabled_active_track
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors
                .disabled_inactive_track
                .unwrap_or_else(|| muted.opacity(0.12)),
            colors
                .disabled_active_tick
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors
                .disabled_inactive_tick
                .unwrap_or_else(|| muted.opacity(0.38)),
            colors.disabled_thumb.unwrap_or_else(|| muted.opacity(0.38)),
        )
    } else {
        (
            colors
                .active_track
                .unwrap_or_else(|| resolve(ColorRole::Primary)),
            colors
                .inactive_track
                .unwrap_or_else(|| resolve(ColorRole::SecondaryContainer)),
            colors
                .active_tick
                .unwrap_or_else(|| resolve(ColorRole::OnPrimary)),
            colors
                .inactive_tick
                .unwrap_or_else(|| resolve(ColorRole::OnSecondaryContainer)),
            colors.thumb.unwrap_or_else(|| resolve(ColorRole::Primary)),
        )
    };
    SliderPalette {
        active,
        inactive,
        active_tick,
        inactive_tick,
        thumb,
        ring: resolve(ColorRole::Secondary),
        active_icon: resolve(ColorRole::OnPrimary),
        inactive_icon: resolve(ColorRole::OnSecondaryContainer),
        disabled_icon: muted.opacity(0.38),
        inverse_surface: resolve(ColorRole::InverseSurface),
        inverse_on_surface: resolve(ColorRole::InverseOnSurface),
    }
}
