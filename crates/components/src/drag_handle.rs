use std::{ops::RangeInclusive, rc::Rc};

use gpui::{
    AccessibleAction, App, Bounds, Context, Corners, ElementId, Entity, FocusHandle, Hitbox,
    HitboxBehavior, HitboxId, Hsla, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseExitEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point, RenderOnce, Role,
    SharedString, Size, StatefulInteractiveElement, Styled, Subscription, TouchDragEvent,
    TouchPhase, Window,
    accesskit::{ActionData, Orientation},
    canvas, div, fill, point, px, size,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    DragCapture, InteractionFeedback, InteractionSource, InteractionState, InteractionTracker,
    Shape, color, window_material_theme,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction};

const REST_SIZE: Size<Pixels> = size(px(4.), px(48.));
const ACTIVE_SIZE: Size<Pixels> = size(px(12.), px(52.));
const DEFAULT_STEP: Pixels = px(8.);
const DRAG_SLOP: Pixels = px(4.);

/// Visual sizes in logical pixels. Omitted values use Material defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleSizes {
    pub resting: Option<Size<Pixels>>,
    pub pressed: Option<Size<Pixels>>,
    pub dragged: Option<Size<Pixels>>,
}

/// Fill colors. Omitted values use the corresponding window theme role.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleColors {
    pub resting: Option<Hsla>,
    pub pressed: Option<Hsla>,
    pub dragged: Option<Hsla>,
}

/// Corner shapes in logical corner order. Omitted values use Material defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleShapes {
    pub resting: Option<CornerShape>,
    pub pressed: Option<CornerShape>,
    pub dragged: Option<CornerShape>,
}

type ResizeRequest = Rc<dyn Fn(Pixels, &mut Window, &mut App)>;

struct ResizeBinding {
    label: SharedString,
    width: Pixels,
    min: Pixels,
    max: Pixels,
    request: ResizeRequest,
}

impl ResizeBinding {
    fn request(&self, width: Pixels, window: &mut Window, cx: &mut App) {
        let width = width.clamp(self.min, self.max);
        if width != self.width {
            (self.request)(width, window, cx);
        }
    }
}

/// A vertical Material handle for a horizontally adjustable boundary.
/// The owner provides the layout and decides which requested widths to apply.
#[derive(IntoElement)]
pub struct VerticalDragHandle {
    id: ElementId,
    sizes: DragHandleSizes,
    colors: DragHandleColors,
    shapes: DragHandleShapes,
    source: Option<Entity<InteractionSource>>,
    resize: Option<Rc<ResizeBinding>>,
    on_resize_end: Option<ResizeRequest>,
    resize_context: Option<Size<Pixels>>,
    keyboard_step: Pixels,
}

impl VerticalDragHandle {
    /// Creates a handle with the default appearance and no resize binding.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sizes: DragHandleSizes::default(),
            colors: DragHandleColors::default(),
            shapes: DragHandleShapes::default(),
            source: None,
            resize: None,
            on_resize_end: None,
            resize_context: None,
            keyboard_step: DEFAULT_STEP,
        }
    }

    pub fn sizes(mut self, sizes: DragHandleSizes) -> Self {
        self.sizes = sizes;
        self
    }

    pub fn colors(mut self, colors: DragHandleColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn shapes(mut self, shapes: DragHandleShapes) -> Self {
        self.shapes = shapes;
        self
    }

    /// Observes hover, press, drag, and keyboard focus under the handle's ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
    }

    /// Configures a focusable splitter with an accessible name, start pane
    /// width, and bounds. The callback receives a requested absolute width.
    /// Invalid or empty width bounds leave a visual handle.
    pub fn on_resize(
        mut self,
        label: impl Into<SharedString>,
        width: Pixels,
        range: RangeInclusive<Pixels>,
        on_resize: impl Fn(Pixels, &mut Window, &mut App) + 'static,
    ) -> Self {
        let (min, max) = range.into_inner();
        self.resize = (min >= px(0.)
            && min < max
            && f32::from(max).is_finite()
            && f32::from(width).is_finite())
        .then(|| {
            Rc::new(ResizeBinding {
                label: label.into(),
                width: width.clamp(min, max),
                min,
                max,
                request: Rc::new(on_resize),
            })
        });
        self
    }

    /// Final pointer value after a completed drag. Cancellation skips this callback.
    pub(crate) fn on_resize_end(
        mut self,
        callback: impl Fn(Pixels, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize_end = Some(Rc::new(callback));
        self
    }

    /// A consumer can invalidate the pointer origin when its viewport changes.
    pub(crate) fn resize_context(mut self, viewport: Size<Pixels>) -> Self {
        self.resize_context = Some(viewport);
        self
    }

    /// Sets the keyboard and accessibility increment. Defaults to 8 logical pixels.
    pub fn keyboard_step(mut self, step: Pixels) -> Self {
        self.keyboard_step = positive_or(step, DEFAULT_STEP);
        self
    }
}

