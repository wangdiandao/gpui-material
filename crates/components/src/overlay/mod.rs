mod menu;
mod tooltip;

pub use menu::{Menu, MenuColor, MenuItem, MenuItemStyle};
pub(crate) use tooltip::init as init_tooltip;
pub use tooltip::{
    Label, Tooltip, TooltipCaret, TooltipColors, TooltipPlacement, TooltipPositioner, TooltipState,
};
mod bottom_sheet;
mod dialog;
mod sheet;
pub use bottom_sheet::{BottomSheetState, BottomSheetThresholds, BottomSheetValue};

pub use dialog::{
    Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties, DialogState,
    DialogVariant,
};
pub use sheet::{
    Sheet, SheetClose, SheetColors, SheetEdge, SheetMargins, SheetMode, SheetPlacement,
};
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
