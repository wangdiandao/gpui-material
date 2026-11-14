//! Optional composite Material components for GPUI desktop applications.

#[cfg(feature = "date-time")]
pub mod date_time;
#[cfg(feature = "expressive-groups")]
mod groups;
#[cfg(feature = "expressive-groups")]
pub use groups::*;
#[cfg(feature = "search")]
mod search;
#[cfg(feature = "search")]
pub use search::*;

#[cfg(feature = "toolbars")]
mod toolbar;
#[cfg(feature = "toolbars")]
pub use toolbar::{
    FabMenu, FabMenuItem, Toolbar, ToolbarAction, ToolbarColor, ToolbarFabPosition,
    ToolbarOrientation, ToolbarPlacement,
};
