use std::time::Duration;

use gpui::Context;
use gpui_base::motion::Easing;
use gpui_material_foundation::RetargetTransition;

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
}

/// Retained presentation of a docked rail. Modal visibility uses DialogState.
pub struct NavigationRailState {
    current: RailMode,
    target: RailMode,
    collapsed: RetargetTransition,
    expanded: RetargetTransition,
    progress: f32,
    duration: Duration,
}

impl Default for NavigationRailState {
    fn default() -> Self {
        Self::new(RailMode::Collapsed)
    }
}

impl NavigationRailState {
    /// Creates retained docked rail state starting in `initial` mode with a 240 ms default transition.
    pub fn new(initial: RailMode) -> Self {
        let weights = WidthWeights::for_mode(initial);
        Self {
            current: initial,
            target: initial,
            collapsed: RetargetTransition::settled(weights.collapsed),
            expanded: RetargetTransition::settled(weights.expanded),
            progress: 1.,
            duration: Duration::from_millis(240),
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

    /// Returns the mode currently settled at or transitioning toward.
    pub fn target_value(&self) -> RailMode {
        self.target
    }

    /// Returns whether a rail width transition is currently running.
    pub fn is_animating(&self) -> bool {
        self.collapsed.is_animating() || self.expanded.is_animating()
    }

    /// Fraction of the active transition, or one at the target.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Animates the rail to [`RailMode::Expanded`].
    pub fn expand(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Expanded, cx);
    }

    /// Animates the rail to [`RailMode::Collapsed`].
    pub fn collapse(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Collapsed, cx);
    }

    /// Toggles between [`RailMode::Expanded`] and [`RailMode::Collapsed`].
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        match self.target {
            RailMode::Expanded => self.collapse(cx),
            _ => self.expand(cx),
        }
    }

    /// Animates the rail to [`RailMode::Hidden`].
    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.animate_to(RailMode::Hidden, cx);
    }

    /// Moves immediately to `mode` without animation.
    pub fn snap_to(&mut self, mode: RailMode, cx: &mut Context<Self>) {
        if self.current != mode || self.target != mode || self.is_animating() {
            self.target = mode;
            self.settle();
            cx.notify();
        }
    }

    /// Animates the rail to `mode`, respecting reduced-motion settings.
    pub fn animate_to(&mut self, mode: RailMode, cx: &mut Context<Self>) {
        if cx.reduce_motion() || self.duration.is_zero() {
            self.snap_to(mode, cx);
            return;
        }
        let (_, changed) = self.sample(cx);
        if self.target != mode {
            self.target = mode;
            let weights = WidthWeights::for_mode(mode);
            let now = cx.background_executor().now();
            self.collapsed.animate_to(
                weights.collapsed,
                self.duration,
                Easing::EaseInOut,
                now,
                false,
            );
            self.expanded.animate_to(
                weights.expanded,
                self.duration,
                Easing::EaseInOut,
                now,
                false,
            );
            self.progress = if self.is_animating() { 0. } else { 1. };
            if !self.is_animating() {
                self.current = mode;
            }
            cx.notify();
        } else if changed {
            cx.notify();
        }
    }

    fn settle(&mut self) {
        self.current = self.target;
        let weights = WidthWeights::for_mode(self.target);
        self.collapsed.snap_to(weights.collapsed);
        self.expanded.snap_to(weights.expanded);
        self.progress = 1.;
    }

    /// Samples the active width transition and returns `(weights, changed)` without calling `cx.notify()`.
    pub(super) fn sample(&mut self, cx: &mut Context<Self>) -> (WidthWeights, bool) {
        let mut changed = false;
        if self.is_animating() {
            let now = cx.background_executor().now();
            let reduced = cx.reduce_motion();
            let prev = (self.collapsed.value(), self.expanded.value());
            let (c, c_run) = self.collapsed.sample(now, reduced);
            let (e, e_run) = self.expanded.sample(now, reduced);
            changed = if !c_run && !e_run {
                self.settle();
                true
            } else {
                let target = WidthWeights::for_mode(self.target);
                let dist = (c - target.collapsed)
                    .abs()
                    .max((e - target.expanded).abs());
                self.progress = (1. - dist).clamp(0., 1.);
                (c, e) != prev
            };
        }
        (
            WidthWeights {
                collapsed: self.collapsed.value(),
                expanded: self.expanded.value(),
            },
            changed,
        )
    }
}
