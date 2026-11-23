use std::{cell::RefCell, mem, rc::Rc};

use gpui::InteractiveElement as _;
use gpui::{
    AnyElement, App, Bounds, Context, Element, ElementId, Entity, EntityId, EventEmitter,
    FocusHandle, GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, IntoElement,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels,
    Point, RenderOnce, Styled, Subscription, TouchDragEvent, TouchPhase, WeakEntity, Window, div,
    point, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{DragCapture, animate, current_material_theme};
use gpui_material_theme::{Direction, MotionKind, MotionSpec, MotionSpeed};

const DEFAULT_THRESHOLD: Pixels = px(56.);
const SWIPE_SLOP: Pixels = px(6.);

/// The resting anchor or a logical dismissal direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwipeToDismissValue {
    /// Resting, undismissed position.
    #[default]
    Settled,
    /// Dismissed from leading edge toward trailing edge (left-to-right in LTR, right-to-left in RTL).
    StartToEnd,
    /// Dismissed from trailing edge toward leading edge (right-to-left in LTR, left-to-right in RTL).
    EndToStart,
}

impl SwipeToDismissValue {
    fn position(self) -> f32 {
        match self {
            Self::Settled => 0.,
            Self::StartToEnd => 1.,
            Self::EndToStart => -1.,
        }
    }

