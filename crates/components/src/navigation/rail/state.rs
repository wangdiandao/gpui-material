use std::time::Duration;

use gpui::Context;
use web_time::Instant;

use super::RailMode;

#[derive(Clone, Copy)]
pub(super) struct WidthWeights {
    pub collapsed: f32,
    pub expanded: f32,
}

impl WidthWeights {
    fn for_mode(mode: RailMode) -> Self {
        Self {
            collapsed: f32::from(mode == RailMode::Collapsed),
            expanded: f32::from(mode == RailMode::Expanded),
        }
    }

    fn interpolate(self, target: Self, fraction: f32) -> Self {
        let mix = |from, to| from + (to - from) * fraction;
        Self {
            collapsed: mix(self.collapsed, target.collapsed),
            expanded: mix(self.expanded, target.expanded),
        }
    }
}

struct Transition {
    from: WidthWeights,
    started: Instant,
    duration: Duration,
}

/// Retained presentation of a docked rail. Modal visibility uses DialogState.
pub struct NavigationRailState {
    current: RailMode,
    target: RailMode,
    width: WidthWeights,
    progress: f32,
    duration: Duration,
    transition: Option<Transition>,
}

impl Default for NavigationRailState {
    fn default() -> Self {
        Self::new(RailMode::Collapsed)
    }
}

impl NavigationRailState {
    pub fn new(initial: RailMode) -> Self {
        Self {
            current: initial,
            target: initial,
            width: WidthWeights::for_mode(initial),
            progress: 1.,
            duration: Duration::from_millis(240),
            transition: None,
        }
    }

    /// Sets the duration used for new transitions.
    pub fn transition_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// The last settled mode, or the initial mode before the first transition.
    pub fn current_value(&self) -> RailMode {
        self.current
    }

    pub fn target_value(&self) -> RailMode {
        self.target
    }

    pub fn is_animating(&self) -> bool {
        self.transition.is_some()
    }

    /// Fraction of the active transition, or one at the target.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn expand(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Expanded, cx);
    }

    pub fn collapse(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Collapsed, cx);
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        match self.target {
            RailMode::Expanded => self.collapse(cx),
            _ => self.expand(cx),
        }
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Hidden, cx);
    }

    pub fn snap_to(&mut self, mode: RailMode, cx: &mut Context<Self>) {
        if self.current != mode || self.target != mode || self.transition.is_some() {
            self.target = mode;
            self.settle();
            cx.notify();
        }
    }

    pub fn animate_to(&mut self, mode: RailMode, cx: &mut Context<Self>) {
        if cx.reduce_motion() || self.duration.is_zero() {
            self.snap_to(mode, cx);
            return;
        }
        self.sample(cx);
        if self.target != mode {
            self.target = mode;
            self.progress = 0.;
            self.transition = Some(Transition {
                from: self.width,
                started: cx.background_executor().now(),
                duration: self.duration,
            });
            cx.notify();
        }
    }

    fn settle(&mut self) {
        self.current = self.target;
        self.width = WidthWeights::for_mode(self.target);
        self.progress = 1.;
        self.transition = None;
    }

    pub(super) fn sample(&mut self, cx: &mut Context<Self>) -> WidthWeights {
        if let Some(transition) = &self.transition {
            let elapsed = cx.background_executor().now() - transition.started;
            if elapsed >= transition.duration || cx.reduce_motion() {
                self.settle();
                cx.notify();
            } else {
                let fraction = (elapsed.as_secs_f64() / transition.duration.as_secs_f64()) as f32;
                if fraction != self.progress {
                    let eased = fraction * fraction * (3. - 2. * fraction);
                    self.width = transition
                        .from
                        .interpolate(WidthWeights::for_mode(self.target), eased);
                    self.progress = fraction;
                    cx.notify();
                }
            }
        }
        self.width
    }
}
