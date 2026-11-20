use std::{collections::HashMap, rc::Rc};

use gpui::{AnyView, App, AppContext, Global, Render, Window, WindowHandle, WindowId, px};
use gpui_base::{Root, Theme, ThemeAppearance};
use gpui_material_theme::{
    Appearance, ColorRole, ColorScheme, CornerRole, MaterialTheme, MotionKind, MotionSpec,
    MotionSpeed, TypeRole, TypographyEmphasis,
};
use web_time::Instant;

use crate::{color, font_for_style, typography::refresh_installed_fonts};

struct ActiveMaterialTheme(Rc<MaterialTheme>);

impl Global for ActiveMaterialTheme {}

#[derive(Default)]
struct WindowMaterialThemes(HashMap<WindowId, Rc<MaterialTheme>>);

impl Global for WindowMaterialThemes {}

struct PaletteTransition {
    from: ColorScheme,
    to: ColorScheme,
    started: Instant,
    frequency: f32,
}

impl PaletteTransition {
    fn progress(&self, now: Instant) -> f32 {
        // Slow Effects is critically damped. A retarget starts at the last
        // painted palette and gives the new target a fresh spring.
        let elapsed = now.saturating_duration_since(self.started).as_secs_f32();
        let x = self.frequency * elapsed;
        1. - (1. + x) * (-x).exp()
    }
}

#[derive(Default)]
struct ThemeAnimator {
    transition: Option<PaletteTransition>,
    driver: Option<WindowId>,
}

impl Global for ThemeAnimator {}

/// Initializes Material and Base before the application opens its first window.
pub struct MaterialRoot;

impl MaterialRoot {
    pub fn init(theme: MaterialTheme, cx: &mut App) {
        gpui_base::init(cx);
        if let Some(animator) = cx.try_global::<ThemeAnimator>()
            && animator.transition.is_some()
        {
            cx.global_mut::<ThemeAnimator>().transition = None;
        }
        refresh_installed_fonts(cx);
        apply_material_theme(theme, true, cx);
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
    window_theme_handle(window, cx)
}

pub(crate) fn window_theme_handle<'a>(window: &Window, cx: &'a App) -> &'a Rc<MaterialTheme> {
    cx.try_global::<WindowMaterialThemes>()
        .and_then(|themes| themes.0.get(&window.window_handle().window_id()))
        .unwrap_or_else(|| &cx.global::<ActiveMaterialTheme>().0)
}

/// Sets or clears the override for Material elements in this window.
///
/// Material controls pass local presentation to their Base-backed parts. This
/// does not replace Base's application-global theme: standalone Base controls,
/// scrollbars, resize handles, and caret-blink policy retain application defaults.
/// Use their explicit styling APIs for local presentation instead of swapping
/// the Base global while rendering a window.
pub fn set_window_material_theme(theme: Option<MaterialTheme>, window: &mut Window, cx: &mut App) {
    refresh_installed_fonts(cx);
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
        themes.insert(id, Rc::new(theme));
    } else {
        themes.remove(&id);
    }
    window.refresh();
}

/// Updates the application theme and projects its current values into Base.
/// Colors follow the new theme's Slow Effects spring from the displayed
/// palette. Typography and layout apply immediately.
pub fn set_material_theme(mut theme: MaterialTheme, cx: &mut App) {
    refresh_installed_fonts(cx);
    let spec = theme
        .motion
        .spec(MotionKind::Effects, MotionSpeed::Slow, cx.reduce_motion());
    let previous = cx.try_global::<ActiveMaterialTheme>();
    if let (MotionSpec::Spring(spring), Some(previous)) = (spec, previous)
        && previous.0.colors != theme.colors
        && !cx.windows().is_empty()
    {
        let shown = previous.0.colors.clone();
        let started = cx.background_executor().now();
        if !cx.has_global::<ThemeAnimator>() {
            cx.set_global(ThemeAnimator::default());
            cx.on_window_closed(|cx, closed| {
                let animator = cx.global_mut::<ThemeAnimator>();
                if animator.driver == Some(closed) {
                    animator.driver = None;
                    schedule_theme_frame(cx);
                }
            })
            .detach();
        }
        let animator = cx.global_mut::<ThemeAnimator>();
        if !animator.transition.as_ref().is_some_and(|pending| {
            pending.to == theme.colors && pending.frequency == spring.stiffness.sqrt()
        }) {
            animator.transition = Some(PaletteTransition {
                from: shown.clone(),
                to: theme.colors,
                started,
                frequency: spring.stiffness.sqrt(),
            });
        }
        theme.colors = shown;
        if animator.driver.is_none() {
            // The caller may already be updating a window. Register the
            // callback after that window is accessible again.
            cx.defer(schedule_theme_frame);
        }
    } else if cx.has_global::<ThemeAnimator>() {
        cx.global_mut::<ThemeAnimator>().transition = None;
    }
    apply_material_theme(theme, true, cx);
}

fn schedule_theme_frame(cx: &mut App) {
    let animator = cx.global::<ThemeAnimator>();
    if animator.transition.is_none() || animator.driver.is_some() {
        return;
    }
    let active = cx.active_window();
    let windows = cx.windows();
    let driver = active
        .into_iter()
        .chain(
            windows
                .iter()
                .copied()
                .filter(|handle| Some(*handle) != active),
        )
        .find(|handle| {
            handle
                .update(cx, |_, window, _| {
                    if window.is_visible() {
                        window.on_next_frame(advance_theme_frame);
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        });
    if let Some(driver) = driver {
        cx.global_mut::<ThemeAnimator>().driver = Some(driver.window_id());
    } else if let Some(transition) = cx.global_mut::<ThemeAnimator>().transition.take() {
        apply_palette(transition.to, cx);
    }
}

fn advance_theme_frame(window: &mut Window, cx: &mut App) {
    let id = window.window_handle().window_id();
    let now = cx.background_executor().now();
    let reduce_motion = cx.reduce_motion();
    let animator = cx.global_mut::<ThemeAnimator>();
    if animator.driver != Some(id) {
        return;
    }
    let Some(transition) = animator.transition.as_ref() else {
        animator.driver = None;
        return;
    };
    let progress = transition.progress(now);
    if reduce_motion || progress >= 0.999 {
        let target = animator.transition.take().unwrap().to;
        animator.driver = None;
        apply_palette(target, cx);
    } else {
        let colors = transition.from.blend(&transition.to, progress);
        apply_palette(colors, cx);
        if window.is_visible() {
            window.on_next_frame(advance_theme_frame);
        } else {
            cx.global_mut::<ThemeAnimator>().driver = None;
            cx.defer(schedule_theme_frame);
        }
    }
}

fn apply_palette(colors: ColorScheme, cx: &mut App) {
    let mut theme = material_theme(cx).clone();
    if theme.colors != colors {
        theme.colors = colors;
        apply_material_theme(theme, false, cx);
    }
}

fn apply_material_theme(theme: MaterialTheme, project_typography: bool, cx: &mut App) {
    use ColorRole::*;

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
    projected.tokens.shadow = gpui_base::ShadowTokens::elevations(role(Shadow).opacity(0.22));
    if project_typography {
        project_base_metrics(&mut projected, &theme, cx);
    }
    cx.set_global(ActiveMaterialTheme(Rc::new(theme)));
    cx.set_global(projected);
    cx.refresh_windows();
}

fn project_base_metrics(projected: &mut Theme, theme: &MaterialTheme, cx: &App) {
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
