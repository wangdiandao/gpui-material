use std::{collections::HashMap, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, RenderOnce, SharedString, Window,
};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::LogicalEdges;

use super::rail::RailPresentation;
use super::tabs::TabVariant;
use crate::{Badge, Icon};

mod palette;
mod row;
mod state;
mod visual;

pub(in crate::navigation) use state::{RowState, row_state_for};

/// Placement of a navigation item's icon relative to its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationItemIconPosition {
    /// Place the icon above the label.
    Top,
    /// Place the icon at the logical start of the label.
    Start,
}

/// Optional pigments for an Expressive navigation destination.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationItemColors {
    /// Icon color when the destination is selected.
    pub selected_icon: Option<Hsla>,
    /// Label color when selected and the icon is positioned above the label.
    pub selected_label_top: Option<Hsla>,
    /// Label color when selected and the icon is positioned at the logical start.
    pub selected_label_start: Option<Hsla>,
    /// Fill color for the selected destination pill indicator.
    pub selected_indicator: Option<Hsla>,
    /// Icon color when the destination is unselected.
    pub unselected_icon: Option<Hsla>,
    /// Label color when the destination is unselected.
    pub unselected_label: Option<Hsla>,
    /// Icon color when the destination is disabled.
    pub disabled_icon: Option<Hsla>,
    /// Label color when the destination is disabled.
    pub disabled_label: Option<Hsla>,
}

/// One destination or tab with an ID unique within its collection.
pub struct NavigationItem {
    pub(super) id: SharedString,
    pub(super) label: SharedString,
    pub(super) icon: Option<Icon>,
    pub(super) icon_position: Option<NavigationItemIconPosition>,
    pub(super) label_content: Option<AnyElement>,
    pub(super) badge: Option<Badge>,
    pub(super) indicator_padding: Option<(LogicalEdges, LogicalEdges)>,
    pub(super) colors: NavigationItemColors,
    pub(super) interaction_source: Option<Entity<InteractionSource>>,
    pub(super) disabled: bool,
}

impl NavigationItem {
    /// Creates a navigation destination or tab with a unique ID and visible label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            icon_position: None,
            label_content: None,
            badge: None,
            indicator_padding: None,
            colors: NavigationItemColors::default(),
            interaction_source: None,
            disabled: false,
        }
    }

    /// Sets the destination's icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Disables pointer and keyboard activation for this destination.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides icon placement inherited from the tab, bar, or rail.
    pub fn icon_position(mut self, position: NavigationItemIconPosition) -> Self {
        self.icon_position = Some(position);
        self
    }

    /// Replaces the visible label while retaining the accessible name.
    pub fn label_content(mut self, content: impl IntoElement) -> Self {
        self.label_content = Some(content.into_any_element());
        self
    }

    /// Anchors a standalone badge to the icon, or to the label if there is no icon.
    /// An accessible badge label becomes the destination's description.
    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Sets the rail indicator's logical content padding without shrinking its target.
    pub fn indicator_padding(mut self, collapsed: LogicalEdges, expanded: LogicalEdges) -> Self {
        self.indicator_padding = Some((collapsed, expanded));
        self
    }

    /// Sets per-state pigments for a navigation or tab destination.
    pub fn colors(mut self, colors: NavigationItemColors) -> Self {
        self.colors = colors;
        self
    }

    /// Observes the destination using its collection ID and item ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }
}

pub(super) type Select = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Controlled destinations. Use the same collection ID across adaptive
/// presentations to preserve focus on the destination that currently owns it.
pub struct NavigationItems {
    id: ElementId,
    label: SharedString,
    items: Vec<NavigationItem>,
    selected: Option<SharedString>,
    on_select: Option<Select>,
    font_scale: f32,
    state: Option<Entity<RowState>>,
    rail_dividers: HashMap<SharedString, AnyElement>,
    retain_roving_stop: bool,
}

