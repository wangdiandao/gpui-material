use gpui::{Context, Pixels, px};

/// Retained Enter Always behavior for a standalone search bar.
pub struct SearchScrollState {
    supplied: Pixels,
    last: Option<Pixels>,
    hidden: Pixels,
    limit: Pixels,
}

impl Default for SearchScrollState {
    fn default() -> Self {
        Self {
            supplied: px(0.),
            last: None,
            hidden: px(0.),
            limit: px(56.),
        }
    }
}

impl SearchScrollState {
    /// Creates a scroll state starting fully visible (`hidden_fraction() == 0.0`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Supplies a positive distance from the content's top. A linked handle takes precedence.
    pub fn set_content_offset(&mut self, offset: Pixels, cx: &mut Context<Self>) {
        let offset = super::presentation::length(offset);
        if self.supplied != offset {
            self.supplied = offset;
            cx.notify();
        }
    }

    /// Returns the fraction of the search bar currently hidden (`0.0` visible to `1.0` hidden).
    pub fn hidden_fraction(&self) -> f32 {
        if self.limit > px(0.) {
            self.hidden / self.limit
        } else {
            0.
        }
    }

    /// Sets the hidden fraction directly, clamped to `0.0..=1.0`.
    pub fn set_hidden_fraction(&mut self, fraction: f32, cx: &mut Context<Self>) {
        if fraction.is_finite() {
            let next = self.limit * fraction.clamp(0., 1.);
            if self.hidden != next {
                self.hidden = next;
                cx.notify();
            }
        }
    }

    /// Snaps the bar either fully visible or fully hidden based on scroll `velocity` or midpoint.
    pub fn settle(&mut self, velocity: Pixels, cx: &mut Context<Self>) {
        let hide = if velocity == px(0.) || !f32::from(velocity).is_finite() {
            self.hidden_fraction() >= 0.5
        } else {
            velocity > px(0.)
        };
        self.set_hidden_fraction(f32::from(hide), cx);
    }

    pub(super) fn measure(
        &mut self,
        linked: Option<Pixels>,
        height: Pixels,
        held_open: bool,
    ) -> Pixels {
        let offset = linked.map_or(self.supplied, super::presentation::length);
        let delta = match self.last.replace(offset) {
            Some(previous) => offset - previous,
            None => px(0.),
        };
        self.limit = height;
        self.hidden = if offset == px(0.) && delta < px(0.) {
            px(0.)
        } else {
            (self.hidden + delta).clamp(px(0.), height)
        };
        if held_open { px(0.) } else { self.hidden }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_non_zero_offset_does_not_spuriously_hide_on_first_frame() {
        let mut state = SearchScrollState::new();
        assert_eq!(state.measure(Some(px(240.)), px(56.), false), px(0.));
        assert_eq!(state.hidden_fraction(), 0.);
        // Subsequent forward scroll hides by the delta from the initial baseline.
        assert_eq!(state.measure(Some(px(268.)), px(56.), false), px(28.));
        assert_eq!(state.hidden_fraction(), 0.5);
        // Scrolling back reveals the bar.
        assert_eq!(state.measure(Some(px(254.)), px(56.), false), px(14.));
    }
}
