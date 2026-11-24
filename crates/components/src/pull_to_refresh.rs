use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, Bounds, Context, ElementId, Entity, GlobalElementId, Hitbox, HitboxBehavior,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, ParentElement, Pixels,
    RenderOnce, ScrollEvent, ScrollHandle, ScrollSource, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, TouchPhase, WeakEntity, Window, WindowId, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{StyledExt, TestSupportExt, motion::Easing};
use gpui_material_foundation::{RetargetTransition, current_material_theme};
use gpui_material_theme::Direction;

use crate::{Button, ButtonVariant, LoadingIndicator};

const DEFAULT_DISTANCE: Pixels = px(80.);
const DRAG_MULTIPLIER: f32 = 0.5;
const SETTLE: Duration = Duration::from_millis(180);

type Refresh = Rc<dyn Fn(&mut Window, &mut App)>;
type ScrollPosition = Rc<dyn Fn(&Window, &App) -> Pixels>;
type Indicator = Box<dyn FnOnce(PullToRefreshProgress, &mut Window, &mut App) -> AnyElement>;

/// Readback supplied to a caller's indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PullToRefreshProgress {
    /// Pull progress relative to the refresh threshold (`0.0` hidden, `1.0` threshold, >`1.0` overshoot).
    pub distance_fraction: f32,
    /// Current downward translation of the indicator in logical pixels.
    pub offset: Pixels,
    /// Whether the caller-owned refreshing state is currently active.
    pub refreshing: bool,
    /// Whether a refresh request has been emitted and is awaiting caller confirmation or rejection.
    pub refresh_pending: bool,
    /// Whether the indicator is currently animating toward its resting or threshold position.
    pub animating: bool,
}

#[derive(Clone, Eq, PartialEq)]
struct ViewportKey {
    window: WindowId,
    id: ElementId,
}

struct PullGesture {
    source: ScrollSource,
    distance: Pixels,
    before_scroll: Pixels,
}

/// Retained gesture and indicator state. The application owns refreshing.
pub struct PullToRefreshState {
    scroll: ScrollHandle,
    viewport: Option<ViewportKey>,
    gesture: Option<PullGesture>,
    fraction: f32,
    motion: RetargetTransition,
    refreshing: bool,
    pending: bool,
    enabled: bool,
}

impl Default for PullToRefreshState {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            viewport: None,
            gesture: None,
            fraction: 0.,
            motion: RetargetTransition::settled(0.),
            refreshing: false,
            pending: false,
            enabled: true,
        }
    }
}

