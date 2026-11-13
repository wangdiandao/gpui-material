//! Material 3 components built on the pinned GPUI and Base interaction primitives.

mod action;
mod button;
mod fab;

pub use button::{
    Button, ButtonShape, ButtonVariant, ControlSize, IconButton, IconButtonVariant,
    IconButtonWidth, IconToggleButton, ToggleButton, ToggleButtonVariant,
};
pub use fab::{ExtendedFab, Fab, FabColor, FabSize, FloatingAction};

mod selection;
pub use selection::{Checkbox, CheckboxState, Radio, RadioGroup, Switch};

mod chip;
pub use chip::{Chip, ChipVariant};

mod slider;
pub use slider::{Slider, SliderEvent, SliderKind, SliderScale, SliderState, SliderValue};

mod display;
mod progress;
mod text;
pub use display::{Badge, Card, CardVariant, Divider, List, ListItem, ListVariant};
pub use progress::{CircularProgress, LinearProgress, LoadingIndicator, ProgressShape};
pub use text::{Icon, Text};
