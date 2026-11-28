use std::{cell::RefCell, ops::Deref, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window, WindowId,
};
use gpui_base::Theme;
use gpui_material_theme::{Density, Direction, MaterialStrings, MaterialTheme};

use crate::{
    root::{
        cached_window_base_theme, project_base_theme, window_override_handle, window_theme_handle,
    },
    strings::app_strings_handle,
    typography::font_cache_revision,
};

type ScopeKey = (*const App, WindowId);

struct ScopeEntry {
    key: ScopeKey,
    material: Option<Rc<MaterialTheme>>,
    projected: RefCell<Option<(u64, Theme)>>,
    strings: Option<Rc<MaterialStrings>>,
}

struct DerivedThemeCacheEntry {
    parent_ptr: *const MaterialTheme,
    direction: Option<Direction>,
    density: Option<Density>,
    parent: Rc<MaterialTheme>,
    derived: Rc<MaterialTheme>,
}

const MAX_DERIVED_CACHE_ENTRIES: usize = 16;

thread_local! {
    static SCOPES: RefCell<Vec<ScopeEntry>> = const { RefCell::new(Vec::new()) };
    static DERIVED_THEMES: RefCell<Vec<DerivedThemeCacheEntry>> = const { RefCell::new(Vec::new()) };
}

fn scope_key(window: &Window, cx: &App) -> ScopeKey {
    (cx, window.window_handle().window_id())
}

fn scoped_theme(window: &Window, cx: &App) -> Option<Rc<MaterialTheme>> {
    let key = scope_key(window, cx);
    SCOPES.with_borrow(|scopes| {
        scopes
            .iter()
            .rev()
            .find_map(|entry| {
                (entry.key == key)
                    .then_some(entry.material.as_ref())
                    .flatten()
            })
            .cloned()
    })
}

fn scoped_strings(window: &Window, cx: &App) -> Option<Rc<MaterialStrings>> {
    let key = scope_key(window, cx);
    SCOPES.with_borrow(|scopes| {
        scopes
            .iter()
            .rev()
            .find_map(|entry| {
                (entry.key == key)
                    .then_some(entry.strings.as_ref())
                    .flatten()
            })
            .cloned()
    })
}

fn derive_scoped_theme(
    base: Rc<MaterialTheme>,
    direction: Option<Direction>,
    density: Option<Density>,
) -> Rc<MaterialTheme> {
    let needs_dir = direction.is_some_and(|dir| base.layout.direction != dir);
    let needs_density = density.is_some_and(|d| base.layout.density != d);
    if !needs_dir && !needs_density {
        return base;
    }
    let parent_ptr = Rc::as_ptr(&base);
    if let Some(cached) = DERIVED_THEMES.with_borrow(|cache| {
        cache
            .iter()
            .find(|entry| {
                entry.parent_ptr == parent_ptr
                    && entry.direction == direction
                    && entry.density == density
                    && Rc::ptr_eq(&entry.parent, &base)
            })
            .map(|entry| entry.derived.clone())
    }) {
        return cached;
    }
    let mut derived = (*base).clone();
    if let Some(direction) = direction {
        derived.layout.direction = direction;
    }
    if let Some(density) = density {
        derived.layout.density = density;
    }
    let derived = Rc::new(derived);
    DERIVED_THEMES.with_borrow_mut(|cache| {
        if cache.len() >= MAX_DERIVED_CACHE_ENTRIES {
            cache.remove(0);
        }
        cache.push(DerivedThemeCacheEntry {
            parent_ptr,
            direction,
            density,
            parent: base,
            derived: derived.clone(),
        });
    });
    derived
}

/// Resolves the closest Material scope, then the window and application defaults.
pub fn current_material_theme(
    window: &Window,
    cx: &App,
) -> impl Deref<Target = MaterialTheme> + use<> {
    scoped_theme(window, cx).unwrap_or_else(|| window_theme_handle(window, cx).clone())
}

/// Resolves the closest [`ThemeScope`] strings override, falling back to the
/// application-wide [`MaterialStrings`] global.
pub fn current_material_strings(
    window: &Window,
    cx: &App,
) -> impl Deref<Target = MaterialStrings> + use<> {
    scoped_strings(window, cx).unwrap_or_else(|| app_strings_handle(cx))
}

/// Projects the active local Material scope (or window/application theme) into
/// a `gpui_base::Theme` without mutating `gpui_base::Theme::global`.
///
/// Reuses cached projections for the application global, window overrides, and active
/// [`ThemeScope`] entries so repeated lookups do not re-project on every frame.
pub fn current_base_theme(window: &Window, cx: &App) -> Theme {
    let key = scope_key(window, cx);
    let revision = font_cache_revision(cx);
    let scoped = SCOPES.with_borrow(|scopes| {
        scopes.iter().rev().find_map(|entry| {
            if entry.key != key {
                return None;
            }
            let material = entry.material.as_ref()?;
            if let Some((cached_rev, cached_theme)) = entry.projected.borrow().as_ref()
                && *cached_rev == revision
            {
                return Some(cached_theme.clone());
            }
            let projected = project_base_theme(material, cx);
            *entry.projected.borrow_mut() = Some((revision, projected.clone()));
            Some(projected)
        })
    });
    scoped.unwrap_or_else(|| cached_window_base_theme(window, cx))
}

