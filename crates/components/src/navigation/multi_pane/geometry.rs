use std::ops::Range;

use gpui::{Bounds, Pixels, Size, point, px, size};
use gpui_material_theme::{Direction, WidthClass};

use super::{PaneLayoutValue, PanePlacement, PaneRole};

/// Available partitions and preferred pane sizes in logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct PaneLayoutDirective {
    /// Maximum number of side-by-side horizontal pane columns (1 to 3).
    pub horizontal_partitions: usize,
    /// Maximum number of stacked vertical partitions when reflowing in a single column.
    pub vertical_partitions: usize,
    /// Horizontal gutter between adjacent columns in logical pixels.
    pub horizontal_gap: Pixels,
    /// Vertical gutter between stacked panes in logical pixels.
    pub vertical_gap: Pixels,
    /// Default preferred width for panes that do not specify one.
    pub preferred_width: Pixels,
    /// Default preferred height for levitated panes that do not specify one.
    pub preferred_height: Pixels,
    /// Full-height excluded regions measured from logical start.
    pub excluded_widths: Vec<Range<Pixels>>,
}

impl Default for PaneLayoutDirective {
    fn default() -> Self {
        Self::for_size(size(px(0.), px(0.)))
    }
}

impl PaneLayoutDirective {
    /// Material window size classes for the available viewport.
    pub fn for_size(viewport: Size<Pixels>) -> Self {
        let class = WidthClass::from_width(viewport.width.into());
        let horizontal_partitions = match class {
            WidthClass::Compact | WidthClass::Medium => 1,
            WidthClass::Expanded => 2,
            WidthClass::Large | WidthClass::ExtraLarge => 3,
        };
        let tall = horizontal_partitions == 1 && length(viewport.height) >= px(900.);
        Self {
            horizontal_partitions,
            vertical_partitions: if tall { 2 } else { 1 },
            horizontal_gap: px(if horizontal_partitions == 1 { 0. } else { 24. }),
            vertical_gap: px(if tall { 24. } else { 0. }),
            preferred_width: px(if horizontal_partitions == 3 {
                412.
            } else {
                360.
            }),
            preferred_height: px(420.),
            excluded_widths: Vec::new(),
        }
    }
}

/// Pane margins in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaneMargins {
    /// Logical start margin in pixels.
    pub start: Pixels,
    /// Logical end margin in pixels.
    pub end: Pixels,
    /// Top margin in pixels.
    pub top: Pixels,
    /// Bottom margin in pixels.
    pub bottom: Pixels,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Measures {
    pub preferred_width: Option<Pixels>,
    pub preferred_height: Option<Pixels>,
    pub minimum_width: Pixels,
    pub margins: PaneMargins,
}

/// Anchors relative to the logical extent of one horizontal band excluding its gutters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaneExpansionAnchor {
    /// Fraction of the band's usable width in `0.0..=1.0`.
    Proportion(f32),
    /// Fixed offset measured from the logical start edge of the band.
    FromStart(Pixels),
    /// Fixed offset measured inward from the logical end edge of the band.
    FromEnd(Pixels),
}

impl PaneExpansionAnchor {
    pub(super) fn position(self, width: Pixels) -> Option<Pixels> {
        let value = match self {
            Self::Proportion(fraction) if fraction.is_finite() => width * fraction.clamp(0., 1.),
            Self::FromStart(offset) if f32::from(offset).is_finite() => length(offset),
            Self::FromEnd(offset) if f32::from(offset).is_finite() => width - length(offset),
            _ => return None,
        };
        Some(value.clamp(px(0.), width))
    }
}

/// Logical alignment of a levitated pane in its horizontal band and available height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneOverlayAlignment {
    /// Top and logical start corner.
    TopStart,
    /// Top edge, horizontally centered.
    TopCenter,
    /// Top and logical end corner.
    TopEnd,
    /// Vertically centered on the logical start edge.
    CenterStart,
    /// Centered both horizontally and vertically.
    #[default]
    Center,
    /// Vertically centered on the logical end edge.
    CenterEnd,
    /// Bottom and logical start corner.
    BottomStart,
    /// Bottom edge, horizontally centered.
    BottomCenter,
    /// Bottom and logical end corner.
    BottomEnd,
}

