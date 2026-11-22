use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, AppContext, Context, DragMoveEvent, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Render, Role,
    SharedString, Size, StatefulInteractiveElement, Styled, Subscription, WeakEntity, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{TestSupportExt as _, motion::Easing};
use gpui_material_foundation::{DragCapture, RetargetTransition};
use web_time::Instant;

use super::{DialogState, SheetClose, SheetMode};

/// Settled values for a retained bottom sheet.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BottomSheetValue {
    #[default]
    Hidden,
    PartiallyExpanded,
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
    pub(super) fn normalized(self) -> Self {
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
pub(super) struct BottomSheetLayout {
    pub(super) available: Size<Pixels>,
    pub(super) expanded: Pixels,
    pub(super) partial: Pixels,
    pub(super) mode: SheetMode,
    pub(super) drag_enabled: bool,
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
    pub(super) dialog: Entity<DialogState>,
    current: BottomSheetValue,
    target: BottomSheetValue,
    enabled: Vec<BottomSheetValue>,
    confirm: Option<Rc<dyn Fn(BottomSheetValue) -> bool>>,
    layout: Option<BottomSheetLayout>,
    height: Pixels,
    motion: RetargetTransition,
    pressed: Option<Pixels>,
    drag: Option<SheetDrag>,
}

impl BottomSheetState {
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

    pub fn available_values(&self) -> &[BottomSheetValue] {
        &self.enabled
    }

    pub fn current_value(&self) -> BottomSheetValue {
        self.current
    }

    pub fn target_value(&self) -> BottomSheetValue {
        self.target
    }

    pub fn is_animating(&self) -> bool {
        self.motion.is_animating()
    }

    pub fn visible_height(&self) -> Pixels {
        self.height
    }

    /// Downward distance from the expanded stop in the sheet's layout.
    pub fn offset(&self) -> Pixels {
        self.layout
            .map_or(px(0.), |layout| layout.expanded - self.height)
    }

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let value = if self.enabled.contains(&BottomSheetValue::PartiallyExpanded) {
            BottomSheetValue::PartiallyExpanded
        } else {
            BottomSheetValue::Expanded
        };
        self.animate_to(value, window, cx)
    }

    pub fn expand(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::Expanded, window, cx)
    }

    pub fn partial_expand(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::PartiallyExpanded, window, cx)
    }

    pub fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.animate_to(BottomSheetValue::Hidden, window, cx)
    }

    pub fn animate_to(
        &mut self,
        value: BottomSheetValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.request(value, true, true, window, cx)
    }

    pub fn snap_to(
        &mut self,
        value: BottomSheetValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.request(value, false, true, window, cx)
    }

    pub(super) fn accept_user_close(
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

    fn move_to_target(&mut self, animate: bool, cx: &App) {
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

    pub(super) fn configure(
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
                self.dialog.update(cx, |dialog, cx| {
                    if layout.mode == SheetMode::Modal && self.target != BottomSheetValue::Hidden {
                        dialog.open(window, cx);
                    } else {
                        dialog.close(window, cx);
                    }
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
            cx.defer(move |cx| {
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

    fn begin_drag(&mut self, y: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        if self.layout.is_none_or(|layout| !layout.drag_enabled) {
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

    fn drag_to(&mut self, y: Pixels, cx: &mut Context<Self>) -> Option<bool> {
        let (Some(drag), Some(layout)) = (&mut self.drag, self.layout) else {
            return None;
        };
        let now = cx.background_executor().now();
        if y != drag.last_position {
            let elapsed = now
                .saturating_duration_since(drag.last_motion)
                .as_secs_f32();
            if elapsed > 0. {
                drag.velocity = (drag.last_position - y) / elapsed;
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

    fn finish_drag(
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

    fn destination(&self, key: Option<&str>) -> Option<BottomSheetValue> {
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

#[derive(Clone)]
struct Driver {
    state: WeakEntity<BottomSheetState>,
    close: SheetClose,
    thresholds: BottomSheetThresholds,
}

impl Driver {
    fn navigate(&self, key: Option<&str>, window: &mut Window, cx: &mut App) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        if let Some(target) = state.read(cx).destination(key) {
            self.request(target, window, cx);
            true
        } else {
            false
        }
    }

    fn request(&self, target: BottomSheetValue, window: &mut Window, cx: &mut App) -> bool {
        if target == BottomSheetValue::Hidden {
            self.close.request(window, cx)
        } else {
            self.state
                .update(cx, |state, cx| state.animate_to(target, window, cx))
                .unwrap_or(false)
        }
    }

    fn release(&self, y: Pixels, window: &mut Window, cx: &mut App) {
        let target = self
            .state
            .update(cx, |state, cx| {
                state.finish_drag(y, self.thresholds, window, cx)
            })
            .ok()
            .flatten();
        if let Some(target) = target {
            if !self.request(target, window, cx) {
                let _ = self.state.update(cx, |state, cx| {
                    state.target = state.current;
                    state.move_to_target(true, cx);
                    cx.notify();
                    window.refresh();
                });
            }
            cx.stop_propagation();
        }
    }
}

struct DragPayload;

impl Render for DragPayload {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(super) struct BottomHandle {
    pub(super) id: ElementId,
    pub(super) state: Entity<BottomSheetState>,
    pub(super) close: SheetClose,
    pub(super) decoration: Option<AnyElement>,
    pub(super) label: SharedString,
    pub(super) grip: bool,
    pub(super) thresholds: BottomSheetThresholds,
    pub(super) tint: Hsla,
    pub(super) active: bool,
}

impl BottomHandle {
    pub(super) fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let model = self.state.read(cx);
        let enabled = self.active && model.available_values().len() > 1;
        let draggable =
            enabled && self.grip && model.layout.is_some_and(|layout| layout.drag_enabled);
        let focus = window
            .use_keyed_state((self.id.clone(), "bottom-handle"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let driver = Driver {
            state: self.state.downgrade(),
            close: self.close,
            thresholds: self.thresholds,
        };
        let decoration = if self.grip {
            self.decoration.unwrap_or_else(|| {
                div()
                    .w(px(32.))
                    .h(px(4.))
                    .rounded_full()
                    .bg(self.tint)
                    .into_any_element()
            })
        } else {
            crate::Text::new("Change sheet height")
                .id("sheet-height-label")
                .into_any_element()
        };
        let grip = div()
            .id("bottom-sheet-resize")
            .test_support()
            .w_full()
            .h(px(48.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .child(decoration);
        if !enabled {
            return grip.into_any_element();
        }
        let keyboard = driver.clone();
        let accessibility = driver.clone();
        let activate = driver.clone();
        grip.role(Role::Button)
            .aria_label(self.label)
            .track_focus(&focus)
            .tab_stop(true)
            .on_click(move |_, window, cx| {
                activate.navigate(None, window, cx);
            })
            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                accessibility.navigate(None, window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if keyboard.navigate(Some(&event.keystroke.key), window, cx) {
                    cx.stop_propagation();
                }
            })
            .when(draggable, |grip| {
                let pressed = driver.state.clone();
                let start = driver.state.clone();
                let moving = driver.state.clone();
                let outside = driver.clone();
                grip.cursor_ns_resize()
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        let _ =
                            pressed.update(cx, |state, _| state.pressed = Some(event.position.y));
                    })
                    .on_drag(DragPayload, move |_, _, window, cx| {
                        focus.focus(window, cx);
                        let y = window.mouse_position().y;
                        let _ = start.update(cx, |state, cx| {
                            state.begin_drag(y, window, cx);
                            state.drag_to(y, cx);
                        });
                        cx.stop_propagation();
                        cx.new(|_| DragPayload)
                    })
                    .on_drag_move(move |event: &DragMoveEvent<DragPayload>, window, cx| {
                        if let Ok(Some(changed)) =
                            moving.update(cx, |state, cx| state.drag_to(event.event.position.y, cx))
                        {
                            if changed {
                                window.refresh();
                            }
                            cx.stop_propagation();
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        driver.release(event.position.y, window, cx)
                    })
                    .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                        outside.release(event.position.y, window, cx)
                    })
            })
            .into_any_element()
    }
}
