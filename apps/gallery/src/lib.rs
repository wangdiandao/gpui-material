//! Interactive Material examples for a GPUI host.
//!
//! Call [`init_gallery`] before creating views. Use [`Gallery`] for the complete
//! catalog or [`Story::new_view`] for a single example. Mount either under a
//! [`gpui_base::Root`] for overlays and keyboard traversal.

mod action_examples;
mod adaptive_examples;
mod carousel_examples;
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
#[cfg(all(test, not(target_arch = "wasm32"), feature = "test-support"))]
mod perf_bench;
#[cfg(all(test, feature = "test-support"))]
mod scrim_tests;
mod search_examples;
mod selection_examples;
mod slider_examples;
mod story;
mod surface_examples;
#[cfg(all(test, feature = "test-support"))]
mod swipe_to_dismiss_tests;
mod text_field_examples;
#[cfg(all(test, feature = "test-support"))]
mod theme_tests;
mod toolbar_examples;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod window_chrome;

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
pub use gallery::Gallery;
pub use story::Story;

#[doc(hidden)]
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub const GALLERY_TITLE: &str = "GPUI Material Gallery";

/// Installs the default Material theme and the component and Gallery keybindings.
/// Call once per application before opening its first window.
pub fn init_gallery(cx: &mut App) {
    MaterialRoot::init(MaterialTheme::default(), cx);
    gpui_material::init_components(cx);
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
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(if local_dark {
                format!("{GALLERY_TITLE} · Dark preview").into()
            } else {
                GALLERY_TITLE.into()
            }),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(1040.), px(744.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    let build_gallery = move |window: &mut Window, cx: &mut App| {
        if local_dark {
            let mut theme = material_theme(cx).clone();
            theme.colors = ColorScheme::baseline(Appearance::Dark);
            set_window_material_theme(Some(theme), window, cx);
        }
        cx.new(|cx| Gallery::new(initial_story, window, cx).with_window_theme_toggle(local_dark))
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
