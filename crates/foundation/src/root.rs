use std::collections::HashMap;

use gpui::{AnyView, App, AppContext, Global, Render, Window, WindowHandle, WindowId, px};
use gpui_base::{Root, Theme, ThemeAppearance};
use gpui_material_theme::{
    Appearance, ColorRole, CornerRole, MaterialTheme, TypeRole, TypographyEmphasis,
};

use crate::{color, typography::font_for_style};

struct ActiveMaterialTheme(MaterialTheme);

impl Global for ActiveMaterialTheme {}

#[derive(Default)]
struct WindowMaterialThemes(HashMap<WindowId, MaterialTheme>);

impl Global for WindowMaterialThemes {}

/// Initializes Material and Base before the application opens its first window.
pub struct MaterialRoot;

impl MaterialRoot {
    pub fn init(theme: MaterialTheme, cx: &mut App) {
        gpui_base::init(cx);
        set_material_theme(theme, cx);
    }

    /// Opens a window whose Base root owns overlays and keyboard traversal.
    pub fn open_window<V: Render>(
        options: gpui::WindowOptions,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut App) -> gpui::Entity<V>,
    ) -> gpui::Result<WindowHandle<Root>> {
        cx.open_window(options, |window, cx| {
            let view = AnyView::from(build(window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
    }
}

/// Returns the shared Material theme installed by [`MaterialRoot::init`].
pub fn material_theme(cx: &App) -> &MaterialTheme {
    &cx.global::<ActiveMaterialTheme>().0
}

/// Resolves a window's override or falls back to the current application theme.
pub fn window_material_theme<'a>(window: &Window, cx: &'a App) -> &'a MaterialTheme {
    cx.try_global::<WindowMaterialThemes>()
        .and_then(|themes| themes.0.get(&window.window_handle().window_id()))
        .unwrap_or_else(|| material_theme(cx))
}

/// Sets or clears the override for Material elements in this window.
pub fn set_window_material_theme(theme: Option<MaterialTheme>, window: &mut Window, cx: &mut App) {
    if !cx.has_global::<WindowMaterialThemes>() {
        cx.set_global(WindowMaterialThemes::default());
        cx.on_window_closed(|cx, id| {
            cx.global_mut::<WindowMaterialThemes>().0.remove(&id);
        })
        .detach();
    }
    let id = window.window_handle().window_id();
    let themes = &mut cx.global_mut::<WindowMaterialThemes>().0;
    if let Some(theme) = theme {
        themes.insert(id, theme);
    } else {
        themes.remove(&id);
    }
    window.refresh();
}

/// Commits the application theme and projects semantic defaults into Base before
/// refreshing windows. A window or subtree can pass an explicit theme to its elements.
pub fn set_material_theme(theme: MaterialTheme, cx: &mut App) {
    use ColorRole::*;

    let typography = &theme.typography;
    let sans = font_for_style(
        typography,
        TypeRole::BodyMedium.style(TypographyEmphasis::Baseline),
        cx,
    )
    .family;
    let mut projected = Theme::global(cx);
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
    let radii = &mut projected.tokens.radius;
    let radius = |role: CornerRole| px(role.shape().radii(100., 100., Default::default())[0]);
    radii.sm = radius(CornerRole::ExtraSmall);
    radii.md = radius(CornerRole::Small);
    radii.lg = radius(CornerRole::Medium);
    radii.xl = radius(CornerRole::Large);
    let text = &mut projected.tokens.typography;
    text.sans = sans;
    for (slot, role) in [
        (&mut text.xs, TypeRole::BodySmall),
        (&mut text.sm, TypeRole::BodyMedium),
        (&mut text.md, TypeRole::BodyLarge),
        (&mut text.lg, TypeRole::TitleMedium),
        (&mut text.xl, TypeRole::TitleLarge),
    ] {
        let style = role.style(TypographyEmphasis::Baseline);
        slot.size = px(style.size);
        slot.line_height = px(style.line_height);
        slot.weight = gpui::FontWeight(f32::from(style.weight));
    }
    projected.tokens.shadow = gpui_base::ShadowTokens::elevations(role(Shadow).opacity(0.22));

    cx.set_global(ActiveMaterialTheme(theme));
    cx.set_global(projected);
    cx.refresh_windows();
}
