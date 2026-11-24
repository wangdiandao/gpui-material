//! Interactive Material examples for a GPUI host.
//!
//! Call [`init_gallery`] before creating views. Use [`Gallery`] for the complete
//! catalog or [`Story::new_view`] for a single example. Mount either under a
//! [`gpui_base::Root`] for overlays and keyboard traversal.

mod action_examples;
mod adaptive_examples;
mod carousel_examples;
#[cfg(not(target_arch = "wasm32"))]
mod catalog;
mod date_time_examples;
#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "desktop",
    test,
    feature = "test-support"
))]
mod desktop_tests;
mod display_examples;
mod drag_handle_examples;
#[cfg(all(test, feature = "test-support"))]
mod drawing;
#[cfg(not(target_arch = "wasm32"))]
mod gallery;
mod group_examples;
#[cfg(all(test, feature = "test-support"))]
mod input_examples;
mod navigation_examples;
mod overlay_examples;
#[cfg(not(target_arch = "wasm32"))]
mod overview;
#[cfg(all(test, not(target_arch = "wasm32"), feature = "test-support"))]
mod perf_bench;
#[cfg(not(target_arch = "wasm32"))]
mod playground;
#[cfg(not(target_arch = "wasm32"))]
mod playgrounds;
#[cfg(all(test, feature = "test-support"))]
mod scrim_tests;
mod search_examples;
mod selection_examples;
mod slider_examples;
mod story;
#[cfg(not(target_arch = "wasm32"))]
mod styles_examples;
mod surface_examples;
#[cfg(all(test, feature = "test-support"))]
mod swipe_to_dismiss_tests;
mod text_field_examples;
#[cfg(all(test, feature = "test-support"))]
mod theme_tests;
mod toolbar_examples;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod window_chrome;

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
use gpui_material::{MaterialRoot, MaterialTheme};
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use gpui_material::{
    foundation::{material_theme, set_window_material_theme},
    theme::{Appearance, ColorScheme},
};

#[cfg(not(target_arch = "wasm32"))]
use gallery::CloseCatalog;

#[cfg(not(target_arch = "wasm32"))]
pub use catalog::GalleryPage;
#[cfg(not(target_arch = "wasm32"))]
pub use gallery::Gallery;
pub use story::Story;

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
        window_bounds: Some(WindowBounds::centered(None, size(px(1040.), px(744.)), cx)),
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
            cx.new(|cx| window_chrome::ChromePreview::new(gallery, window, cx))
        });
    }
    MaterialRoot::open_window(options, cx, build_gallery)
}
