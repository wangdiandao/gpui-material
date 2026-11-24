use std::time::Duration;

use gpui::{Anchor, Edges, Hsla, Pixels, px};
use gpui_material_components::{ButtonColors, ControlShapes, TextFieldColors};
use gpui_material_theme::{CornerShape, Elevation};

mod bar;
mod view;

pub use bar::SearchBar;
pub use view::SearchView;

/// Presentation layout for an expanded [`SearchView`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchViewVariant {
    /// Anchors a floating search panel to the [`SearchBar`] trigger.
    #[default]
    Docked,
    /// Fills the window overlay with a full-width search surface.
    FullScreen,
    /// Fills the window overlay with a centered pill-shaped search header above results.
    FullScreenContained,
}

/// Independent appearance overrides. Omitted values use the current window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStyle {
    /// Color overrides for the collapsed [`SearchBar`] trigger button.
    pub trigger_colors: Option<ButtonColors>,
    /// Corner shape overrides for the collapsed [`SearchBar`] trigger button.
    pub trigger_shapes: Option<ControlShapes>,
    /// Color overrides for the expanded search input [`gpui_material_components::TextField`].
    pub input_colors: Option<TextFieldColors>,
    /// Background fill color for the expanded search surface.
    pub container: Option<Hsla>,
    /// Default foreground content color on the expanded search surface.
    pub content: Option<Hsla>,
    /// Optional 1px divider color drawn between the input header and results region.
    pub divider: Option<Hsla>,
    /// Corner shape override for the search panel (and header when split).
    pub shape: Option<CornerShape>,
    /// Optional separate corner shape for the results surface in a split docked view.
    pub results_shape: Option<CornerShape>,
    /// Tonal elevation override for the search surface.
    pub tonal_elevation: Option<Elevation>,
    /// Shadow elevation override for the search surface.
    pub shadow_elevation: Option<Elevation>,
}

/// Optional screen dimming with a window theme fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchScrim {
    /// Custom scrim color, or `None` to use the theme's scrim role.
    pub color: Option<Hsla>,
    /// Target scrim opacity at full expansion (`0.0..=1.0`, defaults to `0.32`).
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
    /// Logical anchor corner relative to the trigger (mirrored in RTL). Defaults to [`Anchor::TopLeft`].
    pub logical_anchor: Anchor,
    /// Vertical gap in logical pixels between the trigger and the popup. Defaults to `4px`.
    pub offset: Pixels,
    /// Dismisses the docked search view when clicking outside its bounds. Defaults to `true`.
    pub dismiss_on_outside: bool,
    /// Dismisses the docked search view when pressing Escape. Defaults to `true`.
    pub dismiss_on_escape: bool,
    /// Traps keyboard focus inside the docked popup when `true` (also enabled when a scrim is set).
    pub trap_focus: bool,
    /// Optional dimmed backdrop rendered behind the docked popup.
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
    /// Outer margins around the full-screen overlay (`left`/`right` map to logical start/end).
    pub insets: Edges<Pixels>,
    /// Dismisses the overlay when clicking exposed scrim margins. Defaults to `true`.
    pub dismiss_on_outside: bool,
    /// Dismisses the overlay when pressing Escape. Defaults to `true`.
    pub dismiss_on_escape: bool,
    /// Scrim appearance rendered behind the full-screen overlay.
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
    /// Duration of the transition from collapsed to expanded.
    pub expand: Duration,
    /// Duration of the transition from expanded to collapsed.
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
