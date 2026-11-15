use gpui_material_theme::Direction;

use super::CarouselVariant;

const PADDING: f32 = 16.;
const GAP: f32 = 8.;
const FULL_SCREEN_GAP: f32 = 16.;
const MIN_SMALL: f32 = 40.;
const MAX_SMALL: f32 = 56.;

/// A pose uses logical coordinates so keyboard order and RTL are independent
/// of the strategy. Item geometry is interpolated between adjacent snap poses.
pub(super) struct Layout {
    viewport: f32,
    cross: f32,
    variant: CarouselVariant,
    leading: f32,
    trailing: f32,
    gap: f32,
    sizes: Vec<f32>,
    prefix: Vec<f32>,
    preferred: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ItemBounds {
    pub(super) start: f32,
    pub(super) size: f32,
}

impl Layout {
    pub(super) fn new(
        variant: CarouselVariant,
        viewport: f32,
        cross: f32,
        preferred: f32,
        aspects: impl IntoIterator<Item = f32>,
    ) -> Self {
        let vertical = variant.is_vertical();
        let uncontained = variant.is_uncontained();
        let leading = if vertical {
            0.
        } else {
            PADDING.min(viewport / 4.)
        };
        let trailing = if vertical || uncontained { 0. } else { leading };
        let available = (viewport - leading - trailing).max(0.);
        let gap = if vertical { FULL_SCREEN_GAP } else { GAP };
        let preferred = preferred.min(available);
        let sizes: Vec<_> = aspects
            .into_iter()
            .map(|aspect| match variant {
                CarouselVariant::FullScreen => viewport,
                CarouselVariant::UncontainedMultiAspectRatio => (cross * aspect).min(available),
                _ => preferred,
            })
            .collect();
        let mut cursor = 0.;
        let prefix = sizes
            .iter()
            .map(|&size| {
                let start = cursor;
                cursor += size + gap;
                start
            })
            .collect();
        Self {
            viewport,
            cross,
            variant,
            leading,
            trailing,
            gap,
            sizes,
            prefix,
            preferred,
        }
    }

    pub(super) fn cross_inset(&self) -> f32 {
        if self.variant.is_vertical() {
            0.
        } else {
            GAP.min(self.cross / 4.)
        }
    }

    pub(super) fn clip_range(&self) -> (f32, f32) {
        if self.variant.is_vertical() || self.variant.is_uncontained() {
            (0., self.viewport)
        } else {
            (self.leading, self.viewport - self.trailing)
        }
    }

    pub(super) fn step(&self) -> f32 {
        match self.variant {
            CarouselVariant::FullScreen => self.viewport + self.gap,
            CarouselVariant::Hero | CarouselVariant::CenteredHero => self.viewport - MIN_SMALL,
            _ => self.preferred + self.gap,
        }
        .max(1.)
    }

    /// Convert physical scroll to logical page progress. Uncontained mixed
    /// aspect ratios retain each item's actual pixel stride.
    pub(super) fn scroll_delta(&self, position: f32, delta: f32) -> f32 {
        if !self.variant.is_uncontained() || self.prefix.len() < 2 {
            return delta / self.step();
        }
        let base = (position.floor() as usize).min(self.prefix.len() - 1);
        let progress = position.fract();
        let maximum = self.maximum_offset();
        let start = self.prefix[base].min(maximum);
        let next = self.prefix[(base + 1).min(self.prefix.len() - 1)].min(maximum);
        // Painting interpolates clamped snap poses, including the shortened
        // final stride. Scroll conversion must use the same physical offsets.
        let base_offset = start + (next - start) * progress;
        let target = (base_offset + delta).clamp(0., maximum);
        if target == base_offset {
            return 0.;
        }
        let end = self
            .prefix
            .partition_point(|&offset| offset < target)
            .saturating_sub(1);
        let next = self.prefix[(end + 1).min(self.prefix.len() - 1)].min(maximum);
        let fraction = (target - self.prefix[end]) / (next - self.prefix[end]);
        end as f32 + fraction - position
    }

    fn maximum_offset(&self) -> f32 {
        (self.prefix.last().unwrap() + self.sizes.last().unwrap() - (self.viewport - self.leading))
            .max(0.)
    }

