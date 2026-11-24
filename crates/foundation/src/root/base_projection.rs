use gpui::{App, px};
use gpui_base::{ResizableTheme, ScrollbarStyles, ScrollbarTheme, Theme, ThemeAppearance};
use gpui_material_theme::{
    Appearance, ColorRole, CornerRole, MaterialTheme, TypeRole, TypographyEmphasis,
};

use crate::{color, font_for_style};

/// Projects a [`MaterialTheme`] into a matching `gpui_base::Theme` without
/// mutating the application-global Base theme.
///
/// Use this (or [`crate::current_base_theme`]) when styling standalone Base
/// controls inside a window override or [`crate::ThemeScope`].
pub fn project_base_theme(theme: &MaterialTheme, cx: &App) -> Theme {
    let mut projected = Theme::global(cx);
    project_base_colors(&mut projected, theme);
    project_base_metrics(&mut projected, theme, cx);
    projected
}

pub(super) fn project_base_colors(projected: &mut Theme, theme: &MaterialTheme) {
    use ColorRole::*;

    projected.appearance = match theme.colors.appearance() {
        Appearance::Light => ThemeAppearance::Light,
        Appearance::Dark => ThemeAppearance::Dark,
    };
    let role = |role| color(theme.colors.color(role));
    let colors = &mut projected.tokens.colors;
    colors.background = role(Surface);
    colors.foreground = role(OnSurface);
    colors.surface = role(SurfaceContainer);
    colors.surface_foreground = role(OnSurface);
    colors.primary = role(Primary);
    colors.primary_foreground = role(OnPrimary);
    colors.secondary = role(Secondary);
    colors.secondary_foreground = role(OnSecondary);
    colors.muted = role(SurfaceContainerHighest);
    colors.muted_foreground = role(OnSurfaceVariant);
    colors.accent = role(SecondaryContainer);
    colors.accent_foreground = role(OnSecondaryContainer);
    colors.destructive = role(Error);
    colors.destructive_foreground = role(OnError);
    colors.border = role(OutlineVariant);
    colors.input = role(Outline);
    colors.ring = role(Secondary);
    colors.selection = role(Primary).opacity(0.3);
    projected.tokens.shadow = gpui_base::ShadowTokens::elevations(role(Shadow).opacity(0.22));
    let thumb = role(OnSurfaceVariant);
    let styles = ScrollbarStyles::default()
        .track(|track| track.bg(gpui::transparent_black()).width(px(8.)))
        .thumb(|t| {
            t.bg(thumb.opacity(0.38))
                .width(px(4.))
                .radius(px(2.))
                .min_length(px(24.))
        })
        .thumb_hover(|t| t.bg(thumb.opacity(0.56)))
        .thumb_active(|t| t.bg(thumb.opacity(0.72)));
    projected.scrollbar = ScrollbarTheme::new()
        .with_mode(projected.scrollbar.mode())
        .with_motion(projected.scrollbar.motion())
        .with_styles(styles);
    let mut resizable = ResizableTheme::default();
    resizable.handle = Some(role(OutlineVariant));
    resizable.active_handle = Some(role(Primary));
    projected.resizable = resizable;
}

pub(super) fn project_base_metrics(projected: &mut Theme, theme: &MaterialTheme, cx: &App) {
    let typography = &theme.typography;
    let radii = &mut projected.tokens.radius;
    let radius = |role: CornerRole| px(role.shape().radii(100., 100., Default::default())[0]);
    radii.sm = radius(CornerRole::ExtraSmall);
    radii.md = radius(CornerRole::Small);
    radii.lg = radius(CornerRole::Medium);
    radii.xl = radius(CornerRole::Large);
    let text = &mut projected.tokens.typography;
    text.sans = font_for_style(
        typography,
        TypeRole::BodyMedium.style(TypographyEmphasis::Baseline),
        cx,
    )
    .family;
    for (slot, role) in [
        (&mut text.xs, TypeRole::BodySmall),
        (&mut text.sm, TypeRole::BodyMedium),
        (&mut text.md, TypeRole::BodyLarge),
        (&mut text.lg, TypeRole::TitleMedium),
        (&mut text.xl, TypeRole::TitleLarge),
    ] {
        let style = typography.style(role, TypographyEmphasis::Baseline);
        slot.size = px(style.size);
        slot.line_height = px(style.line_height);
        slot.weight = gpui::FontWeight(f32::from(style.weight));
    }
}
