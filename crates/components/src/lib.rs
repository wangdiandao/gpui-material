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
pub use selection::{Checkbox, CheckboxState, Radio, RadioGroup, Switch};

mod chip;
pub use chip::{Chip, ChipVariant};

mod slider;
pub use slider::{Slider, SliderEvent, SliderKind, SliderScale, SliderState, SliderValue};

mod display;
mod drag_handle;
mod loading_indicator;
mod progress;
mod swipe_to_dismiss;
mod text;
pub use display::{Badge, Card, CardVariant, Divider, List, ListItem, ListVariant};
pub use drag_handle::{DragHandleColors, DragHandleShapes, DragHandleSizes, VerticalDragHandle};
pub use loading_indicator::{LoadingIndicator, LoadingIndicatorShape, LoadingIndicatorShapes};
pub use progress::{CircularProgress, LinearProgress, ProgressShape};
pub use swipe_to_dismiss::{SwipeToDismiss, SwipeToDismissState, SwipeToDismissValue};
pub use text::Text;
mod icon;
pub use icon::{Icon, IconName};

pub use overlay::{Menu, MenuColor, MenuItem, MenuItemStyle, MenuSurfaceStyle, Tooltip};
mod input;
pub use input::{DropdownOption, ExposedDropdown, ExposedDropdownState};
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
pub use overlay::{Dialog, DialogState, DialogVariant, Sheet, SheetEdge, SheetMode};
pub use overlay::{Snackbar, SnackbarHost};

mod navigation;
pub use navigation::*;
