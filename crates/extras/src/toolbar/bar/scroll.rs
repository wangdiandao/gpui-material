use std::time::Duration;

use gpui::{Context, Pixels, px};
use gpui_material_theme::{MotionKind, MotionScheme, MotionSpec, MotionSpeed};

/// Logical edge toward which the toolbar exits. Start and End mirror in RTL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarExitDirection {
    Start,
    End,
    Top,
    Bottom,
}

/// Projects release velocity before choosing a toolbar endpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolbarFling {
    pub projection: Duration,
    /// Uses the snap motion when omitted.
    pub motion: Option<MotionSpec>,
}

impl Default for ToolbarFling {
    fn default() -> Self {
        Self {
            projection: Duration::from_millis(150),
            motion: None,
        }
    }
}

/// Controls the toolbar's expansion thresholds and optional slide away from its resting edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolbarScrollBehavior {
    pub exit_direction: Option<ToolbarExitDirection>,
    pub expand_threshold: Pixels,
    pub collapse_threshold: Pixels,
    pub snap_motion: Option<MotionSpec>,
    pub fling: ToolbarFling,
}

impl Default for ToolbarScrollBehavior {
    fn default() -> Self {
        Self {
            exit_direction: None,
            expand_threshold: px(40.),
            collapse_threshold: px(40.),
            snap_motion: None,
            fling: ToolbarFling::default(),
        }
    }
}

#[derive(Clone, Copy)]
enum Request {
    Position(f32),
    Settle(f32),
}

#[derive(Clone, Copy, Default)]
pub(super) enum SettleMotion {
    #[default]
    Direct,
    Snap,
    Fling,
}

