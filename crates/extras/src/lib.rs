//! Optional composite Material components for GPUI desktop applications.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "carousel")]
mod carousel;
#[cfg(feature = "date-time")]
pub mod date_time;
#[cfg(feature = "carousel")]
pub use carousel::{
    Carousel, CarouselAlignment, CarouselBorder, CarouselFling, CarouselFlingContext, CarouselItem,
    CarouselPadding, CarouselParallax, CarouselState, CarouselVariant,
};

#[cfg(feature = "expressive-groups")]
mod groups;
#[cfg(feature = "expressive-groups")]
pub use groups::*;
#[cfg(feature = "expressive-groups")]
mod split_button;
#[cfg(feature = "expressive-groups")]
pub use split_button::{SplitButton, SplitButtonSegmentStyle};
#[cfg(feature = "search")]
mod search;
#[cfg(feature = "search")]
pub use search::*;

#[cfg(feature = "toolbars")]
mod toolbar;
#[cfg(feature = "toolbars")]
pub use toolbar::{
    FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors, FabMenuTriggerAppearance,
    FabMenuTriggerStyle, Toolbar, ToolbarAction, ToolbarArrangement, ToolbarColor, ToolbarColors,
    ToolbarElevations, ToolbarExitDirection, ToolbarFabPosition, ToolbarFling, ToolbarOrientation,
    ToolbarPadding, ToolbarPlacement, ToolbarScrollBehavior, ToolbarScrollState,
};

#[cfg(feature = "window-chrome")]
pub mod window_chrome;
