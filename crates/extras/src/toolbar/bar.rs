use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, ScrollHandle, SharedString,
    Window, px,
};
use gpui_material_components::{ButtonColors, ControlElevations, FabColor, IconName};
use gpui_material_theme::{CornerShape, LogicalEdges, MotionSpec};

use super::{Invoke, activate_key, focus_handle, style_button};
use crate::ContentArrangement;

mod actions;
mod layout;
mod palette;
mod render;
mod scroll;

use scroll::length;
pub use scroll::{ToolbarExitDirection, ToolbarFling, ToolbarScrollBehavior, ToolbarScrollState};

/// Surface layout mode for a [`Toolbar`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarPlacement {
    /// Pill-shaped elevated toolbar floating above content, with optional adjacent FAB.
    #[default]
    Floating,
    /// Edge-to-edge toolbar spanning the full width or height of its container.
    Docked,
}

/// Primary layout axis of a [`Toolbar`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarOrientation {
    /// Arranges actions left-to-right (or right-to-left in RTL).
    #[default]
    Horizontal,
    /// Arranges actions top-to-bottom.
    Vertical,
}

/// Material 3 color treatment preset for a [`Toolbar`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarColor {
    /// Neutral surface container styling.
    #[default]
    Standard,
    /// Expressive primary-container styling for high emphasis.
    Vibrant,
}

/// Optional toolbar pigments. The adjacent FAB uses its own palette.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToolbarColors {
    /// Background fill color for the toolbar strip.
    pub container: Option<Hsla>,
    /// Foreground icon and text color for unselected actions.
    pub content: Option<Hsla>,
    /// Background fill color for selected toggle actions.
    pub selected_container: Option<Hsla>,
    /// Foreground icon and text color for selected toggle actions.
    pub selected_content: Option<Hsla>,
}

/// Places the FAB in logical order. Horizontal positions mirror in RTL.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarFabPosition {
    /// Places the FAB before the toolbar strip (leading edge, mirrored in RTL).
    Start,
    /// Places the FAB after the toolbar strip (trailing edge, mirrored in RTL).
    #[default]
    End,
}

#[derive(Clone)]
pub(super) enum ActionIcon {
    Symbol(IconName),
    Custom(Rc<dyn Fn() -> AnyElement>),
}

/// One labeled command or toggle rendered inside a [`Toolbar`].
pub struct ToolbarAction {
    id: SharedString,
    label: SharedString,
    icon: ActionIcon,
    enabled: bool,
    selected: Option<bool>,
    secondary: bool,
    invoke: Option<Invoke>,
}

impl ToolbarAction {
    /// Creates a toolbar action with a stable `id`, accessible `label`, and Material `icon`.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: IconName,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: ActionIcon::Symbol(icon),
            enabled: true,
            selected: None,
            secondary: false,
            invoke: None,
        }
    }

    /// Creates a toolbar action with a custom repeatable icon factory used both inline
    /// and in the collapsed overflow menu.
    pub fn with_icon<E: IntoElement>(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Fn() -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: ActionIcon::Custom(Rc::new(move || icon().into_any_element())),
            enabled: true,
            selected: None,
            secondary: false,
            invoke: None,
        }
    }

    /// Disables activation for this action when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }

    /// Marks a controlled toggle and specifies its current state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Places this action in the overflow menu when the toolbar is collapsed.
    pub fn secondary(mut self, secondary: bool) -> Self {
        self.secondary = secondary;
        self
    }

    /// Registers a callback invoked when the action is activated via pointer, keyboard, or accessibility.
    pub fn on_activate(mut self, invoke: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.invoke = Some(std::rc::Rc::new(invoke));
        self
    }
}

struct ToolbarFab {
    position: ToolbarFabPosition,
    content: FabContent,
}

enum FabContent {
    Standard {
        icon: IconName,
        label: SharedString,
        invoke: Invoke,
    },
    Custom(AnyElement),
}

/// A floating or docked group of labeled actions with optional adjacent FAB.
/// Supply a bounded main-axis extent for scrollable overflow. The toolbar
/// reveals the focused action even when its scroll container is narrower.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    label: SharedString,
    actions: Vec<ToolbarAction>,
    placement: ToolbarPlacement,
    orientation: ToolbarOrientation,
    color: ToolbarColor,
    fab: Option<ToolbarFab>,
    expanded: bool,
    scroll: Option<ScrollHandle>,
    navigation: Option<AnyElement>,
    overflow_label: Option<SharedString>,
    content: Option<AnyElement>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    leading_expanded: Option<bool>,
    trailing_expanded: Option<bool>,
    arrangement: Option<ContentArrangement>,
    spacing: Pixels,
    expanded_height: Option<Pixels>,
    padding: Option<LogicalEdges>,
    shape: Option<CornerShape>,
    elevations: ControlElevations,
    colors: ToolbarColors,
    fab_color: Option<FabColor>,
    fab_colors: ButtonColors,
    scroll_state: Option<Entity<ToolbarScrollState>>,
    scroll_behavior: Option<ToolbarScrollBehavior>,
    transition_motion: Option<MotionSpec>,
}