fn positive_or(value: Pixels, fallback: Pixels) -> Pixels {
    if value > px(0.) && f32::from(value).is_finite() {
        value
    } else {
        fallback
    }
}

fn handle_size(value: Option<Size<Pixels>>, fallback: Size<Pixels>) -> Size<Pixels> {
    value.map_or(fallback, |value| {
        size(
            positive_or(value.width, fallback.width),
            positive_or(value.height, fallback.height),
        )
    })
}

struct PointerPress {
    mouse_capture: Option<(HitboxId, DragCapture)>,
    origin: Point<Pixels>,
    width_at_press: Option<Pixels>,
    last_request: Option<Pixels>,
    dragged: bool,
}

impl PointerPress {
    fn new(
        origin: Point<Pixels>,
        width: Option<Pixels>,
        mouse_capture: Option<(HitboxId, DragCapture)>,
    ) -> Self {
        Self {
            mouse_capture,
            origin,
            width_at_press: width,
            last_request: width,
            dragged: false,
        }
    }
}

struct HandleState {
    focus: FocusHandle,
    press: Option<PointerPress>,
    hovered: bool,
    direction: Direction,
    adjustable: bool,
    resize_context: Option<Size<Pixels>>,
    tracker: Option<InteractionTracker>,
    _blur: Subscription,
    _activation: Subscription,
}

impl HandleState {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle().tab_stop(true);
        let blur = cx.on_blur(&focus, window, |state, window, cx| {
            state.end(window);
            state.publish(window, cx);
        });
        let activation = cx.observe_window_activation(window, |state, window, cx| {
            if !window.is_window_active() {
                state.end(window);
                state.hovered = false;
                state.publish(window, cx);
            }
        });
        let handle = window.window_handle();
        cx.on_release(move |state: &mut Self, cx| {
            if let Some((hitbox, _)) = state.press.take().and_then(|press| press.mouse_capture) {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, _| release_pointer(hitbox, window));
                });
            }
        })
        .detach();
        Self {
            focus,
            press: None,
            hovered: false,
            direction: Direction::Ltr,
            adjustable: false,
            resize_context: None,
            tracker: None,
            _blur: blur,
            _activation: activation,
        }
    }

    fn snapshot(&self, window: &Window) -> InteractionState {
        let mut state = InteractionState::default();
        state.hovered = self.hovered;
        state.pressed = self.press.is_some();
        state.dragged = self.press.as_ref().is_some_and(|press| press.dragged);
        state.focus_visible =
            self.adjustable && self.focus.is_focused(window) && window.last_input_was_keyboard();
        state
    }

    fn record(&self, window: &Window, cx: &mut App) {
        if let Some(tracker) = &self.tracker {
            tracker.record(self.snapshot(window), cx);
        }
    }

    fn publish(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.record(window, cx);
        cx.notify();
        window.refresh();
    }

    fn end(&mut self, window: &mut Window) {
        if let Some((hitbox, _)) = self.press.take().and_then(|press| press.mouse_capture) {
            release_pointer(hitbox, window);
        }
    }

    fn final_pointer_width(&self) -> Option<Pixels> {
        self.press
            .as_ref()
            .filter(|press| press.dragged)?
            .last_request
    }

    fn mouse_pressed(&self) -> bool {
        self.press
            .as_ref()
            .is_some_and(|press| press.mouse_capture.is_some())
    }

    fn move_pointer(
        &mut self,
        position: Point<Pixels>,
        resize: Option<&ResizeBinding>,
    ) -> Option<Pixels> {
        let press = self.press.as_mut()?;
        let delta = position - press.origin;
        if !press.dragged {
            press.dragged = starts_horizontal_drag(delta);
        }
        if !press.dragged {
            return None;
        }
        let resize = resize?;
        let origin = press.width_at_press?;
        // Use the physical press origin even when the owner moves the handle
        // or rejects a request while it is being dragged.
        let value = (origin + logical_delta(delta.x, self.direction)).clamp(resize.min, resize.max);
        (press.last_request.replace(value) != Some(value)).then_some(value)
    }
}

fn starts_horizontal_drag(delta: Point<Pixels>) -> bool {
    delta.x.abs() >= DRAG_SLOP && delta.x.abs() > delta.y.abs()
}

fn release_pointer(id: HitboxId, window: &mut Window) {
    if window.captured_hitbox() == Some(id) {
        window.release_pointer();
    }
}

fn logical_delta(delta: Pixels, direction: Direction) -> Pixels {
    if direction == Direction::Rtl {
        -delta
    } else {
        delta
    }
}

