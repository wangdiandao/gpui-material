//! Material 3 Expressive controls and theme for GPUI applications.
//!
//! Initialize [MaterialRoot] and call [init_components] before opening windows.
//! Compose controls with GPUI elements and keep controlled selection and
//! command outcomes in the application. Sliders use a retained [SliderState] entity.

pub use gpui_material_components::*;
#[cfg(feature = "date-time")]
pub use gpui_material_extras::date_time;
#[cfg(feature = "expressive-groups")]
pub use gpui_material_extras::{
    ButtonGroup, ButtonGroupItem, ButtonGroupSelection, ButtonGroupVariant, SplitButton,
    SplitButtonSegmentStyle,
};
#[cfg(feature = "carousel")]
pub use gpui_material_extras::{
    Carousel, CarouselAlignment, CarouselBorder, CarouselFling, CarouselFlingContext, CarouselItem,
    CarouselPadding, CarouselParallax, CarouselState, CarouselVariant,
};
#[cfg(feature = "toolbars")]
pub use gpui_material_extras::{
    FabMenu, FabMenuItem, Toolbar, ToolbarAction, ToolbarColor, ToolbarFabPosition,
    ToolbarOrientation, ToolbarPlacement,
};
#[cfg(feature = "search")]
pub use gpui_material_extras::{
    SearchBar, SearchEvent, SearchOutcome, SearchRequest, SearchResult, SearchState, SearchView,
    SearchViewVariant,
};
pub use gpui_material_foundation as foundation;
pub use gpui_material_foundation::{
    InteractionSource, InteractionState, InteractionUpdate, MaterialRoot, Scrim, Surface,
    SurfaceBorder, SurfaceFeedback, material_theme, set_material_theme,
};
pub use gpui_material_theme as theme;
pub use gpui_material_theme::MaterialTheme;
