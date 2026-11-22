use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, ScrollHandle, SharedString,
    Window, px,
};
use gpui_material_components::{ButtonColors, FabColor, IconName};
use gpui_material_theme::{CornerShape, MotionSpec};

use super::{Invoke, activate_key, focus_handle, style_button};

mod layout;
mod render;
mod scroll;

pub use layout::{ToolbarArrangement, ToolbarElevations, ToolbarPadding};
use scroll::length;
pub use scroll::{ToolbarExitDirection, ToolbarFling, ToolbarScrollBehavior, ToolbarScrollState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarPlacement {
    #[default]
    Floating,
    Docked,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarColor {
    #[default]
    Standard,
    Vibrant,
}

/// Optional toolbar pigments. The adjacent FAB uses its own palette.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToolbarColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub selected_container: Option<Hsla>,
    pub selected_content: Option<Hsla>,
}

/// Places the FAB in logical order. Horizontal positions mirror in RTL.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarFabPosition {
    Start,
    #[default]
    End,
}

#[derive(Clone)]
pub(super) enum ActionIcon {
    Symbol(IconName),
    Custom(Rc<dyn Fn() -> AnyElement>),
}

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
    overflow_label: SharedString,
    content: Option<AnyElement>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    leading_expanded: Option<bool>,
    trailing_expanded: Option<bool>,
    arrangement: Option<ToolbarArrangement>,
    spacing: Pixels,
    expanded_height: Option<Pixels>,
    padding: Option<ToolbarPadding>,
    shape: Option<CornerShape>,
    elevations: ToolbarElevations,
    colors: ToolbarColors,
    fab_color: Option<FabColor>,
    fab_colors: ButtonColors,
    scroll_state: Option<Entity<ToolbarScrollState>>,
    scroll_behavior: Option<ToolbarScrollBehavior>,
    transition_motion: Option<MotionSpec>,
}

impl Toolbar {
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
            overflow_label: "More actions".into(),
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
            elevations: ToolbarElevations::default(),
            colors: ToolbarColors::default(),
            fab_color: None,
            fab_colors: ButtonColors::default(),
            scroll_state: None,
            scroll_behavior: None,
            transition_motion: None,
        }
    }

    pub fn placement(mut self, placement: ToolbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

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

    pub fn fab_color(mut self, color: FabColor) -> Self {
        self.fab_color = Some(color);
        self
    }

    pub fn fab_colors(mut self, colors: ButtonColors) -> Self {
        self.fab_colors = colors;
        self
    }

    /// Inserts arbitrary content before the generated actions.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    pub fn leading_content(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    pub fn trailing_content(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    pub fn leading_expanded(mut self, expanded: bool) -> Self {
        self.leading_expanded = Some(expanded);
        self
    }

    pub fn trailing_expanded(mut self, expanded: bool) -> Self {
        self.trailing_expanded = Some(expanded);
        self
    }

    /// Distributes free space in a horizontal docked toolbar.
    pub fn arrangement(mut self, arrangement: ToolbarArrangement) -> Self {
        self.arrangement = Some(arrangement);
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = px(length(spacing).unwrap_or(4.));
        self
    }

    /// Sets the height of a horizontal docked toolbar.
    pub fn expanded_height(mut self, height: Pixels) -> Self {
        self.expanded_height = length(height).filter(|height| *height > 0.).map(px);
        self
    }

    pub fn padding(mut self, padding: ToolbarPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn elevations(mut self, elevations: ToolbarElevations) -> Self {
        self.elevations = elevations;
        self
    }

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

    pub fn scroll_state(mut self, state: &Entity<ToolbarScrollState>) -> Self {
        self.scroll_state = Some(state.clone());
        self
    }

    pub fn scroll_behavior(mut self, behavior: ToolbarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

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

    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }
}
