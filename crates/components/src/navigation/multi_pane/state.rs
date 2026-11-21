use std::{ops::Index, time::Duration};

use gpui::{Bounds, Context, ElementId, Pixels, point, px, size};
use web_time::Instant;

use super::{PaneExpansionAnchor, PaneOverlayAlignment};

/// Roles in a Material three-pane scaffold.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneRole {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

impl PaneRole {
    pub(super) const ALL: [Self; 3] = [Self::Primary, Self::Secondary, Self::Tertiary];

    pub(super) fn index(self) -> usize {
        match self {
            Self::Primary => 0,
            Self::Secondary => 1,
            Self::Tertiary => 2,
        }
    }
}

/// The strategy used when resolving a pane from navigation and the directive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneAdaptStrategy {
    #[default]
    Hide,
    Reflow(PaneRole),
    Levitate(PaneOverlayAlignment),
}

/// A pane's realized presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PanePlacement {
    #[default]
    Hidden,
    Expanded,
    Reflow(PaneRole),
    Levitate(PaneOverlayAlignment),
}

/// Presentation of the three roles, independent of their navigation content keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaneLayoutValue([PanePlacement; 3]);

impl PaneLayoutValue {
    pub fn new(primary: PanePlacement, secondary: PanePlacement, tertiary: PanePlacement) -> Self {
        Self([primary, secondary, tertiary])
    }

    pub fn placement(self, role: PaneRole) -> PanePlacement {
        self[role]
    }

    fn set(&mut self, role: PaneRole, placement: PanePlacement) {
        self.0[role.index()] = placement;
    }
}

impl Index<PaneRole> for PaneLayoutValue {
    type Output = PanePlacement;

    fn index(&self, role: PaneRole) -> &Self::Output {
        &self.0[role.index()]
    }
}

/// A history entry with an optional stable domain content key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneDestination {
    pub role: PaneRole,
    pub content_key: Option<ElementId>,
}

impl PaneDestination {
    pub fn new(role: PaneRole) -> Self {
        Self {
            role,
            content_key: None,
        }
    }

    pub fn content_key(mut self, key: impl Into<ElementId>) -> Self {
        self.content_key = Some(key.into());
        self
    }
}

/// How many destinations a back operation can skip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneBackBehavior {
    PopLatest,
    #[default]
    PopUntilScaffoldValueChange,
    PopUntilCurrentDestinationChange,
    PopUntilContentChange,
}

#[derive(Clone, Copy)]
pub(super) struct Adaptation {
    pub horizontal: usize,
    pub vertical: usize,
    pub strategies: [Option<PaneAdaptStrategy>; 3],
}

impl Default for Adaptation {
    fn default() -> Self {
        Self {
            horizontal: 1,
            vertical: 1,
            strategies: [Some(PaneAdaptStrategy::Hide); 3],
        }
    }
}

impl Adaptation {
    pub fn resolve(self, history: &[PaneDestination], aware: bool) -> PaneLayoutValue {
        let mut value = PaneLayoutValue::default();
        let horizontal = self.horizontal.clamp(1, 3);
        let vertical = self.vertical.clamp(1, 3);
        let active = history.last().map(|destination| destination.role);
        let prior = history
            .iter()
            .rev()
            .take(if aware { history.len() } else { 1 });
        if let Some(role) = active
            && let Some(PaneAdaptStrategy::Levitate(alignment)) = self.strategies[role.index()]
        {
            value.set(role, PanePlacement::Levitate(alignment));
        }
        let mut expanded = 0;
        let mut reflowed = 0;
        let mut seen = [false; 3];
        let mut seen_count = 0;
        for role in prior
            .map(|destination| destination.role)
            .chain(PaneRole::ALL)
        {
            if !std::mem::replace(&mut seen[role.index()], true) {
                seen_count += 1;
            } else {
                continue;
            }
            if value[role] != PanePlacement::Hidden {
                if seen_count == 3 {
                    break;
                }
                continue;
            }
            match self.strategies[role.index()] {
                Some(PaneAdaptStrategy::Reflow(anchor))
                    if horizontal == 1
                        && reflowed + 1 < vertical
                        && anchor != role
                        && self.strategies[anchor.index()].is_some_and(|strategy| {
                            !matches!(strategy, PaneAdaptStrategy::Levitate(_))
                        })
                        && (value[anchor] == PanePlacement::Expanded || expanded == 0) =>
                {
                    value.set(anchor, PanePlacement::Expanded);
                    value.set(role, PanePlacement::Reflow(anchor));
                    expanded = 1;
                    reflowed += 1;
                }
                Some(PaneAdaptStrategy::Hide | PaneAdaptStrategy::Reflow(_))
                    if expanded < horizontal =>
                {
                    value.set(role, PanePlacement::Expanded);
                    expanded += 1;
                }
                _ => {}
            }
            if seen_count == 3 {
                break;
            }
        }
        value
    }