impl NavigationItems {
    /// Creates a controlled navigation collection with an element ID, accessible landmark label, and items.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = NavigationItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            selected: None,
            on_select: None,
            font_scale: 1.,
            state: None,
            rail_dividers: HashMap::new(),
            retain_roving_stop: false,
        }
    }

    /// Sets the ID of the currently selected destination.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected = Some(id.into());
        self
    }

    /// Reports pointer, keyboard activation and accessibility activation.
    /// Arrow keys and Home/End move focus. The caller commits selection.
    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Multiplies item label typography scale by a positive finite factor (default `1.0`).
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        self
    }

    pub(super) fn with_rail_dividers(
        mut self,
        dividers: HashMap<SharedString, AnyElement>,
    ) -> Self {
        self.rail_dividers = dividers;
        self
    }

    // Resolve the state before the adaptive scaffold puts it under a different
    // element path. Otherwise switching from its bar slot to its rail slot
    // replaces the destination focus handles.
    pub(super) fn retain(mut self, window: &mut Window, cx: &mut App) -> Self {
        if self.state.is_none() {
            self.state = Some(self.row_state(window, cx));
        }
        self
    }

    pub(super) fn id(&self) -> &ElementId {
        &self.id
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn into_drawer_parts(
        self,
    ) -> (
        ElementId,
        SharedString,
        Vec<NavigationItem>,
        Option<SharedString>,
        Option<Select>,
        f32,
        Option<Entity<RowState>>,
    ) {
        (
            self.id,
            self.label,
            self.items,
            self.selected,
            self.on_select,
            self.font_scale,
            self.state,
        )
    }

    pub(super) fn retain_roving_stop(mut self) -> Self {
        self.retain_roving_stop = true;
        self
    }

    fn row_state(&self, window: &mut Window, cx: &mut App) -> Entity<RowState> {
        row_state_for(&self.id, window, cx)
    }
}

/// Icon-and-label orientation for items inside a [`NavigationBar`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarLayout {
    /// Stack the icon above the label.
    #[default]
    Vertical,
    /// Place the icon beside the label at the logical start.
    Horizontal,
}

/// Distribution of the navigation items along the width of the bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarArrangement {
    /// Distribute items evenly across the full width of the bar.
    #[default]
    EqualWeight,
    /// Group items together centered along the bar.
    Centered,
}

/// Short navigation bar pigments. Item overrides take precedence over `content`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationBarColors {
    /// Background color override for the navigation bar container.
    pub container: Option<Hsla>,
    /// Default foreground color override for bar item content.
    pub content: Option<Hsla>,
}

/// Material Expressive short navigation bar.
#[derive(IntoElement)]
pub struct NavigationBar {
    items: NavigationItems,
    layout: NavigationBarLayout,
    arrangement: NavigationBarArrangement,
    colors: NavigationBarColors,
}

impl NavigationBar {
    /// Creates a short navigation bar displaying the given [`NavigationItems`].
    pub fn new(items: NavigationItems) -> Self {
        Self {
            items,
            layout: NavigationBarLayout::Vertical,
            arrangement: NavigationBarArrangement::default(),
            colors: NavigationBarColors::default(),
        }
    }

    /// Sets the item icon-and-label layout (default [`NavigationBarLayout::Vertical`]).
    pub fn layout(mut self, layout: NavigationBarLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets how items are distributed across the bar width (default [`NavigationBarArrangement::EqualWeight`]).
    pub fn arrangement(mut self, arrangement: NavigationBarArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    /// Overrides container and default content colors for the bar.
    pub fn colors(mut self, colors: NavigationBarColors) -> Self {
        self.colors = colors;
        self
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.items.render_row(
            Presentation::Bar {
                layout: self.layout,
                arrangement: self.arrangement,
                colors: self.colors,
            },
            window,
            cx,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Presentation {
    Tabs(TabVariant, bool),
    Bar {
        layout: NavigationBarLayout,
        arrangement: NavigationBarArrangement,
        colors: NavigationBarColors,
    },
    Rail(RailPresentation),
}

impl Presentation {
    fn is_tab(self) -> bool {
        matches!(self, Self::Tabs(..))
    }

    fn is_rail(self) -> bool {
        matches!(self, Self::Rail(_))
    }

    fn horizontal_label(self) -> bool {
        matches!(
            self,
            Self::Bar {
                layout: NavigationBarLayout::Horizontal,
                ..
            } | Self::Rail(RailPresentation { expanded: true, .. })
        )
    }

    fn bar_colors(self) -> Option<NavigationBarColors> {
        match self {
            Self::Bar { colors, .. } => Some(colors),
            _ => None,
        }
    }

    fn rail_options(self) -> Option<RailPresentation> {
        match self {
            Self::Rail(options) => Some(options),
            _ => None,
        }
    }

    fn item_colors(self) -> Option<NavigationBarColors> {
        match self {
            Self::Rail(options) => Some(NavigationBarColors {
                content: options.content,
                container: None,
            }),
            _ => self.bar_colors(),
        }
    }

    fn centered(self) -> bool {
        matches!(
            self,
            Self::Bar {
                arrangement: NavigationBarArrangement::Centered,
                ..
            }
        )
    }

    fn is_scrollable(self) -> bool {
        !matches!(self, Self::Tabs(_, false))
    }

    fn minimum_height(self, has_icons: bool) -> f32 {
        match self {
            Self::Tabs(..) if has_icons => 64.,
            Self::Tabs(..) => 48.,
            Self::Bar { .. } | Self::Rail(_) => 64.,
        }
    }
}
