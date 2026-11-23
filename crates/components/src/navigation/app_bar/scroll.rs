use gpui::{Context, Pixels, px};

/// How changes in the linked content offset affect the app bar's height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarScrollBehavior {
    /// Keeps the bar at its expanded height while tracking the scrolled color.
    #[default]
    Pinned,
    /// Collapses on forward scrolling and expands as soon as scrolling reverses.
    EnterAlways,
    /// Collapses on forward scrolling and expands when the content reaches the top.
    ExitUntilCollapsed,
}

#[derive(Clone, Copy, PartialEq)]
enum Request {
    Fraction(f32),
    Settle(f32),
}

/// Retained app bar presentation state. Use a distinct entity per app bar.
#[derive(Default)]
pub struct AppBarScrollState {
    supplied_offset: f32,
    last_offset: Option<f32>,
    last_strategy: Option<AppBarScrollBehavior>,
    target: f32,
    settle: bool,
    request: Option<Request>,
    metrics: Metrics,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Metrics {
    content: f32,
    height: f32,
    limit: f32,
    overlapped: f32,
}

#[derive(Clone, Copy)]
pub(in crate::navigation) struct ScrollPosition {
    pub target: f32,
    pub settling: bool,
    pub overlap: f32,
    content: f32,
    limit: f32,
}

impl AppBarScrollState {
    /// Creates an app bar scroll state initialized at zero scroll offset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Supplies a nonnegative distance from the top for an externally scrolled view.
    /// A linked GPUI ScrollHandle takes precedence when present.
    pub fn set_content_offset(&mut self, offset: Pixels, cx: &mut Context<Self>) {
        let offset = length(offset).unwrap_or(0.);
        if self.supplied_offset != offset {
            self.supplied_offset = offset;
            cx.notify();
        }
    }

    /// Sets the presentation fraction immediately, provided the bar can collapse.
    pub fn set_collapsed_fraction(&mut self, fraction: f32, cx: &mut Context<Self>) {
        if fraction.is_finite() {
            let clamped = fraction.clamp(0., 1.);
            let next = Some(Request::Fraction(clamped));
            if self.request == next
                || (self.request.is_none()
                    && !self.settle
                    && self.target == clamped
                    && self.collapsed_fraction() == clamped)
            {
                return;
            }
            self.request = next;
            cx.notify();
        }
    }

    /// Settles at an endpoint of the current range. Velocity is in logical pixels
    /// per second, positive away from the top. A zero velocity uses the nearest endpoint.
    pub fn settle(&mut self, velocity: f32, cx: &mut Context<Self>) {
        let next = Some(Request::Settle(if velocity.is_finite() {
            velocity
        } else {
            0.
        }));
        if self.request != next {
            self.request = next;
            cx.notify();
        }
    }

    /// Returns the last observed content scroll distance from the top.
    pub fn content_offset(&self) -> Pixels {
        px(self.metrics.content)
    }

    /// Current displayed height offset. Expanded is zero, and collapse is negative.
    pub fn height_offset(&self) -> Pixels {
        px(self.metrics.height)
    }

    /// Returns the maximum negative height offset (zero when the bar cannot collapse).
    pub fn height_offset_limit(&self) -> Pixels {
        px(self.metrics.limit)
    }

    /// Returns the current collapse progress in `0.0..=1.0`.
    pub fn collapsed_fraction(&self) -> f32 {
        if self.metrics.limit < 0. {
            self.metrics.height / self.metrics.limit
        } else {
            0.
        }
    }

    /// Returns the fraction of scrolled-under overlap in `0.0..=1.0`, used for scrolled container color.
    pub fn overlapped_fraction(&self) -> f32 {
        self.metrics.overlapped
    }

    pub(in crate::navigation) fn prepare(
        &mut self,
        handle_offset: Option<f32>,
        limit: f32,
        strategy: Option<AppBarScrollBehavior>,
        enabled: bool,
    ) -> ScrollPosition {
        let content = handle_offset.unwrap_or(self.supplied_offset);
        let previous = self.last_offset.replace(content);
        let change = content - previous.unwrap_or(0.);
        let strategy_changed = self.last_strategy != strategy;
        self.last_strategy = strategy;
        let pinned = strategy == Some(AppBarScrollBehavior::Pinned) || limit == 0.;
        if pinned {
            self.target = 0.;
            self.settle = false;
        } else if enabled && (change != 0. || previous.is_none() || strategy_changed) {
            let fraction = if self.settle {
                self.collapsed_fraction()
            } else {
                self.target
            };
            let distance = fraction * limit;
            let next = match strategy {
                None => content,
                Some(AppBarScrollBehavior::EnterAlways) => distance + change,
                Some(AppBarScrollBehavior::ExitUntilCollapsed) => {
                    if content == 0. {
                        0.
                    } else {
                        distance + change.max(0.)
                    }
                }
                Some(AppBarScrollBehavior::Pinned) => 0.,
            };
            self.target = (next / limit).clamp(0., 1.);
            self.settle = false;
        }
        if let Some(request) = self.request.take().filter(|_| !pinned) {
            match request {
                Request::Fraction(fraction) => {
                    self.target = fraction;
                    self.settle = false;
                }
                Request::Settle(velocity) => {
                    let fraction = if self.settle {
                        self.collapsed_fraction()
                    } else {
                        self.target
                    };
                    let keep_closed = strategy == Some(AppBarScrollBehavior::ExitUntilCollapsed)
                        && content > 0.
                        && fraction > 0.;
                    let collapse =
                        keep_closed || velocity > 0. || velocity == 0. && fraction >= 0.5;
                    self.target = f32::from(collapse);
                    self.settle = true;
                }
            }
        }
        let content = if enabled {
            content
        } else {
            self.metrics.content
        };
        ScrollPosition {
            target: self.target,
            settling: self.settle,
            overlap: (content / limit.max(24.)).clamp(0., 1.),
            content,
            limit,
        }
    }

    pub(in crate::navigation) fn display(
        &mut self,
        position: ScrollPosition,
        fraction: f32,
    ) -> bool {
        let next = Metrics {
            content: position.content,
            height: -position.limit * fraction,
            limit: -position.limit,
            overlapped: position.overlap,
        };
        let changed = self.metrics != next;
        self.metrics = next;
        changed
    }
}

pub(in crate::navigation) fn length(value: Pixels) -> Option<f32> {
    let value = f32::from(value);
    (value.is_finite() && value >= 0.).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_always_resumes_from_the_displayed_height_when_scroll_interrupts_settling() {
        let mut state = AppBarScrollState::new();
        let strategy = Some(AppBarScrollBehavior::EnterAlways);
        let partial = state.prepare(Some(30.), 100., strategy, true);
        assert_eq!(partial.target, 0.3);
        state.display(partial, 0.3);

        state.request = Some(Request::Settle(200.));
        let settling = state.prepare(Some(30.), 100., strategy, true);
        assert!(settling.settling);
        assert_eq!(settling.target, 1.);
        state.display(settling, 0.5);

        let resumed = state.prepare(Some(35.), 100., strategy, true);
        assert!(!resumed.settling);
        assert_eq!(resumed.target, 0.55);
    }
}
