use gpui::{AnyElement, ElementId, Entity, IntoElement, Pixels, ScrollHandle, SharedString};
use gpui_material_theme::{LogicalEdges, MotionSpec};

mod actions;
mod render;
mod scroll;
mod types;

pub use actions::{AppBarColumn, AppBarItem, AppBarMenuState, AppBarRow};
pub(super) use scroll::length;
pub use scroll::{AppBarScrollBehavior, AppBarScrollState};
pub use types::{AppBarAlignment, AppBarColors, AppBarVariant};

use types::Heading;

/// Top app bar with caller-owned navigation, headings, actions, and optional search.
#[derive(IntoElement)]
pub struct AppBar {
    id: ElementId,
    title: Heading,
    subtitle: Heading,
    variant: AppBarVariant,
    navigation: Option<AnyElement>,
    actions: Option<AnyElement>,
    search: Option<AnyElement>,
    logo: Option<AnyElement>,
    second_row: Option<AnyElement>,
    colors: AppBarColors,
    padding: Option<LogicalEdges>,
    collapsed_height: Option<f32>,
    expanded_height: Option<f32>,
    scroll: Option<ScrollHandle>,
    scroll_state: Option<Entity<AppBarScrollState>>,
    pub(super) scroll_behavior: Option<AppBarScrollBehavior>,
    pub(super) scroll_enabled: bool,
    pub(super) scroll_motion: Option<MotionSpec>,
    pub(super) font_scale: f32,
}

impl AppBar {
    /// Creates a small top app bar with the given element ID and title.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: Heading {
                text: Some(title.into()),
                ..Heading::default()
            },
            subtitle: Heading::default(),
            variant: AppBarVariant::Small,
            navigation: None,
            actions: None,
            search: None,
            logo: None,
            second_row: None,
            colors: AppBarColors::default(),
            padding: None,
            collapsed_height: None,
            expanded_height: None,
            scroll: None,
            scroll_state: None,
            scroll_behavior: None,
            scroll_enabled: true,
            scroll_motion: None,
            font_scale: 1.,
        }
    }

    /// Sets the app bar layout variant (default [`AppBarVariant::Small`]).
    pub fn variant(mut self, variant: AppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the secondary subtitle text displayed beneath the title.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle.text = Some(subtitle.into());
        self.subtitle.content = None;
        self
    }

    /// Replaces the visual title while keeping the constructor's accessible name.
    pub fn title_content(self, content: impl IntoElement + 'static) -> Self {
        self.title_for_rows(move |_| content)
    }

    /// Replaces the subtitle with custom content.
    pub fn subtitle_content(self, content: impl IntoElement + 'static) -> Self {
        self.subtitle_for_rows(move |_| content)
    }

    /// Builds the title for the currently displayed row.
    pub fn title_for_rows<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.title.content = Some(Box::new(move |expanded| build(expanded).into_any_element()));
        self
    }

    /// Builds the subtitle for the currently displayed row.
    pub fn subtitle_for_rows<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.subtitle.content = Some(Box::new(move |expanded| build(expanded).into_any_element()));
        self
    }

    /// Overrides the logical horizontal alignment of the title.
    pub fn title_alignment(mut self, alignment: AppBarAlignment) -> Self {
        self.title.alignment = Some(alignment);
        self
    }

    /// Overrides the logical horizontal alignment of the subtitle.
    pub fn subtitle_alignment(mut self, alignment: AppBarAlignment) -> Self {
        self.subtitle.alignment = Some(alignment);
        self
    }

    /// Sets the maximum line count for plain title text (clamped to at least 1).
    pub fn title_max_lines(mut self, lines: usize) -> Self {
        self.title.max_lines = Some(lines.max(1));
        self
    }

    /// Sets the maximum line count for plain subtitle text (clamped to at least 1).
    pub fn subtitle_max_lines(mut self, lines: usize) -> Self {
        self.subtitle.max_lines = Some(lines.max(1));
        self
    }

    /// Places a leading navigation control (such as a back or menu button) at the start of the bar.
    pub fn navigation(mut self, element: impl IntoElement) -> Self {
        self.navigation = Some(element.into_any_element());
        self
    }

    /// Places trailing action controls (such as an [`AppBarRow`]) at the end of the bar.
    pub fn actions(mut self, element: impl IntoElement) -> Self {
        self.actions = Some(element.into_any_element());
        self
    }

    /// Supplies the search control that replaces the heading.
    pub fn search(mut self, element: impl IntoElement) -> Self {
        self.search = Some(element.into_any_element());
        self
    }

    /// Places a brand logo element alongside the title.
    pub fn logo(mut self, element: impl IntoElement) -> Self {
        self.logo = Some(element.into_any_element());
        self
    }

    /// Adds presentation content below the expanded heading in TwoRows.
    pub fn second_row(mut self, element: impl IntoElement) -> Self {
        self.second_row = Some(element.into_any_element());
        self
    }

    /// Overrides container, heading, navigation, and action colors.
    pub fn colors(mut self, colors: AppBarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets explicit logical padding applied to each app bar row.
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.padding = Some(padding.sanitize());
        self
    }

    /// Overrides the collapsed height of the top row in logical pixels (default 64 px).
    pub fn collapsed_height(mut self, height: Pixels) -> Self {
        self.collapsed_height = length(height).filter(|height| *height > 0.);
        self
    }

    /// Overrides the fully expanded height of a two-row bar in logical pixels.
    pub fn expanded_height(mut self, height: Pixels) -> Self {
        self.expanded_height = length(height).filter(|height| *height > 0.);
        self
    }

    /// Links a GPUI [`ScrollHandle`] to drive scroll-under color and collapse behavior.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }

    /// Binds a caller-owned [`AppBarScrollState`] entity for reading or controlling scroll collapse.
    pub fn scroll_state(mut self, state: &Entity<AppBarScrollState>) -> Self {
        self.scroll_state = Some(state.clone());
        self
    }

    /// Sets how scrolling affects the app bar's height (default [`AppBarScrollBehavior::Pinned`]).
    pub fn scroll_behavior(mut self, behavior: AppBarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

    /// Enables or disables scroll-driven collapse updates (default `true`).
    pub fn scroll_enabled(mut self, enabled: bool) -> Self {
        self.scroll_enabled = enabled;
        self
    }

    /// Overrides motion when the caller asks the linked scroll state to settle.
    pub fn scroll_motion(mut self, motion: MotionSpec) -> Self {
        self.scroll_motion = Some(motion);
        self
    }

    /// Multiplies the heading typography scale by a positive finite factor (default `1.0`).
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        self
    }
}