    fn normalize(self, value: PaneLayoutValue) -> PaneLayoutValue {
        let mut safe = value;
        for role in PaneRole::ALL {
            if self.strategies[role.index()].is_none()
                || matches!(value[role], PanePlacement::Reflow(anchor)
                    if self.strategies[anchor.index()].is_none() || value[anchor] != PanePlacement::Expanded)
            {
                safe.set(role, PanePlacement::Hidden);
            }
        }
        safe
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Intent {
    Navigate,
    Animate(PaneLayoutValue),
    Snap(PaneLayoutValue),
    Seek(PaneLayoutValue, f32),
    SeekBack(PaneBackBehavior, f32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Timing {
    Animate,
    Snap,
    Seek,
}

pub(super) type Frames = [Option<Bounds<Pixels>>; 3];

#[derive(Clone, Copy)]
struct Motion {
    from: Frames,
    started: Instant,
    timing: Timing,
}

pub(super) struct DisplayedPanes {
    pub frames: Frames,
    pub animate_next_frame: bool,
}

/// Retained presentation, expansion and navigation state for a single adaptive layout.
pub struct AdaptivePaneState {
    history: Vec<PaneDestination>,
    history_aware: bool,
    adaptation: Adaptation,
    expansion: [Option<PaneExpansionAnchor>; 9],
    current: PaneLayoutValue,
    target: PaneLayoutValue,
    progress: f32,
    intent: Intent,
    motion: Option<Motion>,
    displayed: Frames,
    measured: bool,
}

impl AdaptivePaneState {
    pub fn new(initial: PaneDestination) -> Self {
        let mut value = PaneLayoutValue::default();
        value.set(initial.role, PanePlacement::Expanded);
        Self {
            history: vec![initial],
            history_aware: true,
            adaptation: Adaptation::default(),
            expansion: [None; 9],
            current: value,
            target: value,
            progress: 0.,
            intent: Intent::Navigate,
            motion: None,
            displayed: [None; 3],
            measured: false,
        }
    }

    pub fn current_destination(&self) -> &PaneDestination {
        self.history
            .last()
            .expect("initial destination is retained")
    }

    pub fn history(&self) -> &[PaneDestination] {
        &self.history
    }

    pub fn set_history_aware(&mut self, aware: bool, cx: &mut Context<Self>) {
        if self.history_aware != aware {
            self.history_aware = aware;
            cx.notify();
        }
    }

    pub fn navigate_to(&mut self, destination: PaneDestination, cx: &mut Context<Self>) {
        if self.current_destination() != &destination {
            self.history.push(destination);
            cx.notify();
        }
        self.follow_navigation(cx);
    }

    pub fn can_navigate_back(&self, behavior: PaneBackBehavior) -> bool {
        self.back_index(behavior).is_some()
    }

    pub fn previous_value(&self, behavior: PaneBackBehavior) -> Option<PaneLayoutValue> {
        self.back_index(behavior).map(|ix| self.history_value(ix))
    }

    pub fn navigate_back(&mut self, behavior: PaneBackBehavior, cx: &mut Context<Self>) -> bool {
        let Some(ix) = self.back_index(behavior) else {
            return false;
        };
        self.history.truncate(ix + 1);
        self.intent = Intent::Navigate;
        cx.notify();
        true
    }

    /// Previews a back transition without modifying history.
    pub fn seek_back(
        &mut self,
        behavior: PaneBackBehavior,
        progress: f32,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.can_navigate_back(behavior) {
            return false;
        }
        self.request(Intent::SeekBack(behavior, fraction(progress)), cx);
        true
    }

    pub fn current_value(&self) -> PaneLayoutValue {
        self.current
    }

    pub fn target_value(&self) -> PaneLayoutValue {
        self.target
    }

    /// Transition progress, or zero once the layout settles.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn snap_to(&mut self, value: PaneLayoutValue, cx: &mut Context<Self>) {
        self.request(Intent::Snap(value), cx);
    }

    pub fn animate_to(&mut self, value: PaneLayoutValue, cx: &mut Context<Self>) {
        self.request(Intent::Animate(value), cx);
    }

    pub fn seek_to(&mut self, value: PaneLayoutValue, progress: f32, cx: &mut Context<Self>) {
        self.request(Intent::Seek(value, fraction(progress)), cx);
    }

    pub fn follow_navigation(&mut self, cx: &mut Context<Self>) {
        self.request(Intent::Navigate, cx);
    }

    pub fn expansion(&self, start: PaneRole, end: PaneRole) -> Option<PaneExpansionAnchor> {
        self.expansion[start.index() * 3 + end.index()]
    }

    /// Retains a logical boundary specification, applied when this pair shares a band.
    pub fn set_expansion(
        &mut self,
        start: PaneRole,
        end: PaneRole,
        anchor: PaneExpansionAnchor,
        cx: &mut Context<Self>,
    ) {
        if start != end && anchor.position(px(1.)).is_some() {
            self.replace_expansion(start, end, Some(anchor), cx);
        }
    }

    pub fn clear_expansion(&mut self, start: PaneRole, end: PaneRole, cx: &mut Context<Self>) {
        self.replace_expansion(start, end, None, cx);
    }

    fn replace_expansion(
        &mut self,
        start: PaneRole,
        end: PaneRole,
        value: Option<PaneExpansionAnchor>,
        cx: &mut Context<Self>,
    ) {
        let entry = &mut self.expansion[start.index() * 3 + end.index()];
        if *entry != value {
            *entry = value;
            cx.notify();
        }
    }

    fn request(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.intent != intent {
            self.intent = intent;
            cx.notify();
        }
    }

    fn history_value(&self, end: usize) -> PaneLayoutValue {
        self.adaptation
            .resolve(&self.history[..=end], self.history_aware)
    }

    fn back_index(&self, behavior: PaneBackBehavior) -> Option<usize> {
        let last = self.history.len() - 1;
        let current = self.current_destination();
        let value = self.history_value(last);
        (0..last).rev().find(|&index| {
            let prior = &self.history[index];
            match behavior {
                PaneBackBehavior::PopLatest => true,
                PaneBackBehavior::PopUntilScaffoldValueChange => self.history_value(index) != value,
                PaneBackBehavior::PopUntilCurrentDestinationChange => prior.role != current.role,
                PaneBackBehavior::PopUntilContentChange => {
                    prior.content_key != current.content_key || self.history_value(index) != value
                }
            }
        })
    }

    pub(super) fn project(&mut self, adaptation: Adaptation) -> PaneLayoutValue {
        self.adaptation = adaptation;
        match self.intent {
            Intent::Navigate => self.history_value(self.history.len() - 1),
            Intent::Animate(value) | Intent::Snap(value) | Intent::Seek(value, _) => {
                adaptation.normalize(value)
            }
            Intent::SeekBack(behavior, _) => self
                .previous_value(behavior)
                .unwrap_or_else(|| self.history_value(self.history.len() - 1)),
        }
    }

    pub(super) fn sample(
        &mut self,
        value: PaneLayoutValue,
        target: Frames,
        now: Instant,
        duration: Duration,
        cx: &mut Context<Self>,
    ) -> DisplayedPanes {
        let before = (self.current, self.target, self.progress);
        let (timing, sought) = match self.intent {
            Intent::Snap(_) => (Timing::Snap, 0.),
            Intent::Seek(_, fraction) | Intent::SeekBack(_, fraction) => (Timing::Seek, fraction),
            _ => (Timing::Animate, 0.),
        };
        let unsettled = timing == Timing::Seek || duration > Duration::ZERO;
        let changed = self.target != value;
        self.target = value;
        if !self.measured || timing == Timing::Snap || !unsettled {
            self.motion = None;
            self.current = value;
            self.progress = 0.;
            self.displayed = target;
            self.measured = true;
        } else if changed || self.motion.is_some_and(|motion| motion.timing != timing) {
            self.motion = Some(Motion {
                from: self.displayed,
                started: now,
                timing,
            });
        }
        let mut animate_next_frame = false;
        if let Some(motion) = self.motion {
            let raw = if timing == Timing::Seek {
                sought
            } else {
                ((now - motion.started).as_secs_f32() / duration.as_secs_f32()).clamp(0., 1.)
            };
            if raw >= 1. && timing == Timing::Animate {
                self.motion = None;
                self.current = value;
                self.progress = 0.;
                self.displayed = target;
            } else {
                self.progress = raw;
                let visual = if timing == Timing::Seek {
                    raw
                } else {
                    raw * raw * (3. - 2. * raw)
                };
                self.displayed = std::array::from_fn(|ix| {
                    target[ix].map(|end| blend(motion.from[ix], end, visual))
                });
                animate_next_frame = timing == Timing::Animate;
            }
        } else {
            self.displayed = target;
        }
        if before != (self.current, self.target, self.progress) {
            let state = cx.weak_entity();
            cx.defer(move |cx| {
                let _ = state.update(cx, |_, cx| cx.notify());
            });
        }
        DisplayedPanes {
            frames: self.displayed,
            animate_next_frame,
        }
    }

    pub(super) fn is_transitioning(&self) -> bool {
        self.motion.is_some()
    }
}

fn fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    }
}

fn blend(from: Option<Bounds<Pixels>>, to: Bounds<Pixels>, fraction: f32) -> Bounds<Pixels> {
    let from = from.unwrap_or_else(|| Bounds::new(to.center(), size(px(0.), px(0.))));
    let interpolate = |a, b| a + (b - a) * fraction;
    Bounds::new(
        point(
            interpolate(from.left(), to.left()),
            interpolate(from.top(), to.top()),
        ),
        size(
            interpolate(from.size.width, to.size.width),
            interpolate(from.size.height, to.size.height),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptation_resolve_short_circuits_duplicate_history_entries() {
        let adaptation = Adaptation {
            horizontal: 2,
            vertical: 1,
            strategies: [
                Some(PaneAdaptStrategy::Hide),
                Some(PaneAdaptStrategy::Hide),
                Some(PaneAdaptStrategy::Hide),
            ],
        };
        let mut history = vec![PaneDestination::new(PaneRole::Tertiary)];
        for ix in 0..200 {
            let role = if ix % 2 == 0 {
                PaneRole::Primary
            } else {
                PaneRole::Secondary
            };
            history.push(PaneDestination::new(role).content_key(ix));
        }
        let resolved = adaptation.resolve(&history, true);
        assert_eq!(
            resolved,
            PaneLayoutValue::new(
                PanePlacement::Expanded,
                PanePlacement::Expanded,
                PanePlacement::Hidden,
            )
        );
    }
}