impl SettleMotion {
    pub(super) fn spec(
        self,
        behavior: ToolbarScrollBehavior,
        scheme: MotionScheme,
        reduced: bool,
    ) -> MotionSpec {
        let snap = || {
            behavior
                .snap_motion
                .unwrap_or_else(|| scheme.spec(MotionKind::Effects, MotionSpeed::Default, false))
        };
        match self {
            _ if reduced => MotionSpec::Instant,
            Self::Direct => MotionSpec::Instant,
            Self::Snap => snap(),
            Self::Fling => behavior.fling.motion.unwrap_or_else(snap),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Metrics {
    content: f32,
    offset: f32,
    limit: f32,
    fraction: f32,
}

/// Retained toolbar scroll state. Use one entity per toolbar.
#[derive(Default)]
pub struct ToolbarScrollState {
    supplied_offset: f32,
    last: Option<f32>,
    origin: f32,
    direction: i8,
    collapsed: bool,
    target: f32,
    motion: SettleMotion,
    request: Option<Request>,
    metrics: Metrics,
}

#[derive(Clone, Copy)]
pub(super) struct ScrollPosition {
    pub(super) target: f32,
    pub(super) motion: SettleMotion,
    pub(super) expanded: bool,
    content: f32,
}

impl ToolbarScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Supplies nonnegative logical pixels from the top of externally scrolled content.
    pub fn set_content_offset(&mut self, offset: Pixels, cx: &mut Context<Self>) {
        let offset = length(offset).unwrap_or(0.);
        if self.supplied_offset != offset {
            self.supplied_offset = offset;
            cx.notify();
        }
    }

    pub fn set_collapsed_fraction(&mut self, fraction: f32, cx: &mut Context<Self>) {
        if fraction.is_finite() {
            self.request = Some(Request::Position(fraction.clamp(0., 1.)));
            cx.notify();
        }
    }

    /// Releases an application-owned gesture. Positive logical pixels per second exit the strip.
    pub fn settle(&mut self, velocity: f32, cx: &mut Context<Self>) {
        self.request = Some(Request::Settle(if velocity.is_finite() {
            velocity
        } else {
            0.
        }));
        cx.notify();
    }

    pub fn content_offset(&self) -> Pixels {
        px(self.metrics.content)
    }

    /// The displayed nonnegative travel toward the configured logical edge.
    pub fn offset(&self) -> Pixels {
        px(self.metrics.offset)
    }

    pub fn offset_limit(&self) -> Pixels {
        px(self.metrics.limit)
    }

    pub fn collapsed_fraction(&self) -> f32 {
        self.metrics.fraction
    }

    pub(super) fn prepare(
        &mut self,
        linked_offset: Option<Pixels>,
        limit: f32,
        behavior: ToolbarScrollBehavior,
    ) -> ScrollPosition {
        let content =
            linked_offset.map_or(self.supplied_offset, |offset| length(offset).unwrap_or(0.));
        let previous = self.last.replace(content).unwrap_or(content);
        let delta = content - previous;
        if delta != 0. {
            let direction = if delta > 0. { 1 } else { -1 };
            if direction != self.direction {
                self.direction = direction;
                self.origin = previous;
            }
            if content == 0.
                || direction < 0
                    && self.origin - content >= length(behavior.expand_threshold).unwrap_or(40.)
            {
                self.collapsed = false;
            } else if direction > 0
                && content - self.origin >= length(behavior.collapse_threshold).unwrap_or(40.)
            {
                self.collapsed = true;
            }
            self.motion = SettleMotion::Direct;
            self.target = if behavior.exit_direction.is_none() {
                f32::from(self.collapsed)
            } else if content == 0. {
                0.
            } else {
                ((self.metrics.offset + delta) / limit.max(1.)).clamp(0., 1.)
            };
        }
        if let Some(request) = self.request.take() {
            match request {
                Request::Position(fraction) => {
                    self.target = fraction;
                    self.motion = SettleMotion::Direct;
                }
                Request::Settle(velocity) => {
                    let fraction = if delta != 0. {
                        self.target
                    } else {
                        self.metrics.fraction
                    };
                    let projection = velocity as f64 * behavior.fling.projection.as_secs_f64();
                    let projected = f64::from(fraction) + projection / f64::from(limit.max(1.));
                    self.target = f32::from(projected >= 0.5);
                    self.motion = if projection == 0. {
                        SettleMotion::Snap
                    } else {
                        SettleMotion::Fling
                    };
                }
            }
            self.collapsed = self.target >= 0.5;
        }
        ScrollPosition {
            target: self.target,
            motion: self.motion,
            expanded: !self.collapsed,
            content,
        }
    }

    pub(super) fn display(
        &mut self,
        position: ScrollPosition,
        fraction: f32,
        limit: f32,
        exposed: bool,
    ) -> bool {
        let metrics = Metrics {
            content: position.content,
            offset: if exposed { 0. } else { fraction * limit },
            limit,
            fraction,
        };
        let changed = self.metrics != metrics;
        self.metrics = metrics;
        changed
    }
}

pub(super) fn length(value: Pixels) -> Option<f32> {
    let value = f32::from(value);
    (value.is_finite() && value >= 0.).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directional_thresholds_accumulate_short_deltas() {
        let mut state = ToolbarScrollState::new();
        let behavior = ToolbarScrollBehavior::default();
        for (offset, expanded) in [
            (0., true),
            (39., true),
            (40., false),
            (110., false),
            (72., false),
            (70., true),
        ] {
            assert_eq!(
                state.prepare(Some(px(offset)), 0., behavior).expanded,
                expanded
            );
        }
    }

    #[test]
    fn new_input_interrupts_a_fling_from_painted_position() {
        let mut state = ToolbarScrollState::new();
        let behavior = ToolbarScrollBehavior {
            exit_direction: Some(ToolbarExitDirection::Bottom),
            ..Default::default()
        };
        state.prepare(Some(px(0.)), 100., behavior);
        let partial = state.prepare(Some(px(30.)), 100., behavior);
        assert_eq!(partial.target, 0.3);
        state.display(partial, 0.3, 100., false);
        state.request = Some(Request::Settle(300.));
        let settling = state.prepare(Some(px(30.)), 100., behavior);
        assert_eq!(settling.target, 1.);
        assert!(matches!(settling.motion, SettleMotion::Fling));
        state.display(settling, 0.5, 100., false);
        let interrupted = state.prepare(Some(px(25.)), 100., behavior);
        assert_eq!(interrupted.target, 0.45);
        assert!(matches!(interrupted.motion, SettleMotion::Direct));
    }
}
