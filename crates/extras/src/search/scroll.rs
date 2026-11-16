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

    pub fn hidden_fraction(&self) -> f32 {
        if self.limit > px(0.) {
            self.hidden / self.limit
        } else {
            0.
        }
    }

    pub fn set_hidden_fraction(&mut self, fraction: f32, cx: &mut Context<Self>) {
        if fraction.is_finite() {
            self.hidden = self.limit * fraction.clamp(0., 1.);
            cx.notify();
        }
    }

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
        let delta = offset - self.last.replace(offset).unwrap_or(px(0.));
        self.limit = height;
        self.hidden = if offset == px(0.) && delta < px(0.) {
            px(0.)
        } else {
            (self.hidden + delta).clamp(px(0.), height)
        };
        if held_open { px(0.) } else { self.hidden }
    }
}
