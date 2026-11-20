use std::time::Duration;

use gpui::{Anchor, Edges, Hsla, Pixels, px};
use gpui_material_components::{ButtonColors, ButtonShapes, TextFieldColors};
use gpui_material_theme::{CornerShape, Elevation};

mod bar;
mod view;

pub use bar::SearchBar;
pub use view::SearchView;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchViewVariant {
    #[default]
    Docked,
    FullScreen,
    FullScreenContained,
}

/// Independent appearance overrides. Omitted values use the current window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStyle {
    pub trigger_colors: Option<ButtonColors>,
    pub trigger_shapes: Option<ButtonShapes>,
    pub input_colors: Option<TextFieldColors>,
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub divider: Option<Hsla>,
    pub shape: Option<CornerShape>,
    pub results_shape: Option<CornerShape>,
    pub tonal_elevation: Option<Elevation>,
    pub shadow_elevation: Option<Elevation>,
}

/// Optional screen dimming with a window theme fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchScrim {
    pub color: Option<Hsla>,
    pub opacity: f32,
}

impl Default for SearchScrim {
    fn default() -> Self {
        Self {
            color: None,
            opacity: 0.32,
        }
    }
}

/// Behavior for a docked popup anchored to the search trigger.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchPopupOptions {
    pub logical_anchor: Anchor,
    pub offset: Pixels,
    pub dismiss_on_outside: bool,
    pub dismiss_on_escape: bool,
    pub trap_focus: bool,
    pub scrim: Option<SearchScrim>,
}

impl Default for SearchPopupOptions {
    fn default() -> Self {
        Self {
            logical_anchor: Anchor::TopLeft,
            offset: px(4.),
            dismiss_on_outside: true,
            dismiss_on_escape: true,
            trap_focus: false,
            scrim: None,
        }
    }
}

/// Properties of the full-window search overlay. Insets use left and right as logical start and end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchWindowOptions {
    pub insets: Edges<Pixels>,
    pub dismiss_on_outside: bool,
    pub dismiss_on_escape: bool,
    pub scrim: SearchScrim,
}

impl Default for SearchWindowOptions {
    fn default() -> Self {
        Self {
            insets: Edges::all(px(0.)),
            dismiss_on_outside: true,
            dismiss_on_escape: true,
            scrim: SearchScrim::default(),
        }
    }
}

/// Show and dismissal durations. The default settles immediately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchMotion {
    pub expand: Duration,
    pub collapse: Duration,
}

pub(super) fn length(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() {
        value.max(px(0.))
    } else {
        px(0.)
    }
}

fn valid_dimension(value: Pixels) -> Option<Pixels> {
    let value = length(value);
    (value > px(0.)).then_some(value)
}
