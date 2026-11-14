mod menu;
mod tooltip;

pub use menu::{Menu, MenuItem};
pub use tooltip::Tooltip;
pub(crate) use tooltip::init as init_tooltip;
mod dialog;
mod sheet;

pub use dialog::{Dialog, DialogState, DialogVariant};
pub use sheet::{Sheet, SheetEdge, SheetMode};
mod snackbar;

pub use snackbar::{Snackbar, SnackbarHost};

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

fn close_icon(tint: gpui::Hsla, scale: f32) -> impl gpui::IntoElement {
    use gpui::{Styled, canvas, point, px};
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let mut path = gpui::PathBuilder::stroke(px(2. * scale));
            let at = |x, y| bounds.origin + point(px(x * scale), px(y * scale));
            path.move_to(at(5., 5.));
            path.line_to(at(19., 19.));
            path.move_to(at(19., 5.));
            path.line_to(at(5., 19.));
            if let Ok(path) = path.build() {
                window.paint_path(path, tint);
            }
        },
    )
    .size(px(24. * scale))
}