    fn from_position(position: f32) -> Self {
        if position > 0. {
            Self::StartToEnd
        } else if position < 0. {
            Self::EndToStart
        } else {
            Self::Settled
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SwipeOptions {
    direction: Direction,
    start_to_end: bool,
    end_to_start: bool,
    gestures_enabled: bool,
}

impl Default for SwipeOptions {
    fn default() -> Self {
        Self {
            direction: Direction::Ltr,
            start_to_end: true,
            end_to_start: true,
            gestures_enabled: true,
        }
    }
}

impl SwipeOptions {
    fn allows(self, value: SwipeToDismissValue) -> bool {
        match value {
            SwipeToDismissValue::Settled => true,
            SwipeToDismissValue::StartToEnd => self.start_to_end,
            SwipeToDismissValue::EndToStart => self.end_to_start,
        }
    }

    fn sign(self) -> f32 {
        match self.direction {
            Direction::Ltr => 1.,
            Direction::Rtl => -1.,
        }
    }

    fn can_start(self, delta: Point<Pixels>) -> bool {
        delta.x.abs() >= SWIPE_SLOP
            && delta.x.abs() > delta.y.abs()
            && self.allows(SwipeToDismissValue::from_position(
                f32::from(delta.x) * self.sign(),
            ))
    }
}

#[derive(Default)]
enum SwipeInput {
    #[default]
    None,
    Pressed(Point<Pixels>),
    Mouse {
        origin: Point<Pixels>,
        _capture: DragCapture,
        _escape: Subscription,
    },
    Touch,
    SuppressMouseUp,
}

struct Dismissed(SwipeToDismissValue);

/// Retained swipe state. Give each mounted swipe container its own entity.
pub struct SwipeToDismissState {
    width: Pixels,
    position: f32,
    target: SwipeToDismissValue,
    settled: SwipeToDismissValue,
    moving: bool,
    rebase_motion: bool,
    options: SwipeOptions,
    threshold: Option<Box<dyn Fn(Pixels) -> Pixels>>,
    input: SwipeInput,
    mount: Option<(EntityId, FocusHandle)>,
}

impl Default for SwipeToDismissState {
    fn default() -> Self {
        Self {
            width: px(0.),
            position: 0.,
            target: SwipeToDismissValue::Settled,
            settled: SwipeToDismissValue::Settled,
            moving: false,
            rebase_motion: false,
            options: SwipeOptions::default(),
            threshold: None,
            input: SwipeInput::None,
            mount: None,
        }
    }
}

impl EventEmitter<Dismissed> for SwipeToDismissState {}

impl SwipeToDismissState {
    /// Creates a new swipe state settled at [`SwipeToDismissValue::Settled`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects the initial anchor before mounting the state.
    pub fn initial_value(mut self, value: SwipeToDismissValue) -> Self {
        self.position = value.position();
        self.target = value;
        self.settled = value;
        self
    }

    /// Computes the release threshold from the current width in logical pixels.
    /// The default is 56 logical pixels, limited by the available travel.
    pub fn positional_threshold(mut self, threshold: impl Fn(Pixels) -> Pixels + 'static) -> Self {
        self.threshold = Some(Box::new(threshold));
        self
    }

    /// Returns the nearest anchor to the current foreground position.
    pub fn current_value(&self) -> SwipeToDismissValue {
        SwipeToDismissValue::from_position(self.position.round())
    }

    /// Returns the anchor where the current interaction would land.
    pub fn target_value(&self) -> SwipeToDismissValue {
        self.target
    }

    /// Returns the last fully reached anchor.
    pub fn settled_value(&self) -> SwipeToDismissValue {
        self.settled
    }

    /// Returns the logical direction of the current offset.
    pub fn dismiss_direction(&self) -> SwipeToDismissValue {
        SwipeToDismissValue::from_position(self.position)
    }

    /// Returns the fraction of the width uncovered by the foreground.
    pub fn progress(&self) -> f32 {
        self.position.abs()
    }

    /// Returns the signed horizontal offset in window coordinates.
    pub fn offset(&self) -> Pixels {
        self.width * (self.position * self.options.sign())
    }

    /// Animates to an enabled dismissal direction. Returns whether it was accepted.
    pub fn dismiss(
        &mut self,
        direction: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if direction == SwipeToDismissValue::Settled || !self.options.allows(direction) {
            return false;
        }
        self.cancel_input(window, true);
        self.move_to(direction, window, cx);
        true
    }

    /// Animates the foreground back to its resting position.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_input(window, true);
        self.move_to(SwipeToDismissValue::Settled, window, cx);
    }

    /// Moves immediately to an enabled anchor. Returns whether it was accepted.
    pub fn snap_to(
        &mut self,
        value: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.options.allows(value) {
            return false;
        }
        let cancelled = self.cancel_input(window, true);
        let settled = self.settle(value, window, cx);
        if cancelled || settled {
            window.refresh();
        }
        true
    }

    pub(crate) fn shows_foreground(&self) -> bool {
        self.moving || self.settled == SwipeToDismissValue::Settled
    }

    pub(crate) fn is_dragging(&self) -> bool {
        matches!(self.input, SwipeInput::Mouse { .. } | SwipeInput::Touch)
    }

    fn move_to(&mut self, value: SwipeToDismissValue, window: &mut Window, cx: &mut Context<Self>) {
        if self.moving && self.target == value {
            return;
        }
        let changed = if cx.reduce_motion() || self.position == value.position() {
            self.settle(value, window, cx)
        } else {
            if !self.moving {
                self.rebase_motion = true;
            }
            self.target = value;
            self.moving = true;
            cx.notify();
            true
        };
        if changed {
            window.refresh();
        }
    }

    fn settle(
        &mut self,
        value: SwipeToDismissValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let changed = self.position != value.position() || self.target != value || self.moving;
        self.position = value.position();
        self.target = value;
        self.moving = false;
        self.rebase_motion = false;
        if self.settled != value {
            self.settled = value;
            if value != SwipeToDismissValue::Settled {
                if self
                    .mount
                    .as_ref()
                    .is_some_and(|(_, focus)| focus.contains_focused(window, cx))
                {
                    window.blur(cx);
                }
                cx.emit(Dismissed(value));
            }
            cx.notify();
            true
        } else if changed {
            cx.notify();
            true
        } else {
            false
        }
    }

    fn sample_motion(&mut self, id: &ElementId, window: &mut Window, cx: &mut Context<Self>) {
        if !self.moving {
            return;
        }
        let motion_id = (id.clone(), "swipe-offset");
        if mem::take(&mut self.rebase_motion) {
            animate(
                motion_id.clone(),
                self.position,
                MotionSpec::Instant,
                window,
                cx,
            );
        }
        let target = self.target.position();
        let spec = current_material_theme(window, cx).motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Fast,
            cx.reduce_motion(),
        );
        let position = animate(motion_id, target, spec, window, cx);
        if position == target {
            self.settle(self.target, window, cx);
        } else {
            let position = position.clamp(-1., 1.);
            if self.position != position {
                self.position = position;
                cx.notify();
            }
        }
    }

    fn configure(
        &mut self,
        options: SwipeOptions,
        mount: (EntityId, FocusHandle),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mount = Some(mount);
        let previous = mem::replace(&mut self.options, options);
        if previous != options {
            if !options.allows(self.target) || !options.allows(self.settled) {
                self.cancel_input(window, true);
                self.settle(SwipeToDismissValue::Settled, window, cx);
            } else if !options.gestures_enabled
                || previous.direction != options.direction
                || !options.allows(self.dismiss_direction())
            {
                self.cancel_interaction(window, cx);
            }
            cx.notify();
        }
    }

    fn can_start(&self) -> bool {
        self.options.gestures_enabled
            && (self.options.start_to_end || self.options.end_to_start)
            && self.width > px(0.)
            && self.settled == SwipeToDismissValue::Settled
            && !self.moving
    }

    fn drag_to(&mut self, delta: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        let position = if self.width > px(0.) {
            (delta / self.width * self.options.sign()).clamp(-1., 1.)
        } else {
            0.
        };
        let direction = SwipeToDismissValue::from_position(position);
        let position = if self.options.allows(direction) {
            position
        } else {
            0.
        };
        let threshold = self
            .threshold
            .as_ref()
            .map_or(DEFAULT_THRESHOLD, |f| f(self.width));
        let threshold = if f32::from(threshold).is_finite() {
            threshold
        } else {
            DEFAULT_THRESHOLD
        };
        let threshold = threshold.clamp(px(0.), self.width);
        let target = if self.width * position.abs() >= threshold {
            SwipeToDismissValue::from_position(position)
        } else {
            SwipeToDismissValue::Settled
        };
        if self.position != position || self.target != target {
            self.position = position;
            self.target = target;
            cx.notify();
            window.refresh();
        }
    }

    fn cancel_input(&mut self, window: &mut Window, suppress_mouse_up: bool) -> bool {
        match mem::take(&mut self.input) {
            SwipeInput::Mouse { .. } => {
                window.release_pointer();
                if suppress_mouse_up {
                    self.input = SwipeInput::SuppressMouseUp;
                }
                true
            }
            SwipeInput::Touch => true,
            SwipeInput::SuppressMouseUp if suppress_mouse_up => {
                self.input = SwipeInput::SuppressMouseUp;
                false
            }
            _ => false,
        }
    }

    fn cancel_interaction(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let cancelled = self.cancel_input(window, true);
        if cancelled {
            self.move_to(SwipeToDismissValue::Settled, window, cx);
        }
        cancelled
    }

    fn release(&mut self, delta: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        self.drag_to(delta, window, cx);
        self.cancel_input(window, false);
        self.move_to(self.target, window, cx);
    }
}

type OnDismiss = Rc<dyn Fn(SwipeToDismissValue, &mut Window, &mut App)>;

struct SwipeMount {
    callback: Rc<RefCell<Option<OnDismiss>>>,
    focus: FocusHandle,
    _dismiss: Subscription,
    _activation: Subscription,
}

/// A foreground and background with logical horizontal swipe gestures.
/// The caller owns the content, action commands, and resulting data changes.
#[derive(IntoElement)]
pub struct SwipeToDismiss {
    id: ElementId,
    state: Entity<SwipeToDismissState>,
    content: AnyElement,
    background: Option<AnyElement>,
    options: SwipeOptions,
    on_dismiss: Option<OnDismiss>,
}

impl SwipeToDismiss {
    /// Creates a swipe-to-dismiss container wrapping `content` and bound to `state`.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<SwipeToDismissState>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            content: content.into_any_element(),
            background: None,
            options: SwipeOptions::default(),
            on_dismiss: None,
        }
    }

    /// Sets the background element revealed behind the foreground while swiping.
    pub fn background(mut self, background: impl IntoElement) -> Self {
        self.background = Some(background.into_any_element());
        self
    }

    /// Enables or disables start-to-end dismissal gestures and programmatic dismissal. Defaults to `true`.
    pub fn enable_start_to_end(mut self, enabled: bool) -> Self {
        self.options.start_to_end = enabled;
        self
    }

    /// Enables or disables end-to-start dismissal gestures and programmatic dismissal. Defaults to `true`.
    pub fn enable_end_to_start(mut self, enabled: bool) -> Self {
        self.options.end_to_start = enabled;
        self
    }

    /// Enables direct gestures. Programmatic and accessible commands stay available.
    pub fn gestures_enabled(mut self, enabled: bool) -> Self {
        self.options.gestures_enabled = enabled;
        self
    }

    /// Runs once when the state settles at a new dismissal anchor.
    pub fn on_dismiss(
        mut self,
        on_dismiss: impl Fn(SwipeToDismissValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(on_dismiss));
        self
    }
}

