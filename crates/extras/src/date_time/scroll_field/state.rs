use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{BackgroundExecutor, Context, FocusHandle, ScrollEvent, Task, TouchPhase, Window, px};

pub(super) const HEIGHT: f32 = 200.;
pub(super) const ROW_HEIGHT: f32 = HEIGHT / 3.;
const PRECISE_SCROLL_SETTLE: Duration = Duration::from_millis(120);
// Move near a distant destination before animating the last few rows.
const ANIMATED_ROWS: usize = 3;

struct SettleTimer {
    _task: Task<()>,
    bump: Box<dyn Fn(&BackgroundExecutor)>,
}

/// Caller-owned position and focus for one Material scroll field.
pub struct ScrollFieldState {
    pub(super) item_count: usize,
    pub(super) position: Position,
    pub(super) target: usize,
    pub(super) motion: Option<Motion>,
    pub(super) gesture_start: Option<usize>,
    settle: Option<SettleTimer>,
    pub(super) focus: FocusHandle,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Position {
    pub(super) index: usize,
    pub(super) fraction: f32,
}

impl Position {
    fn at(index: usize) -> Self {
        Self {
            index,
            fraction: 0.,
        }
    }

    fn shift(self, rows: f32, item_count: usize) -> Self {
        let offset = self.fraction + rows;
        let shift = offset.round();
        Self {
            index: wrap(self.index, shift as isize, item_count),
            fraction: offset - shift,
        }
    }

    fn offset_from(self, target: usize) -> f32 {
        let distance = self.index.abs_diff(target).min(ANIMATED_ROWS) as f32;
        self.fraction
            + if self.index < target {
                -distance
            } else {
                distance
            }
    }
}

pub(super) fn wrap(index: usize, rows: isize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let index = index % count;
    let step = rows.unsigned_abs() % count;
    if rows < 0 {
        index
            .checked_sub(step)
            .unwrap_or_else(|| count - (step - index))
    } else if step >= count - index {
        step - (count - index)
    } else {
        index + step
    }
}

#[derive(Clone, Copy)]
pub(super) struct Motion {
    pub(super) start: f32,
    pub(super) restart: bool,
    origin: ScrollOrigin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ScrollOrigin {
    Application,
    Interaction,
}

impl ScrollFieldState {
    /// Creates a position at the requested option, clamped to the available indices.
    pub fn new(item_count: usize, index: usize, cx: &mut Context<Self>) -> Self {
        let index = index.min(item_count.saturating_sub(1));
        Self {
            item_count,
            position: Position::at(index),
            target: index,
            motion: None,
            gesture_start: None,
            settle: None,
            focus: cx.focus_handle(),
        }
    }

    /// Returns the total number of selectable options in this scroll field.
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// Updates the options and centers the clamped destination immediately.
    pub fn set_item_count(&mut self, count: usize, cx: &mut Context<Self>) {
        if self.item_count != count {
            self.item_count = count;
            self.scroll_to_option(self.target, cx);
        }
    }

    /// Reads the nearest option or `None` when the field is empty.
    pub fn selected_option(&self) -> Option<usize> {
        (self.item_count > 0).then_some(self.position.index)
    }

    /// Reads the destination, including while the wheel is settling.
    pub fn target_option(&self) -> Option<usize> {
        (self.item_count > 0).then_some(self.target)
    }

    /// Returns whether a scroll gesture or settling animation is currently active.
    pub fn is_scroll_in_progress(&self) -> bool {
        self.motion.is_some() || self.gesture_start.is_some()
    }

    /// Returns the retained focus handle for this scroll field.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// Centers the clamped option without animation.
    pub fn scroll_to_option(&mut self, index: usize, cx: &mut Context<Self>) {
        let target = index.min(self.item_count.saturating_sub(1));
        let position = Position::at(target);
        if self.target == target
            && self.position == position
            && self.gesture_start.is_none()
            && self.settle.is_none()
            && self.motion.is_none()
        {
            return;
        }
        self.target = target;
        self.gesture_start = None;
        self.settle = None;
        self.motion = None;
        self.position = position;
        cx.notify();
    }

    /// Scrolls to the clamped option using the window's motion policy.
    pub fn animate_scroll_to_option(&mut self, index: usize, cx: &mut Context<Self>) {
        self.animate_to(index, ScrollOrigin::Application, cx);
    }

    pub(super) fn animate_to(
        &mut self,
        index: usize,
        origin: ScrollOrigin,
        cx: &mut Context<Self>,
    ) {
        let index = index.min(self.item_count.saturating_sub(1));
        if self.item_count > 0 && (self.target != index || self.gesture_start.is_some()) {
            self.snap(index, self.position.offset_from(index), origin, cx);
        }
    }

    fn snap(&mut self, index: usize, start: f32, origin: ScrollOrigin, cx: &mut Context<Self>) {
        if start == 0. || cx.reduce_motion() {
            self.scroll_to_option(index, cx);
        } else {
            self.target = index;
            self.gesture_start = None;
            self.settle = None;
            self.motion = Some(Motion {
                start,
                restart: true,
                origin,
            });
            cx.notify();
        }
    }