    pub(super) fn visible(&self, position: f32, direction: Direction) -> Vec<(usize, ItemBounds)> {
        if self.viewport <= 0. || self.sizes.is_empty() {
            return Vec::new();
        }
        let position = position.clamp(0., (self.sizes.len() - 1) as f32);
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;
        let fraction = position.fract();
        let before = self.pose(lower);
        let after = self.pose(upper);
        let (clip_start, clip_end) = self.clip_range();
        let mirror = !self.variant.is_vertical() && direction == Direction::Rtl;
        self.visible_range(&before, &after)
            .filter_map(|index| {
                let a = self.item(&before, index);
                let b = self.item(&after, index);
                let start = a.start + (b.start - a.start) * fraction;
                let size = a.size + (b.size - a.size) * fraction;
                (start + size > clip_start && start < clip_end && size > 0.).then_some((
                    index,
                    ItemBounds {
                        start: if mirror {
                            self.viewport - start - size
                        } else {
                            start
                        },
                        size,
                    },
                ))
            })
            .collect()
    }

    fn start_boundary(&self, pose: &Pose) -> f32 {
        match pose {
            Pose::Track { offset } => *offset,
            Pose::Contained { .. } => 0.,
        }
    }

    fn visible_range(&self, a: &Pose, b: &Pose) -> std::ops::Range<usize> {
        match (a, b) {
            (Pose::Track { .. }, Pose::Track { .. }) => {
                let start = self.start_boundary(a);
                let end = self.start_boundary(b) + self.viewport;
                let first = self
                    .prefix
                    .partition_point(|x| *x <= start)
                    .saturating_sub(1);
                let last = self.prefix.partition_point(|x| *x < end);
                first..last
            }
            (
                Pose::Contained { first, widths },
                Pose::Contained {
                    first: last,
                    widths: end_widths,
                },
            ) => {
                let start = first.saturating_sub(1);
                let end = (first + widths.len()).max(last + end_widths.len()) + 1;
                start..end.min(self.sizes.len())
            }
            _ => unreachable!(),
        }
    }

    fn item(&self, pose: &Pose, index: usize) -> ItemBounds {
        match pose {
            Pose::Track { offset } => ItemBounds {
                start: self.leading + self.prefix[index] - offset,
                size: self.sizes[index],
            },
            Pose::Contained { first, widths } => {
                // Small offscreen anchors let items enter the viewport continuously
                // while a visible keyline expands or contracts.
                let anchor = MIN_SMALL.min(self.viewport);
                let start = if index < *first {
                    self.leading - (*first - index) as f32 * (anchor + self.gap)
                } else {
                    let contained = (index - first).min(widths.len());
                    let end = self.leading
                        + widths[..contained].iter().sum::<f32>()
                        + contained as f32 * self.gap;
                    end + (index - first - contained) as f32 * (anchor + self.gap)
                };
                let size = index
                    .checked_sub(*first)
                    .and_then(|i| widths.get(i))
                    .copied()
                    .unwrap_or(anchor);
                ItemBounds { start, size }
            }
        }
    }