impl RenderOnce for SwipeToDismiss {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.options.direction = current_material_theme(window, cx).layout.direction;
        let mount = window.with_element_namespace((self.id.clone(), "material-swipe"), |window| {
            window.use_keyed_state(("mount", self.state.entity_id()), cx, |window, cx| {
                let callback = Rc::new(RefCell::new(None::<OnDismiss>));
                let dismiss = window.subscribe(&self.state, cx, {
                    let callback = callback.clone();
                    move |_, event: &Dismissed, window, cx| {
                        let callback = callback.borrow().clone();
                        if let Some(callback) = callback {
                            callback(event.0, window, cx);
                        }
                    }
                });
                let state = self.state.downgrade();
                let activation = cx.observe_window_activation(window, {
                    let state = state.clone();
                    move |_, window, cx| {
                        if !window.is_window_active() {
                            let _ = state.update(cx, |state, cx| {
                                state.cancel_interaction(window, cx);
                            });
                        }
                    }
                });
                let handle = window.window_handle();
                let mount = cx.entity_id();
                cx.on_release(move |_: &mut SwipeMount, cx| {
                    let state = state.clone();
                    cx.defer(move |cx| {
                        let _ = handle.update(cx, |_, window, cx| {
                            let _ = state.update(cx, |state, cx| {
                                if state.mount.as_ref().is_some_and(|(id, _)| *id == mount) {
                                    state.mount = None;
                                    state.cancel_input(window, false);
                                    let settled = state.settled;
                                    state.settle(settled, window, cx);
                                }
                            });
                        });
                    });
                })
                .detach();
                SwipeMount {
                    callback,
                    focus: cx.focus_handle().tab_stop(false),
                    _dismiss: dismiss,
                    _activation: activation,
                }
            })
        });
        let mounted = mount.read(cx);
        mounted.callback.replace(self.on_dismiss);
        let focus = mounted.focus.clone();
        self.state.update(cx, |state, cx| {
            state.configure(self.options, (mount.entity_id(), focus.clone()), window, cx);
            state.sample_motion(&self.id, window, cx);
        });
        div()
            .id(self.id)
            .test_support()
            .overflow_hidden()
            .children(
                self.background
                    .map(|background| div().absolute().inset_0().child(background)),
            )
            .child(SwipeContent {
                content: self.content,
                state: self.state.downgrade(),
                focus,
            })
    }
}