impl PaneOverlayAlignment {
    fn factors(self) -> (f32, f32) {
        use PaneOverlayAlignment::*;
        let x = match self {
            TopStart | CenterStart | BottomStart => 0.,
            TopCenter | Center | BottomCenter => 0.5,
            TopEnd | CenterEnd | BottomEnd => 1.,
        };
        let y = match self {
            TopStart | TopCenter | TopEnd => 0.,
            CenterStart | Center | CenterEnd => 0.5,
            BottomStart | BottomCenter | BottomEnd => 1.,
        };
        (x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Boundary {
    pub start: PaneRole,
    pub end: PaneRole,
    pub center: Pixels,
    pub position: Pixels,
    pub min: Pixels,
    pub max: Pixels,
    pub extent: Pixels,
}

impl Boundary {
    pub fn anchors(
        self,
        anchors: &[PaneExpansionAnchor],
    ) -> impl Iterator<Item = (PaneExpansionAnchor, Pixels)> {
        anchors.iter().copied().filter_map(move |anchor| {
            anchor
                .position(self.extent)
                .map(|at| (anchor, at.clamp(self.min, self.max)))
        })
    }
}

#[derive(Default)]
pub(super) struct LayoutGeometry {
    pub panes: [Option<Bounds<Pixels>>; 3],
    pub boundaries: Vec<Boundary>,
}

pub(super) struct GeometryInput<'a> {
    pub viewport: Size<Pixels>,
    pub direction: Direction,
    pub directive: &'a PaneLayoutDirective,
    pub order: [PaneRole; 3],
    pub measures: [Measures; 3],
}

impl GeometryInput<'_> {
    pub fn layout(
        &self,
        value: PaneLayoutValue,
        expansion: impl Fn(PaneRole, PaneRole) -> Option<PaneExpansionAnchor>,
    ) -> LayoutGeometry {
        let width = length(self.viewport.width);
        let height = length(self.viewport.height);
        let bands = available_bands(width, &self.directive.excluded_widths);
        let mut columns = [PaneRole::Primary; 3];
        let mut column_count = 0;
        for role in self.order {
            if value[role] == PanePlacement::Expanded {
                columns[column_count] = role;
                column_count += 1;
            }
        }
        let columns = &columns[..column_count];
        let mut result = LayoutGeometry::default();
        let mut start = 0;
        for (band, count) in allocate_bands(&bands, columns.len()) {
            self.place_band(
                &columns[start..start + count],
                band,
                height,
                value,
                &expansion,
                &mut result,
            );
            start += count;
        }
        if let Some(band) = bands.iter().max_by(|a, b| span(a).total_cmp(&span(b))) {
            for role in self.order {
                if let PanePlacement::Levitate(alignment) = value[role] {
                    let (x, y) = alignment.factors();
                    let measures = self.measures[role.index()];
                    let width = preferred(measures.preferred_width, self.directive.preferred_width)
                        .min(band.end - band.start);
                    let pane_height =
                        preferred(measures.preferred_height, self.directive.preferred_height)
                            .min(height);
                    let bounds = Bounds::new(
                        point(
                            band.start + (band.end - band.start - width) * x,
                            (height - pane_height) * y,
                        ),
                        size(width, pane_height),
                    );
                    result.panes[role.index()] = Some(inset(bounds, measures.margins));
                }
            }
        }
        if self.direction == Direction::Rtl {
            for bounds in result.panes.iter_mut().flatten() {
                bounds.origin.x = width - bounds.right();
            }
            for boundary in &mut result.boundaries {
                boundary.center = width - boundary.center;
            }
        }
        result
    }

    fn place_band(
        &self,
        roles: &[PaneRole],
        band: Range<Pixels>,
        height: Pixels,
        value: PaneLayoutValue,
        expansion: &impl Fn(PaneRole, PaneRole) -> Option<PaneExpansionAnchor>,
        result: &mut LayoutGeometry,
    ) {
        let count = roles.len();
        if count == 0 {
            return;
        }
        let gutter = gap(self.directive.horizontal_gap, band.end - band.start, count);
        let extent = band.end - band.start - gutter * (count - 1) as f32;
        let mut targets = [px(0.); 3];
        let mut minima = [px(0.); 3];
        for (ix, role) in roles.iter().enumerate() {
            let measure = self.measures[role.index()];
            targets[ix] = preferred(measure.preferred_width, self.directive.preferred_width);
            minima[ix] = length(measure.minimum_width);
        }
        let targets = &targets[..count];
        let minima = &minima[..count];
        let widths = distribute(
            targets,
            minima,
            extent,
            roles.iter().position(|role| *role == PaneRole::Primary),
        );
        let mut boundaries = [px(0.); 4];
        let mut wanted = px(0.);
        for i in 1..count {
            wanted += widths[i - 1];
            let min = boundaries[i - 1] + scaled_minimum(minima, i - 1, extent);
            let after: Pixels = (i..count)
                .map(|ix| scaled_minimum(minima, ix, extent))
                .sum();
            let max = (extent - after).max(min);
            boundaries[i] = expansion(roles[i - 1], roles[i])
                .and_then(|anchor| anchor.position(extent))
                .unwrap_or(wanted)
                .clamp(min, max);
        }
        boundaries[count] = extent;
        for (i, &role) in roles.iter().enumerate() {
            let origin = band.start + boundaries[i] + gutter * i as f32;
            let width = (boundaries[i + 1] - boundaries[i]).max(px(0.));
            let mut stack = [role; 3];
            let mut stack_len = 1;
            for child in self.order {
                if value[child] == PanePlacement::Reflow(role) {
                    stack[stack_len] = child;
                    stack_len += 1;
                }
            }
            let stack = &stack[..stack_len];
            let vertical_gap = gap(self.directive.vertical_gap, height, stack.len());
            let stack_space = height - vertical_gap * stack.len().saturating_sub(1) as f32;
            let mut stack_targets = [px(0.); 3];
            for (ix, pane) in stack.iter().enumerate() {
                stack_targets[ix] = preferred(
                    self.measures[pane.index()].preferred_height,
                    self.directive.preferred_height,
                );
            }
            let zero_minima = [px(0.); 3];
            let heights = distribute(
                &stack_targets[..stack.len()],
                &zero_minima[..stack.len()],
                stack_space,
                Some(0),
            );
            let mut y = px(0.);
            for (&pane, pane_height) in stack.iter().zip(heights) {
                result.panes[pane.index()] = Some(inset(
                    Bounds::new(point(origin, y), size(width, pane_height)),
                    self.measures[pane.index()].margins,
                ));
                y += pane_height + vertical_gap;
            }
            if let Some(&next) = roles.get(i + 1) {
                result.boundaries.push(Boundary {
                    start: role,
                    end: next,
                    center: origin + width + gutter / 2.,
                    position: boundaries[i + 1],
                    min: boundaries[i] + scaled_minimum(minima, i, extent),
                    max: boundaries[i + 2] - scaled_minimum(minima, i + 1, extent),
                    extent,
                });
            }
        }
    }
}

pub(super) fn length(value: Pixels) -> Pixels {
    let number = f32::from(value);
    // Capping to GPUI's representable viewport geometry also keeps intermediate sums finite.
    px(if number.is_finite() {
        number.clamp(0., 1_000_000.)
    } else {
        0.
    })
}

fn preferred(value: Option<Pixels>, fallback: Pixels) -> Pixels {
    value
        .map(length)
        .filter(|value| *value > px(0.))
        .unwrap_or_else(|| length(fallback))
}

fn gap(value: Pixels, extent: Pixels, count: usize) -> Pixels {
    length(value).min(extent / count.max(1) as f32)
}

fn span(range: &Range<Pixels>) -> f32 {
    (range.end - range.start).into()
}

fn available_bands(width: Pixels, excluded: &[Range<Pixels>]) -> Vec<Range<Pixels>> {
    let mut cuts: Vec<_> = excluded
        .iter()
        .filter(|cut| f32::from(cut.start).is_finite() && f32::from(cut.end).is_finite())
        .map(|cut| length(cut.start).min(width)..length(cut.end).min(width))
        .filter(|cut| cut.start < cut.end)
        .collect();
    cuts.sort_by(|a, b| f32::from(a.start).total_cmp(&f32::from(b.start)));
    let mut bands = Vec::new();
    let mut start = px(0.);
    for cut in cuts {
        if cut.start > start {
            bands.push(start..cut.start);
        }
        start = start.max(cut.end);
    }
    if width > start {
        bands.push(start..width);
    }
    bands
}

fn allocate_bands(bands: &[Range<Pixels>], columns: usize) -> Vec<(Range<Pixels>, usize)> {
    if columns == 0 || bands.is_empty() {
        return Vec::new();
    }
    let mut available: Vec<_> = bands.iter().cloned().map(|band| (band, 1)).collect();
    if available.len() > columns {
        available.sort_by(|a, b| span(&b.0).total_cmp(&span(&a.0)));
        available.truncate(columns);
        available.sort_by(|a, b| f32::from(a.0.start).total_cmp(&f32::from(b.0.start)));
    }
    for _ in available.len()..columns {
        if let Some((_, count)) = available.iter_mut().max_by(|a, b| {
            (span(&a.0) / (a.1 + 1) as f32).total_cmp(&(span(&b.0) / (b.1 + 1) as f32))
        }) {
            *count += 1;
        }
    }
    available
}

fn scaled_minimum(minima: &[Pixels], ix: usize, extent: Pixels) -> Pixels {
    let sum: Pixels = minima.iter().copied().sum();
    if sum > extent {
        minima[ix] * (extent / sum)
    } else {
        minima[ix]
    }
}

fn distribute(
    targets: &[Pixels],
    minima: &[Pixels],
    extent: Pixels,
    flexible: Option<usize>,
) -> [Pixels; 3] {
    let count = targets.len();
    let mut minimum_sum = px(0.);
    let minimums: [Pixels; 3] = std::array::from_fn(|ix| {
        if ix < count {
            let min = scaled_minimum(minima, ix, extent);
            minimum_sum += min;
            min
        } else {
            px(0.)
        }
    });
    let available = (extent - minimum_sum).max(px(0.));
    let mut needed = px(0.);
    let extras: [Pixels; 3] = std::array::from_fn(|ix| {
        if ix < count {
            let extra = (targets[ix] - minimums[ix]).max(px(0.));
            needed += extra;
            extra
        } else {
            px(0.)
        }
    });
    std::array::from_fn(|ix| {
        if ix >= count {
            return px(0.);
        }
        let minimum = minimums[ix];
        let extra = extras[ix];
        let additional = if needed > available {
            extra * (available / needed)
        } else if flexible == Some(ix) {
            extra + available - needed
        } else if flexible.is_none() {
            extra + (available - needed) / count as f32
        } else {
            extra
        };
        minimum + additional
    })
}

fn inset(bounds: Bounds<Pixels>, margins: PaneMargins) -> Bounds<Pixels> {
    let left = length(margins.start).min(bounds.size.width);
    let right = length(margins.end).min(bounds.size.width - left);
    let top = length(margins.top).min(bounds.size.height);
    let bottom = length(margins.bottom).min(bounds.size.height - top);
    Bounds::new(
        bounds.origin + point(left, top),
        size(
            bounds.size.width - left - right,
            bounds.size.height - top - bottom,
        ),
    )
}

#[cfg(test)]
mod tests {
    use gpui::size;

