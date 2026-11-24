//! Interactive Material examples for a GPUI host.
//!
//! Call [`init_gallery`] before creating views. Use [`Gallery`] for the complete
//! catalog or [`Story::new_view`] for a single example. Mount either under a
//! [`gpui_base::Root`] for overlays and keyboard traversal.

#[cfg(not(target_arch = "wasm32"))]
mod playground;
#[cfg(not(target_arch = "wasm32"))]
mod playgrounds;
#[cfg(not(target_arch = "wasm32"))]
mod shell;
mod stories;
#[cfg(all(test, feature = "test-support"))]
mod tests;

#[cfg(all(test, feature = "test-support"))]
pub(crate) fn painted_quads(window: &gpui::Window) -> Vec<gpui::Quad> {
    let mut merged: Vec<gpui::Quad> = Vec::new();
    for quad in window.painted_quads() {
        if let Some(last) = merged.last_mut()
            && last.bounds == quad.bounds
            && last.background == quad.background
            && last.border_color == quad.border_color
            && last.border_style == quad.border_style
        {
            if last.corner_radii == gpui::Corners::default() {
                last.corner_radii = quad.corner_radii;
            }
            if last.border_widths == gpui::Edges::default() {
                last.border_widths = quad.border_widths;
            }
            last.content_mask.bounds = last.content_mask.bounds.union(&quad.content_mask.bounds);
            continue;
        }
        merged.push(quad);
    }
    merged
}

use gpui::App;
#[cfg(not(target_arch = "wasm32"))]
use gpui::KeyBinding;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use gpui::{
    AppContext, TitlebarOptions, Window, WindowBounds, WindowHandle, WindowOptions, px, size,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use gpui_base::Root;
use gpui_material::MaterialTheme;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use gpui_material::{
    MaterialRoot,
    foundation::{material_theme, set_window_material_theme},
    theme::{Appearance, ColorScheme},
};

#[cfg(not(target_arch = "wasm32"))]
use shell::gallery::CloseCatalog;

#[cfg(not(target_arch = "wasm32"))]
pub use shell::catalog::GalleryPage;
#[cfg(not(target_arch = "wasm32"))]
pub use shell::gallery::Gallery;
pub use stories::story::Story;

pub(crate) use stories::story;

#[doc(hidden)]
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub const GALLERY_TITLE: &str = "GPUI Material Gallery";

/// Installs the default Material theme and the component and Gallery keybindings.
/// Call once per application before opening its first window.
pub fn init_gallery(cx: &mut App) {
    gpui_material::init(MaterialTheme::default(), cx);
    #[cfg(not(target_arch = "wasm32"))]
    cx.bind_keys([KeyBinding::new(
        "escape",
        CloseCatalog,
        Some("GalleryCatalog"),
    )]);
}

#[doc(hidden)]
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn open_gallery(
    cx: &mut App,
    local_dark: bool,
    initial_story: Story,
    window_chrome: bool,
) -> gpui::Result<WindowHandle<Root>> {
    open_gallery_page(
        cx,
        local_dark,
        GalleryPage::Story(initial_story),
        window_chrome,
    )
}

#[doc(hidden)]
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn open_gallery_page(
    cx: &mut App,
    local_dark: bool,
    initial_page: GalleryPage,
    window_chrome: bool,
) -> gpui::Result<WindowHandle<Root>> {
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(if local_dark {
                format!("{GALLERY_TITLE} · Dark preview").into()
            } else {
                GALLERY_TITLE.into()
            }),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(None, size(px(1280.), px(820.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    let build_gallery = move |window: &mut Window, cx: &mut App| {
        if local_dark {
            let mut theme = material_theme(cx).clone();
            theme.colors = ColorScheme::baseline(Appearance::Dark);
            set_window_material_theme(Some(theme), window, cx);
        }
        cx.new(|cx| {
            Gallery::new_at_page(initial_page, window, cx).with_window_theme_toggle(local_dark)
        })
    };
    if window_chrome {
        let options = gpui_material::window_chrome::title_bar_window_options(options);
        return MaterialRoot::open_window(options, cx, |window, cx| {
            let gallery = build_gallery(window, cx);
            cx.new(|cx| shell::window_chrome::ChromePreview::new(gallery, window, cx))
        });
    }
    MaterialRoot::open_window(options, cx, build_gallery)
}