// Keep layout and clipping at the resting position. Only prepaint the
// foreground at its translated position so hit testing and rendering agree.
struct SwipeContent {
    content: AnyElement,
    state: WeakEntity<SwipeToDismissState>,
    focus: FocusHandle,
}

impl IntoElement for SwipeContent {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for SwipeContent {
    type RequestLayoutState = ();
    type PrepaintState = (bool, Option<Hitbox>);

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.content.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (offset, visible) = self
            .state
            .update(cx, |state, cx| {
                if state.width != bounds.size.width {
                    state.width = bounds.size.width;
                    if state.position != 0. || state.moving {
                        cx.notify();
                    }
                }
                (state.offset(), state.shows_foreground())
            })
            .unwrap_or((px(0.), true));
        if visible {
            window.set_focus_handle(&self.focus, cx);
            window.with_element_offset(point(offset, px(0.)), |window| {
                self.content.prepaint(window, cx);
            });
        }
        let hitbox = self.state.upgrade().and_then(|state| {
            let state = state.read(cx);
            (state.can_start() || !matches!(state.input, SwipeInput::None))
                .then(|| window.insert_hitbox(bounds, HitboxBehavior::Normal))
        });
        (visible, hitbox)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        (visible, hitbox): &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if *visible {
            self.content.paint(window, cx);
        }
        if let Some(hitbox) = hitbox {
            listen_for_swipes(self.state.clone(), hitbox.clone(), window);
        }
    }
}

