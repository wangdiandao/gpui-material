//! Material presentation and adapters for GPUI and Base.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod root;
mod surface;
mod theme_scope;
mod typography;

pub use root::{
    MaterialRoot, material_theme, project_base_theme, set_material_theme,
    set_window_material_theme, window_material_theme,
};
pub use surface::{
    Surface, SurfaceBorder, SurfaceFeedback, color, elevation_shadow_pair, elevation_shadows,
    resolve_color_spec, resolve_edges, scrim_color,
};
pub use theme_scope::{ThemeScope, current_base_theme, current_material_theme};
#[doc(hidden)]
pub use typography::font_cache_revision;
pub use typography::{ShapedText, font_for_style, shape_text, text_style, wrapped_text};

pub mod control_feedback;
mod feedback;
mod motion;
mod shape;

pub use feedback::{
    FOCUS_RING_OUTER_OFFSET, FOCUS_RING_WIDTH, FocusRingPlacement, InteractionFeedback,
    InteractionState, Ripple,
};
pub use gpui_material_theme::MaterialShape;
pub use motion::{
    LoopClock, MotionSample, MotionStatus, OverlayMotion, RetargetTransition, ScaledOverlay,
    animate, animate_with_status, is_bounds_visible, sanitize_overlay_scale,
};
pub use shape::{PolygonMorph, Shape, ShapeFit, ShapeMorph, ShapeMorphSequence, ShapePolygon};

mod input;
mod interaction;
mod overlay;
mod range;

pub use input::{DragCapture, move_roving_focus};
pub use interaction::{InteractionSource, InteractionTracker, InteractionUpdate};
pub use overlay::{Scrim, isolated_scroll, logical_anchor};
pub use range::{RangeThumb, range_semantics, value_label};
