use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{App, Context, Entity, ScrollEvent, TouchPhase, Window, px};
use gpui_material_theme::Direction;

use super::{
    CarouselFling, CarouselFlingContext, CarouselState, geometry::Layout, state::ItemFocus,
};

const SCROLL_SETTLE: Duration = Duration::from_millis(120);

#[derive(Clone)]
pub(super) struct Interaction {
    pub(super) layout: Rc<RefCell<Option<Layout>>>,
    pub(super) focus: Rc<[ItemFocus]>,
    pub(super) state: Entity<CarouselState>,
    pub(super) vertical: bool,
    pub(super) direction: Direction,
    pub(super) fling: CarouselFling,
}

impl Interaction {
    pub(super) fn page(&self, delta: isize, window: &mut Window, cx: &mut App) {
        let state = self.state.read(cx);
        let index = (state.requested.round() as usize).saturating_add_signed(delta);
        if index < self.focus.len() {
            self.state
                .update(cx, |state, cx| state.animate_scroll_to_item(index, cx));
            window.refresh();
        }
    }

    pub(super) fn on_scroll(&self, event: &ScrollEvent, window: &mut Window, cx: &mut App) {
        let delta = event.delta.pixel_delta(px(32.));
        let x = f32::from(delta.x);
        let y = f32::from(delta.y);
        let horizontal_input = if event.modifiers.shift && x.abs() < y.abs() {
            y
        } else {
            x
        };
        let main = if self.vertical {
            if x.abs() > y.abs() {
                return;
            }
            y
        } else {
            if !event.modifiers.shift && y.abs() > x.abs() {
                return;
            }
            if self.direction == Direction::Rtl {
                -horizontal_input
            } else {
                horizontal_input
            }
        };
        if self.focus.len() < 2 || !main.is_finite() {
            return;
        }

        let moved = self.state.update(cx, |state, cx| {
            if event.touch_phase == TouchPhase::Cancelled {
                return finish_gesture(state, &self.fling, true, cx);
            }

            let previous = state.requested;
            let limit = (self.focus.len() - 1) as f32;
            if !event.delta.precise() {
                let sign = if main == 0. { 0. } else { main.signum() };
                let target = (previous - sign).clamp(0., limit);
                if target == previous {
                    return false;
                }
                state.animate_scroll_to_item(target.round() as usize, cx);
                return true;
            }

            let delta = {
                let layout = self.layout.borrow();
                let Some(layout) = layout.as_ref() else {
                    return false;
                };
                layout.scroll_delta(previous, -main)
            };
            if !delta.is_finite() {
                return false;
            }
            let target = (previous + delta).clamp(0., limit);
            if target != previous {
                state.gesture_start.get_or_insert(previous);
                state.set_requested(target, true, cx);
            }

            if event.touch_phase == TouchPhase::Ended {
                return finish_gesture(state, &self.fling, false, cx);
            }
            if target == previous {
                return false;
            }

            // Some desktop drivers omit the terminal phase of a precise gesture.
            // Reuse a single settle timer per gesture and bump it on subsequent scroll events.
            state.settle_fling = Some(self.fling.clone());
            if state.settle.is_some() {
                state.settle_bumped = true;
            } else {
                state.settle_bumped = false;
                let weak_state = cx.weak_entity();
                let executor = cx.background_executor().clone();
                state.settle = Some(window.spawn(cx, async move |cx| {
                    loop {
                        executor.timer(SCROLL_SETTLE).await;
                        let mut should_continue = false;
                        let ok = cx.update(|window, cx| {
                            let _ = weak_state.update(cx, |state, cx| {
                                if state.settle_bumped {
                                    state.settle_bumped = false;
                                    should_continue = true;
                                    return;
                                }
                                let fling = state.settle_fling.take().unwrap_or_default();
                                if finish_gesture(state, &fling, false, cx) {
                                    window.refresh();
                                }
                            });
                        });
                        if ok.is_err() || !should_continue {
                            break;
                        }
                    }
                }));
            }
            true
        });
        if !moved {
            return;
        }

        // A pointer scroll can move already-contained focus to a visible item.
        let target = self.state.read(cx).requested;
        let viewport = {
            let layout = self.layout.borrow();
            layout
                .as_ref()
                .map(|layout| layout.visible(target, self.direction))
                .unwrap_or_default()
        };
        let current_focus = self
            .focus
            .iter()
            .position(|item| item.focus.is_focused(window));
        if current_focus.is_some_and(|index| !viewport.iter().any(|(visible, _)| *visible == index))
            && let Some((index, _)) = viewport
                .iter()
                .filter(|(index, _)| self.focus[*index].enabled)
                .max_by(|(_, a), (_, b)| a.size.total_cmp(&b.size))
        {
            self.focus[*index].focus.focus(window, cx);
        }
        window.refresh();
        cx.stop_propagation();
    }
}

fn finish_gesture(
    state: &mut CarouselState,
    fling: &CarouselFling,
    cancelled: bool,
    cx: &mut Context<CarouselState>,
) -> bool {
    let Some(start) = state.gesture_start.take() else {
        return false;
    };
    state.clear_settle();
    let position = state.requested;
    let resting_item = if cancelled {
        if matches!(fling, CarouselFling::Free) {
            state.set_requested(start, true, cx);
            None
        } else {
            Some(start.round() as usize)
        }
    } else {
        let item = position.round() as usize;
        let initial = start.round() as usize;
        match fling {
            CarouselFling::Free => None,
            CarouselFling::Default | CarouselFling::Snap => Some(item),
            CarouselFling::SingleAdvance => {
                Some(item.clamp(initial.saturating_sub(1), initial + 1))
            }
            CarouselFling::Custom(target) => target(CarouselFlingContext {
                start,
                position,
                item_count: state.items.len(),
            }),
        }
    };
    if let Some(item) = resting_item {
        state.animate_scroll_to_item(item, cx);
    }
    true
}

pub(super) fn focus_at_position(items: &[ItemFocus], position: f32) -> Option<usize> {
    let index = (position.round() as usize).min(items.len().saturating_sub(1));
    (index..items.len())
        .chain((0..index).rev())
        .find(|index| items[*index].enabled)
}

pub(super) fn navigation_index(
    items: &[ItemFocus],
    position: f32,
    key: &str,
    vertical: bool,
    direction: Direction,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let current = (position.round() as usize).min(items.len() - 1);
    let forward = match key {
        "home" => return items.iter().position(|item| item.enabled),
        "end" => return items.iter().rposition(|item| item.enabled),
        "up" if vertical => false,
        "down" if vertical => true,
        "left" if !vertical => direction == Direction::Rtl,
        "right" if !vertical => direction == Direction::Ltr,
        _ => return None,
    };
    if forward {
        (current + 1..items.len()).find(|index| items[*index].enabled)
    } else {
        (0..current).rev().find(|index| items[*index].enabled)
    }
}
