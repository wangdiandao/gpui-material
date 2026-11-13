use std::time::{Duration, Instant};

use gpui::{Hsla, Pixels, Point, Window, px};
use gpui_material_theme::StateLayer;

use crate::Shape;

const RIPPLE_DURATION: Duration = Duration::from_millis(320);
const FOCUS_RING_WIDTH: Pixels = px(3.);

/// A snapshot of interaction state from the caller's GPUI/Base behavior.
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct InteractionState {
    pub disabled: bool,
    pub hovered: bool,
    pub focus_visible: bool,
    pub pressed: bool,
    pub dragged: bool,
}

impl InteractionState {
    pub fn layer(self) -> Option<StateLayer> {
        if self.disabled {
            None
        } else if self.dragged {
            Some(StateLayer::Dragged)
        } else if self.pressed {
            Some(StateLayer::Pressed)
        } else if self.focus_visible {
            Some(StateLayer::Focus)
        } else if self.hovered {
            Some(StateLayer::Hover)
        } else {
            None
        }
    }
}

/// One pointer or keyboard ripple. The caller owns its lifetime and clears it
/// when the interaction is cancelled, loses capture or becomes disabled.
pub struct Ripple {
    origin: Point<Pixels>,
    started: Instant,
}

impl Ripple {
    /// Uses the pointer location in the prepared fill, or the center for keyboard activation.
    /// The returned origin is relative to the shape so it moves with layout.
    pub fn start(shape: &Shape, pointer: Option<Point<Pixels>>, now: Instant) -> Option<Self> {
        let bounds = shape.bounds();
        let origin = pointer.unwrap_or_else(|| bounds.center());
        shape.contains(origin).then_some(Self {
            origin: origin - bounds.origin,
            started: now,
        })
    }

    /// Draws above the state layer and requests another frame only while animating.
    pub fn paint(
        &self,
        shape: &Shape,
        color: Hsla,
        now: Instant,
        reduced_motion: bool,
        window: &mut Window,
    ) {
        let elapsed = now.saturating_duration_since(self.started);
        if reduced_motion || elapsed >= RIPPLE_DURATION {
            return;
        }
        let progress = elapsed.as_secs_f32() / RIPPLE_DURATION.as_secs_f32();
        let bounds = shape.bounds();
        let origin = bounds.origin + self.origin;
        let radius = f32::from(bounds.size.width).hypot(f32::from(bounds.size.height));
        let expansion = 1. - (1. - progress).powi(3);
        shape.paint_ripple(
            origin,
            px(radius * expansion),
            color.opacity(StateLayer::Pressed.opacity() * (1. - progress)),
            window,
        );
        window.request_animation_frame();
    }
}

/// Material feedback colors and state policy for a prepared shape.
pub struct InteractionFeedback {
    state: InteractionState,
    foreground: Hsla,
    focus: Hsla,
}

impl InteractionFeedback {
    pub fn new(state: InteractionState, foreground: Hsla, focus: Hsla) -> Self {
        Self {
            state,
            foreground,
            focus,
        }
    }

    /// Draws the state layer, optional ripple, and keyboard focus ring.
    pub fn paint(
        &self,
        shape: &Shape,
        ripple: Option<&Ripple>,
        now: Instant,
        reduced_motion: bool,
        window: &mut Window,
    ) {
        if let Some(layer) = self.state.layer() {
            shape.paint(self.foreground.opacity(layer.opacity()), window);
        }
        if !self.state.disabled {
            if let Some(ripple) = ripple {
                ripple.paint(shape, self.foreground, now, reduced_motion, window);
            }
            if self.state.focus_visible {
                shape.paint_ring(FOCUS_RING_WIDTH, self.focus, window);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_suppresses_the_layer_and_drag_precedes_press() {
        let mut state = InteractionState {
            disabled: false,
            hovered: true,
            focus_visible: true,
            pressed: true,
            dragged: true,
        };
        assert_eq!(state.layer(), Some(StateLayer::Dragged));
        state.dragged = false;
        assert_eq!(state.layer(), Some(StateLayer::Pressed));
        state.pressed = false;
        assert_eq!(state.layer(), Some(StateLayer::Focus));
        state.disabled = true;
        assert_eq!(state.layer(), None);
    }
}
