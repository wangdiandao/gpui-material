use crate::Story;

/// Top-level navigation sections shown in the Gallery app bar and compact drawer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TopSection {
    Components,
    Styles,
    Examples,
}

impl TopSection {
    pub const ALL: [Self; 3] = [Self::Components, Self::Styles, Self::Examples];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Styles => "styles",
            Self::Examples => "examples",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Components => "Components",
            Self::Styles => "Styles",
            Self::Examples => "Examples",
        }
    }
}

/// Navigation groups ordered after the Material 3 catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CatalogGroup {
    Library,
    Actions,
    SelectionInput,
    Navigation,
    Containment,
    Communication,
    Styles,
    Examples,
}

impl CatalogGroup {
    pub const COMPONENT_GROUPS: [Self; 5] = [
        Self::Actions,
        Self::SelectionInput,
        Self::Navigation,
        Self::Containment,
        Self::Communication,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Actions => "actions",
            Self::SelectionInput => "selection-input",
            Self::Navigation => "navigation",
            Self::Containment => "containment",
            Self::Communication => "communication",
            Self::Styles => "styles",
            Self::Examples => "examples",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Library => "Components",
            Self::Actions => "Actions",
            Self::SelectionInput => "Selection & input",
            Self::Navigation => "Navigation",
            Self::Containment => "Containment",
            Self::Communication => "Communication",
            Self::Styles => "Styles",
            Self::Examples => "Examples",
        }
    }

    pub const fn section(self) -> TopSection {
        match self {
            Self::Styles => TopSection::Styles,
            Self::Examples => TopSection::Examples,
            Self::Library
            | Self::Actions
            | Self::SelectionInput
            | Self::Navigation
            | Self::Containment
            | Self::Communication => TopSection::Components,
        }
    }
}

/// Foundation style reference pages in the Styles section.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StylePage {
    Color,
    Typography,
    Shape,
    Motion,
}

impl StylePage {
    pub const ALL: [Self; 4] = [Self::Color, Self::Typography, Self::Shape, Self::Motion];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Typography => "typography",
            Self::Shape => "shape",
            Self::Motion => "motion",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Typography => "Typography",
            Self::Shape => "Shape",
            Self::Motion => "Motion",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Color => {
                "Explore bundled Material 3 color presets, light and dark schemes, and semantic color roles."
            }
            Self::Typography => {
                "Inspect all 15 Material 3 type roles across Baseline and Emphasized scales."
            }
            Self::Shape => {
                "Browse the corner radius scale and all 35 Material 3 Expressive polygon shapes."
            }
            Self::Motion => {
                "Compare Standard and Expressive spring physics for spatial and effects transitions."
            }
        }
    }
}

/// Any page reachable from the Gallery navigation shell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GalleryPage {
    Story(Story),
    Style(StylePage),
}

impl Default for GalleryPage {
    fn default() -> Self {
        Self::Story(Story::Introduction)
    }
}

impl GalleryPage {
    pub fn id(self) -> &'static str {
        match self {
            Self::Story(story) => story.id(),
            Self::Style(style) => style.id(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Story(Story::Introduction) => "Overview",
            Self::Story(story) => story.name(),
            Self::Style(style) => style.name(),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Story(Story::Introduction) => {
                "Material 3 Expressive components, themes, and adaptive layouts for GPUI."
            }
            Self::Story(story) => story.description(),
            Self::Style(style) => style.description(),
        }
    }

    pub fn is_overview(self) -> bool {
        matches!(self, Self::Story(Story::Introduction))
    }

    pub fn story(self) -> Option<Story> {
        match self {
            Self::Story(story) => Some(story),
            Self::Style(_) => None,
        }
    }

    /// Resolves a CLI argument or route identifier to a Gallery page.
    pub fn find(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        if trimmed.eq_ignore_ascii_case("overview") {
            return Some(Self::Story(Story::Introduction));
        }
        if let Some(story) = Story::find(trimmed) {
            return Some(Self::Story(story));
        }
        StylePage::ALL
            .into_iter()
            .find(|style| {
                style.id().eq_ignore_ascii_case(trimmed)
                    || style.name().eq_ignore_ascii_case(trimmed)
            })
            .map(Self::Style)
    }
}

/// One entry in the grouped Gallery catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    pub page: GalleryPage,
    pub group: CatalogGroup,
}