    use super::*;
    use PanePlacement::{Expanded, Hidden};
    use PaneRole::{Primary, Secondary, Tertiary};

    #[test]
    fn material_breakpoints_choose_horizontal_and_tall_vertical_partitions() {
        for (width, height, counts) in [
            (300., 500., (1, 1)),
            (650., 950., (1, 2)),
            (840., 490., (2, 1)),
            (1300., 490., (3, 1)),
        ] {
            let directive = PaneLayoutDirective::for_size(size(px(width), px(height)));
            assert_eq!(
                (
                    directive.horizontal_partitions,
                    directive.vertical_partitions
                ),
                counts
            );
        }
    }

    #[test]
    fn occlusion_and_oversized_preferences_keep_panes_in_finite_nonoverlapping_bands() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let mut directive = PaneLayoutDirective::for_size(size(px(1600.), px(900.)));
            directive.horizontal_gap = px(f32::MAX);
            directive.vertical_gap = px(f32::NAN);
            directive.excluded_widths =
                vec![px(250.)..px(340.), px(300.)..px(400.), px(70.)..px(45.)];
            let measures = Measures {
                preferred_width: Some(px(f32::INFINITY)),
                preferred_height: Some(px(20_000.)),
                minimum_width: px(1800.),
                margins: PaneMargins {
                    start: px(5.),
                    end: px(5.),
                    top: px(-4.),
                    bottom: px(7.),
                },
            };
            let input = GeometryInput {
                viewport: size(px(680.), px(240.)),
                direction,
                directive: &directive,
                order: [Secondary, Primary, Tertiary],
                measures: [measures; 3],
            };
            let geometry = input.layout(
                PaneLayoutValue::new(Expanded, Expanded, Expanded),
                |_, _| Some(PaneExpansionAnchor::FromEnd(px(-400.))),
            );
            let (occlusion_left, occlusion_right) = if direction == Direction::Ltr {
                (px(250.), px(400.))
            } else {
                (px(280.), px(430.))
            };
            let mut bounds: Vec<_> = geometry.panes.into_iter().flatten().collect();
            bounds.sort_by(|a, b| f32::from(a.left()).total_cmp(&f32::from(b.left())));
            assert_eq!(bounds.len(), 3);
            for pane in &bounds {
                assert!(pane.left() >= px(0.) && pane.right() <= px(680.));
                assert!(pane.top() >= px(0.) && pane.bottom() <= px(240.));
                assert!(pane.right() <= occlusion_left || pane.left() >= occlusion_right);
                assert!(pane.size.width > px(0.) && pane.size.height > px(0.));
            }
            assert!(
                bounds
                    .windows(2)
                    .all(|pair| pair[0].right() <= pair[1].left())
            );
            assert_eq!(geometry.boundaries.len(), 1);
            let boundary = geometry.boundaries[0];
            assert!(boundary.min <= boundary.position && boundary.position <= boundary.max);
            let narrow = GeometryInput {
                viewport: size(px(1.), px(1.)),
                ..input
            };
            let limited =
                narrow.layout(PaneLayoutValue::new(Expanded, Hidden, Hidden), |_, _| None);
            assert_eq!(
                limited.panes[Primary.index()].unwrap().size,
                size(px(0.), px(0.))
            );
        }
    }
}
