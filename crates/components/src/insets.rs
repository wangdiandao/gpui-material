use gpui::{Edges, Pixels, px};
use gpui_material_theme::Direction;

/// Logical edges in pixels. Start and end follow the window direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScaffoldInsets {
    pub top: Pixels,
    pub start: Pixels,
    pub bottom: Pixels,
    pub end: Pixels,
}

impl ScaffoldInsets {
    pub fn resolve(self, direction: Direction) -> Edges<Pixels> {
        let (left, right) = if direction == Direction::Rtl {
            (self.end, self.start)
        } else {
            (self.start, self.end)
        };
        Edges {
            top: self.top,
            left,
            bottom: self.bottom,
            right,
        }
    }

    #[cfg_attr(
        not(any(feature = "feedback", feature = "navigation")),
        allow(dead_code)
    )]
    pub(crate) fn normalized(self) -> Self {
        let finite = |edge: Pixels| {
            if f32::from(edge).is_finite() {
                edge.max(px(0.))
            } else {
                px(0.)
            }
        };
        Self {
            top: finite(self.top),
            start: finite(self.start),
            bottom: finite(self.bottom),
            end: finite(self.end),
        }
    }
}
