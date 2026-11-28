//! Material 3 Expressive controls and theme for GPUI applications.
//!
//! Call [`init`] with the initial [`MaterialTheme`] before opening windows.
//! Compose controls with GPUI elements and keep controlled selection and
//! command outcomes in the application. Sliders use a retained `SliderState` entity.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "display")]
pub use gpui_material_components::*;
#[cfg(any(feature = "expressive-groups", feature = "toolbars"))]
pub use gpui_material_extras::ContentArrangement;
#[cfg(feature = "date-time")]
pub use gpui_material_extras::date_time;
#[cfg(feature = "window-chrome")]
pub use gpui_material_extras::window_chrome;
#[cfg(feature = "expressive-groups")]
pub use gpui_material_extras::{
    ButtonGroup, ButtonGroupConnectedShapes, ButtonGroupItem, ButtonGroupSelection,
    ButtonGroupVariant, ButtonGroupVerticalAlignment, ButtonSegmentStyle, SplitButton,
};
#[cfg(feature = "carousel")]
pub use gpui_material_extras::{
    Carousel, CarouselAlignment, CarouselFling, CarouselFlingContext, CarouselItem,
    CarouselParallax, CarouselState, CarouselVariant,
};
#[cfg(feature = "toolbars")]
pub use gpui_material_extras::{
    FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors, FabMenuTriggerAppearance,
    FabMenuTriggerStyle, Toolbar, ToolbarAction, ToolbarColor, ToolbarColors, ToolbarExitDirection,
    ToolbarFabPosition, ToolbarFling, ToolbarOrientation, ToolbarPlacement, ToolbarScrollBehavior,
    ToolbarScrollState,
};
#[cfg(feature = "search")]
pub use gpui_material_extras::{
    SearchBar, SearchEvent, SearchMotion, SearchOutcome, SearchPopupOptions, SearchRequest,
    SearchResult, SearchScrim, SearchScrollState, SearchState, SearchStyle, SearchValue,
    SearchView, SearchViewVariant, SearchWindowOptions,
};
pub use gpui_material_foundation as foundation;
#[cfg(feature = "test-support")]
pub use gpui_material_foundation::test;
pub use gpui_material_foundation::{
    InteractionSource, InteractionState, InteractionUpdate, MaterialRoot, PolygonMorph,
    ReduceMotionPreference, Scrim, Shape, ShapeFit, ShapeMorph, ShapeMorphSequence, ShapePolygon,
    Surface, SurfaceBorder, SurfaceFeedback, ThemeScope, appearance_from_window,
    current_base_theme, current_material_strings, current_material_theme, material_strings,
    material_theme, observe_system_appearance, observe_window_system_appearance,
    project_base_theme, set_material_strings, set_material_theme, set_reduce_motion_preference,
    set_window_material_theme, sync_system_reduce_motion, system_appearance, window_material_theme,
};
pub use gpui_material_theme as theme;
#[cfg(not(feature = "display"))]
pub use gpui_material_theme::LogicalEdges;
pub use gpui_material_theme::{AppearanceMode, MaterialShape, MaterialStrings, MaterialTheme};

/// Installs the application [`MaterialTheme`], initializes [`MaterialRoot`], and
/// registers component keybindings when component features are enabled.
///
/// Call once per application before opening windows.
pub fn init(theme: MaterialTheme, cx: &mut gpui::App) {
    MaterialRoot::init(theme, cx);
    #[cfg(feature = "display")]
    gpui_material_components::init_components(cx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_exports_init_window_theme_and_logical_edges() {
        let _init: fn(MaterialTheme, &mut gpui::App) = init;
        let _get_window_theme: for<'a, 'b> fn(
            &'b gpui::Window,
            &'a gpui::App,
        ) -> &'a MaterialTheme = window_material_theme;
        let _set_window_theme: fn(Option<MaterialTheme>, &mut gpui::Window, &mut gpui::App) =
            set_window_material_theme;
        let _map_appearance: fn(gpui::WindowAppearance) -> theme::Appearance =
            appearance_from_window;
        let _system_appearance: fn(&gpui::App) -> theme::Appearance = system_appearance;
        let _sync_reduce_motion: fn(&mut gpui::App) = sync_system_reduce_motion;
        let _set_reduce_motion: fn(ReduceMotionPreference, &mut gpui::App) =
            set_reduce_motion_preference;
        let _get_strings: fn(&gpui::App) -> &MaterialStrings = material_strings;
        let _set_strings: fn(MaterialStrings, &mut gpui::App) = set_material_strings;
        assert_eq!(MaterialStrings::default().dismiss.as_ref(), "Dismiss");
        assert_eq!(
            AppearanceMode::System.resolve(theme::Appearance::Dark),
            theme::Appearance::Dark
        );
        let edges = LogicalEdges::symmetric(16.0, 8.0);
        assert_eq!(edges.start, 16.0);
        assert_eq!(edges.top, 8.0);
    }
}
