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
    /// Resolves the non-press state layer when a ripple supplies press feedback.
    /// Reduced motion uses the static pressed layer instead of an animated ripple.
    pub fn layer_with_ripple(self, reduced_motion: bool) -> Option<StateLayer> {
        Self {
            pressed: self.pressed && reduced_motion,
            ..self
        }
        .layer()
    }

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
        let (progress, alpha) = self.sample(now);
        if alpha <= 0. && progress >= 1. {
            return;
        }
        let bounds = shape.bounds();
        let center = bounds.center() - bounds.origin;
        let origin = bounds.origin + self.origin + (center - self.origin) * progress;
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let start_radius = width.max(height) * 0.3;
        let end_radius = width.hypot(height) * 0.5 + 10.;
        let expansion = gpui_base::motion::Easing::CubicBezier {
            x1: 0.4,
            y1: 0.,
            x2: 0.2,
            y2: 1.,
        }
        .sample(progress);
        shape.paint_ripple(
            origin,
            px(start_radius + (end_radius - start_radius) * expansion),
            color.opacity(StateLayer::Pressed.opacity() * alpha),
            window,
        );
        if progress < 1. || self.released.is_some() {
            window.request_animation_frame();
        }
    }

    // Compose RippleAnimation waits for radius/center expansion before fading out.
    // A quick release reaches full opacity immediately so short taps remain visible.
    fn sample(&self, now: Instant) -> (f32, f32) {
        let elapsed = now.saturating_duration_since(self.started);
        let fade_start = self
            .released
            .map(|time| time.max(self.started + RIPPLE_EXPANSION));
        let fade = fade_start.map_or(0., |time| {
            now.saturating_duration_since(time).as_secs_f32() / RIPPLE_FADE_OUT.as_secs_f32()
        });
        let progress = (elapsed.as_secs_f32() / RIPPLE_EXPANSION.as_secs_f32()).min(1.);
        let fade_in = if self.released.is_some_and(|released| now >= released) {
            1.
        } else {
            (elapsed.as_secs_f32() / RIPPLE_FADE_IN.as_secs_f32()).min(1.)
        };
        (progress, fade_in * (1. - fade).clamp(0., 1.))
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
            self.state
                .layer_with_ripple(reduced_motion || ripple.is_none())
                .map_or(0., |layer| layer.opacity()),
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
    fn ripple_owns_press_while_hover_focus_and_reduced_motion_remain_visible() {
        let mut state = InteractionState {
            pressed: true,
            ..Default::default()
        };
        assert_eq!(state.layer_with_ripple(false), None);
        assert_eq!(state.layer_with_ripple(true), Some(StateLayer::Pressed));
        state.hovered = true;
        assert_eq!(state.layer_with_ripple(false), Some(StateLayer::Hover));
        state.focus_visible = true;
        assert_eq!(state.layer_with_ripple(false), Some(StateLayer::Focus));
        state.dragged = true;
        assert_eq!(state.layer_with_ripple(false), Some(StateLayer::Dragged));
        state.disabled = true;
        assert_eq!(state.layer_with_ripple(false), None);
        assert_eq!(state.layer_with_ripple(true), None);
    }

    #[test]
    fn quick_tap_finishes_expansion_before_fading_and_held_ripple_settles() {
        let now = Instant::now();
        let mut ripple = Ripple {
            origin: gpui::point(px(0.), px(0.)),
            started: now,
            released: None,
        };
        assert_eq!(ripple.sample(now), (0., 0.));
        assert_eq!(ripple.sample(now + Duration::from_secs(1)), (1., 1.));
        ripple.release(now + Duration::from_millis(10));
        assert_eq!(ripple.sample(now + Duration::from_millis(10)).1, 1.);
        assert_eq!(ripple.sample(now + RIPPLE_EXPANSION), (1., 1.));
        assert_eq!(
            ripple.sample(now + RIPPLE_EXPANSION + RIPPLE_FADE_OUT / 2),
            (1., 0.5)
        );
        assert_eq!(
            ripple.sample(now + RIPPLE_EXPANSION + RIPPLE_FADE_OUT),
            (1., 0.)
        );
        ripple.release(now + Duration::from_secs(2));
        assert_eq!(ripple.sample(now + Duration::from_secs(2)), (1., 0.));
    }

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
