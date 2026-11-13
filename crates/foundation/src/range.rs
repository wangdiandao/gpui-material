use gpui::{
    AccessibleAction, App, Axis, Div, Entity, InteractiveElement, Role, SharedString, Stateful,
    StatefulInteractiveElement, Window,
    accesskit::{ActionData, Orientation},
};
use gpui_base::slider::{SliderState, SliderValue};

/// Identifies the Base slider value represented by a semantic node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeThumb {
    Start,
    End,
}

impl RangeThumb {
    fn value(self, value: SliderValue) -> f32 {
        match self {
            Self::Start => value.start(),
            Self::End => value.end(),
        }
    }
}

/// Formats the same Base slider value that the track and semantics expose.
/// The caller supplies locale-aware formatting and decides when to announce it.
pub fn value_label(
    state: &Entity<SliderState>,
    thumb: RangeThumb,
    cx: &App,
    format: impl FnOnce(f32) -> SharedString,
) -> SharedString {
    format(thumb.value(state.read(cx).value()))
}

/// Adds native slider semantics to a focusable host for Base track and thumb elements.
/// Windows UIA SetValue, increment/decrement and arrow keys share Base's value update.
/// For a disabled control, use the disabled Base Slider semantic host and omit this adapter.
pub fn range_semantics(
    element: Stateful<Div>,
    state: &Entity<SliderState>,
    thumb: RangeThumb,
    axis: Axis,
    cx: &App,
) -> Stateful<Div> {
    let slider = state.read(cx);
    let value = slider.value();
    let (min, max) = match (value.is_range(), thumb) {
        (true, RangeThumb::Start) => (slider.min_value(), value.end()),
        (true, RangeThumb::End) => (value.start(), slider.max_value()),
        _ => (slider.min_value(), slider.max_value()),
    };
    let element = element
        .role(Role::Slider)
        .aria_numeric_value(f64::from(thumb.value(value)))
        .aria_min_numeric_value(f64::from(min))
        .aria_max_numeric_value(f64::from(max))
        .aria_numeric_value_step(f64::from(slider.accessibility_step()))
        .aria_orientation(if axis == Axis::Horizontal {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        });

    let set = state.clone();
    let increment = state.clone();
    let decrement = state.clone();
    let keyboard = state.clone();
    let release = state.clone();
    let release_outside = state.clone();
    element
        .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
            if let Some(ActionData::NumericValue(target)) = data
                && target.is_finite()
            {
                set.update(cx, |state, cx| {
                    let target = target
                        .clamp(f64::from(state.min_value()), f64::from(state.max_value()))
                        as f32;
                    let delta = target - thumb.value(state.value());
                    state.step_thumb(thumb == RangeThumb::Start, delta, window, cx);
                });
            }
        })
        .on_a11y_action(AccessibleAction::Increment, move |_, window, cx| {
            step(&increment, thumb, 1., window, cx)
        })
        .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
            step(&decrement, thumb, -1., window, cx)
        })
        .on_key_down(move |event, window, cx| {
            let direction = match event.keystroke.key.as_str() {
                "left" | "down" => -1.,
                "right" | "up" => 1.,
                _ => return,
            };
            step(&keyboard, thumb, direction, window, cx);
            cx.stop_propagation();
        })
        .on_mouse_up(gpui::MouseButton::Left, move |_, _, cx| {
            release.update(cx, |state, cx| state.handle_release(cx));
        })
        .on_mouse_up_out(gpui::MouseButton::Left, move |_, _, cx| {
            release_outside.update(cx, |state, cx| state.handle_release(cx));
        })
}

fn step(
    state: &Entity<SliderState>,
    thumb: RangeThumb,
    direction: f32,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        state.step_thumb(
            thumb == RangeThumb::Start,
            direction * state.accessibility_step(),
            window,
            cx,
        );
    });
}
