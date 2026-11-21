#[cfg(feature = "feedback")]
use gpui::ElementId;
use gpui::{IntoElement, SharedString};

use crate::Badge;
#[cfg(feature = "feedback")]
use crate::{Menu, MenuItem, Tooltip};

/// Fluent postfix composition helpers for attaching Material badges, tooltips,
/// and context menus to any GPUI element.
pub trait MaterialElementExt: IntoElement + Sized {
    /// Anchors a small dot [`Badge`] to this element's upper trailing edge.
    fn badge_dot(self) -> Badge {
        Badge::dot(self)
    }

    /// Anchors a text [`Badge`] to this element's upper trailing edge.
    fn badge_label(self, text: impl Into<SharedString>) -> Badge {
        Badge::label(self, text)
    }

    /// Anchors a capped numeric [`Badge`] to this element's upper trailing edge.
    fn badge_count(self, count: usize, maximum: usize) -> Badge {
        Badge::count(self, count, maximum)
    }

    /// Wraps this element as the trigger of a plain [`Tooltip`].
    #[cfg(feature = "feedback")]
    fn tooltip(self, id: impl Into<ElementId>, text: impl Into<SharedString>) -> Tooltip {
        Tooltip::plain(id, self, text)
    }

    /// Wraps this element as the trigger of a secondary-click context [`Menu`].
    #[cfg(feature = "feedback")]
    fn context_menu(
        self,
        id: impl Into<ElementId>,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Menu {
        Menu::new(id, self, items).context()
    }
}

impl<T: IntoElement + Sized> MaterialElementExt for T {}
