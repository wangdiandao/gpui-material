mod base_projection;
mod system_sync;
mod transition;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui::{
    AnyView, App, AppContext, Global, Render, Subscription, Window, WindowHandle, WindowId,
};
use gpui_base::{Root, Theme};
use gpui_material_theme::{
    Appearance, AppearanceMode, ColorScheme, MaterialTheme, MotionKind, MotionSpec, MotionSpeed,
};

use crate::{
    strings::init_default_strings,
    typography::{font_cache_revision, refresh_installed_fonts},
};

pub use base_projection::project_base_theme;
use base_projection::{project_base_colors, project_base_metrics};
pub use system_sync::{
    ReduceMotionPreference, appearance_from_window, observe_system_appearance,
    observe_window_system_appearance, set_reduce_motion_preference, sync_system_reduce_motion,
    system_appearance,
};
use transition::PaletteTransition;

struct ActiveMaterialTheme(Rc<MaterialTheme>);

impl Global for ActiveMaterialTheme {}

struct WindowThemeEntry {
    material: Rc<MaterialTheme>,
    projected: RefCell<Option<(u64, Theme)>>,
}

#[derive(Default)]
struct WindowMaterialThemes(HashMap<WindowId, WindowThemeEntry>);

impl Global for WindowMaterialThemes {}

#[derive(Default)]
struct ThemeAnimator {
    transition: Option<PaletteTransition>,
    driver: Option<WindowId>,
}

impl Global for ThemeAnimator {}

/// Initializes Material and Base before the application opens its first window.
pub struct MaterialRoot;

impl MaterialRoot {
    /// Initializes `gpui-base`, refreshes installed fonts, and installs the application-wide
    /// Material theme. Call once at startup before opening windows.
    pub fn init(theme: MaterialTheme, cx: &mut App) {
        gpui_base::init(cx);
        if cx.has_global::<ThemeAnimator>() {
            let animator = cx.global_mut::<ThemeAnimator>();
            animator.transition = None;
            animator.driver = None;
        }
        refresh_installed_fonts(cx);
        init_default_strings(cx);
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

    /// Synchronizes the application-wide Material theme with `window`'s appearance.
    ///
    /// Convenience alias for [`observe_system_appearance`].
    pub fn sync_system_appearance(
        window: &mut Window,
        cx: &mut App,
        mode: AppearanceMode,
        build_theme: impl FnMut(Appearance) -> MaterialTheme + 'static,
    ) -> Subscription {
        observe_system_appearance(window, cx, mode, build_theme)
    }

    /// Synchronizes a single window's Material theme override with `window`'s appearance.
    ///
    /// Convenience alias for [`observe_window_system_appearance`].
    pub fn sync_window_system_appearance(
        window: &mut Window,
        cx: &mut App,
        mode: AppearanceMode,
        build_theme: impl FnMut(Appearance) -> MaterialTheme + 'static,
    ) -> Subscription {
        observe_window_system_appearance(window, cx, mode, build_theme)
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
        .map(|entry| &entry.material)
}

pub(crate) fn window_theme_handle<'a>(window: &Window, cx: &'a App) -> &'a Rc<MaterialTheme> {
    window_override_handle(window, cx).unwrap_or_else(|| &cx.global::<ActiveMaterialTheme>().0)
}

/// Returns the cached projected `gpui_base::Theme` for `window` without re-projecting when
/// the window override (or application global) and installed font revision are unchanged.
pub(crate) fn cached_window_base_theme(window: &Window, cx: &App) -> Theme {
    let Some(entry) = cx
        .try_global::<WindowMaterialThemes>()
        .and_then(|themes| themes.0.get(&window.window_handle().window_id()))
    else {
        return Theme::global(cx);
    };
    let revision = font_cache_revision(cx);
    if let Some((cached_rev, cached_theme)) = entry.projected.borrow().as_ref()
        && *cached_rev == revision
    {
        return cached_theme.clone();
    }
    let projected = project_base_theme(&entry.material, cx);
    *entry.projected.borrow_mut() = Some((revision, projected.clone()));
    projected
}

/// Sets or clears the override for Material elements in this window.
///
/// Material controls pass local presentation to their Base-backed parts without
/// mutating Base's application-global theme. Standalone Base controls inside an
/// overridden window or subtree can call [`crate::current_base_theme`] or
/// [`project_base_theme`] to apply matching local tokens and scrollbar styles.
pub fn set_window_material_theme(theme: Option<MaterialTheme>, window: &mut Window, cx: &mut App) {
    let fonts_changed = refresh_installed_fonts(cx);
    if !fonts_changed && window_override_handle(window, cx).map(Rc::as_ref) == theme.as_ref() {
        return;
    }
    if !cx.has_global::<WindowMaterialThemes>() {
        cx.set_global(WindowMaterialThemes::default());
        cx.on_window_closed(|cx, id| {
            cx.global_mut::<WindowMaterialThemes>().0.remove(&id);
        })
        .detach();
    }
    let id = window.window_handle().window_id();
    let projected = theme
        .as_ref()
        .map(|t| (font_cache_revision(cx), project_base_theme(t, cx)));
    let themes = &mut cx.global_mut::<WindowMaterialThemes>().0;
    if let Some(theme) = theme {
        themes.insert(
            id,
            WindowThemeEntry {
                material: Rc::new(theme),
                projected: RefCell::new(projected),
            },
        );
    } else {
        themes.remove(&id);
    }
    window.refresh();
}

/// Updates the application theme and projects its current values into Base.
/// Colors follow the new theme's Slow Effects motion spec from the displayed
/// palette. Typography and layout apply immediately.
pub fn set_material_theme(mut theme: MaterialTheme, cx: &mut App) {
    let fonts_changed = refresh_installed_fonts(cx);
    let has_transition = cx
        .try_global::<ThemeAnimator>()
        .is_some_and(|animator| animator.transition.is_some());
    if !fonts_changed
        && !has_transition
        && cx
            .try_global::<ActiveMaterialTheme>()
            .is_some_and(|previous| *previous.0 == theme)
    {
        return;
    }
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
    if reduce_motion {
        animator.driver = None;
        if let Some(transition) = animator.transition.take() {
            apply_palette(transition.to, cx);
        }
        return;
    }
    let Some(transition) = animator.transition.as_mut() else {
        animator.driver = None;
        return;
    };
    let (progress, running) = transition.advance(now);
    if !running || progress >= 0.999 {
        animator.driver = None;
        if let Some(transition) = animator.transition.take() {
            apply_palette(transition.to, cx);
        }
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
