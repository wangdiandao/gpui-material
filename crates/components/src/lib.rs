//! Material 3 components built on the pinned GPUI and Base interaction primitives.

mod action;
mod button;
mod button_style;
mod fab;
mod motion;

#[doc(hidden)]
pub use button::composition as button_composition;
pub use button::{
    Button, ButtonShape, ButtonVariant, ControlSize, IconButton, IconButtonVariant,
    IconButtonWidth, IconToggleButton, ToggleButton, ToggleButtonVariant,
};
pub use button_style::{ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes};
pub use fab::{
    ExtendedFab, Fab, FabAlignment, FabColor, FabSize, FabVisibilityAnimation, FloatingAction,
};

mod selection;
pub use selection::{
    Checkbox, CheckboxColors, CheckboxState, CheckboxStateColors, Radio, RadioColors, RadioGroup,
    Switch, SwitchColors, SwitchStateColors,
};

mod chip;
pub use chip::{Chip, ChipColors, ChipElevations, ChipStateColors, ChipVariant};

mod slider;
pub use slider::{
    RangeThumb, Slider, SliderColors, SliderEvent, SliderKind, SliderMarker, SliderScale,
    SliderState, SliderThumbStyle, SliderTrackStyle, SliderValue,
};

mod card;
mod display;
mod display_scrollbar;
mod drag_handle;
mod list;
mod loading_indicator;
mod progress;
mod swipe_to_dismiss;
mod text;
pub use card::{Card, CardBorder, CardColors, CardElevations, CardVariant};
pub use display::{Badge, Divider};
pub use display_scrollbar::{
    DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, ScrollbarHandle,
};
pub use drag_handle::{DragHandleColors, DragHandleShapes, DragHandleSizes, VerticalDragHandle};
pub use list::{
    List, ListItem, ListItemAlignment, ListItemColors, ListItemElevations, ListItemPadding,
    ListItemShapes, ListItemStateColors, ListVariant,
};
pub use loading_indicator::{LoadingIndicator, LoadingIndicatorShape, LoadingIndicatorShapes};
pub use progress::{
    CircularProgress, LinearProgress, ProgressCap, ProgressShape, ProgressStop, ProgressStroke,
};
pub use swipe_to_dismiss::{SwipeToDismiss, SwipeToDismissState, SwipeToDismissValue};
pub use text::Text;
mod icon;
pub use icon::{Icon, IconName};

pub use overlay::{
    Label, Menu, MenuColor, MenuItem, MenuItemStyle, MenuSurfaceStyle, Tooltip, TooltipCaret,
    TooltipColors, TooltipPlacement, TooltipPositioner, TooltipState,
};
mod input;
pub use input::{DropdownAnchor, DropdownOption, ExposedDropdown, ExposedDropdownState};
pub use input::{
    TextAreaState, TextField, TextFieldColorOverrides, TextFieldColors, TextFieldEvent,
    TextFieldLabelAlignment, TextFieldLabelPosition, TextFieldLayout, TextFieldState,
    TextFieldVariant,
};
mod overlay;

/// Installs component keybindings after initializing the Material root.
pub fn init_components(cx: &mut gpui::App) {
    overlay::init_tooltip(cx);
}
pub use overlay::{
    Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties, DialogState,
    DialogVariant, Sheet, SheetEdge, SheetMode,
};
pub use overlay::{
    Snackbar, SnackbarColors, SnackbarHost, SnackbarId, SnackbarParts, SnackbarResult,
};

mod navigation;
pub use navigation::*;
