mod pointer;
mod state;

use std::{ops::RangeInclusive, rc::Rc};

use gpui::{
    AccessibleAction, App, Bounds, Corners, ElementId, Entity, HitboxBehavior, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role, SharedString, Size,
    StatefulInteractiveElement, Styled, Window,
    accesskit::{ActionData, Orientation},
    canvas, div, fill, point, px, size,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, Shape, color, current_material_theme,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape};

use pointer::pointer_listeners;
use state::{HandleState, ResizeBinding, ResizeRequest, logical_delta};

const REST_SIZE: Size<Pixels> = size(px(4.), px(48.));
const ACTIVE_SIZE: Size<Pixels> = size(px(12.), px(52.));
const DEFAULT_STEP: Pixels = px(8.);

/// Visual sizes in logical pixels. Omitted values use Material defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleSizes {
    /// Handle dimensions while idle. Defaults to `4 × 48 px`.
    pub resting: Option<Size<Pixels>>,
    /// Handle dimensions while pressed. Defaults to `12 × 52 px`.
    pub pressed: Option<Size<Pixels>>,
    /// Handle dimensions while actively dragged. Defaults to `12 × 52 px`.
    pub dragged: Option<Size<Pixels>>,
}

/// Fill colors. Omitted values use the corresponding window theme role.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleColors {
    /// Fill color while idle. Defaults to `ColorRole::Outline`.
    pub resting: Option<Hsla>,
    /// Fill color while pressed. Defaults to `ColorRole::OnSurface`.
    pub pressed: Option<Hsla>,
    /// Fill color while actively dragged. Defaults to `ColorRole::OnSurface`.
    pub dragged: Option<Hsla>,
}

/// Corner shapes in logical corner order. Omitted values use Material defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragHandleShapes {
    /// Corner shape while idle. Defaults to `CornerRole::Full`.
    pub resting: Option<CornerShape>,
    /// Corner shape while pressed. Defaults to `CornerRole::Medium`.
    pub pressed: Option<CornerShape>,
    /// Corner shape while actively dragged. Defaults to `CornerRole::Medium`.
    pub dragged: Option<CornerShape>,
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

    /// Overrides the resting, pressed, and dragged handle dimensions.
    pub fn sizes(mut self, sizes: DragHandleSizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// Overrides the resting, pressed, and dragged handle fill colors.
    pub fn colors(mut self, colors: DragHandleColors) -> Self {
        self.colors = colors;
        self
    }

    /// Overrides the resting, pressed, and dragged handle corner shapes.
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
    #[cfg_attr(not(feature = "navigation"), allow(dead_code))]
    pub(crate) fn on_resize_end(
        mut self,
        callback: impl Fn(Pixels, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize_end = Some(Rc::new(callback));
        self
    }

    /// A consumer can invalidate the pointer origin when its viewport changes.
    #[cfg_attr(not(feature = "navigation"), allow(dead_code))]
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

impl RenderOnce for VerticalDragHandle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
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
        let mut root = div()
            .id(self.id)
            .test_support()
            .flex_none()
            .w(width)
            .h(height)
            .cursor_col_resize()
            .child(target);
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
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
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
