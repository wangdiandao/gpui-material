mod actions;
mod communication;
mod containment;
mod navigation;
mod selection;

use crate::{Story, playground::Playground};

/// Returns the declarative playground for any component story.
/// Overview (`Introduction`) and `AdaptiveLayout` use dedicated page layouts.
pub fn playground_for(story: Story) -> Option<&'static Playground> {
    Some(match story {
        Story::Introduction | Story::AdaptiveLayout => return None,
        // Actions
        Story::Button => &actions::BUTTON,
        Story::ToggleButton => &actions::TOGGLE_BUTTON,
        Story::IconButton => &actions::ICON_BUTTON,
        Story::ButtonGroup => &actions::BUTTON_GROUP,
        Story::SplitButton => &actions::SPLIT_BUTTON,
        Story::Fab => &actions::FAB,
        Story::FabMenu => &actions::FAB_MENU,
        Story::ExtendedFab => &actions::EXTENDED_FAB,
        // Selection & input
        Story::Checkbox => &selection::CHECKBOX,
        Story::Chip => &selection::CHIP,
        Story::ChipGroup => &selection::CHIP_GROUP,
        Story::Radio => &selection::RADIO,
        Story::Slider => &selection::SLIDER,
        Story::Switch => &selection::SWITCH,
        Story::TextField => &selection::TEXT_FIELD,
        Story::ExposedDropdown => &selection::EXPOSED_DROPDOWN,
        Story::Menu => &selection::MENU,
        Story::DatePicker => &selection::DATE_PICKER,
        Story::DateRangePicker => &selection::DATE_RANGE_PICKER,
        Story::TimePicker => &selection::TIME_PICKER,
        Story::Stepper => &selection::STEPPER,
        // Navigation
        Story::AppBar => &navigation::APP_BAR,
        Story::BottomAppBar => &navigation::BOTTOM_APP_BAR,
        Story::NavigationBar => &navigation::NAVIGATION_BAR,
        Story::NavigationRail => &navigation::NAVIGATION_RAIL,
        Story::NavigationDrawer => &navigation::NAVIGATION_DRAWER,
        Story::Tabs => &navigation::TABS,
        Story::Search => &navigation::SEARCH,
        Story::CommandPalette => &navigation::COMMAND_PALETTE,
        Story::Toolbars => &navigation::TOOLBARS,
        Story::Breadcrumbs => &navigation::BREADCRUMBS,
        Story::Pagination => &navigation::PAGINATION,
        // Containment
        Story::Accordion => &containment::ACCORDION,
        Story::Card => &containment::CARD,
        Story::Carousel => &containment::CAROUSEL,
        Story::Collapsible => &containment::COLLAPSIBLE,
        Story::DataViews => &containment::DATA_VIEWS,
        Story::Dialog => &containment::DIALOG,
        Story::Divider => &containment::DIVIDER,
        Story::List => &containment::LIST,
        Story::Sheets => &containment::SHEETS,
        Story::Surface => &containment::SURFACE,
        Story::VerticalDragHandle => &containment::VERTICAL_DRAG_HANDLE,
        Story::DropTarget => &containment::DROP_TARGET,
        // Communication
        Story::Avatar => &communication::AVATAR,
        Story::Badge => &communication::BADGE,
        Story::Banner => &communication::BANNER,
        Story::EmptyState => &communication::EMPTY_STATE,
        Story::Icon => &communication::ICON,
        Story::LoadingIndicator => &communication::LOADING_INDICATOR,
        Story::Progress => &communication::PROGRESS,
        Story::Skeleton => &communication::SKELETON,
        Story::Snackbar => &communication::SNACKBAR,
        Story::Text => &communication::TEXT,
        Story::Tooltip => &communication::TOOLTIP,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playground::Config;

    #[test]
    fn every_component_story_has_a_playground_with_balanced_code_in_every_scenario() {
        let mut count = 0;
        for &story in Story::ALL {
            if matches!(story, Story::Introduction | Story::AdaptiveLayout) {
                assert!(playground_for(story).is_none());
                continue;
            }
            let playground = playground_for(story)
                .unwrap_or_else(|| panic!("Story::{story:?} is missing a playground"));
            count += 1;

            let configs = std::iter::once(Config::defaults(playground)).chain(
                playground
                    .scenarios
                    .iter()
                    .map(|scenario| Config::scenario(playground, scenario)),
            );
            for config in configs {
                let code = (playground.code)(&config);
                assert!(
                    !code.trim().is_empty(),
                    "Story::{story:?} generated empty code"
                );
                for (open, close) in [('(', ')'), ('{', '}'), ('[', ']')] {
                    let depth = code.chars().fold(0i32, |d, ch| {
                        d + i32::from(ch == open) - i32::from(ch == close)
                    });
                    assert_eq!(
                        depth, 0,
                        "Story::{story:?} unbalanced {open}{close} in:\n{code}"
                    );
                }
            }
        }
        assert_eq!(count, 55);
    }
}
