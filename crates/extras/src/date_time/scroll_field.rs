use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AccessibleAction, AnyElement, App, Context, Div, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, Role, ScrollEvent,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Task, TouchPhase, Window,
    accesskit::{ActionData, Orientation},
    canvas, div,
    prelude::FluentBuilder as _,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material_components::Text;
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, InteractionTracker, Shape, animate,
    color, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, MaterialTheme, MotionKind, MotionSpec, MotionSpeed,
    TypeRole,
};

const HEIGHT: f32 = 200.;
const ROW_HEIGHT: f32 = HEIGHT / 3.;
const PRECISE_SCROLL_SETTLE: Duration = Duration::from_millis(120);
// Move near a distant destination before animating the last few rows.
const ANIMATED_ROWS: usize = 3;

/// Caller-owned position and focus for one Material scroll field.
pub struct ScrollFieldState {
    item_count: usize,
    position: Position,
    target: usize,
    motion: Option<Motion>,
    gesture_start: Option<usize>,
    settle: Option<Task<()>>,
    focus: FocusHandle,
}

#[derive(Clone, Copy, PartialEq)]
struct Position {
    index: usize,
    fraction: f32,
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

fn wrap(index: usize, rows: isize, count: usize) -> usize {
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
struct Motion {
    start: f32,
    restart: bool,
    origin: ScrollOrigin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollOrigin {
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

    pub fn is_scroll_in_progress(&self) -> bool {
        self.motion.is_some() || self.gesture_start.is_some()
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// Centers the clamped option without animation.
    pub fn scroll_to_option(&mut self, index: usize, cx: &mut Context<Self>) {
        self.target = index.min(self.item_count.saturating_sub(1));
        self.gesture_start = None;
        self.settle = None;
        self.motion = None;
        self.position = Position::at(self.target);
        cx.notify();
    }

    /// Scrolls to the clamped option using the window's motion policy.
    pub fn animate_scroll_to_option(&mut self, index: usize, cx: &mut Context<Self>) {
        self.animate_to(index, ScrollOrigin::Application, cx);
    }

    fn animate_to(&mut self, index: usize, origin: ScrollOrigin, cx: &mut Context<Self>) {
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

    fn sample(&mut self, offset: f32, cx: &mut Context<Self>) {
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

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
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

    fn select_row(&mut self, row: isize, cx: &mut Context<Self>) {
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

    fn scroll(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) -> bool {
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
            // Some discrete touchpad updates omit a terminal phase.
            let timer = cx.background_executor().timer(PRECISE_SCROLL_SETTLE);
            let state = cx.weak_entity();
            self.settle = Some(window.spawn(cx, async move |cx| {
                timer.await;
                let _ = cx.update(|_, cx| {
                    let _ = state.update(cx, |state, cx| state.finish_gesture(false, cx));
                });
            }));
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

    fn has_pending_interaction(&self) -> bool {
        self.gesture_start.is_some()
            || self
                .motion
                .is_some_and(|motion| motion.origin == ScrollOrigin::Interaction)
    }
}

/// Optional overrides for the window theme's scroll field colors.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollFieldColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub selected_content: Option<Hsla>,
    pub disabled_container: Option<Hsla>,
    pub disabled_content: Option<Hsla>,
    pub disabled_selected_content: Option<Hsla>,
}

impl ScrollFieldColors {
    fn resolve(self, enabled: bool, theme: &MaterialTheme) -> (Hsla, Hsla, Hsla) {
        let container = color(theme.colors.color(ColorRole::SurfaceContainerLowest));
        let selected = color(theme.colors.color(ColorRole::OnSurface));
        if enabled {
            (
                self.container.unwrap_or(container),
                self.content
                    .unwrap_or_else(|| color(theme.colors.color(ColorRole::Outline))),
                self.selected_content.unwrap_or(selected),
            )
        } else {
            (
                self.disabled_container
                    .unwrap_or_else(|| container.opacity(0.38)),
                self.disabled_content
                    .unwrap_or_else(|| selected.opacity(0.38)),
                self.disabled_selected_content
                    .unwrap_or_else(|| selected.opacity(0.38)),
            )
        }
    }
}

type ItemContent = Box<dyn Fn(usize, bool, bool) -> AnyElement>;
type ValueDescription = Box<dyn Fn(usize) -> SharedString>;

/// A vertically scrolling selection wheel with a single accessible focus stop.
/// The field fills its parent's width and displays three rows at a time.
#[derive(IntoElement)]
pub struct ScrollField {
    id: ElementId,
    name: SharedString,
    state: Entity<ScrollFieldState>,
    disabled: bool,
    colors: ScrollFieldColors,
    time_field_shape: Option<CornerShape>,
    numeric_offset: usize,
    source: Option<Entity<InteractionSource>>,
    item: Option<ItemContent>,
    description: Option<ValueDescription>,
}

impl ScrollField {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        state: &Entity<ScrollFieldState>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state: state.clone(),
            disabled: false,
            colors: ScrollFieldColors::default(),
            time_field_shape: None,
            numeric_offset: 0,
            source: None,
            item: None,
            description: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn colors(mut self, colors: ScrollFieldColors) -> Self {
        self.colors = colors;
        self
    }

    pub(super) fn time_field(mut self, shape: Option<CornerShape>, numeric_offset: usize) -> Self {
        self.time_field_shape = shape;
        self.numeric_offset = numeric_offset;
        self
    }

    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
    }

    /// Renders a decorative row with its index, selected state and enabled state.
    pub fn item_content<E: IntoElement>(
        mut self,
        item: impl Fn(usize, bool, bool) -> E + 'static,
    ) -> Self {
        self.item = Some(Box::new(move |index, selected, enabled| {
            item(index, selected, enabled).into_any_element()
        }));
        self
    }

    /// Describes an option to assistive technology. By default, uses its index.
    pub fn value_description<S: Into<SharedString>>(
        mut self,
        description: impl Fn(usize) -> S + 'static,
    ) -> Self {
        self.description = Some(Box::new(move |index| description(index).into()));
        self
    }
}

impl RenderOnce for ScrollField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let corner = self
            .time_field_shape
            .unwrap_or_else(|| CornerRole::Large.shape());
        let direction = theme.layout.direction;
        let [tl, tr, br, bl] = corner.radii(HEIGHT, HEIGHT, direction).map(px);
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let motion_spec =
            theme
                .motion
                .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
        if self.disabled && self.state.read(cx).has_pending_interaction() {
            self.state
                .update(cx, |state, cx| state.scroll_to_option(state.target, cx));
        }
        let motion = self.state.read(cx).motion;
        if let Some(motion) = motion {
            let key = (self.id.clone(), "scroll-position");
            if motion.restart {
                animate(key.clone(), motion.start, MotionSpec::Instant, window, cx);
            }
            let offset = animate(key, 0., motion_spec, window, cx);
            self.state.update(cx, |state, cx| state.sample(offset, cx));
        }
        let state = self.state.read(cx);
        let count = state.item_count;
        let position = state.position;
        let target = state.target;
        let focus = state.focus.clone();
        let enabled = !self.disabled && count > 0;
        let (container, content, selected_content) = self
            .colors
            .resolve(enabled, window_material_theme(window, cx));
        let feedback = window
            .use_keyed_state((self.id.clone(), "scroll-field-feedback"), cx, |_, _| {
                Rc::new(Cell::new(InteractionState::default()))
            })
            .read(cx)
            .clone();
        let input = Input {
            tracker: self
                .source
                .as_ref()
                .map(|source| InteractionSource::track(source, &self.id, window, cx)),
            state: self.state,
            feedback,
            numeric_offset: self.numeric_offset,
        };
        let mut interaction = input.feedback.get();
        if !enabled || !window.is_window_active() {
            interaction = InteractionState::default();
        }
        interaction.disabled = !enabled;
        input.update_feedback(interaction, window, cx);
        let feedback = InteractionFeedback::new(input.feedback.get(), selected_content, ring);
        let label = (count > 0).then(|| {
            self.description
                .as_ref()
                .map_or_else(|| target.to_string().into(), |describe| describe(target))
        });
        let rows = (-2_isize..=2)
            .filter(|row| count > 1 || (count == 1 && *row == 0))
            .map(|row| {
                let index = wrap(position.index, row, count);
                let selected = row == 0;
                let visual = self.item.as_ref().map_or_else(
                    || {
                        Text::new(format!("{index:02}"))
                            .role(if selected {
                                TypeRole::DisplayLarge
                            } else {
                                TypeRole::DisplayMedium
                            })
                            .emphasized(selected)
                            .into_any_element()
                    },
                    |item| item(index, selected, enabled),
                );
                let input = input.state.clone();
                div()
                    .absolute()
                    .top(px((1. + row as f32 - position.fraction) * ROW_HEIGHT))
                    .h(px(ROW_HEIGHT))
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(if selected { selected_content } else { content })
                    .id(("scroll-field-row", (row + 2) as usize))
                    .role(Role::Group)
                    .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                    .when(enabled, |row_element| {
                        row_element.on_click(move |_, window, cx| {
                            input.update(cx, |state, cx| {
                                state.focus.focus(window, cx);
                                state.select_row(row, cx);
                            });
                        })
                    })
                    .child(visual)
                    .test_support()
            });
        let field = div()
            .id(self.id)
            .relative()
            .w_full()
            .min_w_0()
            .h(px(HEIGHT))
            .flex_none()
            .overflow_hidden()
            .rounded_tl(tl)
            .rounded_tr(tr)
            .rounded_br(br)
            .rounded_bl(bl)
            .bg(container)
            .role(Role::Slider)
            .aria_label(self.name)
            .aria_orientation(Orientation::Vertical)
            .when_some(label, |field, label| {
                field
                    .aria_value(label)
                    .aria_numeric_value(target as f64 + self.numeric_offset as f64)
                    .aria_min_numeric_value(self.numeric_offset as f64)
                    .aria_max_numeric_value((count - 1) as f64 + self.numeric_offset as f64)
                    .aria_numeric_value_step(1.)
            })
            .when(!enabled, |field| {
                field.a11y_synthetic_children(|tree| tree.parent_node().set_disabled())
            })
            .children(rows)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        if let Some(shape) = Shape::rounded(bounds, corner, direction) {
                            feedback.paint(
                                &shape,
                                None,
                                cx.background_executor().now(),
                                cx.reduce_motion(),
                                window,
                            );
                        }
                    },
                )
                .absolute()
                .inset_0(),
            );
        field
            .when(enabled, |field| input.bind(field))
            .test_support()
            .when(enabled || focus.is_focused(window), |field| {
                field.track_focus(&focus.tab_stop(enabled))
            })
    }
}

#[derive(Clone)]
struct Input {
    state: Entity<ScrollFieldState>,
    feedback: Rc<Cell<InteractionState>>,
    numeric_offset: usize,
    tracker: Option<InteractionTracker>,
}

impl Input {
    fn update_feedback(&self, mut feedback: InteractionState, window: &mut Window, cx: &mut App) {
        let state = self.state.read(cx);
        feedback.focus_visible = !feedback.disabled
            && state.focus.is_focused(window)
            && window.last_input_was_keyboard();
        feedback.dragged = !feedback.disabled && state.gesture_start.is_some();
        if self.feedback.replace(feedback) != feedback {
            window.refresh();
        }
        if let Some(tracker) = &self.tracker {
            tracker.record(feedback, cx);
        }
    }

