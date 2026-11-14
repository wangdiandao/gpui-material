use std::time::{Duration, Instant};

use gpui::{Hsla, Pixels, Point, Window, px};
use gpui_material_theme::StateLayer;

use crate::Shape;

const RIPPLE_EXPANSION: Duration = Duration::from_millis(225);
const RIPPLE_FADE_IN: Duration = Duration::from_millis(75);
const RIPPLE_FADE_OUT: Duration = Duration::from_millis(150);
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

/// One pointer or keyboard ripple. A held ripple is released when the
/// interaction ends; a one-shot ripple releases itself.
#[derive(Clone)]
pub struct Ripple {
    origin: Point<Pixels>,
    started: Instant,
    released: Option<Instant>,
}

impl Ripple {
    /// Starts a one-shot ripple at the activation point or the shape's center.
    pub fn start(shape: &Shape, pointer: Option<Point<Pixels>>, now: Instant) -> Option<Self> {
        let mut ripple = Self::press(shape, pointer, now)?;
        ripple.release(now);
        Some(ripple)
    }

    /// Starts a ripple that holds its settled opacity until released.
    pub fn press(shape: &Shape, pointer: Option<Point<Pixels>>, now: Instant) -> Option<Self> {
        let bounds = shape.bounds();
        let origin = pointer.unwrap_or_else(|| bounds.center());
        shape.contains(origin).then_some(Self {
            origin: origin - bounds.origin,
            started: now,
            released: None,
        })
    }

    /// Begins the exit from the current ripple opacity after a release or cancellation.
    pub fn release(&mut self, now: Instant) {
        self.released.get_or_insert(now);
    }

    /// Draws above the state layer and requests another frame while its radius or opacity changes.
    pub fn paint(
        &self,
        shape: &Shape,
        color: Hsla,
        now: Instant,
        reduced_motion: bool,
        window: &mut Window,
    ) {
        if reduced_motion {
            return;
        }
        let elapsed = now.saturating_duration_since(self.started);
        let fade_start = self
            .released
            .map(|time| time.max(self.started + RIPPLE_FADE_IN));
        let fade = fade_start.map_or(0., |time| {
            now.saturating_duration_since(time).as_secs_f32() / RIPPLE_FADE_OUT.as_secs_f32()
        });
        if fade >= 1. {
            return;
        }
        let progress = (elapsed.as_secs_f32() / RIPPLE_EXPANSION.as_secs_f32()).min(1.);
        let fade_in = (elapsed.as_secs_f32() / RIPPLE_FADE_IN.as_secs_f32()).min(1.);
        let bounds = shape.bounds();
        let origin = bounds.origin + self.origin;
        let dx = f32::from(self.origin.x.max(bounds.size.width - self.origin.x));
        let dy = f32::from(self.origin.y.max(bounds.size.height - self.origin.y));
        let expansion = 1. - (1. - progress).powi(3);
        shape.paint_ripple(
            origin,
            px(dx.hypot(dy) * expansion),
            color.opacity(StateLayer::Pressed.opacity() * fade_in * (1. - fade)),
            window,
        );
        if progress < 1. || self.released.is_some() {
            window.request_animation_frame();
        }
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
        self.paint_with_opacity(
            shape,
            ripple,
            self.state.layer().map_or(0., |layer| layer.opacity()),
            now,
            reduced_motion,
            window,
        );
    }

    /// Paints a state layer sampled by the caller's stable motion channel.
    pub fn paint_with_opacity(
        &self,
        shape: &Shape,
        ripple: Option<&Ripple>,
        opacity: f32,
        now: Instant,
        reduced_motion: bool,
        window: &mut Window,
    ) {
        if !self.state.disabled && opacity > 0. {
            shape.paint(self.foreground.opacity(opacity), window);
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
