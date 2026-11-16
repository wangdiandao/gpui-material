mod menu;
mod tooltip;

pub use menu::{Menu, MenuColor, MenuItem, MenuItemStyle, MenuSurfaceStyle};
pub(crate) use tooltip::init as init_tooltip;
pub use tooltip::{
    Tooltip, TooltipCaret, TooltipColors, TooltipPlacement, TooltipPositioner, TooltipState,
};
mod dialog;
mod sheet;

pub use dialog::{Dialog, DialogState, DialogVariant};
pub use sheet::{Sheet, SheetEdge, SheetMode};
mod snackbar;

pub use snackbar::{
    Snackbar, SnackbarColors, SnackbarHost, SnackbarId, SnackbarParts, SnackbarResult,
};

/// Wrapping overlay copy uses the same theme font and type role as shaped
/// single-line labels. Keep animation scale separate from component geometry.
fn text_style<E: gpui::Styled>(
    element: E,
    role: gpui_material_theme::TypeRole,
    scale: f32,
    window: &gpui::Window,
    cx: &gpui::App,
) -> E {
    let style = role.style(gpui_material_theme::TypographyEmphasis::Baseline);
    gpui_material_foundation::text_style(element, style, scale, window, cx)
}
