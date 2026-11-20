use gpui::{Pixels, px};

pub(super) fn nonnegative(value: Pixels) -> Option<Pixels> {
    (f32::from(value).is_finite() && value >= px(0.)).then_some(value)
}

pub(super) fn visible_count(
    widths: &[Pixels],
    gap: Pixels,
    overflow: Pixels,
    available: Pixels,
) -> usize {
    let required =
        widths.iter().copied().sum::<Pixels>() + gap * widths.len().saturating_sub(1) as f32;
    if required <= available {
        return widths.len();
    }
    let mut remaining = available - overflow;
    widths
        .iter()
        .take_while(|&&width| {
            remaining -= width + gap;
            remaining >= px(0.)
        })
        .count()
}

pub(super) fn weighted_widths(base: &[Pixels], weights: &[f32], available: Pixels) -> Vec<Pixels> {
    let total_weight = weights.iter().map(|&weight| f64::from(weight)).sum::<f64>();
    if total_weight == 0. {
        return base.to_vec();
    }
    let spare = (available - base.iter().copied().sum::<Pixels>()).max(px(0.));
    base.iter()
        .zip(weights)
        .map(|(&width, &weight)| width + spare * (f64::from(weight) / total_weight) as f32)
        .collect()
}

pub(super) fn pressed_widths(
    widths: &[Pixels],
    limits: &[Option<Pixels>],
    active: Option<usize>,
    ratio: f32,
) -> Vec<Pixels> {
    let mut resolved = widths.to_vec();
    let Some(active) = active.filter(|&index| index < widths.len()) else {
        return resolved;
    };
    let neighbors = active
        .checked_sub(1)
        .into_iter()
        .chain((active + 1 < widths.len()).then_some(active + 1));
    let count = neighbors.clone().count().max(1) as f32;
    let requested = widths[active] * (ratio / count);
    for index in neighbors {
        let available = (widths[index] - px(24.)).max(px(0.));
        let contribution =
            requested.min(limits[index].map_or(available, |limit| limit.min(available)));
        resolved[index] -= contribution;
        resolved[active] += contribution;
    }
    resolved
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::*;
    use crate::groups::ButtonGroupSelection;

    #[test]
    fn required_selection_preserves_last_choice_and_single_mode_replaces_it() {
        let a = SharedString::from("a");
        let b = SharedString::from("b");
        for mode in [
            ButtonGroupSelection::SingleRequired,
            ButtonGroupSelection::MultipleRequired,
        ] {
            assert_eq!(mode.next(std::slice::from_ref(&a), &a), None);
        }
        assert_eq!(
            ButtonGroupSelection::SingleRequired.next(std::slice::from_ref(&a), &b),
            Some(vec![b.clone()])
        );
        assert_eq!(
            ButtonGroupSelection::MultipleRequired.next(std::slice::from_ref(&a), &b),
            Some(vec![a, b])
        );
    }

    #[test]
    fn overflow_reserves_the_trigger_and_press_preserves_total_width() {
        let widths = [px(60.), px(90.), px(80.)];
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(234.)), 3);
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(201.)), 1);
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(30.)), 0);
        let adjusted = pressed_widths(&widths, &[None; 3], Some(1), 0.15);
        assert_eq!(
            adjusted.iter().copied().sum::<Pixels>(),
            widths.iter().copied().sum::<Pixels>()
        );
        assert!(adjusted[1] > widths[1]);
        assert!(adjusted[0] < widths[0] && adjusted[2] < widths[2]);
    }

    #[test]
    fn weighted_space_and_neighbor_compression_preserve_the_available_width() {
        let widths = weighted_widths(&[px(48.), px(48.), px(60.)], &[1., 3., 0.], px(236.));
        assert_eq!(widths, [px(68.), px(108.), px(60.)]);
        assert_eq!(
            pressed_widths(&widths, &[Some(px(4.)), None, Some(px(0.))], Some(1), 0.5),
            [px(64.), px(112.), px(60.)]
        );
    }
}
