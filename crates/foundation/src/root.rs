use std::{collections::HashMap, rc::Rc, time::Duration};

use gpui::{
    AnyView, App, AppContext, Global, Render, SpringConfig, SpringState, Window, WindowHandle,
    WindowId, px,
};
use gpui_base::{
    ResizableTheme, Root, ScrollbarStyles, ScrollbarTheme, Theme, ThemeAppearance, motion::Easing,
};
use gpui_material_theme::{
    Appearance, ColorRole, ColorScheme, CornerRole, MaterialTheme, MotionKind, MotionSpec,
    MotionSpeed, TypeRole, TypographyEmphasis,
};
use web_time::Instant;

use crate::{
    color, font_for_style,
    motion::{RetargetTransition, spring_config},
    typography::refresh_installed_fonts,
};

struct ActiveMaterialTheme(Rc<MaterialTheme>);

impl Global for ActiveMaterialTheme {}

#[derive(Default)]
struct WindowMaterialThemes(HashMap<WindowId, Rc<MaterialTheme>>);

impl Global for WindowMaterialThemes {}

enum PaletteCurve {
    Spring {
        config: SpringConfig,
        state: SpringState,
        updated_at: Instant,
    },
    Tween(RetargetTransition),
}

struct PaletteTransition {
    from: ColorScheme,
    to: ColorScheme,
    curve: PaletteCurve,
    spec: MotionSpec,
}

impl PaletteTransition {
    fn new(from: ColorScheme, to: ColorScheme, spec: MotionSpec, now: Instant) -> Option<Self> {
        let curve = match spec {
            MotionSpec::Instant => return None,
            MotionSpec::Spring(spring) => PaletteCurve::Spring {
                config: spring_config(spring)?,
                state: SpringState {
                    position: 0.0,
                    velocity: 0.0,
                },
                updated_at: now,
            },
            MotionSpec::Tween {
                duration_ms,
                easing,
            } => {
                let duration = Duration::from_millis(u64::from(duration_ms));
                let curve =
                    Easing::cubic_bezier(easing.x1, easing.y1, easing.x2, easing.y2).ok()?;
                if duration.is_zero() {
                    return None;
                }
                let mut transition = RetargetTransition::settled(0.0);
                transition.animate_to(1.0, duration, curve, now, false);
                PaletteCurve::Tween(transition)
            }
        };
        Some(Self {
            from,
            to,
            curve,
            spec,
        })
    }

    fn advance(&mut self, now: Instant) -> (f32, bool) {
        match &mut self.curve {
            PaletteCurve::Spring {
                config,
                state,
                updated_at,
            } => {
                let elapsed = now.saturating_duration_since(*updated_at).as_secs_f32();
                let stepped = config.step(*state, 1.0, elapsed);
                *state = stepped;
                *updated_at = now;
                if config.is_settled(stepped, 1.0, 0.001) {
                    (1.0, false)
                } else {
                    (stepped.position.clamp(0.0, 1.0), true)
                }
            }
            PaletteCurve::Tween(transition) => {
                let (value, running) = transition.sample(now, false);
                (value.clamp(0.0, 1.0), running)
            }
        }
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
        if cx.has_global::<ThemeAnimator>() {
            let animator = cx.global_mut::<ThemeAnimator>();
            animator.transition = None;
            animator.driver = None;
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

pub(crate) fn window_override_handle<'a>(
    window: &Window,
    cx: &'a App,
) -> Option<&'a Rc<MaterialTheme>> {
    cx.try_global::<WindowMaterialThemes>()
        .and_then(|themes| themes.0.get(&window.window_handle().window_id()))
}

pub(crate) fn window_theme_handle<'a>(window: &Window, cx: &'a App) -> &'a Rc<MaterialTheme> {
    window_override_handle(window, cx).unwrap_or_else(|| &cx.global::<ActiveMaterialTheme>().0)
}

/// Sets or clears the override for Material elements in this window.
///
/// Material controls pass local presentation to their Base-backed parts without
/// mutating Base's application-global theme. Standalone Base controls inside an
/// overridden window or subtree can call [`crate::current_base_theme`] or
/// [`project_base_theme`] to apply matching local tokens and scrollbar styles.
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

/// Updates the application theme and projects its current values into Base.
/// Colors follow the new theme's Slow Effects motion spec from the displayed
/// palette. Typography and layout apply immediately.
pub fn set_material_theme(mut theme: MaterialTheme, cx: &mut App) {
    refresh_installed_fonts(cx);
    let spec = theme
        .motion
        .spec(MotionKind::Effects, MotionSpeed::Slow, cx.reduce_motion());
    let previous = cx.try_global::<ActiveMaterialTheme>();
    if let Some(previous) = previous
        && previous.0.colors != theme.colors
        && !cx.windows().is_empty()
        && !matches!(spec, MotionSpec::Instant)
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
        if !animator
            .transition
            .as_ref()
            .is_some_and(|pending| pending.to == theme.colors && pending.spec == spec)
        {
            animator.transition =
                PaletteTransition::new(shown.clone(), theme.colors.clone(), spec, started);
        }
        if animator.transition.is_some() {
            theme.colors = shown;
            if animator.driver.is_none() {
                // The caller may already be updating a window. Register the
                // callback after that window is accessible again.
                cx.defer(schedule_theme_frame);
            }
        }
    } else if cx.has_global::<ThemeAnimator>() {
        let animator = cx.global_mut::<ThemeAnimator>();
        animator.transition = None;
        animator.driver = None;
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
    let Some(transition) = animator.transition.as_mut() else {
        animator.driver = None;
        return;
    };
    if reduce_motion {
        let target = animator.transition.take().unwrap().to;
        animator.driver = None;
        apply_palette(target, cx);
        return;
    }
    let (progress, running) = transition.advance(now);
    if !running || progress >= 0.999 {
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
    let current = material_theme(cx);
    if current.colors != colors {
        let theme = MaterialTheme {
            colors,
            typography: current.typography.clone(),
            motion: current.motion,
            layout: current.layout,
        };
        apply_material_theme(theme, false, cx);
    }
}

fn apply_material_theme(theme: MaterialTheme, project_typography: bool, cx: &mut App) {
    let mut projected = Theme::global(cx);
    project_base_colors(&mut projected, &theme);
    if project_typography {
        project_base_metrics(&mut projected, &theme, cx);
    }
    cx.set_global(ActiveMaterialTheme(Rc::new(theme)));
    cx.set_global(projected);
    refresh_inherited_windows(cx);
}

fn refresh_inherited_windows(cx: &mut App) {
    let overridden: Vec<WindowId> = cx
        .try_global::<WindowMaterialThemes>()
        .map(|themes| themes.0.keys().copied().collect())
        .unwrap_or_default();
    if overridden.is_empty() {
        cx.refresh_windows();
        return;
    }
    for handle in cx.windows() {
        if !overridden.contains(&handle.window_id()) {
            let _ = handle.update(cx, |_, window, _| window.refresh());
        }
    }
}

fn project_base_colors(projected: &mut Theme, theme: &MaterialTheme) {
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
