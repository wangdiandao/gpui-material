use gpui::{ElementId, Role, Window};
use gpui_base::test_support as observation;

use super::{ElementSnapshot, ScopedWindow, TextSnapshot};

/// Accessibility-oriented observation queries over a [`Window`] or [`ScopedWindow`].
pub trait TestQueryExt {
    /// Returns all observed elements in the current scope from the last completed
    /// frame (including invisible ones), ordered by `(y, x, path)`.
    fn elements(&self) -> Vec<ElementSnapshot>;

    /// Returns observed text runs painted in the current scope during the last frame.
    fn texts(&self) -> Vec<TextSnapshot>;

    /// Returns the uniquely matching element whose exact, case-sensitive native
    /// accessibility label equals `label`. Panics on absence or ambiguity.
    fn find_by_label(&self, label: &str) -> ElementSnapshot;

    /// Returns the uniquely matching element with accessibility label `label`,
    /// or `None` if absent. Panics on ambiguity.
    fn try_find_by_label(&self, label: &str) -> Option<ElementSnapshot>;

    /// Returns all elements with exact accessibility label `label`, ordered by `(y, x, path)`.
    fn find_all_by_label(&self, label: &str) -> Vec<ElementSnapshot>;

    /// Returns all elements with native AccessKit [`Role`] `role`, ordered by `(y, x, path)`.
    fn find_all_by_role(&self, role: Role) -> Vec<ElementSnapshot>;
}

fn collect_elements(window: &Window, scope: &[ElementId]) -> Vec<ElementSnapshot> {
    let mut items: Vec<_> = observation::snapshots(window)
        .into_iter()
        .filter(|item| item.path().len() > scope.len() && item.path().starts_with(scope))
        .collect();
    items.sort_by(|a, b| {
        (a.bounds().origin.y, a.bounds().origin.x)
            .cmp(&(b.bounds().origin.y, b.bounds().origin.x))
            .then_with(|| format!("{:?}", a.path()).cmp(&format!("{:?}", b.path())))
    });
    items
}

fn collect_texts(window: &Window, scope: &[ElementId]) -> Vec<TextSnapshot> {
    observation::texts(window)
        .into_iter()
        .filter(|item| scope.is_empty() || item.path().starts_with(scope))
        .collect()
}

fn find_all_by_label_in(window: &Window, scope: &[ElementId], label: &str) -> Vec<ElementSnapshot> {
    collect_elements(window, scope)
        .into_iter()
        .filter(|item| item.label() == Some(label))
        .collect()
}

fn try_find_by_label_in(
    window: &Window,
    scope: &[ElementId],
    label: &str,
) -> Option<ElementSnapshot> {
    let mut matches = find_all_by_label_in(window, scope, label);
    assert!(
        matches.len() <= 1,
        "ambiguous accessibility label {label:?} in scope {scope:?}; use within(...) to narrow. \
         Matches: {:?}",
        matches.iter().map(|entry| entry.path()).collect::<Vec<_>>()
    );
    matches.pop()
}

fn find_by_label_in(window: &Window, scope: &[ElementId], label: &str) -> ElementSnapshot {
    try_find_by_label_in(window, scope, label).unwrap_or_else(|| {
        panic!(
            "missing accessibility label {label:?} in scope {scope:?}. Registered paths: {}",
            observation::registered_paths(window)
        )
    })
}

fn find_all_by_role_in(window: &Window, scope: &[ElementId], role: Role) -> Vec<ElementSnapshot> {
    collect_elements(window, scope)
        .into_iter()
        .filter(|item| item.role() == Some(role))
        .collect()
}

impl TestQueryExt for Window {
    fn elements(&self) -> Vec<ElementSnapshot> {
        collect_elements(self, &[])
    }

    fn texts(&self) -> Vec<TextSnapshot> {
        collect_texts(self, &[])
    }

    fn find_by_label(&self, label: &str) -> ElementSnapshot {
        find_by_label_in(self, &[], label)
    }

    fn try_find_by_label(&self, label: &str) -> Option<ElementSnapshot> {
        try_find_by_label_in(self, &[], label)
    }

    fn find_all_by_label(&self, label: &str) -> Vec<ElementSnapshot> {
        find_all_by_label_in(self, &[], label)
    }

    fn find_all_by_role(&self, role: Role) -> Vec<ElementSnapshot> {
        find_all_by_role_in(self, &[], role)
    }
}

impl TestQueryExt for ScopedWindow<'_> {
    fn elements(&self) -> Vec<ElementSnapshot> {
        collect_elements(self.window, &self.scope)
    }

    fn texts(&self) -> Vec<TextSnapshot> {
        collect_texts(self.window, &self.scope)
    }

    fn find_by_label(&self, label: &str) -> ElementSnapshot {
        find_by_label_in(self.window, &self.scope, label)
    }

    fn try_find_by_label(&self, label: &str) -> Option<ElementSnapshot> {
        try_find_by_label_in(self.window, &self.scope, label)
    }

    fn find_all_by_label(&self, label: &str) -> Vec<ElementSnapshot> {
        find_all_by_label_in(self.window, &self.scope, label)
    }

    fn find_all_by_role(&self, role: Role) -> Vec<ElementSnapshot> {
        find_all_by_role_in(self.window, &self.scope, role)
    }
}
