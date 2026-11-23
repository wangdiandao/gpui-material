use std::{cell::RefCell, ops::Deref, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window, WindowId,
};
use gpui_base::Theme;
use gpui_material_theme::{Density, Direction, MaterialTheme};

use crate::root::{project_base_theme, window_override_handle, window_theme_handle};

type ScopeKey = (*const App, WindowId);

thread_local! {
    static SCOPES: RefCell<Vec<(ScopeKey, Rc<MaterialTheme>)>> = const { RefCell::new(Vec::new()) };
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
            .find(|(id, _)| *id == key)
            .map(|(_, theme)| theme.clone())
    })
}

/// Resolves the closest Material scope, then the window and application defaults.
pub fn current_material_theme(
    window: &Window,
    cx: &App,
) -> impl Deref<Target = MaterialTheme> + use<> {
    scoped_theme(window, cx).unwrap_or_else(|| window_theme_handle(window, cx).clone())
}

/// Projects the active local Material scope (or window/application theme) into
/// a `gpui_base::Theme` without mutating `gpui_base::Theme::global`.
pub fn current_base_theme(window: &Window, cx: &App) -> Theme {
    project_base_theme(&current_material_theme(window, cx), cx)
}

fn with_scope<R>(
    theme: Option<&Rc<MaterialTheme>>,
    window: &mut Window,
    cx: &mut App,
    draw: impl FnOnce(&mut Window, &mut App) -> R,
) -> R {
    let Some(theme) = theme else {
        return draw(window, cx);
    };
    struct ScopeGuard;
    impl Drop for ScopeGuard {
        fn drop(&mut self) {
            SCOPES.with_borrow_mut(|scopes| scopes.pop());
        }
    }
    SCOPES.with_borrow_mut(|scopes| scopes.push((scope_key(window, cx), theme.clone())));
    let _guard = ScopeGuard;
    draw(window, cx)
}

/// Applies a Material theme, direction, or density to a child and its Material overlays.
/// The element participates in layout using its child's layout directly.
pub struct ThemeScope {
    child: AnyElement,
    theme: Option<Rc<MaterialTheme>>,
    direction: Option<Direction>,
    density: Option<Density>,
}

impl ThemeScope {
    /// Wraps `child` in a theme scope that inherits the surrounding theme until overridden.
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            theme: None,
            direction: None,
            density: None,
        }
    }

    /// Replaces the entire [`MaterialTheme`] for this subtree.
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = Some(Rc::new(theme));
        self
    }

    /// Overrides the layout [`Direction`] for this subtree.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Overrides the visual [`Density`] for this subtree.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Carries the current subtree scope across a deferred draw. Wrap the
    /// content passed to the positioning or overlay host.
    pub fn inherit(child: impl IntoElement, window: &Window, cx: &App) -> AnyElement {
        match scoped_theme(window, cx) {
            Some(theme) => Self {
                child: child.into_any_element(),
                theme: Some(theme),
                direction: None,
                density: None,
            }
            .into_any_element(),
            None => child.into_any_element(),
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
        move |window, cx| with_scope(theme.as_ref(), window, cx, &action)
    }
}

impl IntoElement for ThemeScope {
    type Element = Self;

    fn into_element(self) -> Self {
        self
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
        if self.direction.is_some() || self.density.is_some() {
            let mut theme = self.theme.take().unwrap_or_else(|| {
                scoped_theme(window, cx).unwrap_or_else(|| window_theme_handle(window, cx).clone())
            });
            if let Some(direction) = self.direction
                && theme.layout.direction != direction
            {
                Rc::make_mut(&mut theme).layout.direction = direction;
            }
            if let Some(density) = self.density
                && theme.layout.density != density
            {
                Rc::make_mut(&mut theme).layout.density = density;
            }
            self.theme = Some(theme);
        }
        let layout = with_scope(self.theme.as_ref(), window, cx, |window, cx| {
            self.child.request_layout(window, cx)
        });
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
        with_scope(self.theme.as_ref(), window, cx, |window, cx| {
            self.child.prepaint(window, cx);
        });
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
        with_scope(self.theme.as_ref(), window, cx, |window, cx| {
            self.child.paint(window, cx);
        });
    }
}