fn listen_for_swipes(state: WeakEntity<SwipeToDismissState>, hitbox: Hitbox, window: &mut Window) {
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if phase.bubble() && event.button == MouseButton::Left && hitbox.is_hovered(window) {
                let _ = state.update(cx, |state, _| {
                    if state.can_start()
                        && matches!(state.input, SwipeInput::None | SwipeInput::SuppressMouseUp)
                    {
                        state.input = SwipeInput::Pressed(event.position);
                    }
                });
            }
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            if !phase.bubble() {
                return;
            }
            let _ = state.update(cx, |swipe, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    if !matches!(swipe.input, SwipeInput::Touch)
                        && swipe.cancel_input(window, false)
                    {
                        swipe.move_to(SwipeToDismissValue::Settled, window, cx);
                    }
                    return;
                }
                match swipe.input {
                    SwipeInput::Pressed(origin) => {
                        let delta = event.position - origin;
                        if delta.x.abs().max(delta.y.abs()) >= SWIPE_SLOP {
                            if swipe.options.can_start(delta)
                                && !cx.has_active_drag()
                                && window.captured_hitbox().is_none()
                            {
                                window.capture_pointer(hitbox.id);
                                swipe.input = watch_mouse_swipe(state.clone(), origin, window, cx);
                                swipe.drag_to(delta.x, window, cx);
                                cx.stop_propagation();
                            } else {
                                swipe.input = SwipeInput::None;
                            }
                        }
                    }
                    SwipeInput::Mouse { origin, .. } => {
                        swipe.drag_to(event.position.x - origin.x, window, cx);
                        cx.stop_propagation();
                    }
                    SwipeInput::SuppressMouseUp => cx.stop_propagation(),
                    _ => {}
                }
            });
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        move |event: &MouseUpEvent, phase, window, cx| {
            if !phase.bubble() || event.button != MouseButton::Left {
                return;
            }
            let _ = state.update(cx, |state, cx| match state.input {
                SwipeInput::Mouse { origin, .. } => {
                    state.release(event.position.x - origin.x, window, cx);
                    cx.stop_propagation();
                }
                SwipeInput::SuppressMouseUp => {
                    state.input = SwipeInput::None;
                    cx.stop_propagation();
                }
                SwipeInput::Pressed(_) => state.input = SwipeInput::None,
                _ => {}
            });
        }
    });
    window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
        if !phase.bubble() {
            return;
        }
        let _ = state.update(cx, |state, cx| {
            let delta = event.position - event.start_position;
            match event.phase {
                TouchPhase::Started
                    if hitbox.is_hovered_at(event.start_position, window)
                        && state.can_start()
                        && matches!(state.input, SwipeInput::None)
                        && state.options.can_start(delta) =>
                {
                    state.input = SwipeInput::Touch;
                    state.drag_to(delta.x, window, cx);
                    window.prevent_default();
                    cx.stop_propagation();
                }
                TouchPhase::Moved | TouchPhase::Ended
                    if matches!(state.input, SwipeInput::Touch) =>
                {
                    if event.phase == TouchPhase::Ended {
                        state.release(delta.x, window, cx);
                    } else {
                        state.drag_to(delta.x, window, cx);
                    }
                    cx.stop_propagation();
                }
                TouchPhase::Cancelled if matches!(state.input, SwipeInput::Touch) => {
                    state.cancel_interaction(window, cx);
                    cx.stop_propagation();
                }
                _ => {}
            }
        });
    });
}

fn watch_mouse_swipe(
    state: WeakEntity<SwipeToDismissState>,
    origin: Point<Pixels>,
    window: &Window,
    cx: &mut App,
) -> SwipeInput {
    let window_id = window.window_handle().window_id();
    let escape = cx.intercept_keystrokes({
        let state = state.clone();
        move |event, window, cx| {
            if event.keystroke.key == "escape" && window.window_handle().window_id() == window_id {
                let cancelled = state.update(cx, |state, cx| state.cancel_interaction(window, cx));
                if cancelled.unwrap_or(false) {
                    cx.stop_propagation();
                }
            }
        }
    });
    let capture = DragCapture::watch_pointer(window, cx, move |window, cx| {
        let _ = state.update(cx, |state, cx| {
            state.cancel_interaction(window, cx);
        });
    });
    SwipeInput::Mouse {
        origin,
        _capture: capture,
        _escape: escape,
    }
}