    fn bind(&self, field: Stateful<Div>) -> Stateful<Div> {
        let scroll = self.clone();
        let hover = self.clone();
        let press = self.clone();
        let release = self.clone();
        let release_out = self.clone();
        let keyboard = self.state.clone();
        let increment = self.state.clone();
        let decrement = self.state.clone();
        let set_value = self.state.clone();
        let numeric_offset = self.numeric_offset as f64;
        field
            .on_scroll(move |event, window, cx| {
                if scroll
                    .state
                    .update(cx, |state, cx| state.scroll(event, window, cx))
                {
                    scroll.update_feedback(scroll.feedback.get(), window, cx);
                    cx.stop_propagation();
                }
            })
            .on_hover(move |hovered, window, cx| {
                let mut feedback = hover.feedback.get();
                feedback.hovered = *hovered;
                hover.update_feedback(feedback, window, cx);
            })
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                press.pointer(true, window, cx)
            })
            .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                release.pointer(false, window, cx)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                release_out.pointer(false, window, cx)
            })
            .on_key_down(move |event, _, cx| {
                match event.keystroke.key.as_str() {
                    "up" => keyboard.update(cx, |state, cx| state.step(-1, cx)),
                    "down" => keyboard.update(cx, |state, cx| state.step(1, cx)),
                    "home" => keyboard.update(cx, |state, cx| {
                        state.animate_to(0, ScrollOrigin::Interaction, cx)
                    }),
                    "end" => keyboard.update(cx, |state, cx| {
                        state.animate_to(
                            state.item_count.saturating_sub(1),
                            ScrollOrigin::Interaction,
                            cx,
                        )
                    }),
                    _ => return,
                }
                cx.stop_propagation();
            })
            .on_a11y_action(AccessibleAction::Increment, move |_, _, cx| {
                increment.update(cx, |state, cx| state.step(1, cx))
            })
            .on_a11y_action(AccessibleAction::Decrement, move |_, _, cx| {
                decrement.update(cx, |state, cx| state.step(-1, cx))
            })
            .on_a11y_action(AccessibleAction::SetValue, move |data, _, cx| {
                let value = match data {
                    Some(ActionData::NumericValue(value)) => Some(*value),
                    Some(ActionData::Value(value)) => value.trim().parse::<f64>().ok(),
                    _ => None,
                };
                if let Some(value) = value.filter(|value| value.is_finite()) {
                    set_value.update(cx, |state, cx| {
                        state.animate_to(
                            (value - numeric_offset).round() as usize,
                            ScrollOrigin::Interaction,
                            cx,
                        )
                    });
                }
            })
    }

    fn pointer(&self, pressed: bool, window: &mut Window, cx: &mut App) {
        let mut feedback = self.feedback.get();
        feedback.pressed = pressed;
        self.update_feedback(feedback, window, cx);
    }
}
