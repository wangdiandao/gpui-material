//! Material presentation and adapters for GPUI and Base.

mod root;
mod surface;
mod typography;

pub use root::{MaterialRoot, material_theme, set_material_theme};
pub use surface::{Surface, color, scrim_color};
pub use typography::{ShapedText, shape_text};
