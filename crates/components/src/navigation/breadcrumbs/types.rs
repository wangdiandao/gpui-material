use std::rc::Rc;

use gpui::{AnyElement, App, Hsla, IntoElement, SharedString, Window};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::ColorRole;

pub(super) type BreadcrumbSelect = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
pub(super) type IconFactory = Rc<dyn Fn() -> AnyElement>;
pub(super) type SeparatorFactory = Rc<dyn Fn() -> AnyElement>;

/// Optional color overrides for [`super::Breadcrumbs`].
///
/// Omitted fields default to `OnSurfaceVariant` for ancestors and separators,
/// `OnSurface` for the current item, and `OnSurface` at 38% opacity for disabled items.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BreadcrumbsColors {
    /// Foreground color override for clickable ancestor segments.
    pub ancestor: Option<Hsla>,
    /// Foreground color override for the current (trailing) segment.
    pub current: Option<Hsla>,
    /// Foreground color override for separators between segments.
    pub separator: Option<Hsla>,
    /// Foreground color override for disabled segments.
    pub disabled: Option<Hsla>,
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedColors {
    pub(super) ancestor: Hsla,
    pub(super) current: Hsla,
    pub(super) separator: Hsla,
    pub(super) disabled: Hsla,
    pub(super) hover: Hsla,
    pub(super) focus_ring: Hsla,
}

impl BreadcrumbsColors {
    pub(super) fn resolve(self, window: &Window, cx: &App) -> ResolvedColors {
        let theme = current_material_theme(window, cx);
        let role = |r| color(theme.colors.color(r));
        let on_surface = role(ColorRole::OnSurface);
        let on_surface_variant = role(ColorRole::OnSurfaceVariant);
        ResolvedColors {
            ancestor: self.ancestor.unwrap_or(on_surface_variant),
            current: self.current.unwrap_or(on_surface),
            separator: self.separator.unwrap_or(on_surface_variant),
            disabled: self.disabled.unwrap_or_else(|| on_surface.opacity(0.38)),
            hover: on_surface.opacity(0.08),
            focus_ring: role(ColorRole::Secondary),
        }
    }
}

/// One path segment inside [`super::Breadcrumbs`].
pub struct BreadcrumbItem {
    pub(super) id: SharedString,
    pub(super) label: SharedString,
    pub(super) icon: Option<IconFactory>,
    pub(super) disabled: bool,
}

impl BreadcrumbItem {
    /// Creates a breadcrumb segment with a unique ID and visible label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    /// Adds an optional leading decorative icon factory used both inline and in the overflow menu.
    pub fn icon<E: IntoElement>(mut self, icon: impl Fn() -> E + 'static) -> Self {
        self.icon = Some(Rc::new(move || icon().into_any_element()));
        self
    }

    /// Disables navigation for this segment.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
