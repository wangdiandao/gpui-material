//! Material 3 Expressive controls and theme for GPUI applications.
//!
//! Initialize [MaterialRoot] and call [init_components] before opening windows.
//! Compose controls with GPUI elements and keep controlled selection and
//! command outcomes in the application. Sliders use a retained [SliderState] entity.

pub use gpui_material_components::*;
pub use gpui_material_foundation as foundation;
pub use gpui_material_foundation::{MaterialRoot, Surface, material_theme, set_material_theme};
pub use gpui_material_theme as theme;
pub use gpui_material_theme::MaterialTheme;
