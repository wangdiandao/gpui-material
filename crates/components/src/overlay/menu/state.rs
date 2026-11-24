use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{App, Bounds, FocusHandle, Pixels, Point, ScrollHandle, Window};
use gpui_material_foundation::OverlayMotion;

use crate::menu_appearance::{MenuColor, MenuSurfaceStyle};

use super::exit::MenuExitRow;
use super::item::{Action, ItemContent, MenuItem};

#[derive(Default)]
pub(super) struct PopupBounds {
    pub(super) trigger: Cell<Bounds<Pixels>>,
    pub(super) popup: Cell<Bounds<Pixels>>,
    pub(super) context_position: Cell<Option<Point<Pixels>>>,
    pub(super) context_was_open: Cell<bool>,
    pub(super) context_focus: RefCell<Option<FocusHandle>>,
    pub(super) context_previous_focus: RefCell<Option<FocusHandle>>,
    pub(super) child: RefCell<Option<Rc<PopupBounds>>>,
    pub(super) close_handler: RefCell<Option<Action>>,
    pub(super) presented: RefCell<Option<PresentedMenu>>,
    pub(super) scroll: ScrollHandle,
    pub(super) motion: Cell<Option<OverlayMotion>>,
    pub(super) open: Rc<Cell<bool>>,
}

#[derive(Clone)]
pub(super) struct MenuPresentation {
    pub(super) color: MenuColor,
    pub(super) appearance: MenuSurfaceStyle,
    pub(super) decoration: Option<ItemContent>,
    pub(super) scroll: ScrollHandle,
}

pub(super) struct PresentedMenu {
    pub(super) items: Rc<[MenuExitRow]>,
    pub(super) presentation: MenuPresentation,
}

impl PopupBounds {
    pub(super) fn popup_contains(&self, point: &Point<Pixels>) -> bool {
        self.popup.get().contains(point)
            || self
                .child
                .borrow()
                .as_ref()
                .is_some_and(|child| child.open.get() && child.contains(point))
    }

    pub(super) fn contains(&self, point: &Point<Pixels>) -> bool {
        self.trigger.get().contains(point) || self.popup_contains(point)
    }

    /// Registers `next` as the active child submenu, closing any previously open sibling submenu.
    pub(super) fn activate_child(&self, next: &Rc<PopupBounds>, window: &mut Window, cx: &mut App) {
        let previous = {
            let mut slot = self.child.borrow_mut();
            let prev = slot
                .as_ref()
                .filter(|prev| !Rc::ptr_eq(prev, next))
                .cloned();
            *slot = Some(next.clone());
            prev
        };
        if let Some(prev) = previous {
            prev.close_now(window, cx);
        }
    }

    /// Recursively closes this menu and any open child submenu.
    pub(super) fn close_now(&self, window: &mut Window, cx: &mut App) {
        let child = self.child.borrow().as_ref().cloned();
        if let Some(child) = child {
            child.close_now(window, cx);
        }
        if self.open.replace(false)
            && let Some(close) = self.close_handler.borrow().as_ref().cloned()
        {
            close(window, cx);
        }
    }

    /// Updates the cached exit snapshot without reallocating row trees when unchanged.
    pub(super) fn sync_presented(&self, items: &[MenuItem], presentation: &MenuPresentation) {
        let mut slot = self.presented.borrow_mut();
        if let Some(existing) = slot.as_mut()
            && MenuExitRow::matches_slice(&existing.items, items)
        {
            existing.presentation = presentation.clone();
            return;
        }
        *slot = Some(PresentedMenu {
            items: items.iter().map(MenuExitRow::from).collect(),
            presentation: presentation.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MenuItem;

    #[test]
    fn sync_presented_reuses_snapshot_allocation_when_items_unchanged() {
        let bounds = PopupBounds::default();
        let presentation = MenuPresentation {
            color: MenuColor::Standard,
            appearance: MenuSurfaceStyle::default(),
            decoration: None,
            scroll: ScrollHandle::new(),
        };
        let items = vec![
            MenuItem::new("a", "Alpha"),
            MenuItem::new("b", "Beta").checked(true),
        ];
        bounds.sync_presented(&items, &presentation);
        let first_ptr = Rc::as_ptr(&bounds.presented.borrow().as_ref().unwrap().items);

        bounds.sync_presented(&items, &presentation);
        let second_ptr = Rc::as_ptr(&bounds.presented.borrow().as_ref().unwrap().items);
        assert_eq!(first_ptr, second_ptr);

        let updated = vec![
            MenuItem::new("a", "Alpha"),
            MenuItem::new("b", "Beta").checked(false),
        ];
        bounds.sync_presented(&updated, &presentation);
        let third_ptr = Rc::as_ptr(&bounds.presented.borrow().as_ref().unwrap().items);
        assert_ne!(first_ptr, third_ptr);
    }
}
