use std::ops::Range;

use gpui::Pixels;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CollapsePlan {
    pub(super) leading: Range<usize>,
    pub(super) collapsed: Range<usize>,
    pub(super) trailing: Range<usize>,
}

pub(super) fn compute_collapse_plan(
    widths: &[Pixels],
    sep_step: Pixels,
    trigger_width: Pixels,
    max_items: Option<usize>,
    available: Pixels,
) -> CollapsePlan {
    let n = widths.len();
    if n == 0 {
        return CollapsePlan {
            leading: 0..0,
            collapsed: 0..0,
            trailing: 0..0,
        };
    }
    if n <= 2 {
        return CollapsePlan {
            leading: 0..n,
            collapsed: 0..0,
            trailing: n..n,
        };
    }

    let cap = max_items.map(|c| c.max(2)).unwrap_or(usize::MAX);
    let full_width = widths.iter().copied().sum::<Pixels>() + sep_step * (n - 1) as f32;
    if n <= cap && full_width <= available {
        return CollapsePlan {
            leading: 0..n,
            collapsed: 0..0,
            trailing: n..n,
        };
    }

    // Keep first segment (0), overflow trigger, and as many trailing segments as fit
    // (always keeping at least the current trailing segment n - 1 and collapsing at least index 1).
    let mut trailing_start = n - 1;
    let mut used = widths[0] + trigger_width + widths[n - 1] + sep_step * 2.;

    for idx in (2..=(n - 2)).rev() {
        let visible_items = 1 + (n - idx);
        if visible_items > cap {
            break;
        }
        let extra = widths[idx] + sep_step;
        if used + extra <= available {
            used += extra;
            trailing_start = idx;
        } else {
            break;
        }
    }

    CollapsePlan {
        leading: 0..1,
        collapsed: 1..trailing_start,
        trailing: trailing_start..n,
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::*;

    #[test]
    fn collapse_plan_fits_when_space_permits_and_collapses_middle_by_max_items_or_width() {
        let widths = [px(60.), px(90.), px(80.), px(50.), px(70.)];
        let sep = px(20.);
        let trigger = px(32.);
        // Full width = 350 + 4 * 20 = 430.

        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(500.)),
            CollapsePlan {
                leading: 0..5,
                collapsed: 0..0,
                trailing: 5..5,
            }
        );

        // Explicit max_items(3) keeps first + 2 trailing, collapsing indices 1..3.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, Some(3), px(500.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..3,
                trailing: 3..5,
            }
        );

        // Explicit max_items(2) keeps first + current, collapsing indices 1..4.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, Some(2), px(500.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..4,
                trailing: 4..5,
            }
        );

        // Width-constrained (340 px): fits Home (60) + sep (20) + trigger (32) + sep (20) + 2026 (50) + sep (20) + Report (70) = 272 <= 340,
        // but adding Projects (80 + 20 = 100) would reach 372 > 340.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(340.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..3,
                trailing: 3..5,
            }
        );

        // Tighter width (220 px): fits Home + trigger + Report (202 <= 220), collapsing 1..4.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(220.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..4,
                trailing: 4..5,
            }
        );

        // Fewer than 3 items never collapse middle segments.
        assert_eq!(
            compute_collapse_plan(&[px(80.), px(90.)], sep, trigger, Some(2), px(100.)),
            CollapsePlan {
                leading: 0..2,
                collapsed: 0..0,
                trailing: 2..2,
            }
        );
    }
}
