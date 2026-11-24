use std::collections::HashMap;

use gpui::{App, ElementId, Entity, FocusHandle, ScrollHandle, SharedString, Window};
use gpui_material_theme::Direction;

use super::{super::tabs::TabReveal, NavigationItem, Presentation};

#[derive(Default)]
pub(in crate::navigation) struct RowState {
    pub(super) selection: Option<SharedString>,
    pub(super) focus: HashMap<SharedString, FocusHandle>,
    pub(super) horizontal_scroll: ScrollHandle,
    pub(super) vertical_scroll: ScrollHandle,
    pub(super) reveal: Option<(Presentation, Direction, SharedString, usize)>,
    pub(super) tab_reveal: TabReveal,
    pub(super) roving_stop: Option<SharedString>,
}

impl RowState {
    /// Synchronizes focus handles for a list of destination IDs, preserving existing handles
    /// across adaptive presentations (e.g. Bar <-> Rail <-> Drawer).
    pub(in crate::navigation) fn sync_focus_map(
        &mut self,
        ids: &[SharedString],
        cx: &mut App,
    ) -> HashMap<SharedString, FocusHandle> {
        let mut previous = std::mem::take(&mut self.focus);
        let mut map = HashMap::with_capacity(ids.len());
        for id in ids {
            let handle = previous.remove(id).unwrap_or_else(|| cx.focus_handle());
            map.insert(id.clone(), handle);
        }
        self.focus = map.clone();
        map
    }
}

/// Returns the shared navigation focus/scroll state entity keyed by collection `id`.
pub(in crate::navigation) fn row_state_for(
    id: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> Entity<RowState> {
    window.use_keyed_state((id.clone(), "navigation-state"), cx, |_, _| {
        RowState::default()
    })
}

pub(super) fn sync_focus_and_scroll(
    state: &Entity<RowState>,
    items: &[NavigationItem],
    rail: bool,
    cx: &mut App,
) -> (Vec<FocusHandle>, ScrollHandle) {
    state.update(cx, |state, cx| {
        let mut previous = std::mem::take(&mut state.focus);
        let mut handles = Vec::with_capacity(items.len());
        state.focus = items
            .iter()
            .map(|item| {
                let handle = previous
                    .remove(&item.id)
                    .unwrap_or_else(|| cx.focus_handle());
                handles.push(handle.clone());
                (item.id.clone(), handle)
            })
            .collect();
        let scroll = if rail {
            &state.vertical_scroll
        } else {
            &state.horizontal_scroll
        };
        (handles, scroll.clone())
    })
}

pub(super) fn resolve_tab_stop(
    state: &Entity<RowState>,
    enabled: &[(SharedString, FocusHandle)],
    focused: Option<&(SharedString, FocusHandle)>,
    selected: Option<&SharedString>,
    retain_roving_stop: bool,
    cx: &mut App,
) -> Option<SharedString> {
    let first = enabled.first().map(|(id, _)| id);
    let remembered = retain_roving_stop
        .then(|| {
            state.update(cx, |state, _| {
                if let Some((id, _)) = focused {
                    state.roving_stop = Some(id.clone());
                } else if state.selection.as_ref() != selected {
                    state.roving_stop = None;
                }
                state.roving_stop.clone()
            })
        })
        .flatten();
    let enabled_id = |id: &&SharedString| enabled.iter().any(|(item, _)| item == *id);
    focused
        .map(|(id, _)| id)
        .or_else(|| remembered.as_ref().filter(enabled_id))
        .or_else(|| selected.filter(enabled_id))
        .or(first)
        .cloned()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_reveal_state(
    state: &Entity<RowState>,
    items: &[NavigationItem],
    presentation: Presentation,
    direction: Direction,
    selected: Option<&SharedString>,
    focused_id: Option<&SharedString>,
    tab: bool,
    scrollable: bool,
    scroll: &ScrollHandle,
    cx: &mut App,
) -> Option<usize> {
    if tab && scrollable {
        return state.update(cx, |state, _| {
            state.tab_reveal.choose(selected, focused_id, items, scroll)
        });
    }
    state.update(cx, |state, _| {
        let reveal_target = if state.selection.as_ref() != selected {
            selected.or(focused_id)
        } else {
            focused_id.or(selected)
        };
        let reveal = reveal_target.and_then(|id| {
            items
                .iter()
                .position(|item| &item.id == id)
                .map(|index| (presentation, direction, id.clone(), index))
        });
        if scrollable
            && state.reveal != reveal
            && let Some((_, _, _, index)) = &reveal
        {
            scroll.scroll_to_item(*index);
        }
        state.reveal = reveal;
        state.selection = selected.cloned();
        state.tab_reveal = TabReveal::default();
    });
    None
}

pub(super) fn next_item(
    key: &str,
    current: usize,
    len: usize,
    vertical: bool,
    rtl: bool,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let forward = match key {
        "home" => return Some(0),
        "end" => return Some(len - 1),
        "down" if vertical => true,
        "up" if vertical => false,
        "right" if !vertical => !rtl,
        "left" if !vertical => rtl,
        _ => return None,
    };
    Some((current + if forward { 1 } else { len - 1 }) % len)
}

#[cfg(test)]
mod tests {
    use super::next_item;

    #[test]
    fn next_item_handles_ltr_rtl_vertical_home_end_and_empty() {
        assert_eq!(next_item("right", 0, 0, false, false), None);
        assert_eq!(next_item("right", 0, 3, false, false), Some(1));
        assert_eq!(next_item("left", 0, 3, false, false), Some(2));
        assert_eq!(next_item("right", 0, 3, false, true), Some(2));
        assert_eq!(next_item("left", 0, 3, false, true), Some(1));
        assert_eq!(next_item("down", 1, 3, true, false), Some(2));
        assert_eq!(next_item("up", 0, 3, true, false), Some(2));
        assert_eq!(next_item("down", 1, 3, false, false), None);
        assert_eq!(next_item("home", 2, 3, false, false), Some(0));
        assert_eq!(next_item("end", 0, 3, true, false), Some(2));
    }
}
