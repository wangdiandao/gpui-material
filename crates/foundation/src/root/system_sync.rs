use gpui::{App, Subscription, Window, WindowAppearance};
pub use gpui_base::{ReduceMotionPreference, set_reduce_motion_preference};
use gpui_material_theme::{Appearance, AppearanceMode, MaterialTheme};

use crate::root::{set_material_theme, set_window_material_theme};

/// Maps a GPUI [`WindowAppearance`] to a Material [`Appearance`].
///
/// [`WindowAppearance::Light`] and [`WindowAppearance::VibrantLight`] resolve to
/// [`Appearance::Light`]; [`WindowAppearance::Dark`] and [`WindowAppearance::VibrantDark`]
/// resolve to [`Appearance::Dark`].
pub const fn appearance_from_window(appearance: WindowAppearance) -> Appearance {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => Appearance::Light,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => Appearance::Dark,
    }
}

/// Returns the current operating-system [`Appearance`] reported by GPUI on `cx`.
///
/// Call this before opening the first window to initialize [`crate::MaterialRoot::init`]
/// with the active system appearance without a light-to-dark flash.
pub fn system_appearance(cx: &App) -> Appearance {
    appearance_from_window(cx.window_appearance())
}

/// Re-reads the operating system's reduced-motion preference and applies it to `cx`.
///
/// Delegates to [`gpui_base::apply_system_reduce_motion`]. `gpui-base` reads the OS
/// preference once during [`crate::MaterialRoot::init`]; call this helper again to
/// refresh the setting on platforms that do not push live updates (such as Windows and
/// macOS). Explicit overrides set via [`set_reduce_motion_preference`] or
/// [`App::set_reduce_motion`] are preserved, and under GPUI's test scheduler the platform
/// is never queried.
pub fn sync_system_reduce_motion(cx: &mut App) {
    gpui_base::apply_system_reduce_motion(cx);
}

struct AppearanceTracker<F> {
    mode: AppearanceMode,
    last_applied: Option<Appearance>,
    build_theme: F,
}

impl<F> AppearanceTracker<F>
where
    F: FnMut(Appearance) -> MaterialTheme,
{
    fn new(mode: AppearanceMode, build_theme: F) -> Self {
        Self {
            mode,
            last_applied: None,
            build_theme,
        }
    }

    fn next_theme(&mut self, window_appearance: WindowAppearance) -> Option<MaterialTheme> {
        let resolved = self.mode.resolve(appearance_from_window(window_appearance));
        if self.last_applied == Some(resolved) {
            return None;
        }
        self.last_applied = Some(resolved);
        Some((self.build_theme)(resolved))
    }
}

/// Synchronizes the application-wide Material theme with `window`'s appearance.
///
/// Immediately resolves `mode` against [`Window::appearance`], builds the theme with
/// `build_theme`, and applies it via [`set_material_theme`]. When `mode` is
/// [`AppearanceMode::System`], subscribes to [`Window::observe_window_appearance`] so
/// subsequent OS light/dark changes smoothly transition the application theme through
/// [`set_material_theme`] while preserving window-level overrides on other windows.
///
/// Redundant notifications that resolve to the same [`Appearance`] (for example,
/// [`WindowAppearance::Light`] to [`WindowAppearance::VibrantLight`] or duplicate
/// platform callbacks) are skipped without invoking `build_theme` again. Retain the
/// returned [`Subscription`] for as long as observation should remain active, or call
/// [`Subscription::detach`] to tie it to the window's lifetime.
pub fn observe_system_appearance(
    window: &mut Window,
    cx: &mut App,
    mode: AppearanceMode,
    build_theme: impl FnMut(Appearance) -> MaterialTheme + 'static,
) -> Subscription {
    let mut tracker = AppearanceTracker::new(mode, build_theme);
    if let Some(theme) = tracker.next_theme(window.appearance()) {
        set_material_theme(theme, cx);
    }
    if mode != AppearanceMode::System {
        return Subscription::new(|| {});
    }
    window.observe_window_appearance(move |window, cx| {
        if let Some(theme) = tracker.next_theme(window.appearance()) {
            set_material_theme(theme, cx);
        }
    })
}

