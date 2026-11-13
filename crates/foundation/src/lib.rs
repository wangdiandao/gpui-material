//! Material presentation and adapters for GPUI and Base.

mod root;
mod surface;
mod typography;

pub use root::{MaterialRoot, material_theme, set_material_theme};
pub use surface::{Surface, color, scrim_color};
pub use typography::{ShapedText, shape_text};

mod feedback;
mod motion;
mod shape;

pub use feedback::{InteractionFeedback, InteractionState, Ripple};
pub use motion::animate;
pub use shape::{PolygonMorph, Shape};