impl CatalogEntry {
    pub fn matches(self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let name = self.page.name().to_ascii_lowercase();
        let id = self.page.id();
        let group = self.group.title().to_ascii_lowercase();
        let legacy_name = self
            .page
            .story()
            .map(|story| story.name().to_ascii_lowercase());
        query.split_whitespace().all(|word| {
            name.contains(word)
                || id.contains(word)
                || group.contains(word)
                || legacy_name
                    .as_deref()
                    .is_some_and(|legacy| legacy.contains(word))
        })
    }
}

const fn story_entry(story: Story, group: CatalogGroup) -> CatalogEntry {
    CatalogEntry {
        page: GalleryPage::Story(story),
        group,
    }
}

const fn style_entry(style: StylePage) -> CatalogEntry {
    CatalogEntry {
        page: GalleryPage::Style(style),
        group: CatalogGroup::Styles,
    }
}

use CatalogGroup::*;

/// All Gallery pages in grouped navigation order.
pub const CATALOG: &[CatalogEntry] = &[
    // Overview
    story_entry(Story::Introduction, Library),
    // Actions
    story_entry(Story::Button, Actions),
    story_entry(Story::ToggleButton, Actions),
    story_entry(Story::IconButton, Actions),
    story_entry(Story::ButtonGroup, Actions),
    story_entry(Story::SplitButton, Actions),
    story_entry(Story::Fab, Actions),
    story_entry(Story::FabMenu, Actions),
    story_entry(Story::ExtendedFab, Actions),
    // Selection & input
    story_entry(Story::Checkbox, SelectionInput),
    story_entry(Story::Chip, SelectionInput),
    story_entry(Story::ChipGroup, SelectionInput),
    story_entry(Story::Radio, SelectionInput),
    story_entry(Story::Slider, SelectionInput),
    story_entry(Story::Switch, SelectionInput),
    story_entry(Story::TextField, SelectionInput),
    story_entry(Story::ExposedDropdown, SelectionInput),
    story_entry(Story::Menu, SelectionInput),
    story_entry(Story::DatePicker, SelectionInput),
    story_entry(Story::DateRangePicker, SelectionInput),
    story_entry(Story::TimePicker, SelectionInput),
    story_entry(Story::Stepper, SelectionInput),
    // Navigation
    story_entry(Story::AppBar, Navigation),
    story_entry(Story::BottomAppBar, Navigation),
    story_entry(Story::NavigationBar, Navigation),
    story_entry(Story::NavigationRail, Navigation),
    story_entry(Story::NavigationDrawer, Navigation),
    story_entry(Story::Tabs, Navigation),
    story_entry(Story::Search, Navigation),
    story_entry(Story::Toolbars, Navigation),
    story_entry(Story::Breadcrumbs, Navigation),
    // Containment
    story_entry(Story::Card, Containment),
    story_entry(Story::Carousel, Containment),
    story_entry(Story::DataViews, Containment),
    story_entry(Story::Dialog, Containment),
    story_entry(Story::Divider, Containment),
    story_entry(Story::List, Containment),
    story_entry(Story::Sheets, Containment),
    story_entry(Story::Surface, Containment),
    story_entry(Story::VerticalDragHandle, Containment),
    story_entry(Story::DropTarget, Containment),
    // Communication
    story_entry(Story::Avatar, Communication),
    story_entry(Story::Badge, Communication),
    story_entry(Story::Banner, Communication),
    story_entry(Story::EmptyState, Communication),
    story_entry(Story::Icon, Communication),
    story_entry(Story::LoadingIndicator, Communication),
    story_entry(Story::Progress, Communication),
    story_entry(Story::Skeleton, Communication),
    story_entry(Story::Snackbar, Communication),
    story_entry(Story::Text, Communication),
    story_entry(Story::Tooltip, Communication),
    // Styles
    style_entry(StylePage::Color),
    style_entry(StylePage::Typography),
    style_entry(StylePage::Shape),
    style_entry(StylePage::Motion),
    // Examples
    story_entry(Story::AdaptiveLayout, Examples),
];

/// A rendered row inside the virtualized sidebar list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogRow {
    GroupHeader(CatalogGroup),
    Entry(usize),
}

pub fn entry_index(page: GalleryPage) -> Option<usize> {
    CATALOG.iter().position(|entry| entry.page == page)
}

pub fn section_of(page: GalleryPage) -> TopSection {
    entry_index(page)
        .map(|ix| CATALOG[ix].group.section())
        .unwrap_or(TopSection::Components)
}

pub fn first_of(section: TopSection) -> GalleryPage {
    CATALOG
        .iter()
        .find(|entry| entry.group.section() == section)
        .map(|entry| entry.page)
        .unwrap_or_default()
}