/// Synchronizes a single window's Material theme override with `window`'s appearance.
///
/// Immediately resolves `mode` against [`Window::appearance`], builds the theme with
/// `build_theme`, and applies it to `window` via [`set_window_material_theme`] without
/// mutating the application-wide theme. When `mode` is [`AppearanceMode::System`],
/// subscribes to [`Window::observe_window_appearance`] and updates only `window` whenever
/// the resolved [`Appearance`] changes.
///
/// Dropping the returned [`Subscription`] stops future updates; call
/// `set_window_material_theme(None, window, cx)` to clear the override and inherit the
/// application theme again.
pub fn observe_window_system_appearance(
    window: &mut Window,
    cx: &mut App,
    mode: AppearanceMode,
    build_theme: impl FnMut(Appearance) -> MaterialTheme + 'static,
) -> Subscription {
    let mut tracker = AppearanceTracker::new(mode, build_theme);
    if let Some(theme) = tracker.next_theme(window.appearance()) {
        set_window_material_theme(Some(theme), window, cx);
    }
    if mode != AppearanceMode::System {
        return Subscription::new(|| {});
    }
    window.observe_window_appearance(move |window, cx| {
        if let Some(theme) = tracker.next_theme(window.appearance()) {
            set_window_material_theme(Some(theme), window, cx);
        }
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use gpui_material_theme::{Density, MotionScheme};

    #[test]
    fn maps_all_window_appearances_to_material_appearance() {
        const LIGHT: Appearance = appearance_from_window(WindowAppearance::Light);
        const VIBRANT_LIGHT: Appearance = appearance_from_window(WindowAppearance::VibrantLight);
        const DARK: Appearance = appearance_from_window(WindowAppearance::Dark);
        const VIBRANT_DARK: Appearance = appearance_from_window(WindowAppearance::VibrantDark);

        assert_eq!(LIGHT, Appearance::Light);
        assert_eq!(VIBRANT_LIGHT, Appearance::Light);
        assert_eq!(DARK, Appearance::Dark);
        assert_eq!(VIBRANT_DARK, Appearance::Dark);
    }

    #[test]
    fn tracker_skips_redundant_updates_and_preserves_custom_builder_settings() {
        let calls = Cell::new(0usize);
        let mut tracker = AppearanceTracker::new(AppearanceMode::System, |appearance| {
            calls.set(calls.get() + 1);
            MaterialTheme::baseline(appearance)
                .motion(MotionScheme::Expressive)
                .density(Density::Compact)
        });

        let first = tracker.next_theme(WindowAppearance::Light).unwrap();
        assert_eq!(first.colors.appearance(), Appearance::Light);
        assert_eq!(first.motion, MotionScheme::Expressive);
        assert_eq!(first.layout.density, Density::Compact);
        assert_eq!(calls.get(), 1);

        // Light -> Light and Light -> VibrantLight resolve to the same Appearance.
        assert!(tracker.next_theme(WindowAppearance::Light).is_none());
        assert!(tracker.next_theme(WindowAppearance::VibrantLight).is_none());
        assert_eq!(calls.get(), 1);

        // Switching to Dark or VibrantDark rebuilds once.
        let dark = tracker.next_theme(WindowAppearance::Dark).unwrap();
        assert_eq!(dark.colors.appearance(), Appearance::Dark);
        assert_eq!(dark.motion, MotionScheme::Expressive);
        assert_eq!(dark.layout.density, Density::Compact);
        assert_eq!(calls.get(), 2);

        assert!(tracker.next_theme(WindowAppearance::VibrantDark).is_none());
        assert_eq!(calls.get(), 2);

        // Returning to VibrantLight rebuilds once back to Light.
        let light_again = tracker.next_theme(WindowAppearance::VibrantLight).unwrap();
        assert_eq!(light_again.colors.appearance(), Appearance::Light);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn fixed_mode_tracker_builds_once_regardless_of_system_changes() {
        for (mode, expected) in [
            (AppearanceMode::Light, Appearance::Light),
            (AppearanceMode::Dark, Appearance::Dark),
        ] {
            let calls = Cell::new(0usize);
            let mut tracker = AppearanceTracker::new(mode, |appearance| {
                calls.set(calls.get() + 1);
                MaterialTheme::baseline(appearance)
            });

            let initial = tracker.next_theme(WindowAppearance::Light).unwrap();
            assert_eq!(initial.colors.appearance(), expected);
            assert_eq!(calls.get(), 1);

            assert!(tracker.next_theme(WindowAppearance::Dark).is_none());
            assert!(tracker.next_theme(WindowAppearance::VibrantDark).is_none());
            assert!(tracker.next_theme(WindowAppearance::VibrantLight).is_none());
            assert_eq!(calls.get(), 1);
        }
    }
}