    pub(super) fn sample(&mut self, offset: f32, cx: &mut Context<Self>) {
        let position = Position::at(self.target).shift(offset, self.item_count);
        let changed = self.position.index != position.index || offset == 0.;
        self.position = position;
        self.motion = self.motion.filter(|_| offset != 0.).map(|mut motion| {
            motion.restart = false;
            motion
        });
        if changed {
            cx.notify();
        }
    }

    pub(super) fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.item_count < 2 {
            return;
        }
        let index = wrap(self.target, delta, self.item_count);
        let (origin, destination) = if delta > 0 {
            (self.position.index, index)
        } else {
            (index, self.position.index)
        };
        let distance = if destination >= origin {
            destination - origin
        } else {
            self.item_count - (origin - destination)
        };
        let start =
            self.position.fraction - delta.signum() as f32 * distance.min(ANIMATED_ROWS) as f32;
        self.snap(index, start, ScrollOrigin::Interaction, cx);
    }

    pub(super) fn select_row(&mut self, row: isize, cx: &mut Context<Self>) {
        if self.item_count > 0 {
            let index = wrap(self.position.index, row, self.item_count);
            self.snap(
                index,
                self.position.fraction - row as f32,
                ScrollOrigin::Interaction,
                cx,
            );
        }
    }

    pub(super) fn scroll(
        &mut self,
        event: &ScrollEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.item_count < 2 {
            return false;
        }
        let delta = event.delta.pixel_delta(px(ROW_HEIGHT));
        let x = f32::from(delta.x);
        let y = f32::from(delta.y);
        let vertical = y.is_finite() && x.abs() <= y.abs() && y != 0.;
        if !event.delta.precise() {
            if vertical {
                self.step(if y < 0. { 1 } else { -1 }, cx);
            }
            return vertical;
        }
        if event.touch_phase == TouchPhase::Cancelled {
            return self.finish_gesture(true, cx);
        }
        if vertical {
            self.gesture_start.get_or_insert(self.position.index);
            self.motion = None;
            self.position = self.position.shift(-y / ROW_HEIGHT, self.item_count);
            self.target = self.position.index;
            cx.notify();
        }
        if event.touch_phase == TouchPhase::Ended {
            return self.finish_gesture(false, cx);
        }
        if vertical {
            // Some discrete touchpad updates omit a terminal phase. Reuse the
            // active settle timer across rapid scroll updates instead of
            // spawning and cancelling a task on every event.
            if let Some(settle) = &self.settle {
                (settle.bump)(cx.background_executor());
            } else {
                let origin = cx.background_executor().now();
                let last_offset = Rc::new(Cell::new(Duration::ZERO));
                let bump_offset = last_offset.clone();
                let bump = Box::new(move |executor: &BackgroundExecutor| {
                    bump_offset.set(executor.now().saturating_duration_since(origin));
                });
                let state = cx.weak_entity();
                let task = window.spawn(cx, async move |cx| {
                    let mut wait = PRECISE_SCROLL_SETTLE;
                    loop {
                        let Ok(timer) = cx.update(|_, cx| cx.background_executor().timer(wait))
                        else {
                            return;
                        };
                        timer.await;
                        let next_wait = cx.update(|_, cx| {
                            let elapsed = cx
                                .background_executor()
                                .now()
                                .saturating_duration_since(origin);
                            let deadline = last_offset.get() + PRECISE_SCROLL_SETTLE;
                            if elapsed < deadline {
                                Some(deadline - elapsed)
                            } else {
                                let _ =
                                    state.update(cx, |state, cx| state.finish_gesture(false, cx));
                                None
                            }
                        });
                        match next_wait {
                            Ok(Some(remaining)) => wait = remaining,
                            _ => return,
                        }
                    }
                });
                self.settle = Some(SettleTimer { _task: task, bump });
            }
        }
        vertical
    }

    fn finish_gesture(&mut self, cancelled: bool, cx: &mut Context<Self>) -> bool {
        let Some(start) = self.gesture_start else {
            return false;
        };
        let index = if cancelled {
            start
        } else {
            self.position.index
        };
        self.snap(
            index,
            self.position.offset_from(index),
            ScrollOrigin::Interaction,
            cx,
        );
        true
    }

    pub(super) fn has_pending_interaction(&self) -> bool {
        self.gesture_start.is_some()
            || self
                .motion
                .is_some_and(|motion| motion.origin == ScrollOrigin::Interaction)
    }
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wrap_guards_against_zero_count_and_wraps_around_bounds() {
        assert_eq!(wrap(0, 0, 0), 0);
        assert_eq!(wrap(5, 3, 0), 0);
        assert_eq!(wrap(5, -3, 0), 0);
        assert_eq!(wrap(0, -1, 12), 11);
        assert_eq!(wrap(11, 1, 12), 0);
        assert_eq!(wrap(2, 25, 12), 3);
        assert_eq!(wrap(2, -25, 12), 1);
    }
}
