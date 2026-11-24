use gpui_material_theme::{Direction, LogicalEdges};

use super::{CarouselAlignment, CarouselVariant};

const PADDING: f32 = 16.;
const GAP: f32 = 8.;
const FULL_SCREEN_GAP: f32 = 16.;
const MIN_SMALL: f32 = 40.;
const MAX_SMALL: f32 = 56.;
const EPSILON: f32 = 1e-5;

#[derive(Clone, Copy)]
pub(super) struct LayoutOptions {
    pub(super) alignment: Option<CarouselAlignment>,
    pub(super) spacing: Option<f32>,
    pub(super) padding: Option<LogicalEdges>,
    pub(super) small: (f32, f32),
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            alignment: None,
            spacing: None,
            padding: None,
            small: (MIN_SMALL, MAX_SMALL),
        }
    }
}

/// A pose uses logical coordinates so keyboard order and RTL are independent
/// of the strategy. Item geometry is interpolated between adjacent snap poses.
pub(super) struct Layout {
    viewport: f32,
    variant: CarouselVariant,
    leading: f32,
    trailing: f32,
    cross_start: f32,
    cross_size: f32,
    cross_end: f32,
    gap: f32,
    small: (f32, f32),
    sizes: Vec<f32>,
    prefix: Vec<f32>,
    preferred: f32,
    contained_base: Option<(Vec<f32>, usize)>,
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
        options: LayoutOptions,
        aspects: impl IntoIterator<Item = f32>,
    ) -> Self {
        let vertical = variant.is_vertical();
        let uncontained = variant.is_uncontained();
        let (leading, trailing, cross_start, cross_end) = if let Some(padding) = options.padding {
            let (main, across) = if vertical {
                ((padding.top, padding.bottom), (padding.start, padding.end))
            } else {
                ((padding.start, padding.end), (padding.top, padding.bottom))
            };
            let (leading, trailing) = fit_padding(viewport, main.0, main.1);
            let (cross_start, cross_end) = fit_padding(cross, across.0, across.1);
            (leading, trailing, cross_start, cross_end)
        } else if vertical {
            (0., 0., 0., 0.)
        } else {
            let leading = PADDING.min(viewport / 4.);
            let trailing = if uncontained { 0. } else { leading };
            let across = GAP.min(cross / 4.);
            (leading, trailing, across, across)
        };
        let available = (viewport - leading - trailing).max(0.);
        let gap = options
            .spacing
            .unwrap_or(if vertical { FULL_SCREEN_GAP } else { GAP })
            .min(viewport);
        let preferred = preferred.min(available);
        let centered = options
            .alignment
            .map_or(variant == CarouselVariant::CenteredHero, |value| {
                value == CarouselAlignment::Center
            });
        let cross_size = (cross - cross_start - cross_end).max(0.);
        let sizes: Vec<_> = aspects
            .into_iter()
            .map(|aspect| match variant {
                CarouselVariant::FullScreen => available,
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
        let contained_base = if !uncontained && !vertical && !sizes.is_empty() {
            let count = sizes.len();
            let widths = match variant {
                CarouselVariant::MultiBrowse => {
                    multi_browse(available, preferred, gap, count, centered, options.small)
                }
                CarouselVariant::Hero | CarouselVariant::CenteredHero => {
                    hero(available, gap, count, centered, options.small)
                }
                _ => unreachable!(),
            };
            let maximum = widths.iter().copied().fold(0., f32::max);
            let focal = widths
                .iter()
                .position(|&width| width == maximum)
                .unwrap_or(0);
            Some((widths, focal))
        } else {
            None
        };
        Self {
            viewport,
            variant,
            leading,
            trailing,
            cross_start,
            cross_size,
            cross_end,
            gap,
            small: options.small,
            sizes,
            prefix,
            preferred,
            contained_base,
        }
    }

    pub(super) fn cross_bounds(&self, direction: Direction) -> (f32, f32) {
        let start = if self.variant.is_vertical() && direction == Direction::Rtl {
            self.cross_end
        } else {
            self.cross_start
        };
        (start, self.cross_size)
    }

    pub(super) fn clip_range(&self, direction: Direction) -> (f32, f32) {
        if self.variant.is_uncontained() {
            (0., self.viewport)
        } else if !self.variant.is_vertical() && direction == Direction::Rtl {
            (self.trailing, self.viewport - self.leading)
        } else {
            (self.leading, self.viewport - self.trailing)
        }
    }

    pub(super) fn step(&self) -> f32 {
        match self.variant {
            CarouselVariant::FullScreen => self.viewport - self.leading - self.trailing + self.gap,
            CarouselVariant::Hero | CarouselVariant::CenteredHero => self.viewport - self.small.0,
            _ => self.preferred + self.gap,
        }
        .max(1.)
    }

    /// Convert physical scroll to logical page progress. Uncontained mixed
    /// aspect ratios retain each item's actual pixel stride.
    pub(super) fn scroll_delta(&self, position: f32, delta: f32) -> f32 {
        if !position.is_finite() || !delta.is_finite() {
            return 0.;
        }
        if !self.variant.is_uncontained() || self.prefix.len() < 2 {
            return delta / self.step();
        }
        let clamped_position = position.clamp(0., (self.prefix.len() - 1) as f32);
        let base = (clamped_position.floor() as usize).min(self.prefix.len() - 1);
        let progress = clamped_position.fract();
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
        let denom = next - self.prefix[end];
        if denom <= EPSILON {
            return 0.;
        }
        let fraction = (target - self.prefix[end]) / denom;
        end as f32 + fraction - clamped_position
    }

    fn maximum_offset(&self) -> f32 {
        let (Some(&last_prefix), Some(&last_size)) = (self.prefix.last(), self.sizes.last()) else {
            return 0.;
        };
        (last_prefix + last_size - (self.viewport - self.leading - self.trailing)).max(0.)
    }

    pub(super) fn visible(&self, position: f32, direction: Direction) -> Vec<(usize, ItemBounds)> {
        if self.viewport <= 0. || self.sizes.is_empty() {
            return Vec::new();
        }
        let position = if position.is_finite() { position } else { 0. }
            .clamp(0., (self.sizes.len() - 1) as f32);
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;
        let fraction = position.fract();
        let before = self.pose(lower);
        let after = self.pose(upper);
        let (clip_start, clip_end) = self.clip_range(Direction::Ltr);
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
                let (clip_start, clip_end) = self.clip_range(Direction::Ltr);
                let start = self.start_boundary(a) + clip_start - self.leading;
                let end = self.start_boundary(b) + clip_end - self.leading;
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
                let anchor = self.small.0.min(self.viewport);
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
        let Some((base_widths, focal)) = &self.contained_base else {
            return Pose::Track {
                offset: self.prefix[index].min(self.maximum_offset()),
            };
        };
        let count = self.sizes.len();
        let mut widths = base_widths.clone();
        let focal = *focal;
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

fn fit_padding(extent: f32, start: f32, end: f32) -> (f32, f32) {
    let start = start.min(extent);
    let end = end.min(extent);
    let limit = extent - extent.min(1.);
    if start + end <= limit {
        (start, end)
    } else {
        let start = limit * (start / (start + end));
        (start, limit - start)
    }
}

fn multi_browse(
    available: f32,
    preferred: f32,
    gap: f32,
    count: usize,
    centered: bool,
    bounds: (f32, f32),
) -> Vec<f32> {
    let (min, max) = bounds;
    let small_count = if centered && count > 2 { 2 } else { 1 };
    if count <= small_count
        || available <= (small_count + 1) as f32 * min + small_count as f32 * gap
    {
        return vec![available];
    }

    // Fit the Compose large / medium / small arrangement to the window.
    // It targets a small item one third of the large item's size and a medium
    // halfway between them, then lets the medium flex up to ten percent.
    let preferred = preferred.max(2. * min);
    let small = (preferred / 3.).clamp(min, max);
    let medium = (preferred + small) / 2.;
    let largest = ((available / preferred).ceil() as usize).clamp(1, count - small_count);
    let smallest = (((available - medium - small_count as f32 * max) / preferred).floor() as usize)
        .clamp(1, largest);
    let mut best: Option<(f32, Vec<f32>)> = None;
    let mut priority = 0.;
    for large_count in (smallest..=largest).rev() {
        for medium_count in [1, 0] {
            if small_count == 2 && medium_count != 0 {
                continue;
            }
            priority += 1.;
            if large_count + medium_count + small_count > count {
                continue;
            }
            let spaces = (large_count + medium_count + small_count - 1) as f32 * gap;
            let remainder = available - spaces;
            let delta = remainder
                - (large_count as f32 * preferred
                    + medium_count as f32 * medium
                    + small_count as f32 * small);
            let small = (small + delta / small_count as f32).clamp(min, max);
            let large_count_f = large_count as f32;
            let medium_count_f = medium_count as f32;
            let mut large = (remainder - (small_count as f32 + medium_count_f / 2.) * small)
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
                let mut widths = Vec::with_capacity(large_count + medium_count + small_count);
                if small_count == 2 {
                    widths.push(small);
                }
                widths.extend(std::iter::repeat_n(large, large_count));
                widths.extend(std::iter::repeat_n(medium, medium_count));
                widths.push(small);
                best = Some((cost, widths));
            }
        }
    }
    best.map_or_else(|| vec![available], |(_, widths)| widths)
}

fn hero(available: f32, gap: f32, count: usize, centered: bool, bounds: (f32, f32)) -> Vec<f32> {
    let (min, max) = bounds;
    let small_count = match (centered, count) {
        (_, 1) => 0,
        (true, 3..) => 2,
        _ => 1,
    };
    if small_count == 0 || available < small_count as f32 * (min + gap) + min * 1.25 {
        return vec![available];
    }
    let small = (available / 3.).clamp(min, max);
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
            LayoutOptions::default(),
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
    fn custom_keylines_use_logical_padding_spacing_and_bounded_small_items() {
        let options = LayoutOptions {
            alignment: Some(CarouselAlignment::Center),
            spacing: Some(14.),
            padding: Some(LogicalEdges {
                start: 22.,
                end: 38.,
                top: 5.,
                bottom: 9.,
            }),
            small: (28., 36.),
        };
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Hero,
            CarouselVariant::CenteredHero,
        ] {
            let layout = Layout::new(variant, 410., 150., 340., options, [1.; 7]);
            assert_eq!(layout.cross_bounds(Direction::Ltr), (5., 136.));
            assert_eq!(layout.clip_range(Direction::Ltr), (22., 372.));
            assert_eq!(layout.clip_range(Direction::Rtl), (38., 388.));
            let centered = layout.visible(3., Direction::Ltr);
            assert_eq!(centered.len(), 3, "{variant:?}");
            let [(before_index, before), (index, focal), (_, after)] = centered[..] else {
                unreachable!()
            };
            assert_eq!((before_index, index), (2, 3));
            assert!((28. ..=36.).contains(&before.size));
            assert_eq!(before.size, after.size);
            assert_eq!(focal.start - (before.start + before.size), 14.);
            assert_eq!(focal.start + focal.size / 2., (22. + 372.) / 2.);
            let mirror = layout.visible(3., Direction::Rtl);
            assert_eq!(mirror[1].1.start, 410. - focal.start - focal.size);
        }
        let start = Layout::new(
            CarouselVariant::CenteredHero,
            410.,
            150.,
            340.,
            LayoutOptions {
                alignment: Some(CarouselAlignment::Start),
                ..options
            },
            [1.; 7],
        );
        assert_eq!(start.visible(3., Direction::Ltr)[0].0, 3);
        let full = Layout::new(
            CarouselVariant::FullScreen,
            240.,
            360.,
            192.,
            options,
            [1.; 3],
        );
        assert_eq!(full.cross_bounds(Direction::Rtl), (38., 300.));
        assert_eq!(full.clip_range(Direction::Rtl), (5., 231.));
        assert_eq!(
            full.visible(2., Direction::Rtl),
            [(
                2,
                ItemBounds {
                    start: 5.,
                    size: 226.
                }
            )]
        );
        for variant in ALL {
            let tiny = Layout::new(variant, 3., 2., 192., options, [1.; 5]);
            assert!(
                tiny.visible(2., Direction::Rtl)
                    .iter()
                    .all(|(_, item)| item.size.is_finite())
            );
        }
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

    #[test]
    fn precomputed_contained_keylines_match_direct_solver_across_indices() {
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Hero,
            CarouselVariant::CenteredHero,
        ] {
            for viewport in [120., 360., 460., 840.] {
                for count in [1, 2, 3, 6, 12] {
                    let geometry = layout(variant, viewport, count);
                    let available =
                        (geometry.viewport - geometry.leading - geometry.trailing).max(0.);
                    let centered = variant == CarouselVariant::CenteredHero;
                    let expected_base = match variant {
                        CarouselVariant::MultiBrowse => multi_browse(
                            available,
                            geometry.preferred,
                            geometry.gap,
                            count,
                            centered,
                            geometry.small,
                        ),
                        CarouselVariant::Hero | CarouselVariant::CenteredHero => {
                            hero(available, geometry.gap, count, centered, geometry.small)
                        }
                        _ => unreachable!(),
                    };
                    let maximum = expected_base.iter().copied().fold(0., f32::max);
                    let expected_focal = expected_base
                        .iter()
                        .position(|&width| width == maximum)
                        .unwrap_or(0);
                    assert_eq!(
                        geometry.contained_base.as_ref(),
                        Some(&(expected_base.clone(), expected_focal))
                    );
                    for index in 0..count {
                        let Pose::Contained { first, widths } = geometry.pose(index) else {
                            panic!("expected contained pose for {variant:?}");
                        };
                        let expected_first = index
                            .saturating_sub(expected_focal)
                            .min(count - expected_base.len());
                        let actual_focal = index - expected_first;
                        let mut expected_widths = expected_base.clone();
                        if actual_focal >= expected_focal {
                            expected_widths.rotate_right(actual_focal - expected_focal);
                        } else {
                            expected_widths.rotate_left(expected_focal - actual_focal);
                        }
                        assert_eq!(first, expected_first);
                        assert_eq!(widths, expected_widths);
                    }
                }
            }
        }
    }

    #[test]
    fn scroll_delta_guards_zero_denominator_non_finite_and_empty_items() {
        // Empty items must not panic in maximum_offset or scroll_delta.
        let empty = layout(CarouselVariant::Uncontained, 400., 0);
        assert_eq!(empty.maximum_offset(), 0.);
        assert_eq!(empty.scroll_delta(0., 25.), 25. / empty.step());

        // Zero-width items with zero spacing produce a zero denominator in uncontained scroll_delta.
        let degenerate = Layout::new(
            CarouselVariant::Uncontained,
            400.,
            200.,
            0.,
            LayoutOptions {
                spacing: Some(0.),
                ..LayoutOptions::default()
            },
            [1., 1., 1.],
        );
        for delta in [-50., 0., 50., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for pos in [0., 0.5, 2., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let result = degenerate.scroll_delta(pos, delta);
                assert!(
                    result.is_finite(),
                    "scroll_delta({pos}, {delta}) produced non-finite {result}"
                );
            }
        }

        // Uncontained layout where total content is narrower than viewport (maximum_offset == 0).
        let fits = Layout::new(
            CarouselVariant::Uncontained,
            800.,
            200.,
            100.,
            LayoutOptions::default(),
            [1., 1.],
        );
        assert_eq!(fits.maximum_offset(), 0.);
        assert_eq!(fits.scroll_delta(0., 30.), 0.);
        assert_eq!(fits.scroll_delta(1., -30.), 0.);
        assert!(
            fits.visible(f32::NAN, Direction::Ltr)
                .iter()
                .all(|(_, b)| b.start.is_finite())
        );
    }
}
