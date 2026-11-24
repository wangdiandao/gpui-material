use std::{rc::Rc, time::Duration};

use gpui::{App, AppContext, Context, Entity, Pixels, Size, Subscription, Window, px};
use gpui_base::motion::Easing;
use gpui_material_foundation::{DragCapture, RetargetTransition};
use web_time::Instant;

use super::super::{DialogState, SheetMode};

/// Settled values for a retained bottom sheet.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BottomSheetValue {
    /// The sheet is dismissed and occupies zero height.
    #[default]
    Hidden,
    /// The sheet is resting at its intermediate peek height.
    PartiallyExpanded,
    /// The sheet is open to its full expanded extent.
    Expanded,
}

impl BottomSheetValue {
    const ORDERED: [Self; 3] = [Self::Hidden, Self::PartiallyExpanded, Self::Expanded];
}

/// Thresholds for settling a released sheet drag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BottomSheetThresholds {
    /// Distance in logical pixels.
    pub position: Pixels,
    /// Speed in logical pixels per second.
    pub velocity: Pixels,
}

impl Default for BottomSheetThresholds {
    fn default() -> Self {
        Self {
            position: px(56.),
            velocity: px(125.),
        }
    }
}

impl BottomSheetThresholds {
    pub(in crate::overlay) fn normalized(self) -> Self {
        let fallback = Self::default();
        let nonnegative = |value: Pixels, default| {
            if f32::from(value).is_finite() && value >= px(0.) {
                value
            } else {
                default
            }
        };
        Self {
            position: nonnegative(self.position, fallback.position),
            velocity: nonnegative(self.velocity, fallback.velocity),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::overlay) struct BottomSheetLayout {
    pub(in crate::overlay) available: Size<Pixels>,
    pub(in crate::overlay) expanded: Pixels,
    pub(in crate::overlay) partial: Pixels,
    pub(in crate::overlay) mode: SheetMode,
    pub(in crate::overlay) drag_enabled: bool,
}

impl BottomSheetLayout {
    fn height(self, value: BottomSheetValue) -> Pixels {
        match value {
            BottomSheetValue::Hidden => px(0.),
            BottomSheetValue::PartiallyExpanded => self.partial,
            BottomSheetValue::Expanded => self.expanded,
        }
    }
}

const SETTLE: Duration = Duration::from_millis(240);
const VELOCITY_AGE: Duration = Duration::from_millis(120);

struct SheetDrag {
    origin: Pixels,
    start_height: Pixels,
    last_position: Pixels,
    last_motion: Instant,
    velocity: Pixels,
    capture: DragCapture,
    _escape: Subscription,
    _activation: Subscription,
}

/// Retained position, available stops and transitions of a bottom sheet.
pub struct BottomSheetState {
    pub(in crate::overlay) dialog: Entity<DialogState>,
    pub(super) current: BottomSheetValue,
    pub(super) target: BottomSheetValue,
    enabled: Vec<BottomSheetValue>,
    confirm: Option<Rc<dyn Fn(BottomSheetValue) -> bool>>,
    pub(super) layout: Option<BottomSheetLayout>,
    height: Pixels,
    motion: RetargetTransition,
    pub(super) pressed: Option<Pixels>,
    drag: Option<SheetDrag>,
}

impl BottomSheetState {
    /// Creates retained bottom-sheet state starting at the given settled stop.
    pub fn new(initial: BottomSheetValue, cx: &mut Context<Self>) -> Self {
        Self {
            dialog: cx.new(DialogState::new),
            current: initial,
            target: initial,
            enabled: BottomSheetValue::ORDERED.to_vec(),
            confirm: None,
            layout: None,
            height: px(0.),
            motion: RetargetTransition::settled(0.),
            pressed: None,
            drag: None,
        }
    }

    /// Configures available stops. An empty set retains the initial stop.
    pub fn enabled_values(mut self, values: impl IntoIterator<Item = BottomSheetValue>) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        self.enabled = BottomSheetValue::ORDERED
            .into_iter()
            .filter(|value| values.contains(value))
            .collect();
        if self.enabled.is_empty() {
            self.enabled.push(self.current);
        }
        if !self.enabled.contains(&self.current) {
            self.current = self.enabled[0];
            self.target = self.current;
        }
        self
    }

    /// Vetoes a requested destination before it changes visibility or focus.
    pub fn confirm_value_change(
        mut self,
        confirm: impl Fn(BottomSheetValue) -> bool + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(confirm));
        self
    }