impl PullToRefreshState {
    /// Creates an idle pull-to-refresh state with its own internal scroll handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// The scroll handle used when the container owns its viewport.
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.scroll.clone()
    }

    /// Zero is hidden and one is the refresh threshold. Pull overshoot can exceed one.
    pub fn distance_fraction(&self) -> f32 {
        self.fraction
    }

    /// Returns `true` while the indicator is transitioning between positions.
    pub fn is_animating(&self) -> bool {
        self.motion.is_animating()
    }

    /// Returns `true` when a refresh callback has been triggered and awaits a state update or [`Self::reject_refresh`].
    pub fn is_refresh_pending(&self) -> bool {
        self.pending
    }

    /// Sets the displayed distance immediately and cancels an active gesture.
    pub fn snap_to(&mut self, fraction: f32, cx: &mut Context<Self>) {
        let target = if fraction.is_finite() {
            fraction.max(0.)
        } else {
            0.
        };
        let had_gesture = self.gesture.take().is_some();
        let was_animating = self.motion.is_animating();
        let changed = had_gesture || self.fraction != target || was_animating;
        self.fraction = target;
        self.motion.snap_to(self.fraction);
        if changed {
            cx.notify();
        }
    }

    /// Cancels any active gesture and animates the indicator to the refresh threshold (`1.0`).
    pub fn animate_to_threshold(&mut self, cx: &mut Context<Self>) {
        self.animate_to_fraction(1., cx);
    }

    /// Cancels any active gesture and animates the indicator back to hidden (`0.0`).
    pub fn animate_to_hidden(&mut self, cx: &mut Context<Self>) {
        self.animate_to_fraction(0., cx);
    }

    fn animate_to_fraction(&mut self, target: f32, cx: &mut Context<Self>) {
        let had_gesture = self.gesture.take().is_some();
        let was_animating = self.motion.is_animating();
        let before = self.fraction;
        self.settle_to(target, cx);
        if had_gesture || was_animating || before != self.fraction || self.motion.is_animating() {
            cx.notify();
        }
    }

    /// Clears the current pull and any unanswered request, keeping the controlled busy value.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.pending = false;
        self.snap_to(if self.refreshing { 1. } else { 0. }, cx);
    }

    /// Rearms refresh after the owner declines a pending request.
    pub fn reject_refresh(&mut self, cx: &mut Context<Self>) {
        if self.pending {
            self.pending = false;
            self.animate_to_hidden(cx);
        }
    }

    fn can_request(&self) -> bool {
        self.enabled && !self.refreshing && !self.pending
    }

    fn configure(
        &mut self,
        viewport: &ViewportKey,
        refreshing: bool,
        enabled: bool,
        window: &mut Window,
        cx: &App,
    ) {
        let retargeted = self.viewport.as_ref() != Some(viewport);
        if retargeted && self.viewport.replace(viewport.clone()).is_some() {
            self.gesture = None;
            self.fraction = f32::from(refreshing || self.pending);
            self.motion.snap_to(self.fraction);
        }
        if refreshing != self.refreshing {
            self.refreshing = refreshing;
            self.pending = false;
            self.gesture = None;
            self.settle_to(f32::from(refreshing), cx);
        } else if self.enabled && !enabled && self.gesture.take().is_some() {
            self.settle_to(0., cx);
        }
        self.enabled = enabled;
        self.sample_motion(window, cx);
    }

    fn settle_to(&mut self, target: f32, cx: &App) {
        let now = cx.background_executor().now();
        self.motion
            .animate_to(target, SETTLE, Easing::EaseOut, now, cx.reduce_motion());
        self.fraction = self.motion.value();
    }

    fn sample_motion(&mut self, window: &mut Window, cx: &App) {
        if self.motion.is_animating() {
            let now = cx.background_executor().now();
            let (value, running) = self.motion.sample(now, cx.reduce_motion());
            self.fraction = value;
            if running && window.is_visible() {
                window.request_animation_frame();
            }
        }
    }

    fn pull_by(&mut self, amount: Pixels, threshold: Pixels, cx: &mut Context<Self>) {
        if let Some(gesture) = &mut self.gesture {
            let next_distance = (gesture.distance + amount).max(px(0.));
            let progress = (next_distance / threshold) * DRAG_MULTIPLIER;
            let overshoot = (progress - 1.).clamp(0., 2.);
            let next_fraction = progress.min(1.) + overshoot - overshoot.powi(2) * 0.25;
            if next_distance != gesture.distance
                || next_fraction != self.fraction
                || self.motion.is_animating()
            {
                gesture.distance = next_distance;
                self.fraction = next_fraction;
                self.motion.snap_to(self.fraction);
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
enum ScrollBinding {
    Handle(ScrollHandle),
    Position(ScrollPosition),
}

impl ScrollBinding {
    fn distance_from_top(&self, window: &Window, cx: &App) -> Pixels {
        let distance = match self {
            Self::Handle(handle) => -handle.offset().y,
            Self::Position(position) => position(window, cx),
        };
        if f32::from(distance).is_finite() {
            distance.max(px(0.))
        } else {
            px(f32::MAX)
        }
    }

    fn remove_overscroll(&self) {
        if let Self::Handle(handle) = self {
            let offset = handle.offset();
            if offset.y > px(0.) {
                handle.set_offset(point(offset.x, px(0.)));
            }
        }
    }
}

/// A vertical viewport with an Expressive refresh indicator and accessible action.
#[derive(IntoElement)]
pub struct PullToRefresh {
    id: ElementId,
    state: Entity<PullToRefreshState>,
    refreshing: bool,
    enabled: bool,
    refresh: Refresh,
    label: SharedString,
    threshold: Pixels,
    indicator_distance: Pixels,
    binding: Option<ScrollBinding>,
    indicator: Option<Indicator>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl PullToRefresh {
    /// Creates a pull-to-refresh container bound to `state`, caller-owned `refreshing` status, and refresh callback.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<PullToRefreshState>,
        refreshing: bool,
        on_refresh: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            refreshing,
            enabled: true,
            refresh: Rc::new(on_refresh),
            label: "Refresh".into(),
            threshold: DEFAULT_DISTANCE,
            indicator_distance: DEFAULT_DISTANCE,
            binding: None,
            indicator: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Enables or disables pull gestures and the accessible refresh action button. Defaults to `true`.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the label used for both the accessible refresh button and default loading indicator. Defaults to `"Refresh"`.
    pub fn refresh_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the pull threshold in logical pixels. Invalid values use the default.
    pub fn threshold(mut self, threshold: Pixels) -> Self {
        if f32::from(threshold).is_finite() && threshold > px(0.) {
            self.threshold = threshold;
        }
        self
    }

    /// Sets indicator travel at the threshold in logical pixels.
    pub fn indicator_distance(mut self, distance: Pixels) -> Self {
        if f32::from(distance).is_finite() && distance >= px(0.) {
            self.indicator_distance = distance;
        }
        self
    }

    /// Wraps a caller-owned viewport that tracks this scroll handle.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.binding = Some(ScrollBinding::Handle(handle.clone()));
        self
    }

    /// Wraps a caller-owned viewport using a positive distance from its top edge.
    pub fn scroll_position(mut self, position: impl Fn(&Window, &App) -> Pixels + 'static) -> Self {
        self.binding = Some(ScrollBinding::Position(Rc::new(position)));
        self
    }

    /// Replaces the loading indicator with decorative content using the same reveal.
    pub fn indicator<E: IntoElement>(
        mut self,
        indicator: impl FnOnce(PullToRefreshProgress, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.indicator = Some(Box::new(move |progress, window, cx| {
            indicator(progress, window, cx).into_any_element()
        }));
        self
    }
}

impl Styled for PullToRefresh {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for PullToRefresh {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for PullToRefresh {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = ViewportKey {
            window: window.window_handle().window_id(),
            id: self.id.clone(),
        };
        let (progress, can_request, scroll) = self.state.update(cx, |state, cx| {
            state.configure(&viewport, self.refreshing, self.enabled, window, cx);
            (
                PullToRefreshProgress {
                    distance_fraction: state.fraction,
                    offset: (self.indicator_distance * state.fraction.min(2.))
                        .min(window.viewport_size().height),
                    refreshing: state.refreshing,
                    refresh_pending: state.pending,
                    animating: state.is_animating(),
                },
                state.can_request(),
                state.scroll_handle(),
            )
        });
        let rtl = current_material_theme(window, cx).layout.direction == Direction::Rtl;
        let owned = self.binding.is_none();
        let mut body = div()
            .id("refresh-scroll")
            .test_support()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .children(self.children);
        if owned {
            body = body.overflow_y_scroll().track_scroll(&scroll);
        }
        let driver = RefreshDriver {
            state: self.state.downgrade(),
            viewport,
            refresh: self.refresh,
            threshold: self.threshold,
            binding: self.binding.unwrap_or(ScrollBinding::Handle(scroll)),
        };
        let command = driver.clone();
        let indicator = (progress.distance_fraction > 0.).then(|| {
            let element = match self.indicator {
                Some(indicator) => indicator(progress, window, cx),
                None => {
                    let loading = LoadingIndicator::new("refresh-progress", self.label.clone())
                        .contained(true);
                    if progress.refreshing {
                        loading.into_any_element()
                    } else {
                        loading.value(progress.distance_fraction).into_any_element()
                    }
                }
            };
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(progress.offset - px(48.))
                .flex()
                .justify_center()
                .child(element)
        });
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .refine_style(&self.style)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .when(rtl, Styled::justify_start)
                    .when(!rtl, Styled::justify_end)
                    .child(
                        Button::new("refresh-action", self.label)
                            .variant(ButtonVariant::Text)
                            .disabled(!can_request)
                            .on_activate(move |window, cx| command.request(window, cx)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .overflow_hidden()
                    .child(RefreshViewport {
                        driver,
                        body: body.into_any_element(),
                    })
                    .children(indicator),
            )
    }
}

#[derive(Clone)]
struct RefreshDriver {
    state: WeakEntity<PullToRefreshState>,
    viewport: ViewportKey,
    refresh: Refresh,
    threshold: Pixels,
    binding: ScrollBinding,
}

impl RefreshDriver {
    fn request(&self, window: &mut Window, cx: &mut App) {
        let accepted = self.state.update(cx, |state, cx| {
            if state.viewport.as_ref() != Some(&self.viewport) || !state.can_request() {
                return false;
            }
            state.pending = true;
            state.gesture = None;
            state.settle_to(1., cx);
            cx.notify();
            true
        });
        if accepted.unwrap_or(false) {
            window.refresh();
            (self.refresh)(window, cx);
        }
    }

    fn listen(self, hitbox: Hitbox, window: &mut Window) {
        window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
            if !event.delta.precise() {
                return;
            }
            let hits = hitbox.should_handle_scroll(window);
            let delta = event.delta.pixel_delta(px(1.));
            let vertical = delta.y != px(0.)
                && delta.y.abs() >= delta.x.abs()
                && f32::from(delta.y).is_finite();
            let before = self.binding.distance_from_top(window, cx);
            let (consumed, request) = self
                .state
                .update(cx, |state, cx| {
                    if state.viewport.as_ref() != Some(&self.viewport) || !state.can_request() {
                        return (false, false);
                    }
                    if phase.capture() {
                        if event.touch_phase == TouchPhase::Started && hits {
                            if state
                                .gesture
                                .as_ref()
                                .is_some_and(|gesture| gesture.distance > px(0.))
                            {
                                state.settle_to(0., cx);
                                cx.notify();
                            }
                            state.gesture = Some(PullGesture {
                                source: event.source,
                                distance: px(0.),
                                before_scroll: before,
                            });
                        }
                        let Some(gesture) = state
                            .gesture
                            .as_mut()
                            .filter(|gesture| gesture.source == event.source)
                        else {
                            return (false, false);
                        };
                        let pulling = gesture.distance > px(0.);
                        if matches!(event.touch_phase, TouchPhase::Ended | TouchPhase::Cancelled)
                            || !hits
                        {
                            let request = hits
                                && event.touch_phase == TouchPhase::Ended
                                && pulling
                                && state.fraction > 1.;
                            state.gesture = None;
                            if pulling && !request {
                                state.settle_to(0., cx);
                                cx.notify();
                            }
                            return (pulling && hits, request);
                        }
                        gesture.before_scroll = before;
                        if pulling && vertical {
                            state.pull_by(delta.y, self.threshold, cx);
                            return (true, false);
                        }
                    } else if hits
                        && vertical
                        && delta.y > px(0.)
                        && let Some(gesture) = state
                            .gesture
                            .as_ref()
                            .filter(|gesture| gesture.source == event.source)
                    {
                        // The scroll owner has handled as much of the delta as fits above its top.
                        let remainder = delta.y - gesture.before_scroll;
                        if remainder > px(0.) && before <= px(0.) {
                            self.binding.remove_overscroll();
                            state.pull_by(remainder, self.threshold, cx);
                            return (true, false);
                        }
                    }
                    (false, false)
                })
                .unwrap_or_default();
            if consumed {
                window.refresh();
                cx.stop_propagation();
            }
            if request {
                self.request(window, cx);
            }
        });
    }
}

/// Registers before the scroll child in paint order so the child receives the bubble first.
struct RefreshViewport {
    driver: RefreshDriver,
    body: AnyElement,
}

impl gpui::Element for RefreshViewport {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.body.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.body.prepaint(window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.driver.clone().listen(hitbox.clone(), window);
        self.body.paint(window, cx);
    }
}
