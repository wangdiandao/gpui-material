use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, SharedString, Window};

mod layout;
mod render;
mod types;

pub use types::{BreadcrumbItem, BreadcrumbsColors};

use types::{BreadcrumbSelect, SeparatorFactory};

/// Desktop path navigation with RTL mirroring and automatic middle-segment
/// overflow collapse into an anchored Material [`crate::Menu`].
#[derive(IntoElement)]
pub struct Breadcrumbs {
    pub(super) id: ElementId,
    pub(super) items: Vec<BreadcrumbItem>,
    pub(super) accessible_label: Option<SharedString>,
    pub(super) overflow_label: Option<SharedString>,
    pub(super) on_select: Option<BreadcrumbSelect>,
    pub(super) max_items: Option<usize>,
    pub(super) separator: Option<SeparatorFactory>,
    pub(super) colors: BreadcrumbsColors,
}

impl Breadcrumbs {
    /// Creates a breadcrumb navigation strip with the given element ID and ordered path segments.
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            accessible_label: None,
            overflow_label: None,
            on_select: None,
            max_items: None,
            separator: None,
            colors: BreadcrumbsColors::default(),
        }
    }

    /// Sets the `Role::Navigation` landmark label. Defaults to
    /// [`gpui_material_theme::MaterialStrings::breadcrumb`] (`"Breadcrumb"`).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Sets the accessible name of the collapsed-segments overflow trigger button.
    /// Defaults to [`gpui_material_theme::MaterialStrings::more_breadcrumbs`] (`"More breadcrumbs"`).
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = Some(label.into());
        self
    }

    /// Invoked when clicking or activating any non-current enabled ancestor segment,
    /// including collapsed segments selected from the overflow [`crate::Menu`].
    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Caps visible inline segments before collapsing middle segments into the overflow menu.
    /// Values below `2` are clamped to `2` when three or more segments exist.
    pub fn max_items(mut self, count: usize) -> Self {
        self.max_items = Some(count.max(2));
        self
    }

    /// Replaces the decorative separator between segments.
    /// Defaults to `Icon::new(IconName::ChevronRight).size(px(16.)).mirror_in_rtl(true)`.
    pub fn separator<E: IntoElement>(mut self, separator: impl Fn() -> E + 'static) -> Self {
        self.separator = Some(Rc::new(move || separator().into_any_element()));
        self
    }

    /// Overrides ancestor, current, separator, and disabled colors.
    pub fn colors(mut self, colors: BreadcrumbsColors) -> Self {
        self.colors = colors;
        self
    }
}
