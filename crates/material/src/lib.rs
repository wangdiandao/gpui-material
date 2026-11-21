//! Material 3 Expressive controls and theme for GPUI applications.
//!
//! Initialize [MaterialRoot] and call [init_components] before opening windows.
//! Compose controls with GPUI elements and keep controlled selection and
//! command outcomes in the application. Sliders use a retained `SliderState` entity.

pub use gpui_material_components::*;
#[cfg(feature = "date-time")]
pub use gpui_material_extras::date_time;
#[cfg(feature = "window-chrome")]
pub use gpui_material_extras::window_chrome;
#[cfg(feature = "expressive-groups")]
pub use gpui_material_extras::{
    ButtonGroup, ButtonGroupAlignment, ButtonGroupConnectedShapes, ButtonGroupItem,
    ButtonGroupItemStyle, ButtonGroupSelection, ButtonGroupVariant, ButtonGroupVerticalAlignment,
    SplitButton, SplitButtonSegmentStyle,
};
#[cfg(feature = "carousel")]
pub use gpui_material_extras::{
    Carousel, CarouselAlignment, CarouselBorder, CarouselFling, CarouselFlingContext, CarouselItem,
    CarouselPadding, CarouselParallax, CarouselState, CarouselVariant,
};
#[cfg(feature = "toolbars")]
pub use gpui_material_extras::{
    FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors, FabMenuTriggerAppearance,
    FabMenuTriggerStyle, Toolbar, ToolbarAction, ToolbarArrangement, ToolbarColor, ToolbarColors,
    ToolbarElevations, ToolbarExitDirection, ToolbarFabPosition, ToolbarFling, ToolbarOrientation,
    ToolbarPadding, ToolbarPlacement, ToolbarScrollBehavior, ToolbarScrollState,
};
#[cfg(feature = "search")]
pub use gpui_material_extras::{
    SearchBar, SearchEvent, SearchMotion, SearchOutcome, SearchPopupOptions, SearchRequest,
    SearchResult, SearchScrim, SearchScrollState, SearchState, SearchStyle, SearchValue,
    SearchView, SearchViewVariant, SearchWindowOptions,
};
pub use gpui_material_foundation as foundation;
pub use gpui_material_foundation::{
    InteractionSource, InteractionState, InteractionUpdate, MaterialRoot, PolygonMorph, Scrim,
    Shape, ShapeFit, ShapeMorph, ShapeMorphSequence, ShapePolygon, Surface, SurfaceBorder,
    SurfaceFeedback, ThemeScope, current_base_theme, current_material_theme, material_theme,
    project_base_theme, set_material_theme,
};
pub use gpui_material_theme as theme;
pub use gpui_material_theme::{MaterialShape, MaterialTheme};