fn with_scope<R>(
    theme: Option<&Rc<MaterialTheme>>,
    strings: Option<&Rc<MaterialStrings>>,
    window: &mut Window,
    cx: &mut App,
    draw: impl FnOnce(&mut Window, &mut App) -> R,
) -> R {
    if theme.is_none() && strings.is_none() {
        return draw(window, cx);
    }
    struct ScopeGuard;
    impl Drop for ScopeGuard {
        fn drop(&mut self) {
            SCOPES.with_borrow_mut(|scopes| scopes.pop());
        }
    }
    SCOPES.with_borrow_mut(|scopes| {
        scopes.push(ScopeEntry {
            key: scope_key(window, cx),
            material: theme.cloned(),
            projected: RefCell::new(None),
            strings: strings.cloned(),
        });
    });
    let _guard = ScopeGuard;
    draw(window, cx)
}

/// Applies a Material theme, direction, density, or [`MaterialStrings`] table to
/// a child and its Material overlays.
///
/// The element participates in layout using its child's layout directly.
pub struct ThemeScope {
    child: AnyElement,
    theme: Option<Rc<MaterialTheme>>,
    direction: Option<Direction>,
    density: Option<Density>,
    strings: Option<Rc<MaterialStrings>>,
}

impl ThemeScope {
    /// Wraps `child` in a theme scope that inherits the surrounding theme and strings until overridden.
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            theme: None,
            direction: None,
            density: None,
            strings: None,
        }
    }

    /// Replaces the entire [`MaterialTheme`] for this subtree.
    pub fn theme(mut self, mut theme: MaterialTheme) -> Self {
        if let Some(direction) = self.direction {
            theme.layout.direction = direction;
        }
        if let Some(density) = self.density {
            theme.layout.density = density;
        }
        self.theme = Some(Rc::new(theme));
        self
    }

    /// Overrides the layout [`Direction`] for this subtree.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        if let Some(theme) = &mut self.theme
            && theme.layout.direction != direction
        {
            Rc::make_mut(theme).layout.direction = direction;
        }
        self
    }

    /// Overrides the visual [`Density`] for this subtree.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        if let Some(theme) = &mut self.theme
            && theme.layout.density != density
        {
            Rc::make_mut(theme).layout.density = density;
        }
        self
    }

    /// Overrides the built-in [`MaterialStrings`] table for this subtree and its
    /// deferred Material overlays without mutating the application global.
    pub fn strings(mut self, strings: MaterialStrings) -> Self {
        self.strings = Some(Rc::new(strings));
        self
    }

    /// Carries the current subtree scope across a deferred draw. Wrap the
    /// content passed to the positioning or overlay host.
    pub fn inherit(child: impl IntoElement, window: &Window, cx: &App) -> AnyElement {
        let theme = scoped_theme(window, cx);
        let strings = scoped_strings(window, cx);
        if theme.is_some() || strings.is_some() {
            Self {
                child: child.into_any_element(),
                theme,
                direction: None,
                density: None,
                strings,
            }
            .into_any_element()
        } else {
            child.into_any_element()
        }
    }

    /// Runs an action in its originating Material scope (subtree or window
    /// override). Use this when an action creates a separate native window from
    /// a scoped trigger.
    pub fn bind_action<F: Fn(&mut Window, &mut App) + 'static>(
        action: F,
        window: &Window,
        cx: &App,
    ) -> impl Fn(&mut Window, &mut App) + use<F> {
        let theme =
            scoped_theme(window, cx).or_else(|| window_override_handle(window, cx).cloned());
        let strings = scoped_strings(window, cx);
        move |window, cx| with_scope(theme.as_ref(), strings.as_ref(), window, cx, &action)
    }
}

impl Element for ThemeScope {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        if self.theme.is_none() && (self.direction.is_some() || self.density.is_some()) {
            let base =
                scoped_theme(window, cx).unwrap_or_else(|| window_theme_handle(window, cx).clone());
            self.theme = Some(derive_scoped_theme(base, self.direction, self.density));
        }
        let layout = with_scope(
            self.theme.as_ref(),
            self.strings.as_ref(),
            window,
            cx,
            |window, cx| self.child.request_layout(window, cx),
        );
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        with_scope(
            self.theme.as_ref(),
            self.strings.as_ref(),
            window,
            cx,
            |window, cx| {
                self.child.prepaint(window, cx);
            },
        );
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        with_scope(
            self.theme.as_ref(),
            self.strings.as_ref(),
            window,
            cx,
            |window, cx| {
                self.child.paint(window, cx);
            },
        );
    }
}
