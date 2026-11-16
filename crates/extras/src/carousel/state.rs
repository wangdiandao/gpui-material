use std::{collections::HashMap, rc::Rc};

use gpui::{Context, FocusHandle, SharedString, Task};

use super::CarouselItem;

pub(super) struct ItemFocus {
    pub(super) id: SharedString,
    pub(super) enabled: bool,
    pub(super) focus: FocusHandle,
}

/// Retained scroll position for one carousel. Observe the entity to read
/// changes from input, programmatic navigation, or item replacement.
pub struct CarouselState {
    pub(super) items: Rc<[ItemFocus]>,
    pub(super) requested: f32,
    pub(super) sampled: f32,
    pub(super) direct: bool,
    pub(super) settle: Option<Task<()>>,
    pub(super) gesture_start: Option<f32>,
    pending_index: Option<usize>,
}

impl CarouselState {
    /// Starts at the given index when the first items are available.
    pub fn new(initial_index: usize) -> Self {
        Self {
            items: Rc::from([]),
            requested: 0.,
            sampled: 0.,
            direct: true,
            settle: None,
            gesture_start: None,
            pending_index: Some(initial_index),
        }
    }

    /// Returns the item nearest the keyline, or no index when empty.
    pub fn current_item(&self) -> Option<usize> {
        (!self.items.is_empty()).then(|| self.sampled.round() as usize)
    }

    /// Returns the stable ID of the item nearest the keyline.
    pub fn current_item_id(&self) -> Option<&SharedString> {
        self.current_item().map(|index| &self.items[index].id)
    }

    /// Signed displacement from the nearest keyline in item units.
    pub fn current_item_offset_fraction(&self) -> f32 {
        self.sampled - self.sampled.round()
    }

    /// Immediately positions the carousel at the clamped index.
    pub fn scroll_to_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.navigate(index, true, cx);
    }

    /// Moves to the clamped index using the window's motion policy.
    pub fn animate_scroll_to_item(&mut self, index: usize, cx: &mut Context<Self>) {
        let immediate = cx.reduce_motion();
        self.navigate(index, immediate, cx);
    }

    fn navigate(&mut self, index: usize, immediate: bool, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            self.pending_index = Some(index);
        } else {
            let index = index.min(self.items.len() - 1);
            self.settle = None;
            self.gesture_start = None;
            self.set_requested(index as f32, immediate, cx);
        }
    }

    pub(super) fn set_requested(&mut self, position: f32, immediate: bool, cx: &mut Context<Self>) {
        if position != self.requested || self.direct != immediate {
            self.requested = position;
            self.direct = immediate;
            cx.notify();
        }
        if immediate {
            self.sample(position, cx);
        }
    }

    pub(super) fn cancel_input(&mut self, cx: &mut Context<Self>) {
        if self.gesture_start.take().is_some() {
            self.settle = None;
            self.set_requested(self.requested, true, cx);
        }
    }

    pub(super) fn sample(&mut self, sample: f32, cx: &mut Context<Self>) {
        let sample = sample.clamp(0., self.items.len().saturating_sub(1) as f32);
        if sample != self.sampled {
            self.sampled = sample;
            cx.notify();
        }
    }

    pub(super) fn sync(&mut self, items: &[CarouselItem], cx: &mut Context<Self>) {
        let same_order = items.len() == self.items.len()
            && items
                .iter()
                .zip(self.items.iter())
                .all(|(item, known)| item.id == known.id);
        if same_order
            && items
                .iter()
                .zip(self.items.iter())
                .all(|(item, known)| item.disabled != known.enabled)
        {
            return;
        }

        let previous_index = self.requested.round() as usize;
        let previous_id = self.items.get(previous_index).map(|item| item.id.clone());
        let previous: HashMap<_, _> = self
            .items
            .iter()
            .map(|item| (&item.id, &item.focus))
            .collect();
        self.items = items
            .iter()
            .map(|item| ItemFocus {
                id: item.id.clone(),
                enabled: !item.disabled,
                focus: previous
                    .get(&item.id)
                    .map_or_else(|| cx.focus_handle(), |focus| (*focus).clone()),
            })
            .collect();
        if !same_order {
            let index = if self.items.is_empty() {
                0
            } else {
                self.pending_index
                    .take()
                    .unwrap_or_else(|| {
                        previous_id
                            .and_then(|id| self.items.iter().position(|item| item.id == id))
                            .unwrap_or(previous_index)
                    })
                    .min(self.items.len() - 1)
            };
            self.settle = None;
            self.gesture_start = None;
            self.set_requested(index as f32, true, cx);
            // A different list can change the current item's ID at the same offset.
            cx.notify();
        }
    }
}