    /// Returns the enabled stops in ascending height order.
    pub fn available_values(&self) -> &[BottomSheetValue] {
        &self.enabled
    }

    /// Returns the last settled stop.
    pub fn current_value(&self) -> BottomSheetValue {
        self.current
    }

    /// Returns the stop the sheet is currently settled at or transitioning toward.
    pub fn target_value(&self) -> BottomSheetValue {
        self.target
    }

    /// Returns whether the sheet is currently animating toward its target stop.
    pub fn is_animating(&self) -> bool {
        self.motion.is_animating()
    }

    /// Returns the sheet's current visible height in logical pixels.
    pub fn visible_height(&self) -> Pixels {
        self.height
    }

    /// Downward distance from the expanded stop in the sheet's layout.
    pub fn offset(&self) -> Pixels {
        self.layout
            .map_or(px(0.), |layout| layout.expanded - self.height)
    }

    /// Animates the sheet to [`BottomSheetValue::PartiallyExpanded`] if enabled, or [`BottomSheetValue::Expanded`] otherwise.
    /// Returns `false` if the target stop is unavailable or vetoed.
    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let value = if self.enabled.contains(&BottomSheetValue::PartiallyExpanded) {
            BottomSheetValue::PartiallyExpanded
        } else {
            BottomSheetValue::Expanded
        };
        self.animate_to(value, window, cx)
    }

    /// Animates the sheet to [`BottomSheetValue::Expanded`]. Returns `false` if unavailable or vetoed.
    pub fn expand(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::Expanded, window, cx)
    }

    /// Animates the sheet to [`BottomSheetValue::PartiallyExpanded`]. Returns `false` if unavailable or vetoed.
    pub fn partial_expand(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::PartiallyExpanded, window, cx)
    }

    /// Animates the sheet to [`BottomSheetValue::Hidden`]. Returns `false` if unavailable or vetoed.
    pub fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::Hidden, window, cx)
    }

    /// Animates the sheet to `value`, respecting reduced-motion settings. Returns `false` if unavailable or vetoed.
    pub fn animate_to(
        &mut self,
        value: BottomSheetValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.request(value, true, true, window, cx)
    }

    /// Moves the sheet immediately to `value` without animation. Returns `false` if unavailable or vetoed.
    pub fn snap_to(
        &mut self,
        value: BottomSheetValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.request(value, false, true, window, cx)
    }

    pub(in crate::overlay) fn accept_user_close(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        (self.target != BottomSheetValue::Hidden || self.drag.is_some())
            && self.request(BottomSheetValue::Hidden, true, false, window, cx)
    }

    fn request(
        &mut self,
        value: BottomSheetValue,
        animate: bool,
        synchronize_focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.enabled.contains(&value)
            || (value != self.target
                && self.confirm.as_ref().is_some_and(|confirm| !confirm(value)))
        {
            return false;
        }
        let had_drag = self.release_drag(window, cx);
        let prev_target = self.target;
        let prev_current = self.current;
        let prev_height = self.height;
        let was_animating = self.motion.is_animating();
        self.target = value;
        let mut dialog_changed = false;
        if synchronize_focus
            && self
                .layout
                .is_some_and(|layout| layout.mode == SheetMode::Modal)
        {
            let want_open = value != BottomSheetValue::Hidden;
            if self.dialog.read(cx).is_open() != want_open {
                dialog_changed = true;
                self.dialog.update(cx, |dialog, cx| match value {
                    BottomSheetValue::Hidden => dialog.close(window, cx),
                    _ => dialog.open(window, cx),
                });
            }
        }
        self.move_to_target(animate, cx);
        let changed = had_drag
            || dialog_changed
            || prev_target != self.target
            || prev_current != self.current
            || prev_height != self.height
            || was_animating
            || self.motion.is_animating();
        if changed {
            cx.notify();
            window.refresh();
        }
        true
    }

    pub(super) fn move_to_target(&mut self, animate: bool, cx: &App) {
        let target = self
            .layout
            .map_or(px(0.), |layout| layout.height(self.target));
        let now = cx.background_executor().now();
        if !animate || cx.reduce_motion() {
            self.height = target;
            self.current = self.target;
            self.motion.snap_to(f32::from(target));
        } else {
            self.motion
                .animate_to(f32::from(target), SETTLE, Easing::EaseOut, now, false);
            self.height = px(self.motion.value());
            if !self.motion.is_animating() {
                self.current = self.target;
            }
        }
    }

    pub(in crate::overlay) fn configure(
        &mut self,
        layout: BottomSheetLayout,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let before = (
            self.height,
            self.current,
            self.target,
            self.is_animating(),
            self.offset(),
        );
        if self.layout != Some(layout) {
            let old = self.layout.replace(layout);
            self.pressed = None;
            if self.drag.is_some() {
                self.release_drag(window, cx);
                self.target = self.current;
            }
            let moving = self.motion.is_animating();
            self.height = if moving {
                self.height.clamp(px(0.), layout.expanded)
            } else {
                layout.height(self.current)
            };
            self.motion.snap_to(f32::from(self.height));
            self.move_to_target(moving, cx);
            if old.is_none_or(|old| old.mode != layout.mode) {
                let dialog = self.dialog.downgrade();
                let want_open =
                    layout.mode == SheetMode::Modal && self.target != BottomSheetValue::Hidden;
                // Defer modal focus/notify changes until GPUI leaves the render phase.
                window.defer(cx, move |window, cx| {
                    let _ = dialog.update(cx, |dialog, cx| {
                        if want_open {
                            dialog.open(window, cx);
                        } else {
                            dialog.close(window, cx);
                        }
                    });
                });
            }
        }
        if self.motion.is_animating() {
            let now = cx.background_executor().now();
            let (height, running) = self.motion.sample(now, cx.reduce_motion());
            self.height = px(height);
            if running {
                if window.is_visible() {
                    window.request_animation_frame();
                }
            } else {
                self.current = self.target;
            }
        }
        let after = (
            self.height,
            self.current,
            self.target,
            self.is_animating(),
            self.offset(),
        );
        if before != after {
            // Deliver the notification after GPUI leaves the draw phase.
            let state = cx.weak_entity();
            window.defer(cx, move |_, cx| {
                let _ = state.update(cx, |_, cx| cx.notify());
            });
        }
    }

    fn release_drag(&mut self, window: &mut Window, cx: &mut App) -> bool {
        self.pressed = None;
        if let Some(drag) = self.drag.take() {
            drag.capture.cancel(window, cx);
            true
        } else {
            false
        }
    }

    fn cancel_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.release_drag(window, cx) {
            self.target = self.current;
            self.move_to_target(true, cx);
            cx.notify();
            window.refresh();
            true
        } else {
            false
        }
    }

    pub(super) fn begin_drag(&mut self, y: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        if !f32::from(y).is_finite() || self.layout.is_none_or(|layout| !layout.drag_enabled) {
            return;
        }
        let origin = self.pressed.take().unwrap_or(y);
        if self.drag.is_some() || self.enabled.len() < 2 {
            return;
        }
        let was_animating = self.motion.is_animating();
        self.motion.snap_to(f32::from(self.height));
        if was_animating
            && self.current == BottomSheetValue::Hidden
            && self.target != BottomSheetValue::Hidden
        {
            self.current = self.target;
        }
        self.target = self.current;
        let state = cx.weak_entity();
        let capture = DragCapture::watch(window, cx, {
            let state = state.clone();
            move |window, cx| {
                let _ = state.update(cx, |state, cx| state.cancel_drag(window, cx));
            }
        });
        let window_id = window.window_handle().window_id();
        let escape = cx.intercept_keystrokes(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && window.window_handle().window_id() == window_id
                && state
                    .update(cx, |state, cx| state.cancel_drag(window, cx))
                    .unwrap_or(false)
            {
                cx.stop_propagation();
            }
        });
        let activation = cx.observe_window_activation(window, |state, window, cx| {
            if !window.is_window_active() {
                state.cancel_drag(window, cx);
            }
        });
        self.drag = Some(SheetDrag {
            origin,
            start_height: self.height,
            last_position: origin,
            last_motion: cx.background_executor().now(),
            velocity: px(0.),
            capture,
            _escape: escape,
            _activation: activation,
        });
    }

    pub(super) fn drag_to(&mut self, y: Pixels, cx: &mut Context<Self>) -> Option<bool> {
        if !f32::from(y).is_finite() {
            return None;
        }
        let (Some(drag), Some(layout)) = (&mut self.drag, self.layout) else {
            return None;
        };
        let now = cx.background_executor().now();
        if y != drag.last_position {
            let elapsed = now
                .saturating_duration_since(drag.last_motion)
                .as_secs_f32();
            if elapsed > 0. {
                let velocity = (drag.last_position - y) / elapsed;
                if f32::from(velocity).is_finite() {
                    drag.velocity = velocity;
                }
            }
            drag.last_position = y;
            drag.last_motion = now;
        }
        let low = layout.height(self.enabled[0]);
        let high = layout.height(*self.enabled.last().expect("an available stop"));
        let next = (drag.start_height + drag.origin - y).clamp(low, high);
        let changed = self.height != next || self.motion.is_animating();
        self.height = next;
        self.motion.snap_to(f32::from(self.height));
        if changed {
            cx.notify();
        }
        Some(changed)
    }

    pub(super) fn finish_drag(
        &mut self,
        position: Pixels,
        thresholds: BottomSheetThresholds,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<BottomSheetValue> {
        self.drag_to(position, cx)?;
        let drag = self.drag.as_ref().expect("the active drag");
        let velocity = if cx
            .background_executor()
            .now()
            .saturating_duration_since(drag.last_motion)
            <= VELOCITY_AGE
        {
            drag.velocity
        } else {
            px(0.)
        };
        let target = self.settle_target(velocity, thresholds);
        self.release_drag(window, cx);
        Some(target)
    }

    fn settle_target(
        &self,
        velocity: Pixels,
        thresholds: BottomSheetThresholds,
    ) -> BottomSheetValue {
        let layout = self.layout.expect("a laid out drag");
        let start = layout.height(self.current);
        let displacement = self.height - start;
        let fling = velocity != px(0.) && velocity.abs() >= thresholds.velocity;
        if !fling && displacement == px(0.) {
            return self.current;
        }
        let positive = if fling {
            velocity > px(0.)
        } else {
            displacement > px(0.)
        };
        let mut stops = self.enabled.iter().copied().filter(|value| {
            let height = layout.height(*value);
            if positive {
                height > start
            } else {
                height < start
            }
        });
        if fling {
            return if positive {
                stops.next()
            } else {
                stops.next_back()
            }
            .unwrap_or(self.current);
        }
        let mut crossed = stops.filter(|value| {
            let distance = (layout.height(*value) - start).abs();
            displacement.abs() >= thresholds.position.min(distance / 2.)
        });
        if positive {
            crossed.next_back()
        } else {
            crossed.next()
        }
        .unwrap_or(self.current)
    }

    pub(super) fn destination(&self, key: Option<&str>) -> Option<BottomSheetValue> {
        let current = self
            .enabled
            .iter()
            .position(|value| *value == self.target)?;
        match key {
            None => self.enabled.get((current + 1) % self.enabled.len()),
            Some("up") => self.enabled.get(current + 1),
            Some("down") => current
                .checked_sub(1)
                .and_then(|index| self.enabled.get(index)),
            Some("home") => self.enabled.first(),
            Some("end") => self.enabled.last(),
            _ => None,
        }
        .copied()
        .filter(|value| *value != self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_sanitize_non_finite_and_negative_values() {
        let defaults = BottomSheetThresholds::default();
        assert_eq!(
            BottomSheetThresholds {
                position: px(f32::NAN),
                velocity: px(-10.),
            }
            .normalized(),
            defaults
        );
        assert_eq!(
            BottomSheetThresholds {
                position: px(24.),
                velocity: px(0.),
            }
            .normalized(),
            BottomSheetThresholds {
                position: px(24.),
                velocity: px(0.),
            }
        );
    }
}