impl RenderOnce for VerticalDragHandle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let minimum = px(theme.layout.density.interactive_size(0.));
        let outline = color(theme.colors.color(ColorRole::Outline));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let ring = color(theme.colors.color(ColorRole::Secondary));

        let state =
            window.use_keyed_state((self.id.clone(), "drag-handle-state"), cx, HandleState::new);
        let tracker = self
            .source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let (focus, interaction) = state.update(cx, |state, cx| {
            if (state.direction != direction
                || state.adjustable != self.resize.is_some()
                || state.resize_context != self.resize_context)
                && state.press.is_some()
            {
                state.end(window);
            }
            state.adjustable = self.resize.is_some();
            state.direction = direction;
            state.resize_context = self.resize_context;
            state.tracker = tracker;
            state.record(window, cx);
            (state.focus.clone(), state.snapshot(window))
        });

        let resting = handle_size(self.sizes.resting, REST_SIZE);
        let pressed = handle_size(self.sizes.pressed, ACTIVE_SIZE);
        let dragged = handle_size(self.sizes.dragged, ACTIVE_SIZE);
        let width = resting
            .width
            .max(pressed.width)
            .max(dragged.width)
            .max(minimum);
        let height = resting
            .height
            .max(pressed.height)
            .max(dragged.height)
            .max(minimum);
        let (mark, tint, shape) = if interaction.dragged {
            (
                dragged,
                self.colors.dragged.unwrap_or(on_surface),
                self.shapes.dragged.unwrap_or(CornerRole::Medium.shape()),
            )
        } else if interaction.pressed {
            (
                pressed,
                self.colors.pressed.unwrap_or(on_surface),
                self.shapes.pressed.unwrap_or(CornerRole::Medium.shape()),
            )
        } else {
            (
                resting,
                self.colors.resting.unwrap_or(outline),
                self.shapes.resting.unwrap_or(CornerRole::Full.shape()),
            )
        };
        let listeners = state.clone();
        let pointer_resize = self.resize.clone();
        let pointer_end = self.on_resize_end;
        let target = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |bounds, hitbox, window, cx| {
                let mark_bounds = Bounds::new(
                    bounds.center() - point(mark.width / 2., mark.height / 2.),
                    mark,
                );
                let [top_left, top_right, bottom_right, bottom_left] = shape
                    .radii(f32::from(mark.width), f32::from(mark.height), direction)
                    .map(px);
                window.paint_quad(fill(mark_bounds, tint).corner_radii(Corners {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                }));
                if (interaction.hovered || interaction.focus_visible)
                    && let Some(path) = Shape::rounded(mark_bounds, shape, direction)
                {
                    let mut feedback = InteractionState::default();
                    feedback.hovered = interaction.hovered;
                    feedback.focus_visible = interaction.focus_visible;
                    InteractionFeedback::new(feedback, on_surface, ring).paint(
                        &path,
                        None,
                        cx.background_executor().now(),
                        cx.reduce_motion(),
                        window,
                    );
                }
                pointer_listeners(listeners, hitbox, pointer_resize, pointer_end, window);
            },
        )
        .size_full();
        let cancel = state.clone();
        let mut root = div()
            .id(self.id)
            .test_support()
            .flex_none()
            .w(width)
            .h(height)
            .cursor_col_resize()
            .child(target)
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" && cancel.read(cx).press.is_some() {
                    cancel.update(cx, |state, cx| {
                        state.end(window);
                        state.publish(window, cx);
                    });
                    cx.stop_propagation();
                }
            });
        if let Some(resize) = self.resize {
            let step = self.keyboard_step;
            let increase = resize.clone();
            let decrease = resize.clone();
            let set_value = resize.clone();
            root = root
                .track_focus(&focus)
                // AccessKit exposes writable range values through Slider.
                .role(Role::Slider)
                .a11y_synthetic_children(|tree| tree.parent_node().set_role_description("splitter"))
                .aria_label(resize.label.clone())
                .aria_orientation(Orientation::Vertical)
                .aria_numeric_value(f64::from(f32::from(resize.width)))
                .aria_min_numeric_value(f64::from(f32::from(resize.min)))
                .aria_max_numeric_value(f64::from(f32::from(resize.max)))
                .aria_numeric_value_step(f64::from(f32::from(step)))
                .on_a11y_action(AccessibleAction::Increment, move |_, window, cx| {
                    increase.request(increase.width + step, window, cx);
                })
                .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
                    decrease.request(decrease.width - step, window, cx);
                })
                .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(ActionData::NumericValue(value)) = data
                        && value.is_finite()
                    {
                        set_value.request(px(*value as f32), window, cx);
                    }
                })
                .on_key_down(move |event, window, cx| {
                    let width = match event.keystroke.key.as_str() {
                        "left" => resize.width + logical_delta(-step, direction),
                        "right" => resize.width + logical_delta(step, direction),
                        "home" => resize.min,
                        "end" => resize.max,
                        _ => return,
                    };
                    resize.request(width, window, cx);
                    cx.stop_propagation();
                });
        }
        root
    }
}

