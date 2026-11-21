//! Material 3 components built on the pinned GPUI and Base interaction primitives.

#[cfg(any(feature = "actions", feature = "selection", feature = "surfaces"))]
#[cfg_attr(
    not(all(feature = "actions", feature = "selection", feature = "surfaces")),
    allow(dead_code)
)]
mod action;
#[cfg_attr(not(feature = "actions"), allow(dead_code))]
mod button_style;
mod display;
mod display_scrollbar;
mod icon;
mod insets;
mod menu_appearance;
#[cfg(any(
    feature = "actions",
    feature = "selection",
    feature = "inputs",
    feature = "surfaces",
    feature = "feedback",
    feature = "navigation"
))]
#[cfg_attr(
    not(all(
        feature = "actions",
        feature = "selection",
        feature = "inputs",
        feature = "surfaces",
        feature = "feedback",
        feature = "navigation"
    )),
    allow(dead_code)
)]
mod motion;
mod text;

pub use button_style::ControlSize;
pub use display::{Badge, Divider};
pub use display_scrollbar::{
    DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, ScrollbarHandle,
};
pub use icon::{Icon, IconName};
pub use insets::ScaffoldInsets;
pub use menu_appearance::MenuSurfaceStyle;
pub use text::Text;

#[cfg(feature = "actions")]
mod button;
#[cfg(feature = "actions")]
mod fab;

#[cfg(feature = "actions")]
#[doc(hidden)]
pub use button::composition as button_composition;
#[cfg(feature = "actions")]
pub use button::{
    Button, ButtonShape, ButtonVariant, IconButton, IconButtonVariant, IconButtonWidth,
    IconToggleButton, ToggleButton, ToggleButtonVariant,
};
#[cfg(feature = "actions")]
pub use button_style::{ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes};
#[cfg(feature = "actions")]
pub use fab::{
    ExtendedFab, Fab, FabAlignment, FabColor, FabSize, FabVisibilityAnimation, FloatingAction,
};

#[cfg(feature = "selection")]
mod chip;
#[cfg(feature = "selection")]
mod selection;
#[cfg(feature = "selection")]
mod slider;

#[cfg(feature = "selection")]
pub use chip::{Chip, ChipColors, ChipElevations, ChipStateColors, ChipVariant};
#[cfg(feature = "selection")]
pub use selection::{
    Checkbox, CheckboxColors, CheckboxState, CheckboxStateColors, Radio, RadioColors, RadioGroup,
    Switch, SwitchColors, SwitchStateColors,
};
#[cfg(feature = "selection")]
pub use slider::{
    RangeThumb, Slider, SliderColors, SliderEvent, SliderKind, SliderMarker, SliderScale,
    SliderState, SliderThumbStyle, SliderTrackStyle, SliderValue,
};

#[cfg(feature = "surfaces")]
mod card;
#[cfg(any(feature = "surfaces", feature = "navigation"))]
mod drag_handle;
#[cfg(feature = "surfaces")]
mod list;
#[cfg(feature = "surfaces")]
mod swipe_to_dismiss;

#[cfg(feature = "surfaces")]
pub use card::{Card, CardBorder, CardColors, CardElevations, CardVariant};
#[cfg(any(feature = "surfaces", feature = "navigation"))]
pub use drag_handle::{DragHandleColors, DragHandleShapes, DragHandleSizes, VerticalDragHandle};
#[cfg(feature = "surfaces")]
pub use list::{
    List, ListItem, ListItemAlignment, ListItemColors, ListItemElevations, ListItemPadding,
    ListItemShapes, ListItemStateColors, ListVariant,
};
#[cfg(feature = "surfaces")]
pub use swipe_to_dismiss::{SwipeToDismiss, SwipeToDismissState, SwipeToDismissValue};

#[cfg(feature = "feedback")]
mod loading_indicator;
#[cfg(feature = "feedback")]
mod overlay;
#[cfg(feature = "feedback")]
mod progress;
#[cfg(feature = "feedback")]
mod pull_to_refresh;

#[cfg(feature = "feedback")]
pub use loading_indicator::{LoadingIndicator, LoadingIndicatorShape, LoadingIndicatorShapes};
#[cfg(feature = "feedback")]
pub use overlay::{
    BottomSheetState, BottomSheetThresholds, BottomSheetValue, Dialog, DialogAction,
    DialogCloseReason, DialogColors, DialogProperties, DialogState, DialogVariant, Label, Menu,
    MenuColor, MenuItem, MenuItemStyle, Sheet, SheetClose, SheetColors, SheetEdge, SheetMargins,
    SheetMode, SheetPlacement, Snackbar, SnackbarColors, SnackbarHost, SnackbarId, SnackbarParts,
    SnackbarResult, Tooltip, TooltipCaret, TooltipColors, TooltipPlacement, TooltipPositioner,
    TooltipState,
};
#[cfg(feature = "feedback")]
pub use progress::{
    CircularProgress, LinearProgress, ProgressCap, ProgressShape, ProgressStop, ProgressStroke,
};
#[cfg(feature = "feedback")]
pub use pull_to_refresh::{PullToRefresh, PullToRefreshProgress, PullToRefreshState};

#[cfg(feature = "inputs")]
mod input;

#[cfg(feature = "inputs")]
pub use input::{
    DropdownAnchor, DropdownOption, ExposedDropdown, ExposedDropdownState, TextAreaState,
    TextField, TextFieldColorOverrides, TextFieldColors, TextFieldEvent, TextFieldLabelAlignment,
    TextFieldLabelPosition, TextFieldLayout, TextFieldMaskPattern, TextFieldObfuscationMode,
    TextFieldState, TextFieldVariant,
};

#[cfg(feature = "navigation")]
mod navigation;

#[cfg(feature = "navigation")]
pub use navigation::*;

/// Installs component keybindings after initializing the Material root.
pub fn init_components(cx: &mut gpui::App) {
    #[cfg(feature = "feedback")]
    overlay::init_tooltip(cx);
    #[cfg(not(feature = "feedback"))]
    let _ = cx;
}
