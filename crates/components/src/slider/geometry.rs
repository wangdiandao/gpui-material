use gpui::{Along, Axis, Bounds, Pixels, Point, Size, Styled, point, px, relative};
use gpui_base::AxisExt;
use gpui_material_foundation::RangeThumb;
use gpui_material_theme::Direction;

#[derive(Clone, Copy)]
pub(super) struct Geometry {
    pub(super) axis: Axis,
    pub(super) direction: Direction,
    pub(super) reverse_vertical: bool,
}

impl Geometry {
    pub(super) fn reversed(self) -> bool {
        if self.axis.is_horizontal() {
            self.direction == Direction::Rtl
        } else {
            self.reverse_vertical
        }
    }

    pub(super) fn position(self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> Point<Pixels> {
        if !self.reversed() {
            pointer
        } else if self.axis.is_horizontal() {
            point(bounds.left() + (bounds.right() - pointer.x), pointer.y)
        } else {
            point(pointer.x, bounds.top() + (bounds.bottom() - pointer.y))
        }
    }

    pub(super) fn fraction(self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> f32 {
        let pointer = self.position(bounds, pointer);
        if self.axis.is_horizontal() {
            (pointer.x - bounds.left()) / bounds.size.width
        } else {
            (bounds.bottom() - pointer.y) / bounds.size.height
        }
    }

    pub(super) fn point_at(self, bounds: Bounds<Pixels>, offset: Pixels) -> Point<Pixels> {
        if self.axis.is_vertical() && self.reversed() {
            point(bounds.center().x, bounds.top() + offset)
        } else if self.axis.is_vertical() {
            point(bounds.center().x, bounds.bottom() - offset)
        } else if self.reversed() {
            point(bounds.right() - offset, bounds.center().y)
        } else {
            point(bounds.left() + offset, bounds.center().y)
        }
    }

    pub(super) fn track_corners(self, start: Pixels, end: Pixels) -> gpui::Corners<Pixels> {
        let (left, right) = if self.reversed() {
            (end, start)
        } else {
            (start, end)
        };
        gpui::Corners {
            top_left: if self.axis.is_vertical() { right } else { left },
            top_right: right,
            bottom_left: left,
            bottom_right: if self.axis.is_vertical() { left } else { right },
        }
    }

    pub(super) fn place(self, element: gpui::Div, position: f32) -> gpui::Div {
        if self.axis.is_vertical() && self.reversed() {
            element.top(relative(position))
        } else if self.axis.is_vertical() {
            element.bottom(relative(position))
        } else if self.reversed() {
            element.right(relative(position))
        } else {
            element.left(relative(position))
        }
    }

    pub(super) fn reversed_key_delta(self, key: &str) -> Option<f32> {
        if !self.reversed() {
            return None;
        }
        match (self.axis, key) {
            (Axis::Horizontal, "left") | (Axis::Vertical, "down") => Some(1.),
            (Axis::Horizontal, "right") | (Axis::Vertical, "up") => Some(-1.),
            _ => None,
        }
    }

    pub(super) fn choose_at(
        self,
        range: bool,
        values: std::ops::Range<f32>,
        handle_sizes: [Size<Pixels>; 2],
        bounds: Bounds<Pixels>,
        pointer: Point<Pixels>,
    ) -> RangeThumb {
        let fraction = self.fraction(bounds, pointer);
        if range && values.start != values.end {
            let hit = |position, handle: Size<Pixels>| {
                let center = self.point_at(bounds, bounds.size.along(self.axis) * position);
                let offset = pointer - center;
                let (along, across) = if self.axis.is_horizontal() {
                    (offset.x, offset.y)
                } else {
                    (offset.y, offset.x)
                };
                along.abs() <= handle.width / 2. && across.abs() <= handle.height / 2.
            };
            match (
                hit(values.start, handle_sizes[0]),
                hit(values.end, handle_sizes[1]),
            ) {
                (true, false) => return RangeThumb::Start,
                (false, true) => return RangeThumb::End,
                _ => {}
            }
        }
        choose_thumb(range, values.start, values.end, fraction)
    }
}

pub(super) fn dimension(value: Option<Pixels>, default: f32, zero_allowed: bool) -> Pixels {
    value
        .filter(|value| {
            f32::from(*value).is_finite() && (*value > px(0.) || zero_allowed && *value == px(0.))
        })
        .unwrap_or(px(default))
}

pub(super) fn value_label_origin(
    thumb: Bounds<Pixels>,
    label: Size<Pixels>,
    visible: Bounds<Pixels>,
    horizontal: bool,
    direction: Direction,
) -> Point<Pixels> {
    let gap = px(12.);
    let x = if horizontal {
        thumb.center().x - label.width / 2.
    } else if direction == Direction::Ltr && thumb.right() + gap + label.width <= visible.right()
        || thumb.left() - gap - label.width < visible.left()
    {
        thumb.right() + gap
    } else {
        thumb.left() - gap - label.width
    };
    let y = if horizontal && thumb.top() - gap - label.height >= visible.top() {
        thumb.top() - gap - label.height
    } else if horizontal {
        thumb.bottom() + gap
    } else {
        thumb.center().y - label.height / 2.
    };
    point(
        x.clamp(
            visible.left(),
            (visible.right() - label.width).max(visible.left()),
        ),
        y.clamp(
            visible.top(),
            (visible.bottom() - label.height).max(visible.top()),
        ),
    )
}

#[derive(Clone, Copy)]
pub(super) struct TrackSegment {
    pub(super) start: Pixels,
    pub(super) end: Pixels,
    pub(super) outer_start: bool,
    pub(super) outer_end: bool,
    pub(super) active: bool,
}

impl TrackSegment {
    pub(super) fn corners(self, outer: Pixels, inner: Pixels) -> (Pixels, Pixels) {
        let limit = (self.end - self.start).max(px(0.)) / 2.;
        let radius = |outside| if outside { outer } else { inner }.min(limit);
        (radius(self.outer_start), radius(self.outer_end))
    }

    pub(super) fn icon_position(
        self,
        side: Pixels,
        outer: Pixels,
        inner: Pixels,
    ) -> Option<(Pixels, bool)> {
        let (start, end) = self.corners(outer, inner);
        let start = start.max(px(4.));
        let end = end.max(px(4.));
        (self.end - self.start >= start + side + end)
            .then_some((self.start + start + side / 2., self.active))
    }
}

pub(super) fn track_segments(
    length: Pixels,
    (first, first_gap): (f32, Pixels),
    (last, last_gap): (f32, Pixels),
) -> [TrackSegment; 3] {
    let first = length * first;
    let last = length * last;
    [
        TrackSegment {
            start: px(0.),
            end: (first - first_gap).min(last - last_gap),
            outer_start: true,
            outer_end: false,
            active: false,
        },
        TrackSegment {
            start: first + first_gap,
            end: last - last_gap,
            outer_start: first == px(0.) && first_gap == px(0.),
            outer_end: last == length && last_gap == px(0.),
            active: true,
        },
        TrackSegment {
            start: (last + last_gap).max(first + first_gap),
            end: length,
            outer_start: false,
            outer_end: true,
            active: false,
        },
    ]
}

pub(super) fn choose_thumb(range: bool, start: f32, end: f32, fraction: f32) -> RangeThumb {
    if range && fraction < (start + end) / 2. {
        RangeThumb::Start
    } else {
        RangeThumb::End
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_labels_keep_material_gap_and_flip_inside_the_viewport() {
        let visible = Bounds::new(point(px(0.), px(0.)), gpui::size(px(200.), px(200.)));
        let label = gpui::size(px(48.), px(44.));
        let thumb = Bounds::new(point(px(98.), px(100.)), gpui::size(px(4.), px(44.)));
        let above = value_label_origin(thumb, label, visible, true, Direction::Ltr);
        assert_eq!(above, point(px(76.), px(44.)));
        assert_eq!(thumb.top() - above.y - label.height, px(12.));

        let top = Bounds::new(point(px(0.), px(0.)), thumb.size);
        let below = value_label_origin(top, label, visible, true, Direction::Rtl);
        assert_eq!(below, point(px(0.), px(56.)));
        let vertical = value_label_origin(thumb, label, visible, false, Direction::Rtl);
        assert_eq!(vertical.x + label.width + px(12.), thumb.left());
    }

    #[test]
    fn track_segments_stop_the_same_distance_from_each_handle() {
        for percentage in [0.1, 0.5, 0.9] {
            let length = px(200.);
            let center = length * percentage;
            let [_, active, inactive] = track_segments(length, (0., px(0.)), (percentage, px(8.)));
            assert_eq!(center - active.end, px(8.));
            assert_eq!(inactive.start - center, px(8.));
            assert_eq!(active.corners(px(12.), px(2.)).1, px(2.));
            assert_eq!(inactive.corners(px(12.), px(2.)).0, px(2.));
        }

        let [before, active, after] = track_segments(px(200.), (0.5, px(7.)), (0.5, px(8.)));
        assert_eq!(before.end, px(92.));
        assert_eq!(after.start, px(108.));
        assert!(active.end < active.start);
    }
}