    fn pose(&self, index: usize) -> Pose {
        if self.variant.is_uncontained() || self.variant.is_vertical() {
            return Pose::Track {
                offset: self.prefix[index].min(self.maximum_offset()),
            };
        }
        let available = (self.viewport - self.leading - self.trailing).max(0.);
        let count = self.sizes.len();
        let mut widths = match self.variant {
            CarouselVariant::MultiBrowse => {
                multi_browse(available, self.preferred, self.gap, count)
            }
            CarouselVariant::Hero | CarouselVariant::CenteredHero => hero(
                available,
                self.gap,
                count,
                self.variant == CarouselVariant::CenteredHero,
            ),
            _ => unreachable!(),
        };
        let maximum = widths.iter().copied().fold(0., f32::max);
        let focal = widths
            .iter()
            .position(|&width| width == maximum)
            .unwrap_or(0);
        let first = index.saturating_sub(focal).min(count - widths.len());
        let actual_focal = index - first;
        if actual_focal >= focal {
            widths.rotate_right(actual_focal - focal);
        } else {
            widths.rotate_left(focal - actual_focal);
        }
        Pose::Contained { first, widths }
    }
}

enum Pose {
    Track { offset: f32 },
    Contained { first: usize, widths: Vec<f32> },
}

fn multi_browse(available: f32, preferred: f32, gap: f32, count: usize) -> Vec<f32> {
    if count == 1 || available <= 2. * MIN_SMALL + gap {
        return vec![available];
    }

    // Fit the Compose large / medium / small arrangement to the window.
    // It targets a small item one third of the large item's size and a medium
    // halfway between them, then lets the medium flex up to ten percent.
    let preferred = preferred.max(2. * MIN_SMALL);
    let small = (preferred / 3.).clamp(MIN_SMALL, MAX_SMALL);
    let medium = (preferred + small) / 2.;
    let largest = ((available / preferred).ceil() as usize).clamp(1, count - 1);
    let smallest =
        (((available - medium - MAX_SMALL) / preferred).floor() as usize).clamp(1, largest);
    let mut best: Option<(f32, Vec<f32>)> = None;
    let mut priority = 0.;
    for large_count in (smallest..=largest).rev() {
        for medium_count in [1, 0] {
            priority += 1.;
            if large_count + medium_count + 1 > count {
                continue;
            }
            let spaces = (large_count + medium_count) as f32 * gap;
            let remainder = available - spaces;
            let delta =
                remainder - (large_count as f32 * preferred + medium_count as f32 * medium + small);
            let small = (small + delta).clamp(MIN_SMALL, MAX_SMALL);
            let large_count_f = large_count as f32;
            let medium_count_f = medium_count as f32;
            let mut large = (remainder - (1. + medium_count_f / 2.) * small)
                / (large_count_f + medium_count_f / 2.);
            let mut medium = (large + small) / 2.;
            if medium_count == 1 {
                let adjustment =
                    ((preferred - large) * large_count_f).clamp(-0.1 * medium, 0.1 * medium);
                large += adjustment / large_count_f;
                medium -= adjustment;
            }
            if large <= small || (medium_count != 0 && (large <= medium || medium <= small)) {
                continue;
            }
            let cost = (preferred - large).abs() * priority;
            if best.as_ref().is_none_or(|(current, _)| cost < *current) {
                let mut widths = vec![large; large_count];
                widths.extend(std::iter::repeat_n(medium, medium_count));
                widths.push(small);
                best = Some((cost, widths));
            }
        }
    }
    best.map_or_else(|| vec![available], |(_, widths)| widths)
}

fn hero(available: f32, gap: f32, count: usize, centered: bool) -> Vec<f32> {
    let small_count = match (centered, count) {
        (_, 1) => 0,
        (true, 3..) => 2,
        _ => 1,
    };
    if small_count == 0 || available < small_count as f32 * (MIN_SMALL + gap) + MIN_SMALL * 1.25 {
        return vec![available];
    }
    let small = (available / 3.).clamp(MIN_SMALL, MAX_SMALL);
    let small = small.min((available - small_count as f32 * gap) / (small_count as f32 + 1.25));
    let large = available - small_count as f32 * (small + gap);
    if small_count == 2 {
        vec![small, large, small]
    } else {
        vec![large, small]
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [CarouselVariant; 6] = [
        CarouselVariant::MultiBrowse,
        CarouselVariant::Uncontained,
        CarouselVariant::UncontainedMultiAspectRatio,
        CarouselVariant::Hero,
        CarouselVariant::CenteredHero,
        CarouselVariant::FullScreen,
    ];

    fn layout(variant: CarouselVariant, viewport: f32, count: usize) -> Layout {
        Layout::new(
            variant,
            viewport,
            196.,
            162.,
            [1.5, 0.75, 1., 1.25, 1.5, 1.5]
                .into_iter()
                .cycle()
                .take(count),
        )
    }

    fn widths(layout: &Layout, point: f32) -> Vec<f32> {
        layout
            .visible(point, Direction::Ltr)
            .into_iter()
            .map(|(_, item)| item.size)
            .collect()
    }

    #[test]
    fn precise_scroll_matches_painted_offsets_at_the_trailing_edge() {
        for variant in [
            CarouselVariant::Uncontained,
            CarouselVariant::UncontainedMultiAspectRatio,
        ] {
            let geometry = layout(variant, 465., 6);
            let offset = |position: f32| {
                let (index, item) = geometry.visible(position, Direction::Ltr)[0];
                geometry.leading + geometry.prefix[index] - item.start
            };
            for position in [0., 2.5, 3.5, 4., 5.] {
                for delta in [-40., 0., 40.] {
                    let next = position + geometry.scroll_delta(position, delta);
                    let expected = (offset(position) + delta).clamp(0., geometry.maximum_offset());
                    assert!(
                        (offset(next) - expected).abs() < 0.001,
                        "{variant:?}: position={position}, delta={delta}, next={next}"
                    );
                    if delta == 0. {
                        assert_eq!(next, position);
                    }
                }
            }
        }
    }

    #[test]
    fn six_layouts_keep_their_distinct_material_geometry() {
        let multi_layout = layout(CarouselVariant::MultiBrowse, 360., 6);
        let Pose::Contained { widths: multi, .. } = multi_layout.pose(0) else {
            unreachable!()
        };
        assert_eq!(multi.len(), 3);
        assert!(multi[0] > multi[1] && multi[1] > multi[2]);
        assert!((MIN_SMALL..=MAX_SMALL).contains(&multi[2]));

        let regular = layout(CarouselVariant::Uncontained, 465., 6);
        assert!(widths(&regular, 0.).iter().all(|width| *width == 162.));
        assert!(
            regular
                .visible(0., Direction::Ltr)
                .last()
                .is_some_and(|(_, item)| item.start + item.size > 465.)
        );

        let mixed = widths(
            &layout(CarouselVariant::UncontainedMultiAspectRatio, 620., 6),
            0.,
        );
        assert!(mixed.len() >= 3);
        assert_eq!(mixed[..3], [294., 147., 196.]);

        let hero = layout(CarouselVariant::Hero, 460., 6);
        let hero_first = hero.visible(0., Direction::Ltr);
        assert_eq!(hero_first[0].1.start, PADDING);
        assert_eq!(hero_first[1].1.size, MAX_SMALL);

        let centered = layout(CarouselVariant::CenteredHero, 460., 6);
        let centered_rest = centered.visible(1., Direction::Ltr);
        assert_eq!(centered_rest.len(), 3);
        assert_eq!(centered_rest[0].1.size, centered_rest[2].1.size);
        let center = centered_rest[1].1;
        assert_eq!(center.start + center.size / 2., 230.);

        let full = layout(CarouselVariant::FullScreen, 320., 6);
        assert_eq!(
            full.visible(1., Direction::Ltr),
            vec![(
                1,
                ItemBounds {
                    start: 0.,
                    size: 320.
                }
            )]
        );
        assert_eq!(full.visible(0.5, Direction::Ltr).len(), 2);
    }

    #[test]
    fn layouts_fit_real_viewports_and_mirror_without_reordering() {
        for variant in ALL {
            for viewport in [0., 30., 90., 260., 430., 1120.] {
                for count in [0, 1, 2, 6, 120] {
                    let geometry = layout(variant, viewport, count);
                    for position in [0., 0.2, 0.5, 1., 2.4, (count.saturating_sub(1)) as f32] {
                        let ltr = geometry.visible(position, Direction::Ltr);
                        let rtl = geometry.visible(position, Direction::Rtl);
                        assert_eq!(ltr.len(), rtl.len());
                        for (left, right) in ltr.iter().zip(&rtl) {
                            assert_eq!(left.0, right.0);
                            assert_eq!(left.1.size, right.1.size);
                            assert!(left.1.size > 0. && left.1.size.is_finite());
                            assert!(left.1.start < viewport && left.1.start + left.1.size > 0.);
                            if variant.is_vertical() {
                                assert_eq!(left, right);
                            } else {
                                assert!(
                                    (left.1.start + left.1.size + right.1.start - viewport).abs()
                                        < 0.001
                                );
                            }
                        }
                        assert!(
                            ltr.windows(2)
                                .all(|pair| pair[0].1.start + pair[0].1.size
                                    <= pair[1].1.start + 0.001),
                            "{variant:?}, {viewport}, {count}, {position}, {ltr:?}"
                        );
                    }
                }
            }
        }
    }
}