impl Toolbar {
    /// Creates a floating horizontal toolbar with an element `id`, accessible `label`, and `actions`.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        actions: impl IntoIterator<Item = ToolbarAction>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            actions: actions.into_iter().collect(),
            placement: ToolbarPlacement::Floating,
            orientation: ToolbarOrientation::Horizontal,
            color: ToolbarColor::Standard,
            fab: None,
            expanded: true,
            scroll: None,
            navigation: None,
            overflow_label: None,
            content: None,
            leading: None,
            trailing: None,
            leading_expanded: None,
            trailing_expanded: None,
            arrangement: None,
            spacing: px(4.),
            expanded_height: None,
            padding: None,
            shape: None,
            elevations: ControlElevations::default(),
            colors: ToolbarColors::default(),
            fab_color: None,
            fab_colors: ButtonColors::default(),
            scroll_state: None,
            scroll_behavior: None,
            transition_motion: None,
        }
    }

    /// Sets whether the toolbar renders as [`ToolbarPlacement::Floating`] or [`ToolbarPlacement::Docked`].
    pub fn placement(mut self, placement: ToolbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Sets the primary layout axis ([`ToolbarOrientation::Horizontal`] or [`ToolbarOrientation::Vertical`]).
    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the Material 3 color treatment ([`ToolbarColor::Standard`] or [`ToolbarColor::Vibrant`]).
    pub fn color(mut self, color: ToolbarColor) -> Self {
        self.color = color;
        self
    }

    /// Adds an adjacent FAB to a floating toolbar. Add the primary action
    /// through the action list when it belongs inside a docked toolbar.
    pub fn fab(
        mut self,
        position: ToolbarFabPosition,
        label: impl Into<SharedString>,
        icon: IconName,
        invoke: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.fab = Some(ToolbarFab {
            position,
            content: FabContent::Standard {
                icon,
                label: label.into(),
                invoke: Rc::new(invoke),
            },
        });
        self
    }

    /// Uses a caller-authored adjacent control in a floating toolbar.
    pub fn fab_content(mut self, position: ToolbarFabPosition, element: impl IntoElement) -> Self {
        self.fab = Some(ToolbarFab {
            position,
            content: FabContent::Custom(element.into_any_element()),
        });
        self
    }

    /// Overrides the adjacent FAB's Material color role preset.
    pub fn fab_color(mut self, color: FabColor) -> Self {
        self.fab_color = Some(color);
        self
    }

    /// Overrides individual container and content colors on the adjacent FAB.
    pub fn fab_colors(mut self, colors: ButtonColors) -> Self {
        self.fab_colors = colors;
        self
    }

    /// Inserts arbitrary content before the generated actions.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    /// Adds an expandable leading slot before the main toolbar content.
    pub fn leading_content(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Adds an expandable trailing slot after the main toolbar content.
    pub fn trailing_content(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Overrides whether the leading slot is expanded independently of the toolbar's expanded state.
    pub fn leading_expanded(mut self, expanded: bool) -> Self {
        self.leading_expanded = Some(expanded);
        self
    }

    /// Overrides whether the trailing slot is expanded independently of the toolbar's expanded state.
    pub fn trailing_expanded(mut self, expanded: bool) -> Self {
        self.trailing_expanded = Some(expanded);
        self
    }

    /// Distributes free space in a horizontal docked toolbar.
    pub fn arrangement(mut self, arrangement: ContentArrangement) -> Self {
        self.arrangement = Some(arrangement);
        self
    }

    /// Sets the gap between toolbar items in logical pixels. Defaults to `4px`.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = px(length(spacing).unwrap_or(4.));
        self
    }

    /// Sets the height of a horizontal docked toolbar.
    pub fn expanded_height(mut self, height: Pixels) -> Self {
        self.expanded_height = length(height).filter(|height| *height > 0.).map(px);
        self
    }

    /// Overrides the logical content padding inside the toolbar strip.
    pub fn padding(mut self, padding: LogicalEdges) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Overrides the corner shape of the toolbar strip.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides the shadow elevations for expanded (`resting`) and collapsed (`disabled`) states.
    ///
    /// Floating toolbars default to [`gpui_material_theme::Elevation::Level3`] when expanded
    /// and [`gpui_material_theme::Elevation::Level0`] when collapsed; docked toolbars default
    /// to [`gpui_material_theme::Elevation::Level0`].
    pub fn elevations(mut self, elevations: ControlElevations) -> Self {
        self.elevations = elevations;
        self
    }

    /// Overrides the toolbar's container, content, and selected-action colors.
    pub fn colors(mut self, colors: ToolbarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Controls whether secondary actions appear directly in the toolbar.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Uses the linked content's vertical scroll offset to collapse and expand
    /// the toolbar after a 40 dp change in the scroll direction.
    pub fn track_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = Some(scroll.clone());
        self
    }

    /// Attaches a caller-owned [`ToolbarScrollState`] entity for scroll-driven collapsing and exit.
    pub fn scroll_state(mut self, state: &Entity<ToolbarScrollState>) -> Self {
        self.scroll_state = Some(state.clone());
        self
    }

    /// Configures scroll thresholds, exit direction, and snap/fling motion.
    pub fn scroll_behavior(mut self, behavior: ToolbarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

    /// Overrides the [`MotionSpec`] used for expanding and collapsing leading/trailing slots.
    pub fn transition_motion(mut self, motion: MotionSpec) -> Self {
        self.transition_motion = Some(motion);
        self
    }

    /// Renders the supplied navigation instead of the toolbar. Use this for
    /// a responsive region shared between navigation and contextual actions.
    pub fn navigation(mut self, navigation: impl IntoElement) -> Self {
        self.navigation = Some(navigation.into_any_element());
        self
    }

    /// Sets the accessible label for the collapsed overflow trigger button. Defaults to `"More actions"`.
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = Some(label.into());
        self
    }
}
