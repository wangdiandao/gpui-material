//! Interactive Material examples for a GPUI host.
//!
//! Call [`init_gallery`] before creating views. Use [`Gallery`] for the complete
//! catalog or [`Story::new_view`] for a single example. Mount either under a
//! [`gpui_base::Root`] for overlays and keyboard traversal.

mod action_examples;
mod adaptive_examples;
mod carousel_examples;
mod date_time_examples;
mod display_examples;
mod drag_handle_examples;
#[cfg(all(test, feature = "test-support"))]
mod drawing;
mod gallery;
mod group_examples;
#[cfg(all(test, feature = "test-support"))]
mod input_examples;
mod navigation_examples;
mod overlay_examples;
#[cfg(all(test, feature = "test-support"))]
mod scrim_tests;
mod search_examples;
mod selection_examples;
mod slider_examples;
mod story;
#[cfg(all(test, feature = "test-support"))]
mod swipe_to_dismiss_tests;
mod text_field_examples;
#[cfg(all(test, feature = "test-support"))]
mod theme_tests;
mod toolbar_examples;

use gpui::{App, KeyBinding};
use gpui_material::{MaterialRoot, MaterialTheme};

use gallery::CloseCatalog;

pub use gallery::Gallery;
pub use story::Story;

/// Installs the default Material theme and the component and Gallery keybindings.
/// Call once per application before opening its first window.
pub fn init_gallery(cx: &mut App) {
    MaterialRoot::init(MaterialTheme::default(), cx);
    gpui_material::init_components(cx);
    cx.bind_keys([KeyBinding::new(
        "escape",
        CloseCatalog,
        Some("GalleryCatalog"),
    )]);
}
