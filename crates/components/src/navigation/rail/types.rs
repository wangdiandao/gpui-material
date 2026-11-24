use std::rc::Rc;

use gpui::{AnyElement, App, Hsla, Pixels, Window};
use gpui_material_theme::{CornerShape, LogicalEdges};

use crate::{DialogCloseReason, DialogProperties, SheetPlacement};

/// Presentation mode of a docked [`super::NavigationRail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailMode {
    /// Narrow rail showing stacked icons and labels.
    #[default]
    Collapsed,
    /// Wide rail showing inline icons beside labels.
    Expanded,
    /// Hidden rail occupying zero width.
    Hidden,
}

/// Distribution of standard destinations along the vertical axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationRailArrangement {
    /// Pack destinations at the top.
    #[default]
    Start,
    /// Center destinations vertically.
    Center,
    /// Pack destinations at the bottom.
    End,
    /// Distribute destinations with equal space between them.
    SpaceBetween,
    /// Distribute destinations with equal space around each item.
    SpaceAround,
    /// Distribute destinations with equal space between and around them.
    SpaceEvenly,
}

/// Per-mode shape overrides. Omitted values use the default docked or sheet shape.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationRailShapes {
    /// Corner shape override when the rail is collapsed.
    pub collapsed: Option<CornerShape>,
    /// Corner shape override when the rail is expanded.
    pub expanded: Option<CornerShape>,
    /// Corner shape override when the rail is presented as a modal sheet.
    pub modal: Option<CornerShape>,
}

/// Independent local colors and scrim opacity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationRailColors {
    /// Background color override for the docked rail container.
    pub container: Option<Hsla>,
    /// Default foreground color override for docked rail content.
    pub content: Option<Hsla>,
    /// Background color override for the modal rail sheet container.
    pub modal_container: Option<Hsla>,
    /// Default foreground color override for modal rail sheet content.
    pub modal_content: Option<Hsla>,
    /// Color override for the modal backdrop scrim.
    pub scrim: Option<Hsla>,
    /// Opacity override for the modal backdrop scrim.
    pub scrim_opacity: Option<f32>,
}

/// Window-local side-sheet policy for the optional modal rail.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RailModalOptions {
    /// Escape and outside-click dismissal rules for the modal sheet.
    pub properties: DialogProperties,
    /// Stacking layer for nested modals.
    pub layer: usize,
    /// Whether the modal side sheet is docked or detached.
    pub placement: SheetPlacement,
    /// Logical margins applied when [`SheetPlacement::Detached`] is active.
    pub margins: LogicalEdges,
    /// When `true`, hides the docked rail (zero width) while collapsed if a modal state is bound.
    pub hide_on_collapse: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::navigation) struct RailPresentation {
    pub expanded: bool,
    pub arrangement: NavigationRailArrangement,
    pub spacing: Pixels,
    pub content: Option<Hsla>,
}

pub(super) type Destinations = Box<dyn FnOnce(RailMode) -> AnyElement>;
pub(super) type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
pub(super) type Closed = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;