pub fn component_count() -> usize {
    CATALOG
        .iter()
        .filter(|entry| CatalogGroup::COMPONENT_GROUPS.contains(&entry.group))
        .count()
}

pub fn entries_in_group(group: CatalogGroup) -> impl Iterator<Item = (usize, CatalogEntry)> {
    CATALOG
        .iter()
        .copied()
        .enumerate()
        .filter(move |(_, entry)| entry.group == group)
}

/// Builds the visible sidebar rows (group headers + matching entries) for a search query.
pub fn filter_catalog_rows(query: &str) -> Vec<CatalogRow> {
    let mut rows = Vec::with_capacity(CATALOG.len() + 8);
    let mut current_group = None;
    for (ix, entry) in CATALOG.iter().copied().enumerate() {
        if !entry.matches(query) {
            continue;
        }
        if current_group != Some(entry.group) {
            current_group = Some(entry.group);
            rows.push(CatalogRow::GroupHeader(entry.group));
        }
        rows.push(CatalogRow::Entry(ix));
    }
    rows
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn catalog_covers_every_story_and_style_exactly_once() {
        let mut seen_pages = HashSet::new();
        let mut seen_ids = HashSet::new();
        for entry in CATALOG {
            assert!(
                seen_pages.insert(entry.page),
                "duplicate page in CATALOG: {:?}",
                entry.page
            );
            assert!(
                seen_ids.insert(entry.page.id()),
                "duplicate page id in CATALOG: {}",
                entry.page.id()
            );
        }
        for story in Story::ALL {
            assert!(
                seen_pages.contains(&GalleryPage::Story(*story)),
                "missing Story::{story:?} from CATALOG"
            );
        }
        for style in StylePage::ALL {
            assert!(
                seen_pages.contains(&GalleryPage::Style(style)),
                "missing StylePage::{style:?} from CATALOG"
            );
        }
        assert_eq!(CATALOG.len(), Story::ALL.len() + StylePage::ALL.len());
    }

    #[test]
    fn catalog_groups_are_contiguous_in_navigation_order() {
        let mut groups = Vec::new();
        for entry in CATALOG {
            if groups.last() != Some(&entry.group) {
                groups.push(entry.group);
            }
        }
        assert_eq!(
            groups,
            vec![
                CatalogGroup::Library,
                CatalogGroup::Actions,
                CatalogGroup::SelectionInput,
                CatalogGroup::Navigation,
                CatalogGroup::Containment,
                CatalogGroup::Communication,
                CatalogGroup::Styles,
                CatalogGroup::Examples,
            ]
        );
    }

    #[test]
    fn sections_resolve_to_expected_first_pages() {
        assert_eq!(
            first_of(TopSection::Components),
            GalleryPage::Story(Story::Introduction)
        );
        assert_eq!(
            first_of(TopSection::Styles),
            GalleryPage::Style(StylePage::Color)
        );
        assert_eq!(
            first_of(TopSection::Examples),
            GalleryPage::Story(Story::AdaptiveLayout)
        );
    }

    #[test]
    fn search_matches_names_ids_and_groups_and_emits_headers_only_for_nonempty_groups() {
        let rows = filter_catalog_rows("slider");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            CatalogRow::GroupHeader(CatalogGroup::SelectionInput)
        );
        assert_eq!(
            rows[1],
            CatalogRow::Entry(entry_index(GalleryPage::Story(Story::Slider)).unwrap())
        );

        let style_rows = filter_catalog_rows("styles");
        assert!(matches!(
            style_rows.first(),
            Some(CatalogRow::GroupHeader(CatalogGroup::Styles))
        ));
        assert_eq!(style_rows.len(), 1 + StylePage::ALL.len());

        assert!(filter_catalog_rows("no matching page").is_empty());
    }

    #[test]
    fn find_resolves_overview_stories_and_styles() {
        assert_eq!(
            GalleryPage::find("overview"),
            Some(GalleryPage::Story(Story::Introduction))
        );
        assert_eq!(
            GalleryPage::find("introduction"),
            Some(GalleryPage::Story(Story::Introduction))
        );
        assert_eq!(
            GalleryPage::find("button"),
            Some(GalleryPage::Story(Story::Button))
        );
        assert_eq!(
            GalleryPage::find("color"),
            Some(GalleryPage::Style(StylePage::Color))
        );
        assert_eq!(
            GalleryPage::find("motion"),
            Some(GalleryPage::Style(StylePage::Motion))
        );
    }
}