fn pointer_listeners(
    state: Entity<HandleState>,
    hitbox: Hitbox,
    resize: Option<Rc<ResizeBinding>>,
    on_resize_end: Option<ResizeRequest>,
    window: &mut Window,
) {
    window.on_mouse_event({
        let state = state.clone();
        let focus = resize.is_some();
        let initial_width = resize.as_ref().map(|resize| resize.width);
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if !phase.bubble()
                || event.button != MouseButton::Left
                || !hitbox.is_hovered(window)
                || window.captured_hitbox().is_some()
                || state.read(cx).press.is_some()
            {
                return;
            }
            state.update(cx, |state, cx| {
                if focus {
                    state.focus.focus(window, cx);
                }
                let owner = cx.weak_entity();
                window.capture_pointer(hitbox.id);
                let watch = DragCapture::watch_pointer(window, cx, move |window, cx| {
                    let _ = owner.update(cx, |state, cx| {
                        state.end(window);
                        state.hovered = false;
                        state.publish(window, cx);
                    });
                });
                state.press = Some(PointerPress::new(
                    event.position,
                    initial_width,
                    Some((hitbox.id, watch)),
                ));
                state.hovered = true;
                state.publish(window, cx);
            });
            cx.stop_propagation();
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let resize = resize.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            let current = state.read(cx);
            let press = current.mouse_pressed();
            // An active gesture still receives moves over other controls.
            if phase.capture() != press {
                return;
            }
            let hovered = hitbox.is_hovered_at(event.position, window);
            if !press && hovered == current.hovered {
                return;
            }
            let request = state.update(cx, |state, cx| {
                state.hovered = hovered;
                let request = if press && event.pressed_button == Some(MouseButton::Left) {
                    state.move_pointer(event.position, resize.as_deref())
                } else {
                    if press {
                        state.end(window);
                    }
                    None
                };
                state.publish(window, cx);
                request
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if press && event.pressed_button == Some(MouseButton::Left) {
                cx.stop_propagation();
            }
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        let hitbox = hitbox.clone();
        let resize = resize.clone();
        let on_resize_end = on_resize_end.clone();
        move |event: &MouseUpEvent, phase, window, cx| {
            if !phase.capture()
                || event.button != MouseButton::Left
                || !state.read(cx).mouse_pressed()
            {
                return;
            }
            let (request, finished) = state.update(cx, |state, cx| {
                let request = state.move_pointer(event.position, resize.as_deref());
                state.hovered = hitbox.is_hovered_at(event.position, window);
                let finished = state.final_pointer_width();
                state.end(window);
                state.publish(window, cx);
                (request, finished)
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if let (Some(position), Some(end)) = (finished, &on_resize_end) {
                end(position, window, cx);
            }
            cx.stop_propagation();
        }
    });
    window.on_mouse_event({
        let state = state.clone();
        move |event: &TouchDragEvent, phase, window, cx| {
            let current = state.read(cx);
            if event.phase == TouchPhase::Started {
                if !phase.bubble()
                    || current.press.is_some()
                    || !hitbox.is_hovered_at(event.start_position, window)
                    || !starts_horizontal_drag(event.position - event.start_position)
                {
                    return;
                }
                window.prevent_default();
            } else if !phase.capture() || current.press.is_none() || current.mouse_pressed() {
                return;
            }
            let (request, finished) = state.update(cx, |state, cx| {
                if event.phase == TouchPhase::Started {
                    let width = resize.as_ref().map(|resize| {
                        state.focus.focus(window, cx);
                        resize.width
                    });
                    state.press = Some(PointerPress::new(event.start_position, width, None));
                }
                let request = if event.phase == TouchPhase::Cancelled {
                    None
                } else {
                    state.move_pointer(event.position, resize.as_deref())
                };
                let finished = (event.phase == TouchPhase::Ended)
                    .then(|| state.final_pointer_width())
                    .flatten();
                if matches!(event.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                    state.end(window);
                }
                state.publish(window, cx);
                (request, finished)
            });
            if let (Some(width), Some(resize)) = (request, &resize) {
                resize.request(width, window, cx);
            }
            if let (Some(position), Some(end)) = (finished, &on_resize_end) {
                end(position, window, cx);
            }
            cx.stop_propagation();
        }
    });
    window.on_mouse_event(move |_: &MouseExitEvent, phase, window, cx| {
        if phase.bubble() && state.read(cx).hovered {
            state.update(cx, |state, cx| {
                state.hovered = false;
                state.publish(window, cx);
            });
        }
    });
}
